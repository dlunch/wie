# LGT / ez-i Java-app ABI

Reverse-engineered ABI for running an **AOT-compiled LGT Java app** (ez-i / Xceed
toolchain) on wie's JVM. Each Java class is emitted as a native ARM record: methods
are real ARM code in `.text`, class/field/method metadata lives in `.data`. The app
talks to the platform through the "java-interface" import module (table `0x64`).

Scope: the PoC lives entirely in `wie_lgt` (`LgtJvmShared`, LGT-specific per #1232);
shared `wie_midp` / `wie_wipi_java` classes are **not** modified. This document is the
consolidated ABI; see `docs/lgt_native_classes.md` for the byte-level descriptor RE.
The `cpNN` tags below mark the reverse-engineering milestone each fact was pinned at.

Notation: addresses are from the ez-i reference app (`.text` `0x1000..0xe7800`,
`.data` `0x1400000..`, `.bss` `0x1500000..`). Anything not pinned by RE is labelled
**(추정)**.

---

## 1. Module / SVC layout

The native code reaches the platform two ways: **import thunks** (16-byte stubs that
trap into wie via SVC) and **trampolines** (per-method SVC stubs wie installs into the
vtables). wie routes everything through SVC categories (`wie_lgt/src/runtime.rs`):

| category | id | handler | role |
|---|---|---|---|
| `SVC_CATEGORY_INIT` | 1 | `init.rs` | boot imports (`java_unk0/5/9/11/12`, `load_classes`, `new`, `0x54`) |
| `SVC_CATEGORY_WIPIC` | 3 | `wipi_c.rs` | WIPI-C clet ABI (framebuffer/graphics) — used by the clet path |
| `SVC_CATEGORY_STDLIB` | 5 | `stdlib.rs` | libc-ish (`strcpy`, `memcpy`, `time`, …) + native `new` (`0x32`) |
| `SVC_CATEGORY_JAVA_TRAMPOLINE` | 7 | `native_jvm.rs` | per-method vtable/static stubs → JVM invoke |
| `SVC_CATEGORY_JAVA_INTERFACE` | 9 | `java/interface.rs` | java-interface imports routed by index (the SVC id **is** the import index) |

### Import thunk format (16 bytes, in `.text`)

```
str  lr, [sp, #-4]!     ; save return
bl   <dispatcher>       ; 0xe31a8 (resolves index, traps to platform)
.word <table>           ; table id (1 = stdlib, 0x64 = java-interface)
.word <index>           ; function index within the table
```

The dispatcher (`0xe31a8`) reads the index word that follows the `bl` and returns to
the caller via the saved `ip` after the platform call. wie resolves a thunk on first
use via `get_import_table(0x64)` / `get_import_function(0x64, <index>)`
(`init.rs`), which hands back the SVC stub for that index. Example: the thunk at
`0xe2c50` has `.word 1, .word 0x32` ⇒ the stdlib **`new`** primitive — so an app call
that `bx`-es to `0xe2c50` is allocating an object.

---

## 2. Native class descriptor format

(Full RE in `docs/lgt_native_classes.md`; mirrored by `native_class.rs`. 283/283
method code pointers validated inside `.text`.)

```
Class header (at H):
  +0x00  tag           (0x21 / 0x31 observed)
  +0x08  ptr_name      -> cstring (obfuscated single letters: "Game", "a", "o", "i", …)
  +0x10  ptr_parent    -> platform class-name cstring, OR another class header H'
  +0x18  access_flags  (java access bits | 0x20000 app marker)
  +0x38  ptr_methods   -> [count:u32, MethodRecord[count]]
  +0x3c  ptr_fields    -> [count:u32, FieldRecord[count]]

Class handle (at H + 0x4c): { 0, 0, H }    <- every member's ptr_class points here,
                                              i.e. handle indirection: record -> handle -> header

MethodRecord (28 bytes):
  +0x00 ptr_class(=handle)  +0x04 ptr_name   +0x08 ptr_signature
  +0x0c access_flags        +0x10 num_locals +0x14 code_ptr(-> .text)   +0x18 unk

FieldRecord (20 bytes):
  +0x00 ptr_class(=handle)  +0x04 ptr_name   +0x08 ptr_type
  +0x0c access_flags        +0x10 index      (declared index within the class)
```

**Class handle indirection**: members carry `ptr_class = H + 0x4c` (the handle), not
`H`. The handle's `+0x08` points back to `H`. `getInstance` and singleton lookups take
a handle and resolve `handle → header → name` (`parse_native_class_from_handle`).

### App class graph (reference app, from `.data` scan)

```
Jlet (platform)
  └─ a            (a.run @0x1f10, a.startApp @0x1ad8, a.b(Lo;I)V @0x2200)
       └─ Game    (Game.a @0x11dc data-load, Game.b @0x1484)
org/kwis/msp/lcdui/Card (platform)
  └─ o
       ├─ d, e, j, l
       └─ b └─ i
```
Class `a`'s descriptor handle = `0x1400df4` (header `0x1400da8`).

---

## 3. Boot sequence

`get_java_interface_method` (`interface.rs`) maps the boot imports; the app drives them
in order during startup:

| import | wie fn | meaning |
|---|---|---|
| `0x03` | `java_unk0` | register main-class metadata `(name="Game", _, flag="true")` |
| `0x07` | `java_unk5` | register the app's OWN native classes (handle array @ a0) |
| `0x06` | `java_unk12` | paired with `0x07` (same struct ptr) — role unconfirmed (추정) |
| `0x14` | `java_load_classes` | declare IMPORTED platform classes + resolve dispatch offsets (see §4) |
| `0x82` | `java_unk9` | boot hook, arg always 0 (추정: lifecycle marker) |
| `0x83` | `java_unk11` | invoke-static `org/kwis/msp/lcdui/Main.main(argv)`, `argv[0]="Game"` |
| `0x0f` | `JavaNewObject` | native object allocator (`obj = 0xf(...); obj.<init>()`) |
| `0x54` | `java_interface_unk84` | per-method entry helper (stack/safepoint check) — no-op (추정) |

`0x83` boots the app's Jlet through the **shared lcdui Main path**
(`invoke_lcdui_main(jvm, "Game")`), identical to the WIPI-C clet boot
(`net/wie/CletWrapper`). That wraps `Game` in `WIPIMIDlet`, creates `net.wie.CardCanvas`,
and `Display.setCurrent(CardCanvas)` — so wie's MIDP paint loop begins ticking (empty)
from here.

---

## 4. Global virtual dispatch (two-level vtable, reserved slot 0)

`java_load_classes` (`0x14`) hands the platform parallel arrays describing imported
classes and the method/field refs they use. `install_platform_tables` builds the
dispatch tables the AOT code reads.

**AOT virtual call shape:**
```
idx = virtual_method_offsets[ref]      ; u16 logical index
bx   vtable[idx + 1]                   ; ldr ip,[r3, #4] after add r3,r3,idx<<2
```
The `+1` (`ldr ip,[r3,#4]`) means **physical slot 0 is reserved** and methods start at
slot 1. wie therefore:

- allocates `global_vtable` of `VTABLE_REFS+1 = 129` words (`VTABLE_REFS = 128`),
- for each method-ref `r` with a real `(...)`-signature: installs a trampoline at
  **physical slot `r+1`** and sets `virtual_method_offsets[r] = r` (the logical index),
- so `vtable[offset[r] + 1] = vtable[r + 1]` dispatches ref `r` correctly.

Verified by RE: pre-fix, `AnnunciatorComponent.show@ref6` misrouted to `vtable[7]` =
`File.read`; the reserved-slot-0 fix corrected it (cp15). Every object's `+0x00` word
points at `global_vtable` (app objects and platform proxies alike).

### Per-class override vtables

A few `java/lang` classes are dispatched by the AOT at **hardcoded physical slots**
that collide with the global identity table. For those, wie copies the global vtable
and overrides the known slots (`known_java_lang_vtable`), storing the result in
`class_vtables[name]`; `bind_pending` repoints the object's `+0x00` to it once the
class is known. Indices are **physical** (reserved slot already baked in):

| class | physical slot → method |
|---|---|
| `java/lang/Runtime` | 13 → `freeMemory()J`, 14 → `gc()V` |
| `java/lang/StringBuffer` | 5 → `toString()`, 19 → `append(Ljava/lang/String;)Ljava/lang/StringBuffer;` |
| `java/lang/Thread` | 11 → `start()V` |
| `java/lang/String` | 35 → `toCharArray()[C` (cp30) |

(These slots are empirically identified — **추정** where not cross-checked against a
second call site. Runtime/StringBuffer/Thread are all confirmed by a
working call.)

### Static methods / fields

Per imported class, static methods get **direct** trampoline pointers written into
`static_method_offsets[smo+j]`; static-field slots get an identity fill into
`field_offsets[sfo+j]`.

---

## 5. Instance object model + field layout

```
guest object block (alloc_native_object / native `new`):
  +0x00  vtable word      -> global_vtable (or a per-class override after <init>)
  +0x04  0
  +0x08  ptr_fields       -> field array (256 words, zeroed)
```

`OBJ_HEADER_SIZE = 0x0c`, `OBJ_PTR_FIELDS_OFFSET = 0x08`, `FIELD_ARRAY_WORDS = 256`.

**Instance field addressing** (AOT): `obj.field[ field_offsets[K] ]`. Left all-zero,
every field-ref aliases slot 0 (this broke `a.startApp`'s "is Display set?" gate —
cp16/cp17). wie computes an **inheritance-aware, inherited-first flat layout**:

> object slot = (count of fields in all app-ancestor classes) + declared field index

Platform ancestors contribute 0 fields to the guest layout (their state lives on the
JVM side). `register_app_classes` computes each class's `(name, type, slot)` list into
`app_field_layouts`; `install_platform_tables` then segments the flat `fields` ref
array by matching each window to a class's exact field set and writes the resolved slot
into `field_offsets[k]`. Field-record parse offsets: name `+0x04`, type `+0x08`,
declared index `+0x10` (all 150 reference-app fields matched after fixing an initial
`+0x00` vs `+0x04` off-by-4).

### Object binding lifecycle

- `new` (stdlib `0x32` / java `0xf`) → `alloc_native_object`: allocates the block with
  the **global** vtable word, inserts the ptr into `pending_new` (**unbound** — no JVM
  class yet).
- The `<init>` trampoline calls `bind_pending(ptr, class_name)`: removes it from
  `pending_new`, repoints `+0x00` to a per-class vtable if one exists, resolves the
  class, and creates the instance. App classes reuse the guest block as an
  `LgtClassInstance`; platform classes are instantiated by the JVM keyed on the ptr.
- `getInstance` (java-interface `0xc`) → `singleton_instance(class_handle)`: cached;
  resolves handle → name, instantiates a bound app instance, returns its guest ptr.
  Must be stable across calls/threads (per-class state like `a.run`'s run-flag at
  `obj+0x20` is shared through it — cp20).

> Roadmap — field unification: an `LgtClassInstance`'s JVM-side fields and the guest
> field array at `guest_ptr` are currently separate stores. They should be unified so a
> field written by ARM code and the same field read via the JVM agree, by mapping each
> JVM field through the `field_offsets` slot map onto the guest array. Not yet needed:
> for the current reach (boot + setup) no field is written on one side and read on the
> other. (cp27 confirmed this for the render path: `o.paint` reads, and `o.k` writes,
> the *same* guest field-array slot — both ARM-side — so the title-render wall is **not**
> a field-store split. See §7.)

---

## 6. java-interface import table (status)

The SVC id **is** the import index. Implemented imports get a real handler; everything
else is logged and returns 0. Three tiers:

### Implemented

| idx | name | semantics |
|---|---|---|
| `0x9` | String factory | `(ctx, utf16_ptr, count, out_slot)` → materialise `java/lang/String` from constant-pool UTF-16, return a guest ptr bound to it (cp10) |
| `0xc` | `getInstance(handle)` | canonical singleton instance for a class handle (cp20) |
| `0xf` | native `new` | allocate guest object; `<init>` binds it (cp8) |
| `0x54` | method-entry helper | resolved first in every native method with a small constant — stack/safepoint check (추정), no-op |
| boot: `0x03 0x06 0x07 0x14 0x82 0x83` | see §3 | |

### No-op-safe (evidence-identified primitives; documented no-ops)

These are called with shapes consistent with GC / safepoint / exception / sync
bookkeeping; leaving them as `→ 0` has caused no regression across full boot + setup.

| idx | observed args | likely role (추정) |
|---|---|---|
| `0x12` | `(0, 0, sp)` | stack-ptr only — safepoint/exception frame |
| `0x1f` | `(0, obj/type, count)` | type/count — GC root or array bookkeeping |
| `0x21` | `(obj, 0, sp)` **and** `(new_obj, …)` | register/track object (see §7 — driver path uses this) |
| `0x22` | `(0, n, …)` | small ints — counter/flag |
| `0xb`,`0xd` | `(handle, …)` pair | per-class bookkeeping invoked inside helper `0x1908` (36× each) |

> Note: `0x21` is listed here because its *common* uses are benign object tracking, but
> it is **also** on the render-driver registration path (§7) where a no-op is *not*
> sufficient. The distinction is the call site, not the index.

### Unresolved — render driver (needs ez-i ABI; see §7)

| idx | observed | blocker |
|---|---|---|
| `0x55` | `(a-singleton, code@0x1ad4, 0)` / `(0,4,8)` (overloaded) | registers app "carried code"; replaying it is inert (cp23) |
| `0x56` | `(this, code@0x1ad4, 0)` | registers app callback/code |
| `0x57` | `(this, …)` in `a.b(Lo;I)V` | called once at setup; a0 = app `this`, not a card (cp25) |
| `0x21` (driver use) | `(new_obj, …)` in `a.run` and `a.b` | hands an **unbound** native object to the platform |

---

## 7. Render model + the one open question

### What the WIPI render model says (public)

- `DisplayProxy.flush()` / `flush(x,y,w,h)` blits a double-buffered back-buffer to
  screen; `isDoubleBuffered()` exists.
- `org.kwis.msp.lcdui.EventDispatcher` (`DisplayProxy.evtDisp`) drives the tick.
- WIPI-C clet: the platform calls `paintClet(x,y,w,h)` per frame. Java equivalent =
  the displayable's per-frame paint + flush.
- wie already drives WIPI-C clets; `CardCanvas.paint` already loops every frame.

### What the reference app actually does (RE, cp21–cp25)

- The app does **not** use `pushCard`/`CardCanvas` normally. It runs full setup
  (data load → 240×320 back-buffer → `getGraphics` → Cards/RNG/Thread) and then **0
  draw calls** are issued.
- `a.run` (the game thread's `run`) is a **confirmed one-shot**: `getInstance(0xe)` →
  helper → check `obj.field[8]` → `0x55(obj)`, `0x56(this)`, `0x1f(0)`, then **returns**
  via the epilogue at `0x2140`. No frame loop inside it.
- `a.b(Lo;I)V` runs **once** at setup: `new(0x32)` → bare object `r4`; `0x57(a0=this)`;
  `0x21(a0=r4)`. The "helper" `0xe2c50` is itself the `new` import thunk.
- The object handed to `0x21` (the render driver / "card") comes straight from `new`:
  it has only the **global vtable word**, is in `pending_new`, and has **no `<init>`**
  before registration ⇒ **no JVM class, no descriptor, no per-class vtable**.

### Why this is blocked

P4 (static-type identification) has nothing to latch onto: the registered object is an
opaque ez-i-native handle with **no bound per-frame method to call**. The per-frame
invocation is the **ez-i runtime's** job — its event loop calls the registered native
object's paint entry each frame — and wie does not emulate that loop (replaying the
carried code `0x55/0x56` is inert, cp23). The app registers nothing into wie's MIDP
card system, so there is no safe wie-side connection point. Calling any native vtable
slot here would be a guess (forbidden).

**cp26 — the app `Card.paint` path is reachable but empty (experiment, reverted).**
The app's `Card` subclasses *are* known JVM classes: `o` (extends
`org/kwis/msp/lcdui/Card`) has a real `paint(Lorg/kwis/msp/lcdui/Graphics;)V`
(`@0xd8d70`, draws on its Graphics arg), inherited by `d`/`e`/`j`/`l`, and each is
created as a bound singleton via `getInstance` (`0xc`). A one-shot experiment pushed the
bound `o`-card into wie's `Display`/`CardCanvas`: **`o.paint` then ticked every frame on
the back-buffer** (the Card.paint path wires cleanly into wie's existing tick) — but
issued **zero draw calls**. `o.paint` ran without error and made no `Graphics`/draw
calls at all, i.e. it took its empty-state early-out. Root cause is the same wall: the
`getInstance` singletons are **empty shells** (zeroed fields); the live title-screen
state lives in the ez-i-native (unbound) objects, not in the JVM-bound cards. So
correctly ticking `Card.paint` still paints nothing. This **excludes the app
`Card.paint` instance path** and re-points at the same missing piece below.

**cp27 — render-field provenance, traced (diagnosis only).** cp26's "empty shell" was
an inference from `g==0`; cp27 traced the actual writer of the one field that gates
`o.paint`, to classify the wall as app-side (fixable) vs platform-side (maintainer).

- *Gate field S.* `o.paint` (`@0xd8d70`) early-outs on `bl 0xd8640` → `getInstance(o)`
  (`0xd85e4`, import idx `0xc`, confirmed) returning the **o class singleton**; the gate
  is that singleton's guest field-array slot 6 (`[obj+8][+0x18]`) = **`o.g : I`** (int,
  declared index 6; `o` extends the platform `Card`, so slot = declared index). The gate
  is the *class singleton*, not `this`.
- *Writers of `o.g`.* Found in the app ARM — both inside one method, **`o.k()V`
  (`@0xda7f8`)**: `0xda8a0` writes `g ← 0` (reset), `0xdb240` writes `g ← 1` (set, from a
  literal). Both reach the singleton via `bl 0xd8640` (`getInstance(o)`), i.e. they write
  **the same singleton and the same guest field-array store that `o.paint` reads**. No
  JVM-side writer exists (the AOT writes its own fields with ARM `str`, not a JVM
  putfield). Data source is a literal `0/1` (a game state flag — not a `.dat` load, not a
  native-import return).
- *Instance identity.* gate object = writer target = the instance cp26 pushed = the
  `getInstance(o)` singleton (`0x48840130`) — all the same. The unbound native objects
  carried by `0x21`/`0x57` are separate, but `o.paint` never reads them (it gates on the
  class singleton).
- *Why `o.g` stays 0.* `o.k()V` (the writer) is a **virtual method with no static call
  site** (no `bl 0xda7f8`, no pointer-word ref anywhere in `.text`) — it is only reachable
  via the two-level vtable dispatch, i.e. called by the game's state machine each
  frame/step. In the reachable run only `o.paint` is dispatched on `o`; `o.k` never runs,
  so `g` is never set.

**Classification — platform-side (§7), now traced, not inferred.** The wall is *not* a
field-store split (§5: writer and reader use the same guest array) and *not* an
instance mis-pick (cp26 pushed the very singleton `o.paint` gates on). The `o.g` writer
exists and targets the right store/instance; it just never runs, because the virtual
`o.k()V` that holds it is driven by the game state machine, which only advances under a
per-frame tick. That tick is the ez-i runtime's job — the same missing piece below.
So **field unification (§5) and "find the live instance" are *not* fixes** for the title
render; the one missing thing remains the ez-i per-frame drive.

**cp28 — force `o.g = 1`: the render path works end-to-end (experiment, reverted).**
A one-shot surgical probe pushed the `o`-singleton into wie's tick (cp26 wiring) and
wrote `1` into its gate slot (`[obj+8][+0x18]`, the `o.g` cp27 traced). Result:
`o.paint` **passed the early-out and drew on the back-buffer** — 39 real draw calls
(**21× `Graphics.setColor`, 18× `Graphics.fillRect`**) through wie's standard paint
event (`Display.handlePaintEvent → CardCanvas.handlePaintEvent(Graphics) → o.paint`),
which presents the buffer to the screen. This **proves `o.g` is the genuine render gate
and the whole render path (gate → `o.paint` → `Graphics` → back-buffer → flush) works**;
the only thing missing for those draws is *setting `o.g`* (i.e. running the virtual
`o.k()V`, i.e. the ez-i per-frame drive).
Caveat: `o.paint` did not run to completion — after the 39 draws it hit a *separate*
`NoSuchMethodError: java/lang/String.e()V` (diagnosed in cp29 below — **not** a second
empty-state field gate; draws did occur). The draws are background/box fills
(`fillRect`), not yet the title sprite/text (`drawImage`/`drawString`), consistent with
`o.paint` aborting partway. Experiment reverted (force-g=1 is a test hook, not a fix);
only the finding is recorded.

**cp29 — the `String.e()V` abort is an ez-i `java/lang/String` vtable slot (platform,
STOP).** Traced the cp28 caveat under the same harness. The receiver is a *genuine*
`java/lang/String` (`"LOADING..."`, just made by the String factory): a draw-text
wrapper `B(Graphics, String)` (`@0x100d8`) does `setColor` → `s.vtable[slot 35]()` →
draw. So the String is the intended argument, **not** a mis-bound app object. The crash
is that the app calls `java/lang/String`'s hardcoded **physical vtable slot 35**
(ref 34), but the app's import data declares `java/lang/String` with **`vmc=0`** (zero
imported virtual methods) — so wie has no per-class String vtable and the global slot 35
holds an unrelated *app* method (`e()V`), giving `String.e()` → `NoSuchMethodError`.
The correct slot-35 method is an entry of **ez-i's own `java/lang/String` vtable**, which
is **not present in the app binary** (`vmc=0`; ez-i provides it). This is the same shape
as the `java/lang/*` per-class vtables (Runtime/StringBuffer/Thread, cp4–6/cp10).
cp29 initially over-classified this as maintainer-gated from a single call site; cp30
corrected and resolved it by RE.

**cp30 — String physical slot 35 = `toCharArray()[C`, RE-confirmed (fix).** Fully
disassembling the draw-text wrapper `B(Graphics, String)` (`@0x100d8`) and its draw
helper (`@0x10228`): `B` calls `s.vtable[35]()` (no args) and the helper then iterates
the result `r` as a char array — `data = [r+8]; len = [data]; for i in 0..len { char =
[data + 4 + i*2] }` (a per-char glyph loop, bitmap font). The only no-arg, char-array-
returning String method is **`toCharArray()[C`**. Added as a per-class String override
(physical slot 35) in `known_java_lang_vtable`. Behaviour-confirmed under the force-g=1
harness: `String.e()V NoSuchMethodError` is **gone**, `String.toCharArray()[C` is now
dispatched at that call site, and `o.paint` runs **without fatal** (cp28 aborted there).
So the platform-side mis-classification was wrong; this was an ordinary `java/lang/*`
vtable slot, RE'd like the others.

**cp31 — char-array guest marshalling (fix), but the glyph loop is gated elsewhere.**
The glyph loop reads a `char[]` as `data = [arr+8]; len = [data]; char = [data+4+i*2]`
(u16 LE). wie marshalled the `toCharArray` result via `register_platform_object`, whose
proxy has `[ptr+8]=0` (empty). cp31 adds `materialize_char_array` (in
`handle_java_trampoline`'s result path, scoped to `[C`): it allocates a guest data
block `{u32 len, u16 chars[len]}` and points the object's `+0x08` at it — the exact
layout RE'd above. Behaviour-confirmed: the `[C` return is materialised, and the block
is correct (logged `char[] len=10 text="LOADING..."`; unit-tested via
`write_char_array_block`).

*(cp31's note that the glyph loop "never enters" was wrong — an `lr` mis-calc; cp32
shows it runs.)* *cp32 target (one line):* find why the glyph loop, once entered,
draws no glyphs.

**cp32 — char-array marshalling confirmed working; glyph render is the real gap.**
Re-traced with corrected `lr`s. The colour "gate" at `0x10298` is **not** a gate:
`g.vtable[ref21]()` = `Graphics.getColor()I` (id 21); both branches (`== 0x00ffffff`
and the `setColor(~c)` else-path) **fall through to `0x102c0`**, so the glyph loop is
always entered. And it **runs**: the loop body's `import 0x55` at `0x102d0` fires
`lr=0x102dc` **30×** (3 frames × 10 chars), reading `data=0x49048000`, `len=10`, chars
`0x4c='L'`… — i.e. `materialize_char_array` works end-to-end and the loop consumes
"LOADING...". (`register_platform_object` readback confirmed `[obj+8]=data`, `[data]=10`.)
What's missing is **glyph drawing**, traced in cp33 below.

**cp33 — the glyph-draw fn runs but has no font image (platform/font gap).** The
glyph-draw fn `@0x109b4` *is* called per char (each char's `0xb(a1=char, a2=x)` lookup
then the fn's body fire). It branches on its first arg `r6` (the font image): at
`0x10b1c cmp r6,#0; 0x10b2c bne 0x10b44`, `r6 != 0` → `r6.vtable[r2]()` (the blit /
drawImage), `r6 == 0` → `import 0x22` (a no-op fallback). Measured: every char takes the
**`r6 == 0`** path — `import 0x22` fires at `lr=0x10b40` once per char — so **no
drawImage is ever issued**. (The earlier `getColor`/clip branches are *not* the skip;
they all reach the glyph fn.) Root cause **(C): the font glyph image is absent** (`r6 ==
0`) — the bitmap-font sheet the AOT expects to blit each glyph from is not present
guest-side, so the draw falls back to a no-op. char data is correct ("LOADING..." is
read); the glyphs just have no font to draw from.

This is a font/image-marshalling task (its own checkpoint), not a one-liner.

**cp34 — glyph blit mechanism RE'd; contract not yet complete → cp35 plan.** The
glyph-draw fn `@0x109b4` has two paths, gated on its font-image arg `r6`:
- **`r6 != 0`** → `g.drawImage(font_sheet, …)` with **`src_x = (char - 0x21) * 10`,
  `width = 10`** (`0x10ac0`: `r3 = char - 0x21; r4 = r3 * 10`) — i.e. a fixed-pitch
  10px ASCII glyph grid blitted from a font sheet (`char 0x21='!'` is grid origin).
- **`r6 == 0`** (measured) → `import 0x22(a0=0, a1=0x11264, a2=x)`. `a1=0x11264` is a
  **.text thunk → `0x10fb0`** (an app native fn that itself calls `getInstance`/imports),
  i.e. `0x22` here is a **carried-code/callback shape** (a1 = a function pointer), and
  wie no-ops it → the native font render never runs. x advances +6 per char (14/20/26…).

So the per-char render is *not* a one-liner: with no font image (`r6==0`), the app falls
to a native font path (`0x22` → fn `0x10fb0`) that wie doesn't run. **Two unknowns block
a confident impl** (so no code this checkpoint, per the "no half-guess" rule):
1. *Why `r6==0`.* Statically `r6` traces to the `Graphics` arg (≠0), but it's measured
   as 0 at the blit — needs a guest-register read (the static trace can't resolve a
   mid-fn reassignment). Determines path (i) JVM-image vs (ii) native.
2. *What `0x22`/`0x10fb0` actually do* — is `0x22` "run carried code a1" (cp23 showed
   replaying carried code can be inert), or a blit primitive? And does `0x10fb0` render
   a glyph to the back-buffer, and from what font data?

*cp35 plan:* (a) guest-register probe at the `0x22` site to fix `r6`/the font image; (b)
RE `0x10fb0`; (c) implement the confirmed path. (Resolved in cp35 below.)

**cp35 — both unknowns resolved; the font path is platform-gated (§7), STOP.** A
one-shot `dump_reg_stack` probe at the per-char `import 0x22` (filtered `a1=0x11264,
a2=14`) settled both:
- **Unknown 1 (corrected cp33/cp34):** `r6` is **not** the font image — `R6 = 0x48840550
  = g (the back-buffer Graphics, ≠0)`. The font image is `import 0x22`'s **a0**
  (`R0 = [singleton.field5] = 0`), measured **0** every char (with `R7=0`, `SB=x`,
  `SL=char`, e.g. `0x4c='L'`). So the **font sheet image is absent/null**, and the glyph
  draw falls back to the native `0x22` path.
- **Unknown 2:** `0x10fb0` (the `0x22` a1 fn) is **`strb` into an object's field array**
  (`[r2+4..+0xb]`, 8 bytes) with **no `Graphics`/drawImage vtable call** — i.e.
  bookkeeping, not an on-screen blit. So the native path renders nothing (cp23-style
  inert), even if run.
- **No font load happens.** A full `debug` run shows `Image.createImage` called **once**
  — the 240×320 **back-buffer** — and **no** `createImage`/`getResource` for a font
  sheet anywhere. So path (i) JVM-image has no load site in the reachable run, and path
  (ii) native is inert.

**Classification: platform-side (§7).** Either the font sheet would be loaded by a later
init step the game never reaches (a.run is one-shot — same per-frame-drive gap as the
render driver), or the font is an ez-i-native resource the `0x22`/`0x10fb0` runtime draws
(which wie doesn't emulate). Both are the §7 missing piece, not an app-side one-liner;
forcing a font would be a guess. *cp36 (one line):* once the per-frame drive (§7) runs,
re-check whether the game then loads the font sheet via `createImage` (→ path i becomes
implementable, `g.drawImage(sheet, src_x=(char-0x21)*10, w=10)`); until then the title
text is blocked on the same §7 gate as the rest of the live render state.

### The single missing answer (for the maintainer)

> In ez-i, when an app `new`s a bare native object and hands it to platform import
> `0x21` (and registers app callbacks via `0x55`/`0x56`), **which registered object's
> which native entry point does the ez-i runtime invoke each frame to paint**, and how
> does its back-buffer reach the screen (the ez-i equivalent of `DisplayProxy.flush`)?
> Equivalently: what is the ez-i native displayable/clet ABI that `0x21`/`0x55`/`0x56`
> bind — so wie can call that per-frame entry from its existing paint tick?

With that, the connection is: on each wie paint tick, invoke the registered object's
per-frame entry with a Graphics, then map its flush to wie's `present`/screen blit —
all from the `wie_lgt` / `LgtJvmShared` side, without touching shared classes.

cp28 narrows what this drive must accomplish: wie's paint→`Graphics`→back-buffer→flush
path already works (forcing one gate flag made `o.paint` draw to screen). The missing
piece is purely **advancing the game state machine each frame** so the per-card render
flags like `o.g` get set.

**cp36 — driving the state methods from wie does NOT substitute for the ez-i tick
(confirmed).** Tested whether wie can just call the state-advance methods each frame
instead of the ez-i runtime:
- *Correction to cp27:* the `o.g` writer is **not** the registered virtual `o.k()V`.
  `o.k @0xda7f8` is a short method (returns at `0xda85c`, no `+0x18`/`o.g` store) — it
  copies a singleton field. The real `o.g` writer is an **unregistered helper
  `@0xda870`** (not in any class's method table): `getInstance(o)` → reset `o.g=0` (and
  siblings) → conditionally `o.g=1` at `0xdb240`.
- *Experiment (reverted):* drove both, 3× each, reading `o.g` after each call. `o.k()`
  (JVM `invoke_virtual`) → `o.g` stays `0`. `fn@0xda870` (native `run_function`) → `o.g`
  stays `0` too: the helper runs and resets `o.g=0`, but its conditional `o.g=1` branch
  (`0xdb240`) is **not taken** — that branch depends on accumulated game state
  (load-complete / timer / input / card-transition), not satisfiable by calling the
  method in isolation.
- *Conclusion:* **a single JVM/native method drive cannot advance the state** — `o.g=1`
  needs the whole game loop's accumulated conditions, which is exactly the ez-i per-frame
  drive. So substituting wie method-calls for the ez-i tick is ruled out; the §7 entry
  (the real per-frame driver the ez-i runtime invokes) is the single remaining gate, the
  same one that gates the live render state, `o.g`, and the font load. Maintainer path.

**cp37 — per-frame-driver recon (disasm + word dumps; logging-only, reverted). Verdict:
self-contained (A), NOT external-injection (B).** Settled whether the §7 driver is
derivable from the app binary, by examining the three things ez-i registers.

- *Recon 1 — `code@0x1ad4` (the "carried code" of `0x55`/`0x56`).* `0x1ad4 = b 0x1a24`.
  Full disasm of `0x1a24`: prologue → `bl 0x1908` (getInstance fetches its own object,
  no `this` arg) → writes **constants** into the object's field array (`field[9]=0`,
  `field[13]=1`, `field[14]=1`, `field[15]=<import ret>`, `field[17]=3`, others `0`) →
  return. It reads/writes **no accumulated state** (no timer/frame-counter increment),
  takes **no `r0-r3` arg**. ⇒ this is a **one-shot init/reset helper, not a per-frame
  step**. (Matches cp23: replaying it is inert.)
- *Recon 2 — the unbound object handed to `0x21` (word dump).* Instrumented the `0x21`
  handler (logging only, returns 0). Both driver objects dump identically:

  | site | a0 (obj) | a1 | `[obj+0]` | `[obj+8]`=fldptr | field array `[+0..+0x80]` |
  |---|---|---|---|---|---|
  | a.b `lr=0x227c` | `0x48840540` | `0x48840120` | `0x4010022c` (global vtable) | `0x4010a9a0` | **all 0** |
  | a.run `lr=0x2108` | `0x48840550` | `0x1ad4` (carried code) | `0x4010022c` (global vtable) | `0x4010ada8` | **all 0** |

  ⇒ the registered object is just `{global-vtable word, empty field array}` — **no code
  pointer anywhere** in header or fields. There is no per-frame native entry *stored in
  the object*; ez-i would dispatch through a vtable slot, but the object is unbound (no
  class/methods). Confirms cp25.
- *Recon 3 — what actually sets `o.g=1` (corrects cp27/cp36).* The `o.g` writer is **not**
  `o.k()V` (`@0xda7f8`, a short field-copy that returns at `0xda85c`) nor the helper
  `0xda870`. It is a **separate function `@0xdb200`**: `r4 = r0` (arg) → getInstance(o) →
  `o.field[12] = max(o.field[12], r4)` → **unconditionally `o.g = 1`** (`0xdb240`). It has
  **no `bl` callers**; it's reached only via 8 `ldr+bx` (vtable/dispatch) sites
  (`0x1d4e0`, `0x1dd58`, `0x2d714…`, `0x702fc`, `0x747a0`). At each site the arg is a
  **literal constant** — `0x1dd58: r0=#9`, `0x1d4e0: r0=#1` — **not** an elapsed-time /
  input / event value.

**Verdict (A), with evidence.** `o.g=1` is set **unconditionally from a constant** — no
external per-frame value is injected, so **(B) is ruled out**: ez-i does **not** need to
pass elapsed-time/input each frame for the state to advance. The state machine is
self-contained; it just needs its step methods *called*. So the §7 gap is **not** an
ABI-data problem (no per-frame argument signature is required) but purely a **call-trigger**
one: the `0xdb200`-calling game methods (card-step logic, in the `0x1d…`/`0x2d…` region)
are never dispatched in the reachable run (trace shows no `lr=0x1d…/0x2d…`), because
`a.run` is one-shot and nothing drives the step loop.

*Next move (cp38):* drive the state step from wie's tick — unlike cp36 (which drove the
wrong methods: `o.k`/`0xda870`, both non-writers), the real `o.g=1` writer is `@0xdb200`,
reached from card-step methods. RE which registered/bound method on the card or `a`
singleton sits on the `0xdb200` path, then invoke it each frame from wie's existing
`CardCanvas.paint` tick (no args needed — self-contained). If that lands `o.g=1`, cp28
already proved `o.paint` then draws ⇒ live render from the wie side, no maintainer ABI.

**cp38 — game-flow RE: `i` is the title card; its scene-setup `i.a(I)V` runs the o.g=1
path; the gap is the card lifecycle, not an ABI (disasm + trace).** Traced the whole
reachable game flow and where it stops:

- *Flow.* `a.startApp` (gate `a.field[slot]==0` → init, else polymorphic
  `a.vtable[slot]` = `Game.startApp`) → `Game.a` (data load) / `Game.b` → i-card init
  (the `0x1c…` region runs: `i.Q` etc. — array clears) → registration (`0x57` show-card
  with a1=`0x48840120`, `0x21` driver `0x48840540`) → `Thread.start(a)` → `a.run`
  (one-shot: `0x55/0x56/0x1f/0x57/0x21`, returns). Native methods dispatched:
  `Game.a/b`, `a.startApp`, `a.run` — **no card paint/step**.
- *`i` is the title card.* The card handed to `0x57` (`0x48840120`) is constructed via
  `Card.<init>` from an `o.<init>` chain (`lr=0xd86c4`) — i.e. an instance of `i`
  (`i extends b extends o extends Card`). `i` has 151 methods (the game's main logic).
- *`i.a(I)V` (@0x1d4ac) is the scene-setup.* Disasm: takes a state arg `I`, calls the
  `o.g` setter `@0xdb200` with **literal constants** (`r0=#1`, `#2`, …), runs object/
  resource setup and many `bl` helpers + vtable calls. It is the per-scene **enter/setup
  (run once on card entry)**, not a per-frame step, and it is **never reached** in the
  run (the `0x1d4ac+` range shows no `lr` in trace; only `i`-init `0x1c…` runs).
- *Where it stops.* The game registers the card (`0x57`) and the driver (`0x21`) and
  returns from the one-shot `a.run`. In real ez-i, `0x57` (show card) then drives the
  card's lifecycle — setup (→ `i.a`) and the per-frame paint loop. wie no-ops `0x57`, so
  the card is never *entered* (no `i.a`, so `o.g` stays 0) and never *painted* by the
  game (only the manual `pushCard` of cp26/28 paints, on an empty singleton).

**This refines, not contradicts, cp37 (A).** `o.g=1` is still self-contained (constants).
The missing piece is the **card lifecycle that `0x57` should trigger**: enter the shown
card (run its setup, which sets `o.g=1` and loads resources) and add it to the per-frame
paint loop. That is app-side work wie can do from the `0x57` handler — no ez-i argument
needed. *cp39 target:* implement `0x57` (show-card) as a real card-enter in
`LgtJvmShared` — bind/resolve the shown card instance, run its enter/setup, push it to
wie's `CardCanvas` paint tick — then re-drive and chase the resulting cascade
(font/sprite loads, missing imports) to the title.

**cp39 — implemented show-card + card lifecycle → FIRST PIXELS (o.g set, `o.paint`
draws).** A live trace (`RUST_LOG=wie_lgt=debug`) pinned the §7 gap to a single
no-op'd import and turned it into working render:

- *The gap was one import.* `a.run` calls java-interface `0x57(jlet, card=0x48840120,
  jlet)` — the show-card / `Display.setCurrent` equivalent — which wie left as a no-op.
  So the card was never pushed to wie's Display and **`o.paint` never ran** (0 frames).
  (`0x57` is overloaded: `a.run` also calls `0x57(jlet, <carried code 0x1ad4>, jlet)`,
  arg1 a `.text` pointer not a card.)
- *show-card (`LgtJvmShared::show_card`).* The card guest block was bound to the
  platform `Card` base by its `<init>` trampoline (only `super Card.<init>` runs through
  wie, so the app's most-derived class isn't visible at bind time → `paint` would resolve
  to the empty platform `Card.paint`). Rebind it to the app class `i` (cp38) as an
  `LgtClassInstance` reusing the **same** guest pointer, then `Display.pushCard`. `paint`
  now dispatches through `i→b→o` to native `o.paint` (@0xd8d70). Guarded to genuine bound
  heap objects (ignores the carried-code `0x57`).
- *card lifecycle (`drive_card_step`, called from `LgtMethod::run` just before the
  card's `o.paint`).* First paint tick: run scene-enter `i.a(I)V` (@0x1d4ac) once; every
  tick: run per-frame step `i.aE()V` (@0x72f2c). Both RE'd o.g-setter reachers (cp38).
  The enter MUST run at the paint boundary, **not** mid-`a.run` — driving native code on
  the same `ArmCore` mid-SVC clobbers the in-flight context (an enter from `show_card`
  faulted the whole flow).
- *Result (verified vs ROM).* `o.g` goes 0→1 (`i.a`'s prologue sets it via the `@0xdb200`
  setter during **real** scene setup — not a force), `o.paint` passes the gate
  (`ldr r7,[fields+0x18]; cmp r7,#0; beq`) and **draws each frame through the real MIDP
  Graphics: 18× `fillRect` + 24× `setColor` + 3× `getColor`** (background fills). This is
  the first time the LGT/ez-i app renders pixels in wie.

*Remaining cascade (next cp).* `i.a`'s deep setup (state `0`) builds resource paths
(`"img/map"`, `".dat"`) and in the sub-helper `i.a(Z)V` (@0x2fd94) does
`ldr r3,[r5]; ldr ip,[r3,#0x60]; bx ip` — a hardcoded `vtable[24]` call on a String/
resource object whose class isn't in the import tables, so wie's **global** by-name
vtable misroutes slot 24 to `Display.pushCard` (the cp14/25 unbound-vtable problem) →
swallowed fatal, so sprite/image/text loads don't complete (only `fillRect`/`setColor`
so far, no `drawImage`/`drawString`). Next: resolve that per-object vtable slot (or the
correct title scene state — `0` is a starting value) to unblock the full title.

**cp40 — StringBuffer physical slot 24 = `append(I)`; scene-enter completes.** Resolved
cp39's residual blocker with the cp30 methodology. `i.a(Z)V` (@0x2fd94) builds resource
paths via `append(String).append(int)…` (e.g. `"img/map" + id`): physical vtable slot 19
(offset 0x4c) = `append(String)`, slot 24 (offset 0x60) = `append(int)`. The app calls
slot 24 on a StringBuffer (`r5.slot24(r1)` returns the chainable receiver; `r1 =
singleton.field[0x74]`), but slot 24 wasn't overridden so it fell through to global slot
24 = `Display.pushCard` → swallowed fatal, aborting the scene's loads. Classified by call
shape + a logging-only runtime probe (raw arg = `8`, a small int) ⇒ `append(I)`; added
`(24, "append", "(I)Ljava/lang/StringBuffer;")` to StringBuffer's `known_java_lang_vtable`.
Verified: `i.a(card, 0)` now completes (no fatal); the deep setup loads `txt/*.dat` data
(`mon_info`, `SUB_QST_INFO`, `mon_name`, …). Background still the only draw.

**cp41 — RE map: the next blocker is the resource-load subsystem (+ continuous frame
drive), not a single slot.** Dumped the full imported-platform-API vocabulary and traced
the loader and the post-load idle. Findings:

- *Import vocabulary (30 platform classes).* Relevant: `org/kwis/msp/io/File`
  (`read([B)I` + statics — raw file bytes), `org/kwis/msp/lcdui/Image`
  (`createImage(String)`, `createImage([BII)`, `createImage(II)`, `getGraphics`,
  `getWidth/Height`), `Graphics.drawImage(Image,III)`. wie's shared classes already
  implement `File` + `Image` over `system.filesystem()`, and the JAR carries the 802
  resources (`img/*.mbm` Symbian bitmaps, `img/*.png`, `txt/*.dat`).
- *Loader.* `b.a(Ljava/lang/String;Z)V` (@0x5874) is the app's resource loader: it builds
  `path + ext` with a StringBuffer (slot 19/24/5), then calls into the app's **own**
  resource-manager object (`r6 = this.slot1(); r6.slot17(path)`), which bottoms out in a
  java-interface **import** (the `0xe2c50` thunk → dispatcher `0xe31a8`) — currently
  no-op'd, so the file read returns 0. So no `File`/`Image`/`createImage` is reached;
  the manager uses low-level imports, not the platform classes directly.
- *Post-load idle (NOT a spin, NOT a proven external wall).* After the scene-enter +
  data load + 3 paints (background), the guest goes fully idle in ~0.2 s: `a.run` (the
  game thread) is one-shot and returns; wie's MIDP painted 3× then stopped. The app
  requests frames via `Card.repaint(IIII)` (Card vref 3) / import `0xe2`, which wie
  no-ops, so no further paints/steps run. This is plausibly downstream of the empty
  loads (the loading scene can't advance without real data) and/or a missing continuous
  repaint drive — to be disambiguated after the load import lands. cp37's "no external
  injection" still holds (the frame request is self-contained: `repaint` → paint → step).

*cp42+ target:* implement the resource-load import (read the named file from
`system.filesystem()` and return bytes/handle the manager expects), wire `Card.repaint`
/ `0xe2` to wie's repaint so the frame loop self-sustains, then chase
`createImage`/MBM-decode → `drawImage` (sprites) and the glyph font sheet (cp33-35) →
`drawString`/glyphs. Per-frame image/char allocs must pool/free.

**cp42 — resource-I/O RE corrections (no code): cp41's read-import claim was wrong.**
Read every thunk's `.word table; .word index`:
- `0xe2c50` = stdlib (table 1) **index 0x32 = `new`**, not a read import (cp41 misread it).
- The file read is **not stdlib** (wie's stdlib `_ =>` is `FatalError`; it never fires,
  so the 0x3e9-0x3ec ops in that thunk table are never called) and **not** the `File`/
  `Image` platform classes (no `File`/`Image` trampolines fire — see cp45).
- `b.a(String,Z)V` @0x5874 uses java-interface `0x54`(safepoint)/`0x1f`/`0x12`/`0xf` and
  StringBuffer/System statics; it is **config/setup, not the byte reader** (with `new`
  always non-zero it takes the no-read branch). The app's resource-read path does not use
  the expected platform APIs and is obfuscated — exact contract still unpinned.

**cp44 — self-sustaining frame loop (continuous card tick). IMPLEMENTED + verified.**
`drive_card_step` now schedules the card's `repaint()` after each step, so the shown
card's step+paint run every frame (continuous render mirroring the platform tick;
`repaint` only enqueues a paint event — not a force). **o.paint 3 → 362 frames over 8 s
(~45 fps), self-sustaining**, each frame drawing background (~6 `fillRect` + ~8
`setColor`). The idle is gone; the game still shows only the background.

**cp45 — KEY CORRECTION: the I/O is never reached; the game is stuck pre-load.** Over
362 continuous frames the app exercises **zero** `File`/`Image`/`InputStream`/
`createImage`/`getResourceAsStream` — it builds path strings (StringBuffer) but never
opens them. So the blocker is **not** "implement the load import" (that code is never
reached) — the **scene state machine is stuck in an early state before loading**:
- `i.aE` (per-frame step) iterates a scene-object array (`[singleton+0xd4]`, calling
  slot 18 per element); the array is **empty** (no objects loaded) so each frame is a
  no-op update over background.
- import `0x22` (the image blit, fired 64× during enter) no-ops because its image arg is
  **0** — the image was never `createImage`'d, because the create-image code is never
  reached.

So the real open question (revised): **why does the driven scene (`i.a(0)` + `i.aE`) not
advance to the resource-load state?** Candidates to RE next, by evidence (no guessing):
(a) the forced initial scene `0` may be the wrong entry (a state that idles vs the
splash/loading state the game would normally enter — the scene state still unconfirmed);
(b) the loader's read branch is gated behind a condition (e.g. `b.a`'s `import 0x12`
result) that wie returns 0 for. This is **not** a proven external input/time wall
(`currentTimeMillis` is called once, not polled; `keyNotify` is processed) — cp37 holds;
it is an unresolved internal advance condition needing focused RE.

**cp47 — DECISIVE measurement of the `[+0xd4]` keystone (logging-only probe).** `i.aE`
reads the scene array from `getInstance(class b = handle 0x1400fc4)` (via `0x564c`);
the would-be writers (`i.b`/`o.g`/…) are dispatched on `this=card 0x48840120`. Dumped
`[+0xd4]` of both after the scene-enter+step:
- `getInstance(b) = 0x488401c0 ≠ card 0x48840120` → **instance-identity split is real**
  (cp26/27): wie mints a fresh per-class singleton instead of returning the card.
- **but `[+0xd4] = 0` on _both_** → the populate never wrote to either object. Cross-ref:
  `i.b` is an arg-dispatched command (switch on the int); its `[+0xd4]` write branch
  needs arg `~0x46`, but `i.b` only ran with `0x3e9` (a different branch). **The
  populate command was never dispatched** ⇒ the blocker is hyp **(3)** (populate not run),
  not hyp (1) (identity).

**cp48 — identity alias (hyp 1): implemented, REVERTED (not the keystone + regressed).**
Made `getInstance` for the card's app classes `{i,b,o}` return the card (cp17: the card
is the single instance holding all inherited fields). Verified it unifies them
(`getInstance(b)=card`), but `[+0xd4]` stayed 0 **and** it *changed control flow
harmfully*: the resource-load-by-ID methods that ran before (`o.g(0x3e9)`, `o.e/o.f(0x23)`,
`i.b(0x3e9)`) **stopped running** (the card's pre-existing field state made the loader
think work was already done). Reverted to baseline. The identity split is a real latent
issue but **not** the blocker, and blanket-aliasing is the wrong fix.

**cp49 — the populate chain bottoms out at the obfuscated resource read (same root).**
RE'd the loader `o.g(I)V @0xda518` (called with resource id `0x3e9`): it stores the id at
`field[0x74]=0x3e9` and `field[0x78]=0` (the *data* slot, empty), then dispatches a
polymorphic virtual `this.vtable[X](id)` — the read that should fill `field[0x78]` lives
in that handler and bottoms out in the no-op'd obfuscated import (cp42–45). The populate
`i.b(0x46) → [+0xd4] = [obj.field+0x44]` is gated on `field[0x78]` filling. So the chain
is: **obfuscated resource read no-ops → `field[0x78]` stays 0 → loaded-data object is
null → populate command never fires / writes null → `[+0xd4]` empty → `i.aE` idles over
an empty scene → no sprites/text.** The genuine root is the **obfuscated resource-read
import** (multi-layer: app loader stores id → virtual handler → import that returns 0),
still unpinned and not the standard `File`/`Image`/stdlib APIs (cp42/45). This is an
internal I/O mechanism, **not** an external input/time wall (cp37 holds) — but cracking it
needs sustained focused RE; per the guardrails I did not guess past it.

**cp50 — traced the loader chain to its leaf: there is NO single clean read import; the
data path is a multi-layer obfuscated in-memory system.** Followed the cp49 chain all the
way down (full disasm + `.word table;index` on every thunk + runtime arg cross-ref):
- `o.g(I)V @0xda518`(id) → `bx this.vtable[X](id)` where `X=[0x15009ac+0x3a]` resolves
  (by trace order) to the handler **`i.b(I)V`**.
- `i.b`'s arg-switch routes `id=0x3e9` → `0x20860`, which calls helper **`0x706c`** (in
  `b.e`) with `getInstance(b).field[0xc0/0xc4]`.
- **`0x706c` is in-memory scene-object array management** — not a file read: it reads
  `getInstance(b).field[0x44]` as a count, scales by `<<4` (16-byte elements), and runs
  alloc/copy/`Math` ops (`static_method_offsets[76/78/80]`) to grow/copy the object
  array, storing to `field[0x84]`.
- The only imports anywhere on this path are `0x54` (safepoint) and the class/static
  field tier **`0xb`/`0xd`** (called 186×/179×, both no-op'd) and `0x22` — **none a clean
  `read(id|path) → bytes`**. The resource *bytes* never enter via a single identifiable
  import; the IDs (`0x3e9`, `0x23`) index an in-memory data system fed by the no-op'd
  `0xb`/`0xd` static/class-field tier.

**Conclusion (refines the mission premise).** The blocker is **not** one measurable
"resource-read import" that can be implemented from a pinned contract — it is the app's
**entire obfuscated resource/data subsystem** (id→in-memory struct via the `0xb`/`0xd`
class-field tier + multi-layer array builders `i.b`/`b.e`/`0x706c`), which does not use
the standard `File`/`Image`/stream APIs and exposes no single leaf byte-read with a
measurable `(input → field[0x78]/bytes)` contract. Implementing it would require RE'ing
the whole subsystem (the `0xb`/`0xd` static-field semantics + the id→data table + the
in-memory object layout) — a large effort, not a one-import fix. Per the guardrails
(no fabricating a read without a measured byte contract, and none is pinnable here) this
is the honest stopping point for the resource cascade. It remains an **internal** system,
not an external input/time wall (cp37 holds). Next-RE thread: pin the `0xb`/`0xd`
class/static-field import contract (result usage: used vs discarded) — that tier, not a
file-read, is where the id-indexed data originates.

**cp51 — `0xb`/`0xd` measured + implemented (lazy class/instance init); A/B gate ⇒ B.**
Pinned the `0xb`/`0xd` contract by the call shape in the getInstance helpers and a
runtime probe:
- **`0xd(instance, init_fn)` = lazy instance init.** Guard `if [inst.field+0x10] != 5 {
  0xd(inst, init_fn) }` (`5` = initialised). Runs the instance initialiser once
  (`init_fn` is passed by the call site, e.g. `i.c @0x788a0` for the `b`/`o` singleton).
- **`0xb(class)` = lazy class init.** Guard `if [[class+8]+0x1a] != 3 { 0xb(class) }`
  (`3` = initialised). Marks the class header init flag.
- Both no-op'd: `0xb` spun **3665×/run** (the flag never reached 3) and instances/classes
  never initialised. Implemented in `LgtJvmShared` (`lazy_instance_init` runs `init_fn` +
  marks `field[0x10]=5`; `lazy_class_init` marks `header[+0x1a]=3`). Verified: spin gone
  (`0xb` 3665→0), 16 instance inits run, no crash, frame loop intact. **This is a genuine
  fix** (a real no-op'd init subsystem + per-frame spin), kept.
- **Decisive A/B measurement (this refutes cp50's premise):** implementing `0xb`/`0xd`
  does **not** populate the scene data — after init + scene-enter + step,
  `getInstance(b).field[0x44]=0` (count), `field[0x84]=0` (the `0x706c` array),
  `field[0xd4]=0`, **all zero**. So `0xb`/`0xd` are *init*, **not** the static-field data
  source cp50 guessed; the scene data still comes only from the deeper obfuscated resource
  mechanism (cp49/50: `o.g→i.b→0x706c`), which has **no pinnable single contract** and is
  **not reached** this run. ⇒ **Path B**: the data is in another layer not supplied by the
  measurable tier; converge to a **Foundation PR** of the resolved baseline (boot + first
  pixels + self-sustaining background render — §7 resolved). The full title (sprites/text)
  is scoped future work blocked on that whole subsystem. Still **internal**, not an
  external input/time wall (cp37 holds).

**cp52 — the load-trigger question, answered (single decision session). The load is
*requested* but never *completes*; the completion is the unpinnable subsystem ⇒ (ii)
wall, Foundation PR finalised.** A per-frame probe of the scene singleton
`getInstance(b)` over 293 frames showed the state **totally stuck**: `field[0x1c]=0`
(`i.a` state), `field[0x54]=0` (`i.aE` scene-machine state — never advances),
`field[0x44]=0` (scene count), `field[0xd4]=0` (scene array) — **but `field[0x74]=8`**
(a resource id *is* requested) while **`field[0x78]=0`** (its data **never fills**). So:
- The load **is reached/triggered** — `o.g(8)` requested resource id `8` (`field[0x74]=8`)
  and the game **polls** `field[0x78]` every frame for completion. It is *not* a simply
  "missing trigger" (the earlier cp45 "never reached" was the *image* load; the *data*
  load is requested here). The whole state machine is gated on `field[0x78]` filling.
- The completion (`field[0x78]` fill) is **not** a drivable hook: every `field[0x78]`
  writer (`o.g`, `o.h`, …) only **clears** it to 0 (request markers); the actual *fill*
  with loaded bytes is the obfuscated resource subsystem (cp50) — **no single measurable
  contract, no clean per-frame processor**, diffuse across the same multi-layer
  mechanism. Candidate A (a no-op'd platform hook): none — `o.g` requests via no resource
  import. Candidate B/C (missing lifecycle / `a.run` one-shot): the would-be load
  processor is part of the obfuscated subsystem, **not a single confirmable drivable
  method this session**.
- Per the decision gate (a deeper-keystone candidate that can't be confirmed as a single
  drivable trigger ⇒ treat as (ii) evidence; no 7th-keystone chase): **(ii) — the
  practical wall is the obfuscated resource read/completion subsystem.** It is **reached
  but no-ops**, has no pinnable contract (cp50), uses no standard `File`/`Image`/stream,
  and is **internal** — not an external input/time wall (cp37 holds: `field[0x78]` is
  polled internally, no input/timer dependency). **Foundation PR finalised.**

---

## 8. Current reach

| stage | state |
|---|---|
| app classes registered, methods run as real ARM | ✅ |
| boot (`0x64` java-interface → `Main.main` → Jlet) | ✅ |
| two-level vtable + per-class overrides + instance field layout | ✅ |
| `getInstance` singletons, `Thread.start`, game thread spawns `a.run` | ✅ |
| data load → 240×320 back-buffer → `getGraphics` → Cards/RNG | ✅ |
| app `Card.paint` ticked in wie's loop (cp26 experiment) | ◑ wires in & runs per-frame, but `o.paint` gates on `o.g` which its (never-run) virtual `o.k()V` writer would set → **0 draws** (cp27, §7) |
| render path with `o.g` forced to 1 (cp28 experiment) | ✅ `o.paint` draws (21× setColor, 18× fillRect) to back-buffer + flushes → **render path works end-to-end**; only "set `o.g`" (ez-i per-frame drive) is missing (§7) |
| `java/lang/String` slot 35 = `toCharArray()[C` (cp30) | ✅ per-class override added; `String.e` abort gone, `o.paint` runs without fatal. Title text still blocked on char-array guest marshalling (cp31) |
| char-array guest marshalling (cp31) | ✅ `materialize_char_array` → `{u32 len, u16 chars}` at `[arr+8]` (RE'd, unit-tested; `len=10 "LOADING..."`) |
| glyph loop runs, consumes chars (cp32) | ✅ confirmed: loop runs 30× (3 frames × 10 chars), reads "LOADING..."; the `0x10298` "gate" is just `getColor` (both paths fall through) |
| glyph-draw fn runs; no font image (cp33) | ◑ `@0x109b4` is called per char but takes its `r6==0` (no font image) path → `import 0x22` no-op, **0 drawImage**. Root cause: bitmap-font sheet absent guest-side |
| glyph blit mechanism RE'd (cp34) | ◑ blit = `g.drawImage(sheet, src_x=(char-0x21)*10, w=10)`; font path via `import 0x22(a0=font_img, a1=0x11264→fn 0x10fb0)` |
| font path resolved → platform-gated (cp35) | ⛔ probe: `r6=g` (not the font img — cp33/4 corrected); font img = `0x22` a0 = **0** every char; `0x10fb0` = strb bookkeeping (no draw); **no font `createImage`** in the reachable run (only the 240×320 back-buffer). Font load/native render is §7-gated, not an app one-liner |
| wie can't substitute the ez-i tick (cp36) | ⛔ `o.k()` (registered) and `fn@0xda870` (unregistered) both driven from wie → `o.g` stays 0 — but cp37 shows these were the *wrong* methods (neither is the `o.g` writer) |
| per-frame driver is self-contained, not ABI (cp37) | ◑ recon: carried code = const init; `0x21` obj has no code ptr; real `o.g=1` writer `@0xdb200` sets g **unconditionally from a constant arg** ⇒ **(A) self-contained, (B) external-injection ruled out**. Gap is a call-trigger, not an ABI signature |
| game-flow RE: `i` = title card, gap = card lifecycle `0x57` (cp38) | ✅ traced flow stops at one-shot `a.run`; `0x57` show-card no-op'd → card never pushed, never painted |
| **show-card `0x57` + card lifecycle → FIRST PIXELS (cp39)** | ✅ `0x57`→`show_card` rebinds card to `i` + `Display.pushCard`; `drive_card_step` runs scene-enter `i.a` once + per-frame step `i.aE` before paint → **`o.g` 0→1 (genuine setup, not force), `o.paint` draws 18× `fillRect` + 24× `setColor` through real MIDP Graphics**. First LGT pixels in wie |
| StringBuffer slot 24 = `append(I)`; scene-enter completes (cp40) | ✅ vtable[24] misdispatch fixed; `i.a(card,0)` runs to completion (loads `txt/*.dat`); background still the only draw |
| self-sustaining frame loop (cp44) | ✅ `drive_card_step` schedules `repaint()` each tick → o.paint 3 → 362 frames (~45 fps); idle gone; background renders continuously |
| `[+0xd4]` keystone measured (cp47) | ✅ identity split real (`getInstance(b)≠card`) but **not** the blocker — `[+0xd4]=0` on both ⇒ the populate command (`i.b(0x46)`) never runs (hyp 3). cp48 identity-alias reverted (regressed flow) |
| `0xb`/`0xd` lazy class/instance init (cp51) | ✅ implemented (`0xd`=run instance init_fn at `field[0x10]!=5`; `0xb`=mark class flag at `[[class+8]+0x1a]!=3`); removed a 3665×/run spin; genuine init fix |
| full title (logo/sprites/text) — **(ii) wall, scoped future work** | ⛔ cp52: the data load **is requested** (`getInstance(b).field[0x74]=8`) but its data **never fills** (`field[0x78]=0` over 293 frames; state machine stuck, `field[0x54]=0`). The completion (`field[0x78]` fill) is the obfuscated resource subsystem (cp49/50) — `field[0x78]` writers only *clear* it; the *fill* has **no single measurable contract / no drivable hook** (cp50/52). Not File/Image/stdlib; **internal**, not an input/time wall (cp37). Whole-subsystem RE, not a one-import fix |
| clet regression (`test_helloworld`) / `clippy -p wie_lgt` | ✅ clean |
