// Generates the placeholder app icon: a keycap with a lit top face on a darker edge.
// Replace src-tauri/icon-source.png with the owner's logo when there is one, then run `npm run icon`
// (the make-icon step is skipped if you call `tauri icon` directly).
import fs from 'node:fs';
import zlib from 'node:zlib';

const N = 1024;
const SS = 3; // supersampling per axis
const rgba = new Uint8Array(N * N * 4);

// Signed distance to a rounded box centered at (cx, cy) with half size (hw, hh) and radius r.
const box = (x, y, cx, cy, hw, hh, r) => {
  const qx = Math.abs(x - cx) - (hw - r);
  const qy = Math.abs(y - cy) - (hh - r);
  return Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - r;
};
const mix = (a, b, t) => a.map((v, i) => v + (b[i] - v) * t);

const paper = [0xf3, 0xee, 0xe4];
const edge = [0xcf, 0xc3, 0xad];
const top = [0xff, 0xfd, 0xf8];
const amber = [0xe9, 0xa2, 0x3b];

function sample(x, y) {
  const outer = box(x, y, 512, 540, 400, 380, 120); // whole keycap
  if (outer > 0) return null;
  const face = box(x, y, 512, 500, 340, 310, 90); // top face
  if (face <= 0) {
    const glow = box(x, y, 512, 500, 150, 120, 50); // the lit legend area
    const base = mix(top, paper, Math.min(1, Math.max(0, (y - 240) / 520)) * 0.35);
    return glow <= 0 ? amber : base;
  }
  return edge;
}

for (let py = 0; py < N; py++) {
  for (let px = 0; px < N; px++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const c = sample(px + (sx + 0.5) / SS, py + (sy + 0.5) / SS);
        if (c) { r += c[0]; g += c[1]; b += c[2]; a++; }
      }
    }
    const i = (py * N + px) * 4;
    const n = SS * SS;
    if (a) { rgba[i] = r / a; rgba[i + 1] = g / a; rgba[i + 2] = b / a; }
    rgba[i + 3] = Math.round((a / n) * 255);
  }
}

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
const crc = (buf) => {
  let c = 0xffffffff;
  for (const byte of buf) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
};
const chunk = (type, data) => {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type), data]);
  const sum = Buffer.alloc(4);
  sum.writeUInt32BE(crc(body));
  return Buffer.concat([len, body, sum]);
};

const raw = Buffer.alloc((N * 4 + 1) * N);
for (let y = 0; y < N; y++) {
  raw[y * (N * 4 + 1)] = 0;
  Buffer.from(rgba.buffer, y * N * 4, N * 4).copy(raw, y * (N * 4 + 1) + 1);
}
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(N, 0);
ihdr.writeUInt32BE(N, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // RGBA
const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);
fs.writeFileSync(new URL('../src-tauri/icon-source.png', import.meta.url), png);
console.log('wrote src-tauri/icon-source.png', png.length, 'bytes');
