const assert = require('node:assert/strict');
const input = require('node:fs').readFileSync(0);
const length = input.readUInt32LE(0);
const bytes = input.subarray(4, 4 + length);
assert(WebAssembly.validate(bytes));
const module_ = new WebAssembly.Module(bytes);
assert.deepEqual(WebAssembly.Module.imports(module_).map(({module, name, kind}) => [module,name,kind]), [
    ['wie','memory','memory'], ['wie','load','function'], ['wie','store','function'], ['wie','sample_prepare','function'],
]);
const helpers = new WebAssembly.Instance(new WebAssembly.Module(input.subarray(4 + length))).exports;
const memory = helpers.memory;
const F = 256, A = 1234;
Object.defineProperty(globalThis, 'samples', {get() {
    const count = new DataView(memory.buffer).getUint32(144, true);
    assert(count <= 1, 'generated code must exit after selected retirement');
    return count ? [Array.from(new Uint32Array(memory.buffer,128,4))] : [];
}});
const instance = new WebAssembly.Instance(module_, {wie: helpers});
const entry = instance.exports.region_0;
function get(offset) { return new DataView(memory.buffer).getUint32(F + offset, true); }
function set(offset, value) { new DataView(memory.buffer).setUint32(F + offset, value, true); }
function reset({pc = 0x1000, end = 0xfffffff0, budget = 100, sample = 100, cpsr = 0x30} = {}) {
    new Uint8Array(memory.buffer, F, 92).fill(0);
    set(28,77); set(60,pc); set(64,cpsr); set(68,end); set(72,budget); set(76,sample);
    new Uint8Array(memory.buffer,128,20).fill(0);
}
