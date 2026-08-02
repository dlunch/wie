# LGT Platform Architecture

## Table of contents

- [Overview](#overview)
- [Application package](#application-package)
- [Native initialization](#native-initialization)
- [Import resolution](#import-resolution)
- [Java bootstrap](#java-bootstrap)
- [AOT Java data model](#aot-java-data-model)
- [Link-time metadata](#link-time-metadata)
- [Java runtime interface](#java-runtime-interface)
- [ABI details](#abi-details)

## Overview

LGT WIPI applications use an ARM ELF executable named `binary.mod`. Clets contain native application code, while Java applications contain Java methods that have already been AOT-compiled into ARM code together with the metadata needed to bind them to the Java runtime.

Both application types use the same ELF initialization and import-table mechanism. Java binaries additionally describe generated classes, imported classes and members, object layouts, and ARM entrypoints for generated methods.

## Application package

An installed application consists of:

- `app_info`, which describes values including the AID, PID, and main class;
- an application archive containing `binary.mod` and application resources.

`binary.mod` is an ELF32 ARM executable. Its addressed sections are loaded at the virtual addresses recorded in the ELF section headers.

## Native initialization

Initialization follows this sequence:

```text
load binary.mod sections
  -> allocate a host output block and an import resolver block
  -> call the ELF entrypoint with both blocks
  -> read the binary initialization descriptor from the output block
  -> call the initializer referenced by the descriptor
```

The host output block is platform-owned memory through which the binary returns a pointer to its initialization descriptor. The import resolver block provides callbacks for resolving an import table and a function within that table. The ELF entrypoint records these inputs and publishes the generated initializer through the descriptor.

ARM initializer and method pointers are Thumb pointers and commonly have bit 0 set.

## Import resolution

Generated binaries call platform functions through numbered import tables. The import resolver block provides one callback to identify an import table and another to resolve a function index within that table.

Known tables are:

| Table ID | Purpose |
| --- | --- |
| `0x1fb` | WIPI C functions |
| `0x64` | LGT Java runtime functions |
| `0x1` | C standard library functions |
| `0x1f8`, `0x1fc`, `0x1ff`, `0x201` | Additional LGT bootstrap services whose contracts are only partly known |

Generated code uses lazy 16-byte import thunks. On first use, a thunk calls the import resolver and patches itself with the returned target. Subsequent calls jump directly to that target.

## Java bootstrap

The generated Java initializer performs these operations:

1. Obtain the application archive path through import table `0x1f8`, index `0x17`.
2. Pass the archive-related value to Java import `0x82`.
3. Establish the generated class table and runtime context through Java import `0x07`.
4. Link imported and public class metadata through Java imports `0x14` and `0x13`.
5. Prepare and initialize generated classes through Java imports `0x0b`, `0x0c`, and `0x0d`.
6. Invoke Java import `0x83` with the WIPI Java entry class and startup arguments.

The operation names are inferred from how their inputs and outputs are used.

## AOT Java data model

### Class metadata

Generated classes use the following relationship:

```text
generated class record
  +0x00 unknown
  +0x04 unknown
  +0x08 -> generated class descriptor

generated class descriptor
  +0x08 -> class name
  +0x10 -> superclass name
  +0x2c -> generated callback
  +0x30 -> generated callback
  +0x34 -> generated callback
  +0x38 -> method table
  +0x3c -> field table
```

A generated field record contains pointers to the declaring class, field name, and descriptor together with metadata words and a mutable field-slot index.

A generated method record contains pointers to the declaring class, method name, and descriptor together with metadata words and a generated Thumb method pointer. One metadata word correlates with the Java argument slot count, including `this` for instance methods.

### Object layout and dispatch

Generated objects use this layout:

```text
generated object record
  +0x00 -> vtable/method pointer array
  +0x04 unknown runtime word
  +0x08 -> field slot array
```

Generated instance methods access fields and virtual methods directly:

```text
value = this->fields[field_slot]
target = object->vtable[virtual_method_slot + 1]
target(object, ...)
```

Field and virtual-method slots are patched during linking. The purpose of the leading vtable entry is not yet known.

## Link-time metadata

The generated image contains three class collections:

- a collection of generated application class pointers;
- a collection of external Java and WIPI classes referenced by generated code;
- a collection of generated classes whose members are exported for cross-class linking.

Each imported-class record divides flat name/descriptor tables into per-class ranges for static fields, virtual methods, and static methods.

Java import `0x14` receives 11 pointers to generated input tables and patch output areas:

| Argument | Table or role | Confidence |
| ---: | --- | --- |
| `r0` | imported classes | Observed |
| `r1` | field name/descriptor pairs | Observed |
| `r2` | static field name/descriptor pairs | Observed |
| `r3` | virtual method name/descriptor pairs | Observed |
| stack 0 | additional method metadata | Unknown |
| stack 1 | static method imports | Observed |
| stack 2 | output or offset base | Unknown |
| stack 3 | field slot output | Inferred |
| stack 4 | virtual method slot output | Inferred |
| stack 5 | imported static method output | Inferred |
| stack 6 | imported class or method target output | Unknown |

The Java runtime resolves each class and member by name and descriptor, then writes field slots, vtable slots, static storage references, or callable targets into the corresponding output tables. Generated methods consume these patched values directly.

Output tables for field and virtual-method slots use `u16` entries rather than byte or host-sized offsets.

## Java runtime interface

The following table lists the known subset of Java import table `0x64`:

| Index | Observed or inferred role |
| ---: | --- |
| `0x03` | bootstrap helper |
| `0x06` | bootstrap or class helper |
| `0x07` | establish the generated class table and runtime context |
| `0x09` | create or cache a Java string from UTF-16 data |
| `0x0b` | prepare or register a generated class |
| `0x0c` | obtain a runtime class handle |
| `0x0d` | ensure class initialization |
| `0x0e` | obtain an array type |
| `0x0f` | instantiate an object |
| `0x10` | instantiate an array |
| `0x11` | runtime helper |
| `0x12` | runtime helper |
| `0x13` | link a public or generated class |
| `0x14` | link imported classes and members |
| `0x1f` | frame or exception helper |
| `0x20` | frame or exception helper |
| `0x21` | frame or exception helper |
| `0x22` | raise `NullPointerException` |
| `0x23` | raise an array index exception |
| `0x25` | raise an arithmetic exception |
| `0x54` | generated method prologue |
| `0x55` | companion frame or runtime helper |
| `0x61` | runtime helper |
| `0x82` | Java application bootstrap setup |
| `0x83` | invoke or start the Java application |
| `0xe1` | runtime helper |
| `0xe2` | runtime helper |
| `0xfa` | runtime helper |

## ABI details

- Instance methods pass `this` in `r0`; later arguments follow the ARM calling convention.
- Static methods, 64-bit argument alignment, and native-method wrappers require further confirmation.
- Class descriptors contain several unknown fields and three generated callback pointers.
- One descriptor value appears to represent total inherited instance field slots rather than the declared field count.
- Generated Java string imports receive UTF-16 data together with a cache or output slot.
- Class initialization state is visible to generated code and is managed separately from metadata registration.
- Generated method prologue and exception helpers maintain ARM frame state used by Java exception paths.
- The raw array header and data layout are not yet known.
- Static Java storage and imported static member targets occupy ARM-visible slots patched during linking.
