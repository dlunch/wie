Hi! 👋 This PR adds a `wie_lgt` foundation for **AOT-compiled LGT/ez-i Java apps** — the
ones where each Java class is emitted as native ARM with a `.data` descriptor instead of
JVM bytecode. It boots through wie's existing shared lcdui `Main.main` path and runs the
app's real native methods, dispatched through reconstructed platform tables (a global
vtable with a reserved slot 0, per-class overrides, inheritance-aware instance fields).

The part I want to highlight: the **ez-i per-frame render driver**. Early on it looked
like an undocumented displayable/clet ABI and seemed to be the wall. It turned out *not*
to be an unknown ABI — it was a no-op'd `0x57` (show-card) import plus the card's own
lifecycle, now driven from `LgtJvmShared`. As a check that this is genuine rather than a
hack, the `o.g` render gate is set by the **app's own scene setup**, not forced — and
`o.paint` then draws every frame (~45 fps) through wie's MIDP loop.

The boundary is precise and documented: the title's sprites/text are blocked because the
app's internal scene-state machine (`i.a()V` @0x6fac4, a `switch` on `field[0x74]`) sits on
a state value it doesn't handle, so the scene never advances to populate its objects. It's
an internal mechanism, not an external input/time dependency. Details and the full RE trail
are in `docs/lgt_abi.md` §7/§8.

(Correction for transparency: an earlier version of this description called the blocker a
resource "data slot" `field[0x78]` that never fills. On closer tracing that field is
actually a counter/state field, and the real blocker is the unhandled state in the
scene-state `switch` above — I've corrected the PR description and ABI doc accordingly.)

Everything is LGT-specific in `LgtJvmShared`; shared `wie_midp` / `wie_wipi_java` classes
are untouched. The branch is a long series of small RE checkpoints, so **"Squash and
merge" is probably the cleanest** if you take it.

I'd really value your read on (a) whether a `wie_lgt` foundation in roughly this shape is
something you'd consider, and (b) anything you'd want done differently — I may well have
missed existing context or conventions, and I'm happy to adjust.

Verification (CI gates, all green locally):
- `cargo fmt --all -- --check`
- `cargo clippy --all -- -D warnings`
- `cargo test -p wie_lgt` (5) and `cargo test -p wie_ktf test_helloworld` (clet regression)
- `cargo build --workspace`

Thanks for taking a look!
