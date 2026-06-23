# feat(lgt): native-backed JVM foundation for AOT-compiled LGT/ez-i Java apps

> **TL;DR** — Adds a `wie_lgt` foundation for **AOT-compiled LGT/ez-i Java apps** (each
> class emitted as native ARM + a `.data` descriptor, not JVM bytecode). It **boots
> through wie's shared lcdui path and reaches a self-sustaining render loop** (the card
> paints every frame, ~45 fps, real MIDP Graphics — background). The boundary is precise:
> the title's sprites/text are blocked at one spot — a requested resource's data slot
> (`field[0x78]`) never fills, inside the app's obfuscated internal data subsystem. Shared
> `wie_midp` / `wie_wipi_java` classes are **not modified** (#1232) — all PoC code is in
> `LgtJvmShared`.
>
> **Why this might be useful** — The hard part was the ez-i **per-frame render driver**,
> which earlier looked like an undocumented displayable/clet ABI. RE showed it is *not* an
> unknown ABI: it was a no-op'd `0x57` (show-card) import plus the card's own lifecycle,
> now driven from `LgtJvmShared`. As a sanity check that this is genuine and not a hack,
> the `o.g` render gate is set by the **app's own scene setup**, not forced.
>
> **What I'd like feedback on** — (a) whether a `wie_lgt` foundation in roughly this shape
> is something you'd consider taking, and what you'd want changed; (b) the branch is 73
> small RE checkpoints, so **"Squash and merge" is probably best**. I may well have missed
> existing context or conventions here — happy to adjust.

## Summary

Adds a foundation in `wie_lgt` for running **AOT-compiled LGT Java apps** (ez-i / Xceed
toolchain), where each Java class is emitted as native ARM code with `.data` metadata
rather than JVM bytecode. The app boots through wie's existing lcdui `Main.main` path
and runs its real native methods, dispatched through reconstructed platform tables.

Reverse-engineered against one ez-i reference app. It reaches **boot + game setup +
a self-sustaining render loop**: class registration, platform dispatch, data load,
240×320 back-buffer, `getGraphics`, Cards/RNG/Thread, and — after implementing the
show-card import and driving the card lifecycle — the card's `o.paint` runs **every
frame (~45 fps, continuous)** and draws to the back-buffer (`fillRect`/`setColor`,
background). The **central open question of the earlier draft — the ez-i per-frame
render driver (§7) — is resolved**: it was not an undocumented ABI but a no-op'd
show-card import plus the card's own lifecycle, now driven from `LgtJvmShared`. The
**full title (logo/sprites/text)** is scoped as future work, blocked on the app's
obfuscated resource/data subsystem (see *Deferred*).

The PoC keeps everything LGT-specific in `LgtJvmShared` (per #1232); shared
`wie_midp` / `wie_wipi_java` classes are **not modified**.

## What's added

- **Native class descriptor parser** (`runtime/java/native_class.rs`, read-only):
  decodes the ez-i class/method/field records in `.data` (handle indirection, 28-byte
  method records, 20-byte field records). Full byte layout in
  `docs/lgt_native_classes.md`.
- **Boot path** (`runtime/init.rs`, `runtime/java/interface.rs`): the `0x64`
  java-interface module — register main metadata (`0x03`), app classes (`0x07`),
  imported platform classes + offset resolution (`0x14`), and invoke-static
  `Main.main` (`0x83`) into the shared lcdui boot.
- **Two-level virtual dispatch** (`runtime/java/native_jvm.rs`): global vtable with a
  **reserved slot 0** (`vtable[virtual_method_offsets[ref] + 1]`), per-class override
  vtables for `java/lang` classes the AOT calls by hardcoded slot
  (Runtime/StringBuffer/Thread), and **inheritance-aware instance field layout**.
- **Object model**: native `new` primitive (stdlib `0x32` / java `0xf`) +
  `<init>`-trampoline binding to JVM instances, `getInstance` singletons
  (java-interface `0xc`), and the native String factory (`0x9`).
- **Render driver** (`LgtJvmShared`): java-interface `0x57` (show-card) binds + pushes
  the title card to wie's Display; `drive_card_step` runs the card lifecycle (scene-enter
  `i.a`, per-frame step `i.aE`) before each `o.paint` and schedules `repaint()` so the
  frame loop self-sustains (~45 fps) — the resolution of the §7 per-frame-driver question.
- **Lazy init + slot fixes**: lazy class/instance init (java-interface `0xb`/`0xd` —
  run an instance/class initialiser on first use, removing a 3665×/run no-op spin);
  `StringBuffer.append(int)` per-class override (a hardcoded scene-setup vtable slot).
- **Unit tests** (`cargo test -p wie_lgt`, 5 tests): the descriptor parser against a
  hand-encoded fixture (header offsets, 28/20-byte record strides, in-`.text`
  code-pointer invariant, handle indirection); the reserved-slot-0 vtable model
  (install slot == dispatch slot for every ref, no slot-0 use, no collisions) and the
  per-class override slots; and the inheritance-aware field layout on a known
  hierarchy. Two pure helpers (`physical_vtable_slot`, `compute_field_layouts`) were
  extracted to make the core invariants testable without a live app.
- **Docs**: `docs/lgt_abi.md` (consolidated, reverse-engineered ABI) and
  `docs/lgt_native_classes.md` (descriptor byte layout).

## Architecture (one paragraph)

Native methods run as real ARM under `wie_core_arm`. They reach the platform via
import thunks (`SVC_CATEGORY_JAVA_INTERFACE`) and per-method trampolines
(`SVC_CATEGORY_JAVA_TRAMPOLINE`) that bridge into wie's JVM. wie reconstructs the
dispatch tables the AOT code reads — a global vtable with reserved slot 0, per-class
overrides for hardcoded `java/lang` slots, and an inherited-first instance field
layout — so virtual/static calls and field access route correctly. Objects are bare
guest blocks (vtable word + 256-word field array) bound to JVM instances at `<init>`.

## Behavior

- ✅ App classes registered; native methods execute as real ARM.
- ✅ Boot: `0x64` java-interface → `Main.main` → `Game` Jlet → `CardCanvas` →
  `Display.setCurrent` (wie's MIDP paint loop ticks).
- ✅ Platform dispatch (two-level vtable, per-class overrides, instance fields),
  `getInstance` singletons, `Thread.start`, game thread spawns `a.run`.
- ✅ Game setup: data load → 240×320 back-buffer → `getGraphics` → Cards/RNG.
- ✅ **Render: the card draws every frame, self-sustaining.** java-interface `0x57`
  (show-card) + the card lifecycle (scene-enter `i.a` once, per-frame step `i.aE`) sets
  the `o.g` render gate, and `drive_card_step` schedules `repaint()` each tick so
  `o.paint` runs **continuously (~45 fps)** through wie's MIDP loop →
  **`fillRect` / `setColor` to the back-buffer** (background). `o.g` is set by the app's
  **own** scene setup (not a force).
- ✅ `StringBuffer.append(int)` per-class override (a hardcoded vtable slot the scene
  setup uses); lazy class/instance init (`0xb`/`0xd`) — both real no-op'd subsystems.
- ◑ Full title (logo / sprites / text) — scoped future work; blocked on the app's
  obfuscated resource/data subsystem (see *Deferred*).

## Deferred (out of scope for this PR)

**ez-i per-frame render driver — RESOLVED (cp38–44, `docs/lgt_abi.md` §7).** The earlier
draft's single open question ("which entry does ez-i invoke per frame to paint?") turned
out **not** to be an undocumented displayable/clet ABI but a no-op'd import plus the
card's own lifecycle: `a.run` hands the platform the title card via java-interface `0x57`
(show-card / `Display.setCurrent`), which wie left as a no-op, so the card was never
pushed and `o.paint` never ran. `LgtJvmShared::show_card` now binds + `pushCard`s the
card; `drive_card_step` runs the genuine card lifecycle (`i.a` enter, `i.aE` step) and
schedules `repaint()` so the loop self-sustains. The back-buffer flushes through wie's
existing MIDP path — **no shared-class changes** (#1232). This is the landmark the
foundation set out to find.

**Remaining for the full title — the app's resource/data subsystem (precise wall).**
A per-frame probe of the scene singleton (cp52) pinned exactly where it stops: the data
load **is requested** — `getInstance(b).field[0x74] = 8` (resource id) — and the game
**polls `field[0x78]` (the data slot) every frame for completion**, but it **never fills**
(0 over 293 frames), so the scene-machine state (`field[0x54]`) never advances, the
scene-object array (`field[0xd4]`) stays empty, and no `createImage`/`drawImage` is ever
reached. The completion (`field[0x78]` fill) is **not** a single drivable hook: every
`field[0x78]` writer only *clears* it (request markers); the actual *fill* with bytes is
the app's obfuscated resource subsystem (`o.g(id)`→`i.b(id)`→`0x706c`, traced to its leaf
in `docs/lgt_abi.md` §7, cp42/45/49/50/52) which uses **no** standard `File`/`Image`/
stream API and exposes **no single measurable `read(id)→bytes` contract**. The
class/instance lazy-init tier (`0xb`/`0xd`) was measured and implemented (cp51) but proven
**not** to be that data source. Unblocking the full title is a large, self-contained
RE effort (the id→data mapping + in-memory layout of that subsystem). It is an **internal**
mechanism — **not** an external input/time dependency (cp37: `field[0x78]` is polled
internally) — so it is implementable, just sizeable; the precise unknowns are in §7/§8.

## Verification

CI gates (`.github/workflows/rust.yml`), all green locally:

- `cargo fmt --all -- --check` — **clean**.
- `cargo clippy --all -- -D warnings` — **clean** (workspace, warnings denied).
- `cargo test -p wie_lgt` — **5 passed** (descriptor parser, vtable model, field layout).
- `cargo test -p wie_ktf test_helloworld` (clet regression) — **pass**.
- `cargo build --workspace` — **builds**.

Scope / hygiene:

- Diff touches **only** `wie_lgt/` + `docs/` (14 files, +3539/−39); no ROMs / binaries /
  logs / `.DS_Store` / probe code (`.gitignore` covers `*.jar`/`*.mod`/`*.log`/etc.).
- Shared `wie_midp` / `wie_wipi_java` classes are **not modified** (#1232); the Java-app
  PoC is entirely in `LgtJvmShared`.
- `wasm32-unknown-unknown` clippy (a CI target) not run locally; the `wie_lgt` additions
  are pure `no_std` (`core`/`alloc` + workspace crates, no new external deps), so they are
  wasm-safe and CI will confirm.
