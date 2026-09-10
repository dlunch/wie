const input = JSON.parse(require("node:fs").readFileSync(0, "utf8"));
const memory = new WebAssembly.Memory({ initial: 1 });
const frame = new Uint32Array(memory.buffer, 0, 24);
frame.set(input.frame);
const effects = [];
const pages = input.pages.map(([base, bytes], index) => {
    const mapped = new Uint8Array(memory.buffer, 4096 + index * 1024, bytes.length);
    mapped.set(bytes);
    return [base, mapped];
});
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
            return 0;
        },
        sample_prepare: (_, pc, cpsr, r7) => {
            // Direct generated stores must be visible when the Rust sampler reads the stack.
            effects.push({ Memory: pages.map(([base, bytes]) => [base, Array.from(bytes)]) });
            effects.push({ Sample: [pc >>> 0, cpsr >>> 0, r7 >>> 0, frame[23]] });
        },
        word_range: (_, address, words) => {
            address >>>= 0;
            const admitted = Array.from({ length: words }, (_, index) => (address + 4 * index) >>> 0)
                .every(word => find(word, 4)) ? 1 : 0;
            effects.push({ WordRange: [address, words, admitted] });
            if (!admitted) return 0n;
            const [base, bytes] = find(address, 4);
            const firstLength = Math.min(words * 4, bytes.length - (address - base));
            const first = bytes.byteOffset + address - base;
            let second = 0;
            if (firstLength < words * 4) {
                const next = (address + firstLength) >>> 0;
                const [nextBase, nextBytes] = find(next, 4);
                second = nextBytes.byteOffset + next - nextBase;
            }
            frame[22] = firstLength;
            return BigInt(first) | (BigInt(second) << 32n);
        },
    },
});
const exit = instance.exports[`region_${input.slot}`](0, 0);
effects.push({ Memory: pages.map(([base, bytes]) => [base, Array.from(bytes)]) });
process.stdout.write(JSON.stringify({ frame: Array.from(frame), effects, exit }));
