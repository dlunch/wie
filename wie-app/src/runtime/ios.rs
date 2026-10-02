use std::ptr::NonNull;

use anyhow::{Context, Result};
use block2::RcBlock;
use objc2::{
    MainThreadMarker,
    rc::Retained,
    runtime::{NSObjectProtocol, ProtocolObject},
};
use objc2_foundation::{NSNotification, NSNotificationCenter};
use objc2_ui_kit::{UIApplication, UIApplicationDidBecomeActiveNotification, UIApplicationState, UIApplicationWillResignActiveNotification};
use tauri::{AppHandle, Manager, Resource};

use super::AppState;

struct Observers {
    center: Retained<NSNotificationCenter>,
    tokens: [Retained<ProtocolObject<dyn NSObjectProtocol>>; 2],
}

// Tokens are opaque and only passed to the thread-safe notification center for removal.
// Their callbacks capture a Send + Sync AppHandle and use AppState's runtime mutex.
unsafe impl Send for Observers {}
unsafe impl Sync for Observers {}

impl Resource for Observers {}

impl Drop for Observers {
    fn drop(&mut self) {
        for token in &self.tokens {
            unsafe { self.center.removeObserver(token.as_ref()) };
        }
    }
}

/// Register on the main thread after AppState has been installed.
pub(crate) fn register(app: &AppHandle) -> Result<()> {
    let main = MainThreadMarker::new().context("iOS lifecycle registration requires the main thread")?;
    let center = NSNotificationCenter::defaultCenter();
    let notifications = unsafe {
        [
            (UIApplicationWillResignActiveNotification, true),
            (UIApplicationDidBecomeActiveNotification, false),
        ]
    };
    let tokens = notifications.map(|(name, suspended)| {
        let app = app.clone();
        let callback = RcBlock::new(move |_: NonNull<NSNotification>| {
            app.state::<AppState>().suspend(suspended);
        });
        // A nil queue delivers synchronously on UIKit's posting thread.
        unsafe { center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &callback) }
    });
    // Tauri explicitly clears resources on exit, releasing the blocks' AppHandles too.
    app.resources_table().add(Observers { center, tokens });
    app.state::<AppState>()
        .suspend(UIApplication::sharedApplication(main).applicationState() != UIApplicationState::Active);
    Ok(())
}
