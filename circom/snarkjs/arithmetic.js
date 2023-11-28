import * as curves from "./src/curves.js";
import * as fs from "fs";

let curve = await curves.getCurveFromName("BN254");
const Fr = curve.Fr;
let n = parseInt(process.argv[2]);
let measurement = {};
measurement["threads"] = curve.tm.concurrency
measurement["size"] = n;
let multiplier = Fr.e(Math.floor(n/2));
const scalars = [];
const mul_start = Date.now();
for (let i = 0; i < n; i++) {
    scalars.push(Fr.e(i+1) * multiplier);
}
const mul_end = Date.now();
measurement["mul"] = mul_end - mul_start;
let result = Fr.e(0);

const add_start = Date.now();
for (let i = 0; i < n; i++) {
    result = result + scalars[i];
}
const add_end = Date.now();
measurement["add"] = add_end - add_start;

fs.writeFileSync(`circom_arithmetic_${n}.json`, JSON.stringify(measurement), function(err) {
    console.log("Writing error");
});

process.exit()
