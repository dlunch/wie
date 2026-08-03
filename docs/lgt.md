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
  +0x00 Java class access flags
  +0x04 -> next generated class record, or zero at the end of the list
  +0x08 -> class name
  +0x0c -> class word in the instance-field-initializer record, when present
  +0x10 -> superclass name
  +0x18 u16: total instance field slots including inherited fields
  +0x1a u16: generated-class link state
  +0x20 -> instance-field-initializer record, when present
  +0x28 -> implemented-interface name table, when present
  +0x2c -> member-linking callback
  +0x30 -> initialized-class lookup callback
  +0x34 -> runtime-class lookup callback
  +0x38 -> method table
  +0x3c -> field table
```

The three callbacks form two related paths:

```text
runtime-class lookup
  register the generated class if its link state is not ready
  resolve its runtime class from the generated class record and runtime context

initialized-class lookup
  obtain the runtime class
  run the generated class initializer if its initialization state is not ready

member linking
  match the runtime class to an exported generated-class record
  link its fields and methods and patch the generated slot tables
```

The member-linking callback is empty for classes that do not export members. It is not the Java class initializer. The instance field initializer referenced by the descriptor writes initial values to a newly created object's fields; it is separate from the `<init>` constructor method. The class initializer passed to Java import `0x0d` manages static class initialization.

Generated interface metadata has a different descriptor form. Its access flags include `ACC_INTERFACE | ACC_ABSTRACT`, but the descriptor pointer and callback fields do not follow the concrete-class offsets above and require separate layout analysis.

A generated field record contains pointers to the declaring class, field name, and descriptor together with flags, an unknown metadata word, and a mutable field slot.

A generated method record contains pointers to the declaring class, method name, and descriptor together with access flags, the Java argument slot count, additional metadata, and a generated Thumb method pointer. The argument slot count includes `this` for instance methods.

### Object layout and dispatch

Generated objects use this layout:

```text
generated object record
  +0x00 -> class dispatch table
  +0x04 unknown runtime word
  +0x08 -> field slot array

class dispatch table
  +0x00 unknown or reserved
  +0x04 virtual method slot 0
  +0x08 virtual method slot 1
  ...
```

Generated instance methods access fields and virtual methods directly:

```text
value = this->fields[field_slot]
dispatch_table = *(u32 **)object
target = dispatch_table[virtual_method_slot + 1]
target(object, ...)
```

Field and virtual-method slots are patched during linking. Generated dynamic-call sequences explicitly add four bytes between the dispatch-table base and virtual method slot zero. The purpose of the leading table entry is not yet known.

## Link-time metadata

The generated image contains three class collections:

- a collection of generated application class pointers;
- a collection of external Java and WIPI classes referenced by generated code;
- a collection of generated classes whose members are exported for cross-class linking.

Each imported or exported class record divides flat name/descriptor tables into per-class ranges for instance fields, static fields, virtual methods, interface methods, and non-virtual methods.

Java import `0x14` receives 11 pointers to generated input tables and patch output areas:

| Argument | Table or role | Confidence |
| ---: | --- | --- |
| `r0` | imported class records | Observed |
| `r1` | instance-field imports | Observed |
| `r2` | static-field imports | Observed |
| `r3` | virtual-method imports | Observed |
| stack 0 | interface-method imports | Inferred |
| stack 1 | non-virtual method imports | Observed |
| stack 2 | instance-field slot outputs | Inferred |
| stack 3 | static-field slot outputs | Inferred |
| stack 4 | virtual-method slot outputs | Observed |
| stack 5 | interface-method slot outputs | Inferred |
| stack 6 | non-virtual method target outputs | Observed |

The Java runtime resolves each class and member by name and descriptor, then writes field slots, virtual method slots, or callable targets into the corresponding output tables. Non-virtual method imports include constructors, static methods, and other calls whose target can be resolved without receiver-based dispatch. Slot outputs are `u16`; non-virtual method targets are ARM-callable pointers. Generated methods consume these patched values directly.

## Java runtime interface

The following table lists the known subset of Java import table `0x64`:

| Index | Observed or inferred role |
| ---: | --- |
| `0x03` | initialize Java application bootstrap state; exact contract unresolved |
| `0x06` | destroy the generated Java runtime context |
| `0x07` | create a runtime context from the generated class collection and runtime metadata |
| `0x09` | resolve and cache a Java string literal from UTF-16 data |
| `0x0b` | register or link a generated class record |
| `0x0c` | resolve a runtime class from a generated class record and runtime context |
| `0x0d` | ensure class initialization using a generated initializer callback |
| `0x0e` | obtain an array type |
| `0x0f` | instantiate an object |
| `0x10` | instantiate an array |
| `0x11` | no references observed |
| `0x12` | test whether a caught exception matches a class |
| `0x13` | link an exported generated class and patch its member slots |
| `0x14` | link imported classes and members |
| `0x1f` | push an exception-handler context |
| `0x20` | pop an exception-handler context |
| `0x21` | rethrow the current exception |
| `0x22` | raise `NullPointerException` |
| `0x23` | raise an array index exception |
| `0x25` | raise an arithmetic exception |
| `0x54` | generated method prologue |
| `0x55` | generated-code safepoint or runtime poll |
| `0x61` | store a reference-array element with runtime checks or barriers |
| `0x82` | set the Java application JAR path |
| `0x83` | start the Java application entry class |
| `0xe1` | obtain the `java/lang/String` runtime class |
| `0xe2` | obtain the `[Ljava/lang/String;` runtime array type |
| `0xfa` | no references observed |

## ABI details

- Instance methods pass `this` in `r0`; later arguments follow the ARM calling convention.
- Static methods, 64-bit argument alignment, and native-method wrappers require further confirmation.
- Concrete class descriptors contain additional packed metadata and runtime state whose roles are not yet known.
- The descriptor's instance field slot count includes inherited fields rather than only declared fields.
- Generated Java string imports receive UTF-16 data together with a cache or output slot.
- Class initialization state is visible to generated code and is managed separately from metadata registration.
- Generated method prologue, safepoint, and exception helpers maintain runtime state around generated ARM code.
- The raw array header and data layout are not yet known.
- Static Java storage and imported static member targets occupy ARM-visible slots patched during linking.
