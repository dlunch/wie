const input = JSON.parse(require("node:fs").readFileSync(0, "utf8"));
const memory = new WebAssembly.Memory({ initial: 1 });
const frame = new Uint32Array(memory.buffer, 0, 23);
frame.set(input.frame);
const effects = [];
const pages = input.pages.map(([base, bytes]) => [base, Uint8Array.from(bytes)]);
function find(address, width) {
    return address % width === 0 && pages.find(([base, bytes]) => address >= base && address + width <= base + bytes.length);
}
const instance = new WebAssembly.Instance(new WebAssembly.Module(Uint8Array.from(input.wasm)), {
    wie: {
        memory,
        load: (_, address, width, out) => {
            address >>>= 0;
            const page = find(address, width);
            if (!page) return 1;
            const [base, bytes] = page;
            let value = 0;
            for (let i = 0; i < width; i++) value |= bytes[address - base + i] << (8 * i);
            new DataView(memory.buffer).setUint32(out, value >>> 0, true);
            return 0;
        },
        store: (_, address, width, value) => {
            address >>>= 0;
            const page = find(address, width);
            if (!page) return 1;
            const [base, bytes] = page;
            for (let i = 0; i < width; i++) bytes[address - base + i] = value >>> (8 * i);
            effects.push({ Store: [address, width, value >>> 0] });
            return input.source.some(stamp => stamp.page === ((address & 0xffff0000) >>> 0)) ? 3 : 0;
        },
        sample_prepare: (_, pc, cpsr, r7) => effects.push({ Sample: [pc >>> 0, cpsr >>> 0, r7 >>> 0] }),
    },
});
const exit = instance.exports[`region_${input.slot}`](0, 0);
process.stdout.write(JSON.stringify({ frame: Array.from(frame), effects, exit }));
