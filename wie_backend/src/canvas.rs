mod lbmp;

use alloc::{borrow::Cow, boxed::Box, string::ToString, vec, vec::Vec};
use core::mem::size_of;

use ab_glyph::{Font, FontRef, ScaleFont};
use bytemuck::{Pod, cast_slice, pod_collect_to_vec};
use image::ImageReader;
use num_traits::{Num, Zero};

use wie_util::{Result, WieError};

use self::lbmp::decode_lbmp;

lazy_static::lazy_static! {
    static ref FONT: FontRef<'static> = FontRef::try_from_slice(include_bytes!("../../fonts/neodgm.ttf")).unwrap();
}

pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Image: Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes_per_pixel(&self) -> u32;
    fn get_pixel(&self, x: i32, y: i32) -> Color;
    fn raw(&self) -> Cow<'_, [u8]>;
    fn colors(&self) -> Vec<Color>;
}

pub trait ImageBuffer: Send {
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]);
    fn xor_pixel(&mut self, x: i32, y: i32, color: Color);
}

#[allow(clippy::too_many_arguments)]
pub trait Canvas: Send {
    fn image(&self) -> &dyn Image;
    fn set_xor_mode(&mut self, xor_mode: bool);
    fn copy_area(&mut self, dx: i32, dy: i32, sx: i32, sy: i32, w: u32, h: u32, clip: Clip);
    fn draw(&mut self, dx: i32, dy: i32, w: u32, h: u32, src: &dyn Image, sx: i32, sy: i32, clip: Clip);
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, clip: Clip);
    fn draw_text(&mut self, string: &str, x: i32, y: i32, text_alignment: TextAlignment, color: Color, clip: Clip);
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip);
    fn draw_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip);
    fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip);
    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip);
    fn fill_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip);
    fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip);
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
}

pub trait PixelType: Send {
    type DataType: Copy + Pod + Num + Send;
    fn from_color(color: Color) -> Self::DataType;
    fn to_color(raw: Self::DataType) -> Color;
    fn xor_color(raw: Self::DataType, color: Color) -> Self::DataType;
}

pub struct Rgb332Pixel;

impl PixelType for Rgb332Pixel {
    type DataType = u8;

    fn from_color(color: Color) -> Self::DataType {
        let r = ((color.r as u16 * 7 + 127) / 255) as u8;
        let g = ((color.g as u16 * 7 + 127) / 255) as u8;
        let b = ((color.b as u16 * 3 + 127) / 255) as u8;

        (r << 5) | (g << 2) | b
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = (raw >> 5) & 0x7;
        let g = (raw >> 2) & 0x7;
        let b = raw & 0x3;

        Color {
            a: 0xff,
            r: r * 36,
            g: g * 36,
            b: b * 85,
        }
    }

    fn xor_color(raw: Self::DataType, color: Color) -> Self::DataType {
        raw ^ Self::from_color(color)
    }
}

pub struct Rgb565Pixel;

impl PixelType for Rgb565Pixel {
    type DataType = u16;

    fn from_color(color: Color) -> Self::DataType {
        let r = (color.r as u16) >> 3;
        let g = (color.g as u16) >> 2;
        let b = (color.b as u16) >> 3;

        (r << 11) | (g << 5) | b
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = ((raw >> 11) & 0x1f) as u8;
        let g = ((raw >> 5) & 0x3f) as u8;
        let b = (raw & 0x1f) as u8;

        let r = ((r as u32 * 255 + 15) / 31) as u8;
        let g = ((g as u32 * 255 + 31) / 63) as u8;
        let b = ((b as u32 * 255 + 15) / 31) as u8;

        Color { a: 0xff, r, g, b }
    }

    fn xor_color(raw: Self::DataType, color: Color) -> Self::DataType {
        raw ^ Self::from_color(color)
    }
}

pub struct Rgb8Pixel;

impl PixelType for Rgb8Pixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let b = (raw & 0xff) as u8;

        Color { a: 0xff, r, g, b }
    }

    fn xor_color(raw: Self::DataType, color: Color) -> Self::DataType {
        raw ^ Self::from_color(color)
    }
}

pub struct ArgbPixel;

impl PixelType for ArgbPixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        ((color.a as u32) << 24) | ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let a = ((raw >> 24) & 0xff) as u8;
        let r = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let b = (raw & 0xff) as u8;

        Color { a, r, g, b }
    }

    fn xor_color(raw: Self::DataType, color: Color) -> Self::DataType {
        let source = ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32;

        (raw ^ source) | 0xff000000
    }
}

pub struct AbgrPixel;

impl PixelType for AbgrPixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        ((color.a as u32) << 24) | ((color.b as u32) << 16) | ((color.g as u32) << 8) | color.r as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let a = ((raw >> 24) & 0xff) as u8;
        let b = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let r = (raw & 0xff) as u8;

        Color { a, r, g, b }
    }

    fn xor_color(raw: Self::DataType, color: Color) -> Self::DataType {
        let source = ((color.b as u32) << 16) | ((color.g as u32) << 8) | color.r as u32;

        (raw ^ source) | 0xff000000
    }
}

pub struct VecImageBuffer<T>
where
    T: PixelType,
{
    width: u32,
    height: u32,
    data: Vec<T::DataType>,
}

impl<T> VecImageBuffer<T>
where
    T: PixelType,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![T::DataType::zero(); (width * height) as usize],
        }
    }

    pub fn from_raw(width: u32, height: u32, raw: Vec<T::DataType>) -> Self {
        Self { width, height, data: raw }
    }
}

impl<T> Image for VecImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn bytes_per_pixel(&self) -> u32 {
        size_of::<T::DataType>() as u32
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        let raw = self.data[((y as u32) * self.width + (x as u32)) as usize];

        T::to_color(raw)
    }

    fn raw(&self) -> Cow<'_, [u8]> {
        cast_slice(&self.data).into()
    }

    fn colors(&self) -> Vec<Color> {
        self.data.iter().map(|&x| T::to_color(x)).collect()
    }
}

impl<T> ImageBuffer for VecImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
            return;
        }

        let raw = T::from_color(color);

        self.data[((y as u32) * self.width + (x as u32)) as usize] = raw;
    }

    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]) {
        for (i, color) in colors.iter().enumerate() {
            let x = x + (i as i32 % (width as i32));
            let y = y + (i as i32 / (width as i32));

            if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
                continue;
            }

            let raw = T::from_color(*color);

            self.data[((y as u32) * self.width + (x as u32)) as usize] = raw;
        }
    }

    fn xor_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
            return;
        }

        let offset = ((y as u32) * self.width + (x as u32)) as usize;
        self.data[offset] = T::xor_color(self.data[offset], color);
    }
}

pub struct ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    image_buffer: T,
    xor_mode: bool,
}

impl<T> ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    pub fn new(image_buffer: T) -> Self {
        Self {
            image_buffer,
            xor_mode: false,
        }
    }

    pub fn into_inner(self) -> T {
        self.image_buffer
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.compose_pixel(x, y, color, true);
    }

    fn compose_pixel(&mut self, x: i32, y: i32, color: Color, blend: bool) {
        if x < 0 || y < 0 || (x as u32) >= self.image_buffer.width() || (y as u32) >= self.image_buffer.height() {
            return;
        }

        if self.xor_mode {
            if color.a == 0 {
                return;
            }

            let color = if blend && color.a < 255 {
                Color {
                    a: 255,
                    r: ((color.r as u16 * color.a as u16 + 127) / 255) as u8,
                    g: ((color.g as u16 * color.a as u16 + 127) / 255) as u8,
                    b: ((color.b as u16 * color.a as u16 + 127) / 255) as u8,
                }
            } else {
                color
            };
            self.image_buffer.xor_pixel(x, y, color);
            return;
        }

        if !blend {
            self.image_buffer.put_pixel(x, y, color);
            return;
        }

        let bg = self.image_buffer.get_pixel(x, y);
        let factor = color.a as f32 / 255.0;

        let computed_color = Color {
            a: 0xff,
            r: (color.r as f32 * factor + bg.r as f32 * (1.0 - factor)) as u8,
            g: (color.g as f32 * factor + bg.g as f32 * (1.0 - factor)) as u8,
            b: (color.b as f32 * factor + bg.b as f32 * (1.0 - factor)) as u8,
        };

        self.image_buffer.put_pixel(x, y, computed_color);
    }

    fn plot(&mut self, x: i32, y: i32, color: Color, clip: &Clip) {
        if x < 0 || y < 0 || (x as u32) >= self.image_buffer.width() || (y as u32) >= self.image_buffer.height() {
            return;
        }
        if x < clip.x || x >= clip.x + clip.width as i32 || y < clip.y || y >= clip.y + clip.height as i32 {
            return;
        }
        self.put_pixel(x, y, color);
    }

    #[allow(clippy::too_many_arguments)]
    fn stroke_arc(&mut self, cx: f32, cy: f32, a: f32, b: f32, start_deg: f32, sweep_deg: f32, color: Color, clip: &Clip) {
        let radius = a.max(b).max(1.0);
        let sweep_rad = sweep_deg.to_radians();
        let start_rad = start_deg.to_radians();
        // the visible portion of any arc is bounded by the image perimeter, so cap
        // the step count; otherwise a guest-supplied huge radius spins for minutes
        let max_steps = 16 * (self.image_buffer.width() as i64 + self.image_buffer.height() as i64).max(1);
        let steps = ((sweep_rad.abs() * radius).ceil() as i64 * 2).clamp(1, max_steps) as i32;

        for i in 0..=steps {
            let theta = start_rad + sweep_rad * (i as f32) / (steps as f32);
            let px = (cx + a * theta.cos()).round() as i32;
            let py = (cy - b * theta.sin()).round() as i32;
            self.plot(px, py, color, clip);
        }
    }
}

/// Liang-Barsky clip of a segment to the image bounds (expanded by one pixel so
/// endpoints just outside still step in correctly). Endpoints already inside are
/// returned unchanged to keep the exact bresenham pixel pattern.
fn clip_segment(x1: i32, y1: i32, x2: i32, y2: i32, width: u32, height: u32) -> Option<(i32, i32, i32, i32)> {
    let (min_x, min_y) = (-1.0, -1.0);
    let (max_x, max_y) = (width as f64, height as f64);

    let inside = |x: i32, y: i32| (min_x..=max_x).contains(&(x as f64)) && (min_y..=max_y).contains(&(y as f64));
    if inside(x1, y1) && inside(x2, y2) {
        return Some((x1, y1, x2, y2));
    }

    let (fx1, fy1) = (x1 as f64, y1 as f64);
    let (dx, dy) = (x2 as f64 - fx1, y2 as f64 - fy1);

    let mut t0 = 0.0f64;
    let mut t1 = 1.0f64;
    for (p, q) in [(-dx, fx1 - min_x), (dx, max_x - fx1), (-dy, fy1 - min_y), (dy, max_y - fy1)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    if t0 > t1 {
        return None;
    }

    Some((
        (fx1 + dx * t0).round() as i32,
        (fy1 + dy * t0).round() as i32,
        (fx1 + dx * t1).round() as i32,
        (fy1 + dy * t1).round() as i32,
    ))
}

/// Intersect the span [start, start + len) with [0, max) in i64 so extreme
/// guest-supplied coordinates can neither overflow i32 nor iterate off-image.
fn clamp_span(start: i32, len: u32, max: u32) -> core::ops::Range<i32> {
    let s = (start as i64).max(0);
    let e = (start as i64 + len as i64).min(max as i64);

    if s >= e { 0..0 } else { s as i32..e as i32 }
}

/// Whether the point lies within the pie sweep starting at `start_deg` spanning
/// `sweep_deg` degrees (counterclockwise for positive, clockwise for negative).
/// Angles follow the J2ME/WIPI convention: 0° is 3 o'clock, positive is CCW.
fn point_in_sweep(px: f32, py: f32, cx: f32, cy: f32, start_deg: f32, sweep_deg: f32) -> bool {
    if sweep_deg.abs() >= 360.0 {
        return true;
    }
    if sweep_deg == 0.0 {
        return false;
    }

    let dx = px - cx;
    let dy = py - cy;
    if dx * dx + dy * dy < 1.0 {
        return true; // apex of the pie
    }

    // screen y grows downward, so negate dy to match the math convention
    let angle = (-dy).atan2(dx).to_degrees();
    let (start, sweep) = if sweep_deg > 0.0 {
        (start_deg, sweep_deg)
    } else {
        (start_deg + sweep_deg, -sweep_deg)
    };

    (angle - start).rem_euclid(360.0) <= sweep
}

#[allow(clippy::too_many_arguments)]
impl<T> Canvas for ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    fn image(&self) -> &dyn Image {
        &self.image_buffer
    }

    fn set_xor_mode(&mut self, xor_mode: bool) {
        self.xor_mode = xor_mode;
    }

    fn copy_area(&mut self, dx: i32, dy: i32, sx: i32, sy: i32, w: u32, h: u32, clip: Clip) {
        let image_width = (self.image_buffer.width() as i64).min(i32::MAX as i64);
        let image_height = (self.image_buffer.height() as i64).min(i32::MAX as i64);

        let x_start = (dx as i64).max(clip.x as i64).max(0);
        let y_start = (dy as i64).max(clip.y as i64).max(0);
        let x_end = (dx as i64 + w as i64).min(clip.x as i64 + clip.width as i64).min(image_width);
        let y_end = (dy as i64 + h as i64).min(clip.y as i64 + clip.height as i64).min(image_height);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        let mut pixels = Vec::with_capacity(((x_end - x_start) * (y_end - y_start)) as usize);
        for y_dst in y_start..y_end {
            for x_dst in x_start..x_end {
                let x_src = sx as i64 + x_dst - dx as i64;
                let y_src = sy as i64 + y_dst - dy as i64;

                if x_src < 0 || y_src < 0 || x_src >= image_width || y_src >= image_height {
                    continue;
                }

                pixels.push((x_dst as i32, y_dst as i32, self.image_buffer.get_pixel(x_src as i32, y_src as i32)));
            }
        }

        for (x, y, color) in pixels {
            self.image_buffer.put_pixel(x, y, color);
        }
    }

    fn draw(&mut self, dx: i32, dy: i32, w: u32, h: u32, src: &dyn Image, sx: i32, sy: i32, clip: Clip) {
        // iterate only the overlap of destination, source, and image bounds; i64 keeps
        // extreme guest offsets from overflowing or spinning through offscreen pixels
        let x_start = 0i64.max(-(dx as i64)).max(-(sx as i64));
        let x_end = (w as i64)
            .min(self.image_buffer.width() as i64 - dx as i64)
            .min(src.width() as i64 - sx as i64);
        let y_start = 0i64.max(-(dy as i64)).max(-(sy as i64));
        let y_end = (h as i64)
            .min(self.image_buffer.height() as i64 - dy as i64)
            .min(src.height() as i64 - sy as i64);

        for y in y_start..y_end {
            for x in x_start..x_end {
                let px = (dx as i64 + x) as i32;
                let py = (dy as i64 + y) as i32;
                if px < clip.x || px >= clip.x + (clip.width as i32) || py < clip.y || py >= clip.y + (clip.height as i32) {
                    continue;
                }

                // TODO blend multiple pixels at once for performance
                self.blend_pixel(px, py, src.get_pixel((sx as i64 + x) as i32, (sy as i64 + y) as i32));
            }
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, clip: Clip) {
        // pre-clip to image bounds: guest can pass extreme coordinates whose deltas
        // overflow i32 and whose bresenham walk would take billions of steps
        let Some((x1, y1, x2, y2)) = clip_segment(x1, y1, x2, y2, self.image_buffer.width(), self.image_buffer.height()) else {
            return;
        };

        // bresenham's line drawing
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            self.plot(x, y, color, &clip);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_text(&mut self, string: &str, x: i32, y: i32, text_alignment: TextAlignment, color: Color, clip: Clip) {
        let size = 10.0; // TODO
        let font = FONT.as_scaled(FONT.pt_to_px_scale(size).unwrap());

        let total_width = string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>();
        let x = match text_alignment {
            TextAlignment::Left => x,
            TextAlignment::Center => x - (total_width / 2.0) as i32,
            TextAlignment::Right => x - total_width as i32,
        };

        let mut position = 0.0;
        for c in string.chars() {
            if c.is_control() {
                continue;
            }

            let glyph = font.scaled_glyph(c);
            let h_advance = font.h_advance(glyph.id);

            if let Some(outlined_glyph) = font.outline_glyph(glyph) {
                outlined_glyph.draw(|glyph_x: u32, glyph_y, c| {
                    let bounds = outlined_glyph.px_bounds();
                    let px = x + (glyph_x as f32 + bounds.min.x + position) as i32;
                    let py = y + (glyph_y as f32 + bounds.min.y + size) as i32;
                    if px < clip.x || px >= clip.x + clip.width as i32 || py < clip.y || py >= clip.y + clip.height as i32 {
                        return;
                    }
                    self.blend_pixel(
                        px,
                        py,
                        Color {
                            a: (c * 255.0) as u8,
                            r: color.r,
                            g: color.g,
                            b: color.b,
                        },
                    )
                });
            }

            position += h_advance;
        }
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }

        // degenerate rect is a line; also avoids plotting pixels twice, which would
        // cancel out in xor mode
        if w == 1 || h == 1 {
            return self.fill_rect(x, y, w, h, color, clip);
        }

        let width = self.image_buffer.width();
        let height = self.image_buffer.height();
        let right = x as i64 + w as i64 - 1;
        let bottom = y as i64 + h as i64 - 1;

        for px in clamp_span(x, w, width) {
            self.plot(px, y, color, &clip);
            if bottom < height as i64 {
                self.plot(px, bottom as i32, color, &clip);
            }
        }

        let py_start = (y as i64 + 1).max(0);
        let py_end = bottom.min(height as i64);
        for py in py_start..py_end {
            self.plot(x, py as i32, color, &clip);
            if right < width as i64 {
                self.plot(right as i32, py as i32, color, &clip);
            }
        }
    }

    fn draw_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }

        let a = (w as f32 - 1.0) / 2.0;
        let b = (h as f32 - 1.0) / 2.0;
        let cx = x as f32 + a;
        let cy = y as f32 + b;

        self.stroke_arc(cx, cy, a, b, start_angle as f32, arc_angle as f32, color, &clip);
    }

    fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }
        if arc_width == 0 || arc_height == 0 {
            return self.draw_rect(x, y, w, h, color, clip);
        }

        let rx = (arc_width.min(w) as f32 / 2.0).max(0.5);
        let ry = (arc_height.min(h) as f32 / 2.0).max(0.5);
        let rxi = rx.round() as i64;
        let ryi = ry.round() as i64;

        let width = self.image_buffer.width() as i64;
        let height = self.image_buffer.height() as i64;
        let left = x as i64;
        let right = x as i64 + w as i64 - 1;
        let top = y as i64;
        let bottom = y as i64 + h as i64 - 1;

        let px_start = (left + rxi).max(0);
        let px_end = (right - rxi + 1).min(width);
        for px in px_start..px_end {
            if (0..height).contains(&top) {
                self.plot(px as i32, top as i32, color, &clip);
            }
            if (0..height).contains(&bottom) {
                self.plot(px as i32, bottom as i32, color, &clip);
            }
        }

        let py_start = (top + ryi).max(0);
        let py_end = (bottom - ryi + 1).min(height);
        for py in py_start..py_end {
            if (0..width).contains(&left) {
                self.plot(left as i32, py as i32, color, &clip);
            }
            if (0..width).contains(&right) {
                self.plot(right as i32, py as i32, color, &clip);
            }
        }

        self.stroke_arc(right as f32 - rx, top as f32 + ry, rx, ry, 0.0, 90.0, color, &clip);
        self.stroke_arc(left as f32 + rx, top as f32 + ry, rx, ry, 90.0, 90.0, color, &clip);
        self.stroke_arc(left as f32 + rx, bottom as f32 - ry, rx, ry, 180.0, 90.0, color, &clip);
        self.stroke_arc(right as f32 - rx, bottom as f32 - ry, rx, ry, 270.0, 90.0, color, &clip);
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip) {
        // TODO use put_pixels
        for py in clamp_span(y, h, self.image_buffer.height()) {
            for px in clamp_span(x, w, self.image_buffer.width()) {
                self.plot(px, py, color, &clip);
            }
        }
    }

    fn fill_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }

        let a = (w as f32 - 1.0) / 2.0;
        let b = (h as f32 - 1.0) / 2.0;
        let cx = x as f32 + a;
        let cy = y as f32 + b;
        let da = a.max(0.5);
        let db = b.max(0.5);
        let start = start_angle as f32;
        let sweep = arc_angle as f32;

        for py in clamp_span(y, h, self.image_buffer.height()) {
            for px in clamp_span(x, w, self.image_buffer.width()) {
                let nx = (px as f32 - cx) / da;
                let ny = (py as f32 - cy) / db;
                if nx * nx + ny * ny > 1.0 {
                    continue;
                }
                if !point_in_sweep(px as f32, py as f32, cx, cy, start, sweep) {
                    continue;
                }
                self.plot(px, py, color, &clip);
            }
        }
    }

    fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }
        if arc_width == 0 || arc_height == 0 {
            return self.fill_rect(x, y, w, h, color, clip);
        }

        let rx = (arc_width.min(w) as f32 / 2.0).max(0.5);
        let ry = (arc_height.min(h) as f32 / 2.0).max(0.5);

        let left_center = x as f32 + rx;
        let right_center = (x as i64 + w as i64 - 1) as f32 - rx;
        let top_center = y as f32 + ry;
        let bottom_center = (y as i64 + h as i64 - 1) as f32 - ry;

        for py in clamp_span(y, h, self.image_buffer.height()) {
            let cy = if (py as f32) < top_center {
                top_center
            } else if (py as f32) > bottom_center {
                bottom_center
            } else {
                py as f32
            };
            for px in clamp_span(x, w, self.image_buffer.width()) {
                let cx = if (px as f32) < left_center {
                    left_center
                } else if (px as f32) > right_center {
                    right_center
                } else {
                    px as f32
                };

                let nx = (px as f32 - cx) / rx;
                let ny = (py as f32 - cy) / ry;
                if nx * nx + ny * ny <= 1.0 {
                    self.plot(px, py, color, &clip);
                }
            }
        }
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.compose_pixel(x, y, color, false);
    }
}

pub struct Clip {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Clip {
    pub fn intersect(&self, other: &Clip) -> Clip {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x as i64 + self.width as i64).min(other.x as i64 + other.width as i64) - x as i64;
        let height = (self.y as i64 + self.height as i64).min(other.y as i64 + other.height as i64) - y as i64;

        Clip {
            x,
            y,
            width: width.clamp(0, u32::MAX as i64) as _,
            height: height.clamp(0, u32::MAX as i64) as _,
        }
    }
}

pub fn decode_image(data: &[u8]) -> Result<Box<dyn Image>> {
    extern crate std; // XXX

    use std::io::Cursor;

    if data[0] == b'L' && data[1] == b'B' && data[2] == b'M' && data[3] == b'P' {
        return decode_lbmp(data);
    }

    let image = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()
        .map_err(|x| WieError::FatalError(x.to_string()))?
        .decode()
        .map_err(|x| WieError::FatalError(x.to_string()))?;
    let rgba = image.into_rgba8();

    let data = rgba.pixels().flat_map(|x| [x.0[2], x.0[1], x.0[0], x.0[3]]).collect::<Vec<_>>();

    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(
        rgba.width(),
        rgba.height(),
        pod_collect_to_vec(&data),
    )) as Box<_>)
}

pub fn string_width(string: &str, pt_size: f32) -> f32 {
    let font = FONT.as_scaled(FONT.pt_to_px_scale(pt_size).unwrap());

    string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>()
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use wie_util::Result;

    use crate::canvas::{Clip, Image, ImageBufferCanvas};

    use super::{ArgbPixel, Canvas, Color, Rgb332Pixel, TextAlignment, VecImageBuffer};

    #[test]
    fn test_canvas() -> Result<()> {
        let image_buffer = VecImageBuffer::<ArgbPixel>::new(10, 10);
        let mut canvas = ImageBufferCanvas::new(image_buffer);

        let clip = Clip {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        canvas.fill_rect(0, 0, 10, 10, Color { r: 0, g: 0, b: 0, a: 255 }, clip);

        let image_buffer = canvas.into_inner();
        let raw = image_buffer.raw();

        assert_eq!(raw.len(), 10 * 10 * 4);
        for i in 0..10 * 10 {
            assert_eq!(raw[i * 4], 0);
            assert_eq!(raw[i * 4 + 1], 0);
            assert_eq!(raw[i * 4 + 2], 0);
            assert_eq!(raw[i * 4 + 3], 255);
        }

        Ok(())
    }

    fn full_clip(size: u32) -> Clip {
        Clip {
            x: 0,
            y: 0,
            width: size,
            height: size,
        }
    }

    const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    const BACKGROUND: Color = Color {
        a: 255,
        r: 0x12,
        g: 0x34,
        b: 0x56,
    };

    const XOR_COLOR: Color = Color {
        a: 255,
        r: 0xf0,
        g: 0x0f,
        b: 0xaa,
    };

    fn is_set(image: &impl Image, x: i32, y: i32) -> bool {
        let c = image.get_pixel(x, y);
        c.r != 0 || c.g != 0 || c.b != 0
    }

    fn assert_color(image: &dyn Image, x: i32, y: i32, expected: Color) {
        let actual = image.get_pixel(x, y);
        assert_eq!((actual.a, actual.r, actual.g, actual.b), (expected.a, expected.r, expected.g, expected.b));
    }

    #[test]
    fn test_xor_mode_fill_rect_toggles_pixels() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(4, 4));

        canvas.fill_rect(0, 0, 4, 4, BACKGROUND, full_clip(4));
        canvas.set_xor_mode(true);
        canvas.fill_rect(1, 1, 2, 2, XOR_COLOR, full_clip(4));

        assert_color(
            canvas.image(),
            1,
            1,
            Color {
                a: 255,
                r: BACKGROUND.r ^ XOR_COLOR.r,
                g: BACKGROUND.g ^ XOR_COLOR.g,
                b: BACKGROUND.b ^ XOR_COLOR.b,
            },
        );
        assert_color(canvas.image(), 0, 0, BACKGROUND);

        canvas.fill_rect(1, 1, 2, 2, XOR_COLOR, full_clip(4));

        assert_color(canvas.image(), 1, 1, BACKGROUND);
    }

    #[test]
    fn test_xor_mode_transparent_source_is_noop() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(1, 1));
        canvas.fill_rect(0, 0, 1, 1, BACKGROUND, full_clip(1));
        canvas.set_xor_mode(true);

        let src = VecImageBuffer::<ArgbPixel>::from_raw(1, 1, vec![0x00ff0000]);
        canvas.draw(0, 0, 1, 1, &src, 0, 0, full_clip(1));

        assert_color(canvas.image(), 0, 0, BACKGROUND);
    }

    #[test]
    fn test_xor_mode_rgb332_toggles_native_pixel() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<Rgb332Pixel>::new(1, 1));
        let background = Color { a: 255, r: 0, g: 0, b: 0 };
        let source = Color { a: 255, r: 19, g: 0, b: 0 };

        canvas.fill_rect(0, 0, 1, 1, background, full_clip(1));
        canvas.set_xor_mode(true);
        canvas.fill_rect(0, 0, 1, 1, source, full_clip(1));
        canvas.fill_rect(0, 0, 1, 1, source, full_clip(1));

        assert_color(canvas.image(), 0, 0, background);
    }

    #[test]
    fn test_copy_area_uses_source_snapshot_for_overlap() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(4, 1));
        let red = Color { a: 255, r: 255, g: 0, b: 0 };
        let green = Color { a: 255, r: 0, g: 255, b: 0 };
        let blue = Color { a: 255, r: 0, g: 0, b: 255 };
        let black = Color { a: 255, r: 0, g: 0, b: 0 };

        canvas.put_pixel(0, 0, red);
        canvas.put_pixel(1, 0, green);
        canvas.put_pixel(2, 0, blue);
        canvas.put_pixel(3, 0, black);

        canvas.copy_area(1, 0, 0, 0, 3, 1, full_clip(4));

        assert_color(canvas.image(), 0, 0, red);
        assert_color(canvas.image(), 1, 0, red);
        assert_color(canvas.image(), 2, 0, green);
        assert_color(canvas.image(), 3, 0, blue);
    }

    #[test]
    fn test_copy_area_respects_clip() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(4, 1));
        let red = Color { a: 255, r: 255, g: 0, b: 0 };
        let green = Color { a: 255, r: 0, g: 255, b: 0 };
        let blue = Color { a: 255, r: 0, g: 0, b: 255 };
        let black = Color { a: 255, r: 0, g: 0, b: 0 };
        let clip = Clip {
            x: 2,
            y: 0,
            width: 1,
            height: 1,
        };

        canvas.put_pixel(0, 0, red);
        canvas.put_pixel(1, 0, green);
        canvas.put_pixel(2, 0, blue);
        canvas.put_pixel(3, 0, black);

        canvas.copy_area(1, 0, 0, 0, 3, 1, clip);

        assert_color(canvas.image(), 0, 0, red);
        assert_color(canvas.image(), 1, 0, green);
        assert_color(canvas.image(), 2, 0, green);
        assert_color(canvas.image(), 3, 0, black);
    }

    #[test]
    fn test_fill_arc_full_circle() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.fill_arc(0, 0, 32, 32, 0, 360, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 16, 16), "center should be filled");
        assert!(!is_set(&image, 0, 0), "corner should be outside the ellipse");
        assert!(!is_set(&image, 31, 31), "corner should be outside the ellipse");
    }

    #[test]
    fn test_fill_arc_quadrant() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        // 0..90 deg sweeps from 3 o'clock CCW to 12 o'clock => upper-right quadrant
        canvas.fill_arc(0, 0, 32, 32, 0, 90, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 24, 8), "upper-right quadrant should be filled");
        assert!(!is_set(&image, 8, 8), "upper-left quadrant should be empty");
        assert!(!is_set(&image, 8, 24), "lower-left quadrant should be empty");
        assert!(!is_set(&image, 24, 24), "lower-right quadrant should be empty");
    }

    #[test]
    fn test_fill_arc_negative_sweep_is_clockwise() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        // 0..-90 deg sweeps CW from 3 o'clock to 6 o'clock => lower-right quadrant
        canvas.fill_arc(0, 0, 32, 32, 0, -90, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 24, 24), "lower-right quadrant should be filled");
        assert!(!is_set(&image, 24, 8), "upper-right quadrant should be empty");
    }

    #[test]
    fn test_draw_arc_endpoint() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.draw_arc(0, 0, 32, 32, 0, 360, WHITE, full_clip(32));
        let image = canvas.into_inner();

        // bounding box 32x32 => center (15.5, 15.5), radius 15.5; nominal endpoints round to:
        assert!(is_set(&image, 31, 16), "3 o'clock (0 deg) point");
        assert!(is_set(&image, 16, 0), "12 o'clock (90 deg) point");
        assert!(is_set(&image, 0, 16), "9 o'clock (180 deg) point");
        assert!(is_set(&image, 16, 31), "6 o'clock (270 deg) point");
        assert!(!is_set(&image, 16, 16), "arc outline must not fill the interior");
    }

    #[test]
    fn test_draw_round_rect_outline() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.draw_round_rect(0, 0, 32, 32, 16, 16, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 16, 0), "top straight edge");
        assert!(is_set(&image, 16, 31), "bottom straight edge");
        assert!(is_set(&image, 0, 16), "left straight edge");
        assert!(is_set(&image, 31, 16), "right straight edge");
        assert!(!is_set(&image, 0, 0), "rounded corner cut away");
        assert!(!is_set(&image, 16, 16), "outline must not fill the interior");
    }

    #[test]
    fn test_fill_arc_degenerate_width() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(8, 8));
        canvas.fill_arc(3, 0, 1, 8, 0, 360, WHITE, full_clip(8));
        let image = canvas.into_inner();

        for y in 0..8 {
            assert!(is_set(&image, 3, y), "degenerate 1px-wide column should be fully filled (y={y})");
        }
    }

    #[test]
    fn test_fill_round_rect_corners_empty() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.fill_round_rect(0, 0, 32, 32, 16, 16, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 16, 16), "center should be filled");
        assert!(is_set(&image, 16, 0), "straight top edge should be filled");
        assert!(!is_set(&image, 0, 0), "rounded corner should be empty");
        assert!(!is_set(&image, 31, 0), "rounded corner should be empty");
    }

    #[test]
    fn test_clip_intersect_disjoint_is_empty() {
        let a = Clip {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        let b = Clip {
            x: 20,
            y: 20,
            width: 5,
            height: 5,
        };

        let result = a.intersect(&b);

        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_clip_intersect_partial() {
        let a = Clip {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        let b = Clip {
            x: 5,
            y: 5,
            width: 10,
            height: 10,
        };

        let result = a.intersect(&b);

        assert_eq!(result.x, 5);
        assert_eq!(result.y, 5);
        assert_eq!(result.width, 5);
        assert_eq!(result.height, 5);
    }

    #[test]
    fn test_clip_intersect_extreme_values_no_overflow() {
        let a = Clip {
            x: 1,
            y: 1,
            width: u32::MAX,
            height: u32::MAX,
        };
        let b = Clip {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };

        let result = a.intersect(&b);

        assert_eq!(result.x, 1);
        assert_eq!(result.y, 1);
        assert_eq!(result.width, 9);
        assert_eq!(result.height, 9);
    }

    #[test]
    fn test_draw_rect_clips_edges_independently() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        let clip = Clip {
            x: 0,
            y: 16,
            width: 32,
            height: 16,
        };
        canvas.draw_rect(4, 4, 10, 20, WHITE, clip);
        let image = canvas.into_inner();

        assert!(is_set(&image, 4, 23), "bottom edge inside clip should be drawn");
        assert!(is_set(&image, 8, 23), "bottom edge interior pixels inside clip should be drawn");
        assert!(is_set(&image, 13, 23), "bottom edge inside clip should be drawn");
        assert!(!is_set(&image, 4, 4), "top edge outside clip must not be drawn");
        assert!(!is_set(&image, 4, 15), "vertical edges above clip must not be drawn");
        assert!(is_set(&image, 4, 16), "left edge inside clip should be drawn");
        assert!(is_set(&image, 13, 20), "right edge inside clip should be drawn");
        assert!(!is_set(&image, 5, 20), "rect interior must not be filled");
    }

    #[test]
    fn test_draw_line_includes_endpoint() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));
        canvas.draw_line(1, 1, 5, 5, WHITE, full_clip(10));
        let image = canvas.into_inner();

        assert!(is_set(&image, 1, 1), "start point should be drawn");
        assert!(is_set(&image, 5, 5), "end point should be drawn");
    }

    #[test]
    fn test_draw_line_extreme_coordinates() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));
        canvas.draw_line(-2_000_000_000, 5, 2_000_000_000, 5, WHITE, full_clip(10));
        let image = canvas.into_inner();

        for x in 0..10 {
            assert!(is_set(&image, x, 5), "visible span of extreme line should be drawn (x={x})");
        }
    }

    #[test]
    fn test_draw_text_respects_clip() {
        let empty_clip = Clip {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(30, 20));
        canvas.draw_text("A", 2, 2, TextAlignment::Left, WHITE, empty_clip);
        let clipped = canvas.into_inner();

        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(30, 20));
        canvas.draw_text("A", 2, 2, TextAlignment::Left, WHITE, full_clip(30));
        let unclipped = canvas.into_inner();

        let count_set = |image: &VecImageBuffer<ArgbPixel>| {
            (0..20)
                .flat_map(|y| (0..30).map(move |x| (x, y)))
                .filter(|&(x, y)| is_set(image, x, y))
                .count()
        };
        assert_eq!(count_set(&clipped), 0, "empty clip must draw nothing");
        assert!(count_set(&unclipped) > 0, "full clip must draw glyph pixels");
    }

    #[test]
    fn test_extreme_dimensions_terminate() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));

        // must complete quickly and draw only the visible portion, whatever the guest passes
        canvas.draw_rect(0, 0, u32::MAX, u32::MAX, WHITE, full_clip(10));
        canvas.fill_rect(-5, -5, u32::MAX, u32::MAX, WHITE, full_clip(10));
        canvas.draw_rect(2, 2, 0x8000_0000, 5, WHITE, full_clip(10));
        canvas.fill_arc(0, 0, u32::MAX, u32::MAX, 0, 360, WHITE, full_clip(10));
        canvas.fill_round_rect(0, 0, u32::MAX, u32::MAX, 4, 4, WHITE, full_clip(10));
        canvas.draw_round_rect(-100, -100, u32::MAX, u32::MAX, 4, 4, WHITE, full_clip(10));
        canvas.draw_arc(0, 0, u32::MAX, u32::MAX, 0, 360, WHITE, full_clip(10));
        canvas.draw(
            -2_000_000_000,
            -2_000_000_000,
            u32::MAX,
            u32::MAX,
            &VecImageBuffer::<ArgbPixel>::new(4, 4),
            0,
            0,
            full_clip(10),
        );

        let image = canvas.into_inner();
        assert!(is_set(&image, 0, 0), "full-coverage fill must reach the visible area");
    }

    #[test]
    fn test_draw_rect_edges_visible_when_partially_offscreen() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));

        // right/bottom edges far offscreen: only top and left edges are visible
        canvas.draw_rect(2, 2, 1_000_000, 1_000_000, WHITE, full_clip(10));
        let image = canvas.into_inner();

        assert!(is_set(&image, 5, 2), "top edge should be drawn");
        assert!(is_set(&image, 2, 5), "left edge should be drawn");
        assert!(!is_set(&image, 5, 5), "interior must stay empty");
    }

    #[test]
    fn test_draw_rect_xor_toggles_corners_once() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(8, 8));
        canvas.fill_rect(0, 0, 8, 8, BACKGROUND, full_clip(8));
        canvas.set_xor_mode(true);
        canvas.draw_rect(1, 1, 4, 4, XOR_COLOR, full_clip(8));

        let toggled = Color {
            a: 255,
            r: BACKGROUND.r ^ XOR_COLOR.r,
            g: BACKGROUND.g ^ XOR_COLOR.g,
            b: BACKGROUND.b ^ XOR_COLOR.b,
        };
        assert_color(canvas.image(), 1, 1, toggled);
        assert_color(canvas.image(), 4, 4, toggled);
        assert_color(canvas.image(), 2, 1, toggled);
        assert_color(canvas.image(), 1, 2, toggled);
    }

    #[test]
    fn test_draw_line_respects_clip() {
        let clip = Clip {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));
        canvas.draw_line(6, 6, 9, 9, WHITE, clip);
        let image = canvas.into_inner();

        for i in 6..10 {
            assert!(!is_set(&image, i, i), "line outside clip must not be drawn ({i},{i})");
        }

        let clip = Clip {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));
        canvas.draw_line(0, 0, 9, 9, WHITE, clip);
        let image = canvas.into_inner();

        assert!(is_set(&image, 2, 2), "part of the line inside clip should be drawn");
        assert!(!is_set(&image, 5, 5), "part of the line outside clip must not be drawn");
    }

    #[test]
    fn test_draw_line_single_point() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(10, 10));
        canvas.draw_line(3, 3, 3, 3, WHITE, full_clip(10));
        let image = canvas.into_inner();

        assert!(is_set(&image, 3, 3));
    }

    #[test]
    fn test_arc_respects_clip() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        let clip = Clip {
            x: 0,
            y: 0,
            width: 16,
            height: 32,
        };
        canvas.fill_arc(0, 0, 32, 32, 0, 360, WHITE, clip);
        let image = canvas.into_inner();

        assert!(is_set(&image, 8, 16), "inside clip should be filled");
        assert!(!is_set(&image, 24, 16), "outside clip must not be filled");
    }
}
