//! Card render lifecycle on `LgtJvmShared`: show-card (`0x57`), card-class
//! derivation, lifecycle-pointer resolution, and the per-frame drive. See
//! `docs/lgt_abi.md` §7.

use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};

use jvm::{ClassDefinition, ClassInstance, JavaError, JavaValue};
use spin::Mutex;

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmSupport;
use wie_util::Result;

use super::class_model::{LgtClassDefinition, LgtClassInstance};
use super::registration::deepest_card_subclass;
use super::shared::LgtJvmShared;

impl LgtJvmShared {
    /// Derive the app class of the shown card from the registered class graph rather
    /// than assuming an obfuscated symbol name. The shown object is already known to be
    /// a `Card` (bound to the platform `org/kwis/msp/lcdui/Card` base by its `<init>`);
    /// among the app's classes, the title card is the unique deepest subclass of that
    /// base (reference app: `Card <- o <- b <- i`, so `i`). Returns `None` if no app
    /// class extends `Card`, or if the deepest depth is not unique — `show_card` then
    /// leaves the object as the platform `Card` (graceful fallback, no forcing).
    ///
    /// Identifying *which* obfuscated app class is the title card is per-app reverse
    /// engineering — a documented PoC boundary. The graph heuristic is derived from app
    /// metadata and verifiable (it must resolve to the title card and render), not a
    /// hardcoded symbol. See `docs/lgt_abi.md` §7.
    fn resolve_card_class(&self) -> Option<String> {
        deepest_card_subclass(&self.app_class_graph.lock())
    }

    /// LGT java-interface import `0x57` (show-card): the app `new`'d a card and hands
    /// it to the platform to display (observed `0x57(jlet, card_guest, jlet)` in
    /// `a.run` — cp39 trace). The guest block was bound to the platform `Card` base by
    /// its `<init>` trampoline: only the `super Card.<init>` runs through wie, so the
    /// app's most-derived class isn't visible at bind time and `paint` would resolve to
    /// the empty platform `Card.paint`. Rebind the guest block to the app's title-card
    /// class (resolved from the shown object's class hierarchy — see `resolve_card_class`)
    /// as an [`LgtClassInstance`] reusing the SAME guest pointer, so `paint` dispatches
    /// through the app card chain to the native `o.paint` (the real draw @0xd8d70), then
    /// push it to wie's `Display` so the MIDP paint loop ticks it each frame. Left no-op
    /// before cp39, which is why `o.paint` never ran. See `docs/lgt_abi.md` §7.
    pub async fn show_card(&self, card_guest: u32) -> Result<()> {
        if card_guest == 0 {
            return Ok(());
        }
        // `0x57` is overloaded: the Game-flow call passes a real card guest object
        // (heap-allocated, already bound by its `<init>`), but `a.run` also calls
        // `0x57(jlet, <carried code ptr>, jlet)` where arg1 is a `.text` code pointer,
        // not a card. Only act on a genuine bound object (present in `instances`) so the
        // carried-code call is ignored.
        if !self.instances.lock().contains_key(&card_guest) {
            tracing::trace!("LGT show_card: {card_guest:#x} is not a bound object (carried-code 0x57?); ignoring");
            return Ok(());
        }
        // Derive WHICH app card class the shown object is from the registered class
        // graph — the title-card symbol is app-scoped; resolve it from the shown
        // object's class hierarchy, do not assume the name. See `resolve_card_class`.
        let card_class = match self.resolve_card_class() {
            Some(c) => c,
            None => {
                tracing::warn!("LGT show_card: could not derive app card class; card {card_guest:#x} left as platform Card");
                return Ok(());
            }
        };
        let jvm = self.jvm.clone();

        let class = match jvm.resolve_class(&card_class).await {
            Ok(c) => c,
            Err(_) => {
                tracing::warn!("LGT show_card: app card class {card_class:?} not registered; card {card_guest:#x} left as platform Card");
                return Ok(());
            }
        };
        let definition = match class.definition.as_any().downcast_ref::<LgtClassDefinition>() {
            Some(def) => def.clone(),
            None => {
                tracing::warn!("LGT show_card: {card_class:?} is not an app class; card {card_guest:#x} left as-is");
                return Ok(());
            }
        };
        // Resolve the card's lifecycle method addresses from its descriptor (not
        // hardcoded) for the per-frame driver. See `resolve_card_lifecycle`.
        self.resolve_card_lifecycle(&definition);

        let card: Box<dyn ClassInstance> = Box::new(LgtClassInstance {
            guest_ptr: card_guest,
            definition,
            jvm_fields: Arc::new(Mutex::new(BTreeMap::new())),
        });
        self.instances.lock().insert(card_guest, card.clone());
        tracing::debug!("LGT show_card: rebound card {card_guest:#x} -> {card_class:?}; pushing to Display");

        let display_val: JavaValue = match jvm
            .invoke_static(
                "org/kwis/msp/lcdui/Display",
                "getDefaultDisplay",
                "()Lorg/kwis/msp/lcdui/Display;",
                Vec::<JavaValue>::new(),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        let display = match display_val {
            JavaValue::Object(Some(d)) => d,
            _ => {
                tracing::warn!("LGT show_card: getDefaultDisplay returned null");
                return Ok(());
            }
        };
        let args = alloc::vec![JavaValue::Object(Some(card))];
        let _: JavaValue = match jvm.invoke_virtual(&display, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", args).await {
            Ok(v) => v,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        // The scene-enter is driven from the first paint tick (a clean dispatch
        // boundary), not here: `show_card` runs mid-`a.run` and driving native code on
        // the same core would clobber that in-flight context. See `drive_card_step`.
        *self.card_entered.lock() = false;
        Ok(())
    }

    /// Resolve the shown card's lifecycle method addresses from its descriptor and
    /// stash them for `drive_card_step`. The scene-enter `a(I)V` and per-frame step
    /// `aE()V` are looked up by name+signature in the card class's parsed method table,
    /// so the addresses come from the app's own metadata rather than literals. The
    /// method *names* are this reference app's obfuscated symbols (app-scoped, like the
    /// card class itself). The reference-app addresses are kept only to flag drift
    /// (`tracing::error!`, non-fatal — no panic) if a descriptor ever resolves something
    /// unexpected. See `docs/lgt_abi.md` §7.
    fn resolve_card_lifecycle(&self, def: &LgtClassDefinition) {
        const SCENE_ENTER: (&str, &str) = ("a", "(I)V");
        const STEP: (&str, &str) = ("aE", "()V");
        const REF_ENTER_PTR: u32 = 0x1d4ac;
        const REF_STEP_PTR: u32 = 0x72f2c;

        let enter = def.method_code_ptr(SCENE_ENTER.0, SCENE_ENTER.1);
        let step = def.method_code_ptr(STEP.0, STEP.1);
        match enter {
            Some(p) if p != REF_ENTER_PTR => {
                tracing::error!("LGT show_card: scene-enter resolved to {p:#x}, expected reference {REF_ENTER_PTR:#x}")
            }
            Some(p) => tracing::debug!("LGT show_card: scene-enter {}.a(I)V resolved to {p:#x}", def.name()),
            None => tracing::warn!("LGT show_card: card class {} declares no scene-enter a(I)V", def.name()),
        }
        match step {
            Some(p) if p != REF_STEP_PTR => tracing::error!("LGT show_card: step resolved to {p:#x}, expected reference {REF_STEP_PTR:#x}"),
            Some(p) => tracing::debug!("LGT show_card: step {}.aE()V resolved to {p:#x}", def.name()),
            None => tracing::warn!("LGT show_card: card class {} declares no step aE()V", def.name()),
        }
        *self.card_enter_ptr.lock() = enter;
        *self.card_step_ptr.lock() = step;
    }

    /// Per-frame card step (cp39): run the shown card's step method `aE()V`
    /// (@0x72f2c) on the card `this`. ez-i ticks the current card each frame — advance
    /// the scene state machine, then paint — but wie only drives `paint`, so we run the
    /// step here, immediately before the card's `o.paint` (see [`LgtMethod::run`]).
    /// `i.aE` is the per-frame state machine that reaches the `o.g` render-gate setter
    /// (one of the four `0xdb200` callers — cp38); `o.paint` early-returns forever while
    /// `o.g==0`, so without this tick the card never renders. Errors are logged and
    /// swallowed so a step fault doesn't abort the frame. See `docs/lgt_abi.md` §7.
    pub async fn drive_card_step(&self, core: &mut ArmCore, card_this: u32) {
        if card_this == 0 {
            return;
        }
        const INITIAL_SCENE: u32 = 0; // scene index, app-scoped
        let enter_ptr = *self.card_enter_ptr.lock();
        let step_ptr = *self.card_step_ptr.lock();

        // First tick: enter the initial scene. The scene-enter's prologue sets the `o.g`
        // render gate (via the @0xdb200 setter) and runs the scene setup that
        // initialises the state the per-frame step advances; without it the step
        // early-returns on uninitialised state and `o.paint` stays gated (cp38).
        let need_enter = {
            let mut entered = self.card_entered.lock();
            if *entered {
                false
            } else {
                *entered = true;
                true
            }
        };
        if need_enter {
            match enter_ptr {
                Some(enter_ptr) => {
                    let r: Result<u32> = core.run_function(enter_ptr, &[card_this, INITIAL_SCENE]).await;
                    match r {
                        Ok(_) => tracing::debug!("LGT drive_card_step: entered scene via a@{enter_ptr:#x}({card_this:#x}, {INITIAL_SCENE})"),
                        Err(e) => tracing::warn!("LGT scene-enter a@{enter_ptr:#x} this={card_this:#x} failed: {e}"),
                    }
                }
                None => tracing::warn!("LGT drive_card_step: no resolved scene-enter pointer; skipping enter"),
            }
        }

        match step_ptr {
            Some(step_ptr) => {
                let r: Result<u32> = core.run_function(step_ptr, &[card_this]).await;
                if let Err(e) = r {
                    tracing::warn!("LGT card step aE@{step_ptr:#x} this={card_this:#x} failed: {e}");
                }
            }
            None => tracing::warn!("LGT drive_card_step: no resolved step pointer; skipping step"),
        }

        // cp44: sustain the frame loop. `a.run` is one-shot and the app's per-frame
        // repaint request (import `0xe2`) is not yet wired, so wie paints a few times
        // then idles. Schedule the next repaint so the card's step+paint run each frame
        // — a legitimate continuous render mirroring the platform's per-frame tick (not
        // a force; `repaint` only enqueues a paint event, which runs after this one).
        let card = self.instances.lock().get(&card_this).cloned();
        if let Some(card) = card {
            let _: core::result::Result<JavaValue, JavaError> = self.jvm.invoke_virtual(&card, "repaint", "()V", Vec::<JavaValue>::new()).await;
        }
    }
}
