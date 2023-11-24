import * as curves from "./src/curves.js";
import { BigBuffer, Scalar } from "ffjavascript";


let curve = await curves.getCurveFromName("BN254");
const Fr = curve.Fr;
const G1 = curve.G1;
const G2 = curve.G2;
let n = parseInt(process.argv[2]);
console.log("Current threads:", curve.tm.concurrency);
console.log("Number of MSM elements:", n);
const scalars = new BigBuffer(n * Fr.n8);
const g1affines = new BigBuffer(n * G1.F.n8*2);
const g2affines = new BigBuffer(n * G2.F.n8*2);
for (let i = 0; i < n; i++) {
    const num = Fr.e(i+1);
    scalars.set(Fr.fromMontgomery(num), i*Fr.n8);
    g1affines.set(G1.toAffine(G1.g), i*G1.F.n8*2);
    g2affines.set(G2.toAffine(G2.g), i*G2.F.n8*2);
}
const start = performance.now();
await G1.multiExpAffine(g1affines, scalars);
const end = performance.now();
console.log(`MSM on G1 cost: ${end - start}ms`);

const start2 = performance.now();
await G2.multiExpAffine(g1affines, scalars);
const end2 = performance.now();
console.log(`MSM on G2 cost: ${end2 - start2}ms`);


process.exit()