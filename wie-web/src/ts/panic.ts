export let rustPanic: Error | undefined;

export function recordPanic(message: string): void {
  rustPanic = new Error(message);
  rustPanic.name = "RustPanic";
}
