# LGT Platform Architecture

> LGT WIPI application packaging, native bootstrap, and AOT Java runtime ABI.

## Table of contents

- [Overview](#overview)
- [Application package](#application-package)
- [Native initialization](#native-initialization)
- [Import resolution](#import-resolution)
- [Java bootstrap](#java-bootstrap)
- [AOT Java data model](#aot-java-data-model)
- [Class registration and initialization](#class-registration-and-initialization)
- [Link-time metadata](#link-time-metadata)
- [Java runtime interface](#java-runtime-interface)
- [ABI details](#abi-details)
- [Unresolved areas](#unresolved-areas)

## Overview

LGT WIPI applications use an ARM ELF executable named `binary.mod`. Clets contain native application code. Java applications package Java methods already AOT-compiled into ARM code together with metadata that lets the platform runtime load classes, construct object layouts, and bind symbolic Java references.

Both application types use the same ELF loading and numbered import-table mechanism. AOT Java binaries additionally contain generated class records, imported and public member tables, field and virtual-method index patch areas, and ARM entrypoints for generated methods.

## Application package

An installed application contains:

- `app_info`, including identifiers and the Java entry class;
- an application archive containing `binary.mod` and application resources.

`binary.mod` is an ELF32 ARM executable. Addressed ELF sections are loaded at their recorded virtual addresses.

## Native initialization

The platform passes two writable native-memory blocks to the ELF entrypoint:

```text
load addressed binary.mod sections
  -> prepare an output block and an import-resolver block
  -> call the Thumb ELF entrypoint(output, resolver, 0)
  -> read the initialization descriptor pointer published in the output block
  -> call the initializer from that descriptor
```

The known block fields are:

```text
output block
  +0x000..+0x213 unknown/bootstrap storage
  +0x214 -> initialization descriptor

initialization descriptor
  +0x00 unknown
  +0x04 -> initializer
  +0x08 -> string "init"

import-resolver block
  +0x00 -> resolve import table
  +0x04 -> resolve function within an import table
  +0x08 unknown
  +0x0c unknown
```

ARM initializer and generated-method pointers are Thumb pointers and commonly have bit 0 set.

## Import resolution

Generated binaries call platform functions through numbered import tables. The first resolver callback accepts a table ID. The second accepts a table ID and function index and returns an ARM-callable target.

Known tables are:

| Table ID | Purpose |
| --- | --- |
| `0x1fb` | WIPI C functions |
| `0x64` | LGT Java runtime functions |
| `0x1` | C standard-library functions and an exception bridge at index `0x32` |
| `0x1f8` | Bootstrap services, including application archive lookup |
| `0x1fc`, `0x1ff`, `0x201` | Additional bootstrap services with unresolved contracts |

Generated code uses lazy 16-byte import thunks. On first use, a thunk resolves its target and patches itself. Later calls jump directly to the resolved function.

## Java bootstrap

The generated Java initializer follows this sequence:

1. Obtain the application archive path through table `0x1f8`, index `0x17`.
2. Pass that path to Java import `0x82`.
3. Install the generated-class collection and its accompanying opaque metadata through import `0x07`.
4. Link imported classes and public generated classes through imports `0x14` and `0x13`.
5. Register, resolve, and initialize generated classes through imports `0x0b`, `0x0c`, and `0x0d`.
6. Start the Java entry class through import `0x83`.

The generated-class collection is a bucket table. Its first word is the last bucket index, each following word is a bucket head, and classes within a bucket are connected through the next pointer in their descriptors.

## AOT Java data model

### Class metadata

Concrete generated classes use the following records:

```text
generated class record
  +0x00 -> linked virtual table
  +0x04 unknown runtime word
  +0x08 -> generated class descriptor

generated class descriptor
  +0x00 u32: Java access flags
  +0x04 -> next generated class record, or zero
  +0x08 -> class name
  +0x0c -> a word within an optional record used while creating instances
  +0x10 -> superclass name
  +0x14 unknown
  +0x18 u16: total instance field words, including inherited fields
  +0x1a u16: class status; value 3 indicates ready
  +0x1c unknown
  +0x20 -> optional record used while creating instances
  +0x24 packed metadata with unresolved meaning
  +0x28 -> count-prefixed implemented-interface name table
  +0x2c -> callback that patches exported member outputs
  +0x30 -> callback that returns the class object after class initialization
  +0x34 -> callback that registers the class and returns its class object
  +0x38 -> count-prefixed method table
  +0x3c -> count-prefixed field table
  +0x40..+0x48 unknown
```

The three callbacks have separate roles:

- The callback at `+0x34` registers the generated class when necessary and returns its Java class object.
- The callback at `+0x30` obtains that class object and ensures the class initializer has completed.
- The callback at `+0x2c` binds exported fields and methods and patches the generated output tables.

The callback at `+0x2c` is not `<clinit>`. The callback associated with the optional record at `+0x20` is also distinct from both `<init>` and `<clinit>`: it writes generated initial field values into newly allocated object storage.

### Fields and methods

A generated field record contains:

```text
+0x00 -> declaring class record
+0x04 -> field name
+0x08 -> field descriptor
+0x0c u16: access flags
+0x0e u16: unresolved metadata
+0x10 u32: mutable field word index
```

Field indexes address 32-bit words. `long` and `double` occupy two consecutive words. The descriptor's instance-field word count covers the complete inherited object layout, not only fields declared by that class.

A generated method record contains:

```text
+0x00 -> declaring class record
+0x04 -> method name
+0x08 -> method descriptor
+0x0c u16: access flags
+0x0e u16: argument word count
+0x10 unresolved metadata
+0x14 -> generated Thumb method
+0x18 unresolved metadata
```

The argument count is measured in 32-bit words and includes `this` for instance methods. `long` and `double` arguments consume two words.

### Objects, arrays, and virtual calls

Objects use a 12-byte header:

```text
object record
  +0x00 -> virtual table
  +0x04 unknown runtime word
  +0x08 -> field storage
```

Generated code accesses fields and virtual methods directly:

```text
value = object->fields[field_word_index]
target = object->virtual_table[virtual_method_index + 1]
target(object, ...)
```

The leading virtual-table word precedes virtual method index zero. Its purpose is unresolved. The class record and its instances share the linked virtual-table base.

Arrays use the same object header. Their field-storage block begins with a 32-bit length followed immediately by packed elements:

```text
array field storage
  +0x00 u32: element count
  +0x04: element 0
  ...
```

Reference-array elements are 32-bit object pointers. Primitive elements use their Java primitive widths.

Class objects use ordinary field storage. Word index 2 receives a pointer to the represented class name. Word index 4 (`fields + 0x10`) records whether the class initializer has completed; ready is value `5`.

## Class registration and initialization

Before a generated class becomes usable, the platform resolves its superclass and constructs a complete virtual table from the inherited layout and all declared methods in the generated descriptor. Platform ABI methods retain their fixed indexes, overrides reuse inherited indexes, and new generated virtual methods occupy new entries.

The descriptor status at `+0x1a` becomes `3` when the class is ready. The callback at `+0x2c` then receives the Java class object and patches the class's exported member tables.

Object creation allocates the complete inherited field-word array. Instance-field initializer callbacks run from the highest participating superclass down to the concrete class. Java constructors remain ordinary generated methods and run separately.

Running the class initializer is separate from making the class available. Import `0x0d` checks class-object word index 4, invokes the supplied callback when needed, and records ready value `5`. The callback at descriptor `+0x34` does not run the class initializer; the callback at `+0x30` does.

UTF-16 string literals carry a cache cell. Import `0x09` returns the cached Java string when present, otherwise creates the string, keeps it reachable for the application lifetime, and writes its object pointer to the cache cell.

## Link-time metadata

The generated image contains three related class collections:

- generated application class records;
- external Java and WIPI classes imported by generated code;
- public generated classes whose members are exported for cross-class linking.

Imported and public classes use the same range record shape. Each record names a class and divides flat member tables into ranges for instance fields, static fields, virtual methods, interface methods, and non-virtual methods.

Java import `0x14` receives 11 input and output table pointers:

| Argument | Table or role |
| ---: | --- |
| `r0` | imported class range records |
| `r1` | instance-field name/descriptor pairs |
| `r2` | static-field name/descriptor pairs |
| `r3` | virtual-method name/descriptor pairs |
| stack 0 | interface-method name/descriptor pairs |
| stack 1 | non-virtual method name/descriptor pairs |
| stack 2 | instance-field word-index outputs |
| stack 3 | static-field word-index outputs |
| stack 4 | virtual-method index outputs |
| stack 5 | interface-method index outputs |
| stack 6 | non-virtual method target outputs |

Field and virtual-method outputs are `u16`; non-virtual targets are 32-bit ARM-callable pointers. The first two non-virtual outputs for a class are the callbacks stored at descriptor offsets `+0x30` and `+0x34`. Remaining entries represent constructors, static methods, private methods, and other direct calls.

Virtual-method linking looks up an index in the virtual table already built when the class became ready. It patches the generated output table, not the virtual table itself. Generated ARM code subsequently consumes the patched field indexes, virtual indexes, and direct targets without symbolic lookup.

Interface-method ranges and outputs are present in the metadata format, but their exact binding rules remain unresolved.

## Java runtime interface

Known functions in Java import table `0x64` are:

| Index | Role |
| ---: | --- |
| `0x03` | unresolved bootstrap helper |
| `0x06` | teardown helper paired with `0x07` |
| `0x07` | install the generated-class collection and accompanying opaque metadata |
| `0x09` | resolve and cache a UTF-16 Java string literal |
| `0x0b` | register a generated class record |
| `0x0c` | resolve a runtime class object from a generated class record |
| `0x0d` | ensure class initialization using a generated callback |
| `0x0e` | obtain an array class |
| `0x0f` | instantiate an object |
| `0x10` | instantiate a one-dimensional array |
| `0x11` | instantiate a multidimensional array |
| `0x12` | test whether the pending exception matches a class |
| `0x13` | link a public generated class and patch member outputs |
| `0x14` | link imported classes and patch member outputs |
| `0x1f` | push an exception-handler frame |
| `0x20` | pop an exception-handler frame |
| `0x21` | rethrow an exception through the current frame |
| `0x22` | raise `NullPointerException` |
| `0x23` | raise an array-index exception |
| `0x25` | raise an arithmetic exception |
| `0x54` | unresolved generated-code helper |
| `0x55` | unresolved generated-code helper |
| `0x61` | checked reference-array store |
| `0x82` | set the application archive path |
| `0x83` | start the Java application entry class |
| `0xe1` | obtain the `java/lang/String` runtime class |
| `0xe2` | obtain the `java/lang/String[]` runtime class |
| `0xfa` | unchecked reference-array store |

## ABI details

- Instance methods receive `this` in `r0`; remaining arguments follow the ARM procedure-call convention.
- Java references cross the generated/runtime boundary as 32-bit object pointers.
- `long` and `double` values use two 32-bit words in arguments, return values, fields, and arrays.
- Generated method calls preserve the low/high word ordering used by the ARM ABI.
- Instance and static field links produce word indexes rather than byte offsets.
- Virtual call sites add one word to the virtual-table base before applying the linked virtual index.
- Descriptor status `3` and class-object word value `5` represent separate stages.
- Exception imports maintain a native handler stack and expose the currently pending Java exception to generated ARM code.

## Unresolved areas

- The purpose of the second class-record word and the leading virtual-table word is unresolved.
- Several class-descriptor metadata fields remain unnamed.
- Interface-method calls and interface-specific generated metadata need further analysis.
- The exact work performed by imports `0x03`, `0x54`, and `0x55` beyond their call positions is unresolved.
- Bootstrap services in tables `0x1fc`, `0x1ff`, and `0x201` remain only partially understood.
