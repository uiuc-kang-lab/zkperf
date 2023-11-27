import * as curves from "./src/curves.js";
import { BigBuffer, Scalar } from "ffjavascript";


let curve = await curves.getCurveFromName("BN254");
const Fr = curve.Fr;
let n = parseInt(process.argv[2]);
console.log("Current threads:", curve.tm.concurrency);
console.log("Number of FFT/IFFT elements:", n);
const scalars = new BigBuffer(n * Fr.n8);
for (let i = 0; i < n; i++) {
    const num = Fr.e(i+1);
    scalars.set(num, i*Fr.n8);
}
const start = Date.now();
const a = await Fr.fft(scalars, "", "");
const end = Date.now();
console.log(`FFT on Fr cost: ${end - start}ms`);

const start2 = Date.now();
Fr.ifft(a, "", "");
const end2 = Date.now();
console.log(`IFFT on Fr cost: ${end2 - start2}ms`);

process.exit()