# KTF Platform Architecture

KTF (Korea Telecom Freetel) is a WIPI-based mobile platform.

## App Structure

A KTF app consists of:
- A JAR containing:
  - `client.bin{bss_size}` — a raw ARM binary with all AOT-compiled Java classes
  - App resources
- An ADF file (separate from the JAR) — app descriptor (AID, PID, MClass)

There are no `.class` files in the JAR. All Java classes are AOT-compiled and exist only inside `client.bin` as ARM code + metadata.

Java source is compiled to bytecode, then **AOT-compiled into ARM native code** by the KTF toolchain. The resulting `client.bin` contains:
- AOT-compiled method bodies (ARM machine code)
- Java class/method metadata structures (platform-specific binary format)

Both pure Java apps and Clets (C native apps) share the same binary format. Clets are Java apps that call C native functions through JNI — the native code is included in the same `client.bin`.

## Platform Interfaces

The platform provides two kinds of integration points to the app binary during initialization:

### Direct Java runtime callbacks

Passed directly as function pointers for operations such as:
- throwing a Java exception
- allocating Java objects and arrays
- loading Java classes by name
- checking type relationships
- obtaining additional interface tables
- allocating memory

### Java bridge table

Obtained through the platform's interface lookup. This table provides helpers for:
- calling Java methods from AOT-generated ARM code
- calling JNI native methods
- resolving methods by class and signature
- registering classes with the runtime
- registering Java string constants

### WIPI C Interface

Obtained through the same interface lookup mechanism. This is the standard WIPI C API providing kernel, graphics, database, input, and timer functions.

## Initialization Sequence

1. Platform loads `client.bin` into memory
2. Calls the binary's entrypoint with the BSS size and gets back a top-level executable descriptor
3. Calls the executable's initialization function with a set of platform-owned context blocks, including:
   - exception handling state
   - JVM context storage
   - primitive type descriptors
   - Java support callbacks (see above)
4. Calls the app-level initialization function
5. Calls `Main.main()` to start the app

## Method Dispatch

AOT-compiled methods live in the ARM binary. When one method calls another:
1. The caller uses a platform-provided Java-call trampoline with the target method address and arguments
2. The trampoline calls the target method in a new execution frame
3. The target method runs as ARM code and returns its result

For JNI native methods, a separate native-call trampoline is used instead, passing arguments through a data pointer.

## Exception Handling

KTF uses a setjmp/longjmp mechanism:
- Each try block saves registers (r4-lr) into an exception handler record
- The `current_java_exception_handler` pointer in the exception context tracks the active handler
- On exception, the runtime restores the saved registers and jumps to the catch target

## How We Emulate This

- **ARM execution**: `wie_core_arm::ArmCore` emulates ARM instructions. Rust callbacks are registered at sentinel addresses; when the ARM engine hits one, the `run_function` loop dispatches to Rust.
- **JVM**: `KtfJvmImplementation` reads class/method metadata directly from ARM memory. The Rust JVM delegates to ARM code for AOT-compiled method bodies via `core.run_function()`.
- **Class loading**: `KtfClassLoader` calls the native `fn_get_class` to discover classes defined in the ARM binary.
- **Trampolines**: `java_jump_1/2/3` and `call_native` each call `core.run_function()`, creating nested execution frames.
- **Exception unwind**: When `handle_exception` finds a catch handler, it returns `WieError::JavaExceptionUnwind` instead of a normal result. This error propagates through nested `run_function` frames (skipping their context restore), and the trampoline converts it back into a `JavaMethodResult` that resumes execution through the restore helper.
