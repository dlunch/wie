use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Graphics, Image};

const CLASS: &str = "javax/microedition/lcdui/game/TiledLayer";
const LAYER: &str = "javax/microedition/lcdui/game/Layer";

// Snapshot of a tiled layer's grid, with animated tiles resolved to the static tiles they currently show.
pub struct Tiles {
    pub x: i32,
    pub y: i32,
    pub tile_width: i32,
    pub tile_height: i32,
    pub columns: i32,
    pub rows: i32,
    pub image: ClassInstanceRef<Image>,
    cells: Vec<i32>,
    tiles_per_row: i32,
}

impl Tiles {
    // Static tile index of a cell; 0 is an empty cell.
    pub fn tile(&self, column: i32, row: i32) -> i32 {
        self.cells[(row * self.columns + column) as usize]
    }

    // Upper-left corner of a static tile within the tile set image.
    pub fn source(&self, tile: i32) -> (i32, i32) {
        (
            (tile - 1) % self.tiles_per_row * self.tile_width,
            (tile - 1) / self.tiles_per_row * self.tile_height,
        )
    }
}

// class javax.microedition.lcdui.game.TiledLayer
pub struct TiledLayer;

#[allow(clippy::too_many_arguments)]
impl TiledLayer {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/TiledLayer",
            parent_class: Some("javax/microedition/lcdui/game/Layer"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(IILjavax/microedition/lcdui/Image;II)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("createAnimatedTile", "(I)I", Self::create_animated_tile, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setAnimatedTile", "(II)V", Self::set_animated_tile, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getAnimatedTile", "(I)I", Self::get_animated_tile, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setCell", "(III)V", Self::set_cell, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCell", "(II)I", Self::get_cell, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillCells", "(IIIII)V", Self::fill_cells, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getCellWidth",
                    "()I",
                    Self::get_cell_width,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new(
                    "getCellHeight",
                    "()I",
                    Self::get_cell_height,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new(
                    "getColumns",
                    "()I",
                    Self::get_columns,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new("getRows", "()I", Self::get_rows, MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL),
                JavaMethodProto::new(
                    "setStaticTileSet",
                    "(Ljavax/microedition/lcdui/Image;II)V",
                    Self::set_static_tile_set,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::paint,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tileWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tileHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tileCount", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("columns", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("rows", "I", FieldAccessFlags::PRIVATE),
                // Row-major tile indices: 0 is empty, positive is a static tile, negative is an animated tile.
                JavaFieldProto::new("cells", "[I", FieldAccessFlags::PRIVATE),
                // Static tile shown by animated tile -(index + 1).
                JavaFieldProto::new("animatedTiles", "[I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        columns: i32,
        rows: i32,
        image: ClassInstanceRef<Image>,
        tile_width: i32,
        tile_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::<init>({this:?}, {columns}, {rows}, {image:?}, {tile_width}, {tile_height})");

        let _: () = jvm.invoke_special(&this, LAYER, "<init>", "()V", ()).await?;

        if columns < 1 || rows < 1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid grid size").await);
        }

        jvm.put_field(&mut this, CLASS, "columns", "I", columns).await?;
        jvm.put_field(&mut this, CLASS, "rows", "I", rows).await?;
        let cells = jvm.instantiate_array("I", (columns as usize) * (rows as usize)).await?;
        jvm.put_field(&mut this, CLASS, "cells", "[I", cells).await?;
        let animated_tiles = jvm.instantiate_array("I", 0).await?;
        jvm.put_field(&mut this, CLASS, "animatedTiles", "[I", animated_tiles).await?;

        jvm.invoke_virtual(
            &this,
            CLASS,
            "setStaticTileSet",
            "(Ljavax/microedition/lcdui/Image;II)V",
            (image, tile_width, tile_height),
        )
        .await
    }

    async fn set_static_tile_set(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        tile_width: i32,
        tile_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::setStaticTileSet({this:?}, {image:?}, {tile_width}, {tile_height})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let image_width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let image_height: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await?;
        if tile_width < 1 || tile_height < 1 || image_width % tile_width != 0 || image_height % tile_height != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid tile size").await);
        }

        let columns: i32 = jvm.get_field(&this, CLASS, "columns", "I").await?;
        let rows: i32 = jvm.get_field(&this, CLASS, "rows", "I").await?;
        let old_tile_count: i32 = jvm.get_field(&this, CLASS, "tileCount", "I").await?;
        let tile_count = (image_width / tile_width) * (image_height / tile_height);
        // A smaller tile set can leave cells pointing at tiles that no longer exist, so the grid starts over.
        if tile_count < old_tile_count {
            let cells = jvm.instantiate_array("I", (columns as usize) * (rows as usize)).await?;
            jvm.put_field(&mut this, CLASS, "cells", "[I", cells).await?;
            let animated_tiles = jvm.instantiate_array("I", 0).await?;
            jvm.put_field(&mut this, CLASS, "animatedTiles", "[I", animated_tiles).await?;
        }

        jvm.put_field(&mut this, CLASS, "image", "Ljavax/microedition/lcdui/Image;", image)
            .await?;
        jvm.put_field(&mut this, CLASS, "tileWidth", "I", tile_width).await?;
        jvm.put_field(&mut this, CLASS, "tileHeight", "I", tile_height).await?;
        jvm.put_field(&mut this, CLASS, "tileCount", "I", tile_count).await?;
        jvm.put_field(&mut this, LAYER, "width", "I", columns.saturating_mul(tile_width)).await?;
        jvm.put_field(&mut this, LAYER, "height", "I", rows.saturating_mul(tile_height)).await?;

        Ok(())
    }

    async fn create_animated_tile(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, static_tile_index: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::createAnimatedTile({this:?}, {static_tile_index})");

        let tile_count: i32 = jvm.get_field(&this, CLASS, "tileCount", "I").await?;
        if static_tile_index < 0 || static_tile_index > tile_count {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid static tile index").await);
        }

        let animated_tiles: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "animatedTiles", "[I").await?;
        let mut tiles: Vec<i32> = jvm.load_array(&animated_tiles, 0, jvm.array_length(&animated_tiles).await?).await?;
        tiles.push(static_tile_index);
        let count = tiles.len();
        let mut array = jvm.instantiate_array("I", count).await?;
        jvm.store_array(&mut array, 0, tiles).await?;
        jvm.put_field(&mut this, CLASS, "animatedTiles", "[I", array).await?;

        Ok(-(count as i32))
    }

    async fn set_animated_tile(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        animated_tile_index: i32,
        static_tile_index: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::setAnimatedTile({this:?}, {animated_tile_index}, {static_tile_index})");

        let tile_count: i32 = jvm.get_field(&this, CLASS, "tileCount", "I").await?;
        let mut animated_tiles: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "animatedTiles", "[I").await?;
        let count = jvm.array_length(&animated_tiles).await? as i32;
        if animated_tile_index >= 0 || animated_tile_index < -count || static_tile_index < 0 || static_tile_index > tile_count {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid tile index").await);
        }

        jvm.store_array(&mut animated_tiles, (-animated_tile_index - 1) as usize, [static_tile_index])
            .await
    }

    async fn get_animated_tile(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, animated_tile_index: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::getAnimatedTile({this:?}, {animated_tile_index})");

        let animated_tiles: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "animatedTiles", "[I").await?;
        let count = jvm.array_length(&animated_tiles).await? as i32;
        if animated_tile_index >= 0 || animated_tile_index < -count {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid animated tile index").await);
        }

        Ok(jvm.load_array(&animated_tiles, (-animated_tile_index - 1) as usize, 1).await?[0])
    }

    async fn set_cell(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, column: i32, row: i32, tile_index: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::setCell({this:?}, {column}, {row}, {tile_index})");

        jvm.invoke_virtual(&this, CLASS, "fillCells", "(IIIII)V", (column, row, 1, 1, tile_index))
            .await
    }

    async fn get_cell(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, column: i32, row: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::getCell({this:?}, {column}, {row})");

        let columns: i32 = jvm.get_field(&this, CLASS, "columns", "I").await?;
        let rows: i32 = jvm.get_field(&this, CLASS, "rows", "I").await?;
        if column < 0 || column >= columns || row < 0 || row >= rows {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid cell").await);
        }

        let cells: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "cells", "[I").await?;
        Ok(jvm.load_array(&cells, (row * columns + column) as usize, 1).await?[0])
    }

    async fn fill_cells(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        column: i32,
        row: i32,
        num_columns: i32,
        num_rows: i32,
        tile_index: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::fillCells({this:?}, {column}, {row}, {num_columns}, {num_rows}, {tile_index})");

        if num_columns < 0 || num_rows < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Negative region size").await);
        }

        let columns: i32 = jvm.get_field(&this, CLASS, "columns", "I").await?;
        let rows: i32 = jvm.get_field(&this, CLASS, "rows", "I").await?;
        let tile_count: i32 = jvm.get_field(&this, CLASS, "tileCount", "I").await?;
        let animated_tiles: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "animatedTiles", "[I").await?;
        let animated_count = jvm.array_length(&animated_tiles).await? as i32;
        let in_grid = column >= 0 && row >= 0 && column as i64 + num_columns as i64 <= columns as i64 && row as i64 + num_rows as i64 <= rows as i64;
        if !in_grid || tile_index > tile_count || tile_index < -animated_count {
            return Err(jvm
                .exception("java/lang/IndexOutOfBoundsException", "Invalid cell region or tile index")
                .await);
        }

        let mut cells: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "cells", "[I").await?;
        for row in row..row + num_rows {
            jvm.store_array(&mut cells, (row * columns + column) as usize, vec![tile_index; num_columns as usize])
                .await?;
        }

        Ok(())
    }

    async fn get_cell_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::getCellWidth({this:?})");

        jvm.get_field(&this, CLASS, "tileWidth", "I").await
    }

    async fn get_cell_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::getCellHeight({this:?})");

        jvm.get_field(&this, CLASS, "tileHeight", "I").await
    }

    async fn get_columns(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::getColumns({this:?})");

        jvm.get_field(&this, CLASS, "columns", "I").await
    }

    async fn get_rows(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::getRows({this:?})");

        jvm.get_field(&this, CLASS, "rows", "I").await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.TiledLayer::paint({this:?}, {g:?})");

        if g.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "g is null").await);
        }
        let visible: bool = jvm.get_field(&this, LAYER, "visible", "Z").await?;
        if !visible {
            return Ok(());
        }

        let tiles = Self::tiles(jvm, &this).await?;
        let mut clip = [0; 4];
        for (value, getter) in clip.iter_mut().zip(["getClipX", "getClipY", "getClipWidth", "getClipHeight"]) {
            *value = jvm.invoke_virtual(&g, "javax/microedition/lcdui/Graphics", getter, "()I", ()).await?;
        }
        let [clip_x, clip_y, clip_width, clip_height] = clip.map(i64::from);
        if clip_width <= 0 || clip_height <= 0 {
            return Ok(());
        }

        // Only the cells that intersect the clip are drawn.
        let (x, y) = (i64::from(tiles.x), i64::from(tiles.y));
        let (tile_width, tile_height) = (i64::from(tiles.tile_width), i64::from(tiles.tile_height));
        let first_column = (clip_x - x).div_euclid(tile_width).max(0);
        let last_column = (clip_x + clip_width - 1 - x).div_euclid(tile_width).min(i64::from(tiles.columns) - 1);
        let first_row = (clip_y - y).div_euclid(tile_height).max(0);
        let last_row = (clip_y + clip_height - 1 - y).div_euclid(tile_height).min(i64::from(tiles.rows) - 1);
        for row in first_row..=last_row {
            for column in first_column..=last_column {
                let (column, row) = (column as i32, row as i32);
                let tile = tiles.tile(column, row);
                if tile == 0 {
                    continue;
                }
                let (source_x, source_y) = tiles.source(tile);
                let _: () = jvm
                    .invoke_virtual(
                        &g,
                        "javax/microedition/lcdui/Graphics",
                        "drawRegion",
                        "(Ljavax/microedition/lcdui/Image;IIIIIIII)V",
                        [
                            tiles.image.clone().into(),
                            source_x.into(),
                            source_y.into(),
                            tiles.tile_width.into(),
                            tiles.tile_height.into(),
                            0.into(),
                            (tiles.x + column * tiles.tile_width).into(),
                            (tiles.y + row * tiles.tile_height).into(),
                            (4 | 16).into(), // LEFT | TOP
                        ],
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn tiles(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Tiles> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, CLASS, "image", "Ljavax/microedition/lcdui/Image;").await?;
        let image_width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let tile_width: i32 = jvm.get_field(this, CLASS, "tileWidth", "I").await?;

        let animated_tiles: ClassInstanceRef<Array<i32>> = jvm.get_field(this, CLASS, "animatedTiles", "[I").await?;
        let animated_tiles: Vec<i32> = jvm.load_array(&animated_tiles, 0, jvm.array_length(&animated_tiles).await?).await?;
        let cells: ClassInstanceRef<Array<i32>> = jvm.get_field(this, CLASS, "cells", "[I").await?;
        let cells: Vec<i32> = jvm.load_array(&cells, 0, jvm.array_length(&cells).await?).await?;

        Ok(Tiles {
            x: jvm.get_field(this, LAYER, "x", "I").await?,
            y: jvm.get_field(this, LAYER, "y", "I").await?,
            tile_width,
            tile_height: jvm.get_field(this, CLASS, "tileHeight", "I").await?,
            columns: jvm.get_field(this, CLASS, "columns", "I").await?,
            rows: jvm.get_field(this, CLASS, "rows", "I").await?,
            image,
            cells: cells
                .into_iter()
                .map(|tile| if tile < 0 { animated_tiles[(-tile - 1) as usize] } else { tile })
                .collect(),
            tiles_per_row: image_width / tile_width,
        })
    }
}
