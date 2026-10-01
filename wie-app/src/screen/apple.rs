use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    ptr::NonNull,
    rc::Rc,
};

use anyhow::{Result, anyhow};
use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, WindowHandle};
use softbuffer::{Context, Surface};
use tauri::webview::PlatformWebview;

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSAutoresizingMaskOptions, NSView as HostView};
#[cfg(target_os = "ios")]
use objc2_foundation::NSArray;
#[cfg(target_os = "ios")]
use objc2_ui_kit::{NSLayoutConstraint, UILayoutConstraintAxis, UIStackView, UITraitEnvironment, UIView as HostView};
#[cfg(target_os = "macos")]
use raw_window_handle::AppKitWindowHandle;
#[cfg(target_os = "ios")]
use raw_window_handle::UiKitWindowHandle;

use super::Frame;
#[cfg(target_os = "macos")]
use super::{CONTROLS_HEIGHT, GAME_HEIGHT};

struct GameView(Retained<HostView>);

impl HasWindowHandle for GameView {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        let pointer = NonNull::from(&*self.0).cast();
        #[cfg(target_os = "macos")]
        let raw = AppKitWindowHandle::new(pointer);
        #[cfg(target_os = "ios")]
        let raw = UiKitWindowHandle::new(pointer);
        // The retained view outlives its surface and is accessed only on the main thread.
        Ok(unsafe { WindowHandle::borrow_raw(raw.into()) })
    }
}

pub(super) struct View {
    parent: Retained<HostView>,
    #[cfg(target_os = "macos")]
    webview: Retained<HostView>,
    game: Rc<GameView>,
    surface: RefCell<Option<Surface<DisplayHandle<'static>, Rc<GameView>>>>,
    frame: RefCell<Option<Frame>>,
    playing: Cell<bool>,
    #[cfg(target_os = "ios")]
    stack: Retained<UIStackView>,
    #[cfg(target_os = "ios")]
    game_height: Retained<NSLayoutConstraint>,
}

impl View {
    pub(super) fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
        let main = MainThreadMarker::new().ok_or_else(|| anyhow!("Native view requires the main thread"))?;
        let webview = unsafe { Retained::<HostView>::retain(webview.inner().cast()) }.ok_or_else(|| anyhow!("WebView is unavailable"))?;
        #[cfg(target_os = "macos")]
        let parent = unsafe { webview.superview() };
        #[cfg(target_os = "ios")]
        let parent = webview.superview();
        let parent = parent.ok_or_else(|| anyhow!("WebView has no parent"))?;
        let game = Rc::new(GameView(HostView::initWithFrame(
            HostView::alloc(main),
            NSRect::new(NSPoint::ZERO, NSSize::ZERO),
        )));
        game.0.setHidden(true);

        #[cfg(target_os = "macos")]
        {
            webview.setAutoresizingMask(NSAutoresizingMaskOptions::empty());
            parent.addSubview(&game.0);
        }
        #[cfg(target_os = "ios")]
        let (stack, game_height) = {
            let stack = UIStackView::initWithFrame(UIStackView::alloc(main), parent.bounds());
            stack.setAxis(UILayoutConstraintAxis::Vertical);
            stack.setTranslatesAutoresizingMaskIntoConstraints(false);
            webview.removeFromSuperview();
            webview.setTranslatesAutoresizingMaskIntoConstraints(false);
            game.0.setTranslatesAutoresizingMaskIntoConstraints(false);
            parent.addSubview(&stack);
            stack.addArrangedSubview(&game.0);
            stack.addArrangedSubview(&webview);
            let safe_area = parent.safeAreaLayoutGuide();
            NSLayoutConstraint::activateConstraints(
                &NSArray::from_retained_slice(&[
                    stack.topAnchor().constraintEqualToAnchor(&safe_area.topAnchor()),
                    stack.bottomAnchor().constraintEqualToAnchor(&safe_area.bottomAnchor()),
                    stack.leadingAnchor().constraintEqualToAnchor(&safe_area.leadingAnchor()),
                    stack.trailingAnchor().constraintEqualToAnchor(&safe_area.trailingAnchor()),
                ]),
                main,
            );
            let height = game
                .0
                .heightAnchor()
                .constraintEqualToAnchor_multiplier(&stack.heightAnchor(), 480.0 / 740.0);
            (stack, height)
        };
        Ok(Rc::new(Self {
            parent,
            #[cfg(target_os = "macos")]
            webview,
            game,
            surface: RefCell::new(None),
            frame: RefCell::new(None),
            playing: Cell::new(false),
            #[cfg(target_os = "ios")]
            stack,
            #[cfg(target_os = "ios")]
            game_height,
        }))
    }

    pub(super) fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        if playing && self.surface.borrow().is_none() {
            #[cfg(target_os = "macos")]
            let display = DisplayHandle::appkit();
            #[cfg(target_os = "ios")]
            let display = DisplayHandle::uikit();
            let context = Context::new(display).map_err(|error| anyhow!(error.to_string()))?;
            let surface = Surface::new(&context, self.game.clone()).map_err(|error| anyhow!(error.to_string()))?;
            self.surface.replace(Some(surface));
        } else if !playing {
            self.frame.take();
            self.surface.take();
            // Softbuffer removes its observers, but its sublayer remains attached.
            #[cfg(target_os = "macos")]
            if let Some(layer) = self.game.0.layer() {
                unsafe { layer.setSublayers(None) };
            }
            #[cfg(target_os = "ios")]
            unsafe {
                self.game.0.layer().setSublayers(None)
            };
        }
        #[cfg(target_os = "ios")]
        self.game_height.setActive(playing);
        self.game.0.setHidden(!playing);
        self.resize()
    }

    pub(super) fn present(&self, frame: Frame) -> Result<()> {
        self.frame.replace(Some(frame));
        self.draw()
    }

    pub(super) fn resize(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let size = self.parent.bounds().size;
            let game_height = if self.playing.get() {
                size.height * GAME_HEIGHT / (GAME_HEIGHT + CONTROLS_HEIGHT)
            } else {
                0.0
            };
            let controls_height = size.height - game_height;
            let (game_y, controls_y) = if self.parent.isFlipped() {
                (0.0, game_height)
            } else {
                (controls_height, 0.0)
            };
            self.game
                .0
                .setFrame(NSRect::new(NSPoint::new(0.0, game_y), NSSize::new(size.width, game_height)));
            self.webview
                .setFrame(NSRect::new(NSPoint::new(0.0, controls_y), NSSize::new(size.width, controls_height)));
        }
        #[cfg(target_os = "ios")]
        {
            self.parent.layoutIfNeeded();
            self.game.0.setContentScaleFactor(unsafe { self.game.0.traitCollection().displayScale() });
        }
        self.draw()
    }

    fn draw(&self) -> Result<()> {
        let mut surface = self.surface.borrow_mut();
        let Some(surface) = surface.as_mut() else { return Ok(()) };
        let size = self.game.0.bounds().size;
        #[cfg(target_os = "macos")]
        let scale = self.game.0.window().map_or(1.0, |window| window.backingScaleFactor());
        #[cfg(target_os = "ios")]
        let scale = self.game.0.contentScaleFactor();
        let (Some(width), Some(height)) = (
            NonZeroU32::new((size.width * scale).round() as u32),
            NonZeroU32::new((size.height * scale).round() as u32),
        ) else {
            return Ok(());
        };
        surface.resize(width, height).map_err(|error| anyhow!(error.to_string()))?;
        let mut buffer = surface.buffer_mut().map_err(|error| anyhow!(error.to_string()))?;
        if let Some(frame) = self.frame.borrow().as_ref() {
            frame.draw_into(&mut buffer, width.get(), height.get());
        } else {
            buffer.fill(0);
        }
        buffer.present().map_err(|error| anyhow!(error.to_string()))
    }

    pub(super) fn close(&self) {
        self.surface.take();
        self.frame.take();
        self.game.0.removeFromSuperview();
        #[cfg(target_os = "ios")]
        {
            self.game_height.setActive(false);
            self.stack.removeFromSuperview();
        }
    }
}
