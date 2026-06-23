# LGT native class-descriptor format (reverse-engineering notes)

Reference app: one ez-i AOT-compiled LGT WIPI app (a Java game).
Scope: this is a **read-only RE pass**. It documents the byte layout of the native
class descriptors that the app hands to the platform via `java_unk5` (java-interface
import `0x07`). No JVM registration is implemented here.

Parser implementation: [`wie_lgt/src/runtime/java/native_class.rs`](../wie_lgt/src/runtime/java/native_class.rs).

## 1. Background

The app's `binary.mod` is a 1 MB ARM ELF and the jar contains **no `.class` files** —
The app is **AOT-compiled Java** (ez-i / Xceed-style toolchain). Each Java class
is emitted as a metadata record in the ELF `.data` segment; the class's methods are
plain native ARM functions whose entry points live in `.text`. The platform is expected
to read these records to expose the app's classes to the (platform-provided) Java
runtime — i.e. this is the native↔JVM bridge data.

### Memory map (`binary.mod`, from ELF section headers)

| section | vaddr range | role |
|---|---|---|
| `.text` | `0x001000 .. 0x0e7800` | native code **and** read-only strings (names/signatures live near the top, ~`0xe4xxx..0xe77xx`) |
| `.data` | `0x1400000 .. 0x14046fc` | class descriptor records, handles, registries |
| `.bss`  | `0x1500000 .. 0x1500a78` | output tables (e.g. `java_load_classes` offset tables) |

ELF entry point `0xe325c` (in `.text`). All pointers in `.data` are absolute and require
no relocation (fixed load address), so the descriptors can be decoded statically from the
file — which is how the layout below was confirmed, then cross-checked against a live run.

## 2. `java_unk5(a0, a1)` — the app class registry

`a0` (observed `0x1400154`) is a container:

```
a0[0]            = handle_count            (16 for the reference app)
a0[1]            = 0
a0[2 .. 2+n]     = `count` class HANDLES   (each = class_header + 0x4c)
a0[2+n ..]       = trailing per-entry byte array (small values 0x01..0x11;
                   likely per-class vtable/slot counts — role UNCONFIRMED)
```

Each handle is **not** the class header; it is the small sub-structure at
`header + 0x4c` (see §4). To reach the header, read the handle's `+0x08` word.

`a1` (observed `0x1400878`) starts with a count (`0x14a`) followed by a large zero-filled
region — **role unconfirmed** (likely a runtime scratch / interned-object table; it is all
zero at registration time).

The 16 handles in `a0` enumerate classes `p, q, r, s, d, e, f, g, h, i, j, k, l, m, n, o`.
Note this set does **not** include the main class `Game` nor `a`, `b`, `c`; those records
exist in `.data` too (see §6) and are reached via other paths (`Game` is named in
`java_unk0`/`java_unk11`). Whether `a0` is intentionally a subset or there is a second
registry is **unconfirmed**.

## 3. Class header record

All offsets little-endian; `H` = header address. **Confirmed** fields:

| offset | type | field | notes |
|---|---|---|---|
| `+0x00` | u32 | `tag` | `0x21` or `0x31` (also `0x601` on one entry); exact meaning unconfirmed |
| `+0x08` | u32 | `ptr_name` | → cstring, obfuscated (`"Game"`, `"i"`, `"l"`, …) |
| `+0x10` | u32 | `ptr_parent` | polymorphic — see §5 |
| `+0x18` | u32 | `access_flags` | low half = Java access bits; bit `0x20000` set on every app class |
| `+0x38` | u32 | `ptr_methods` | → method table: `[count: u32, MethodRecord × count]` |
| `+0x3c` | u32 | `ptr_fields` | → field table: `[count: u32, FieldRecord × count]` |

Words `+0x04/0x0c/0x14/0x20…` are mostly zero; `+0x24` (`0x00190705` on `Game`) and the
three `.text` pointers around `+0x2c..+0x34` (e.g. `0x1000/0x1084/0x1028`) are
**unconfirmed** (candidate: clinit / finalizer / static-init thunks). They are not needed
to enumerate methods and fields.

## 4. Class handle (`H + 0x4c`)

```
H+0x4c:  { 0x00000000, 0x00000000, H }      // third word points back to the header
```

This 12-byte handle is the canonical "class identity": **every** `FieldRecord.ptr_class`
and `MethodRecord.ptr_class` equals `H + 0x4c` of the owning class, and the `a0` registry
stores these handles. Confirmed: `Game` header `0x1400000`, handle `0x140004c = {0,0,0x1400000}`,
and `Game`'s field/method records all carry `ptr_class = 0x140004c`.

## 5. Parent reference (`+0x10`) — polymorphic

The parent word is resolved in three observed forms:

1. **cstring** → platform/superclass name. e.g. `o → "org/kwis/msp/lcdui/Card"`,
   most leaf classes → `"java/lang/Object"`.
2. **app-class handle** (`{0,0,header}`) → an app superclass. e.g. `Game → a`,
   `d/e/j/l → o`, `i → b`, `q → r`. Resolve as `handle → +0x08 → header → +0x08 → name`.
3. (fallback) a bare header pointer.

> Note: an earlier draft mis-reported a base class `"!"`. That was an artifact of reading
> the `o` header's `tag` byte (`0x21` = `'!'`) as a string; with handle-aware resolution the
> real parent of `d/e/j/l` is `o`. There is no `"!"` class.

## 6. Method record (28 bytes) — **CONFIRMED**

| offset | type | field | notes |
|---|---|---|---|
| `+0x00` | u32 | `ptr_class` | = owning class handle (`H+0x4c`) |
| `+0x04` | u32 | `ptr_name` | → cstring (`"<init>"`, `"a"`, `"pauseApp"`, …) |
| `+0x08` | u32 | `ptr_signature` | → cstring JVM descriptor (`"()V"`, `"(I)V"`, …) |
| `+0x0c` | u32 | `access_flags` | e.g. `0x10001`, `0x10011`, `0x8001` |
| `+0x10` | u32 | `num_locals` | small int (0 for `<init>`, 1 for `(I)V` setters); interpretation tentative |
| `+0x14` | u32 | `code_ptr` | **native ARM entry point** (in `.text`) |
| `+0x18` | u32 | `unk` | usually 0 |

**Validation: 283 / 283** method `code_ptr` values across all app classes fall inside
`.text` (`0x1000..0xe7800`) — zero exceptions. This confirms both the record stride (28)
and the `code_ptr` offset (`+0x14`).

## 7. Field record (20 bytes) — **CONFIRMED**

| offset | type | field | notes |
|---|---|---|---|
| `+0x00` | u32 | `ptr_class` | = owning class handle |
| `+0x04` | u32 | `ptr_name` | → cstring (`"a"`, `"b"`, …) |
| `+0x08` | u32 | `ptr_type` | → cstring type descriptor (`"I"`, `"Z"`, `"Lj;"`, `"[Lorg/kwis/msp/lcdui/Image;"`) |
| `+0x0c` | u32 | `access_flags` | e.g. `0x8001`, `0x8002` |
| `+0x10` | u32 | `index` | slot index / offset (0,1,2,… within the class) |

Type descriptors use standard JVM grammar with obfuscated class names (`Lj;`, `Ld;`, `[Lq;`).

## 8. App class inventory (static scan of `.data`)

| header | name | tag | access | parent | #methods | #fields |
|---|---|---|---|---|---|---|
| `0x1400000` | `Game` | 0x21 | 0x2000f | `a` | 5 | 5 |
| `0x14011e4` | `c` | 0x31 | 0x20000 | `java/lang/Object` | 0 | 0 |
| `0x1401268` | `d` | 0x31 | 0x2001c | `o` | 11 | 4 |
| `0x140144c` | `e` | 0x31 | 0x20024 | `o` | 13 | 12 |
| `0x1401720` | `f` | 0x31 | 0x2000c | `java/lang/Object` | 0 | 0 |
| `0x14017bc` | `g` | 0x31 | 0x20000 | `java/lang/Object` | 0 | 0 |
| `0x1401840` | `h` | 0x31 | 0x20003 | `java/lang/Object` | 0 | 0 |
| `0x14018c4` | `i` | 0x31 | 0x2004d | `b` | 151 | 52 |
| `0x1402db8` | `j` | 0x31 | 0x2001d | `o` | 8 | 5 |
| `0x1402f5c` | `k` | 0x31 | 0x20009 | `java/lang/Object` | 0 | 0 |
| `0x1402ff8` | `l` | 0x31 | 0x20050 | `o` | 72 | 56 |
| `0x1403c98` | `m` | 0x31 | 0x20003 | `java/lang/Object` | 0 | 0 |
| `0x1403d20` | `n` | 0x31 | 0x20002 | `java/lang/Object` | 0 | 0 |
| `0x1403dbc` | `o` | 0x21 | 0x20018 | `org/kwis/msp/lcdui/Card` | 23 | 11 |
| `0x14041e0` | `q` | 0x31 | 0x20077 | `r` | 0 | 0 |
| `0x1404300` | `r` | 0x21 | 0x20011 | `java/lang/Object` | 0 | 0 |
| `0x14043a4` | `s` | 0x31 | 0x20018 | `java/lang/Object` | 0 | 0 |

(`a`, `b` headers also exist — `a` is `Game`'s base; `b` is `i`'s base. They are referenced
only as parents, not via `a0`, so they sit just outside the strict header-scan heuristic.)

Reading the structure: `o` extends `org/kwis/msp/lcdui/Card` (the on-screen card/canvas);
`d/e/j/l` extend `o` (game screens/views); `i` (151 methods) and `l` (72 methods) hold the
bulk of game logic; the 0-method classes (`c,f,g,h,k,m,n,q,r,s`) are interfaces/markers or
near-empty helpers. `Game` (5 methods incl. `pauseApp`/`resumeApp`) is the Jlet entry,
extending app class `a`.

## 9. Confirmed vs. unconfirmed

**Confirmed** (cross-validated statically + live, 283/283 code-ptr check):
- class header offsets `ptr_name +0x08`, `ptr_parent +0x10`, `access +0x18`,
  `ptr_methods +0x38`, `ptr_fields +0x3c`
- class handle at `+0x4c` = `{0,0,header}`; equals members' `ptr_class`
- method record (28 B) and field record (20 B) layouts incl. `code_ptr +0x14`
- parent polymorphism (cstring vs app handle)

**Unconfirmed / future work:**
- `tag` (`0x21`/`0x31`/`0x601`) semantics and the `access_flags` high bits beyond `0x20000`
- header words `+0x24` and the `.text` thunk pointers near `+0x2c..+0x34` (clinit?)
- `MethodRecord +0x10` (`num_locals`?) and `+0x18`
- the `a0` trailing byte array (per-class counts?) and the `a1` aux table
- why `a0` enumerates only 16 of the ~19 app classes (subset vs. second registry)
- interface list location (no `ptr_interfaces` field identified yet)

## 10. Appendix — raw dumps

### `Game` class header @ `0x1400000`
```
+0x00 0x00000021   tag
+0x08 0x000e4fe2 -> "Game"          ptr_name
+0x10 0x01400df4 -> handle(a)        ptr_parent  (= class a's handle 0x1400da8+0x4c)
+0x18 0x0002000f                     access_flags
+0x38 0x014000c0 -> methods[5]       ptr_methods
+0x3c 0x01400058 -> fields[5]        ptr_fields
+0x4c 0x140004c = {0,0,0x1400000}    class handle
```

### `Game` field table @ `0x1400058` (count=5, 20-byte records)
```
count=5
[0] ptr_class=0x140004c name="a" type="Lj;" access=0x8001 index=0
[1] ptr_class=0x140004c name="b" type="Ld;" access=0x8001 index=1
[2] ptr_class=0x140004c name="c" type="Li;" access=0x8001 index=2
...
```

### `Game` method table @ `0x14000c0` (count=5, 28-byte records)
```
count=5
[0] ptr_class=0x140004c name="<init>"    sig="()V" access=0x10001 locals=0 code=0x10c8
[1] ptr_class=0x140004c name="a"         sig="()V" access=0x10011 locals=1 code=0x11dc
[2] ptr_class=0x140004c name="b"         sig="()V" access=0x10011 locals=1 code=0x1484
[3] ptr_class=0x140004c name="pauseApp"  sig="()V" access=0x10001 locals=1 code=0x14f8
[4] ptr_class=0x140004c name="resumeApp" sig="()V" access=0x10001 locals=1 code=0x1578
```

### Live `java_unk5` dump (`RUST_LOG=wie_lgt=debug`, first entries)
```
java_unk5: app registry @ 0x1400154 (16 class handles, aux @ 0x1400878) — not yet bridged
  class[4] "d" (tag=0x31 access=0x2001c) parent=Some("o") methods=11 fields=4
      <init>()V code=0x1132c locals=0
      a(I)V code=0x113a8 locals=1
      b(I)V code=0x11614 locals=1
  class[9] "i" (tag=0x31 access=0x2004d) parent=Some("b") methods=151 fields=52
      <init>()V code=0x1c348 locals=0
  class[15] "o" (tag=0x21 access=0x20018) parent=Some("org/kwis/msp/lcdui/Card") methods=23 fields=11
      <init>()V code=0xd8684 locals=0
```

## 11. How to reproduce

```sh
# live decode (parses each a0 handle through native_class.rs and logs it)
RUST_LOG=wie_lgt=debug,wie_core_arm=warn \
  cargo run -p wie_cli -- /path/to/<app>.jar
```
Static decode was done by parsing `binary.mod`'s ELF `.data`/`.text` directly (absolute
pointers, no relocation needed); the live decode matches it byte-for-byte.
