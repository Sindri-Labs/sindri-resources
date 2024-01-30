const fs = require('fs');
const program = require('commander');
const path = require("path");
const process = require("process");

program
    .option('-p, --proof <string>', 'Proof')
    .option('-m, --pub <string>', 'Public inputs')
    .parse(process.argv);

if (!program.opts().proof || !program.opts().pub) {
    console.error('Error: Please provide proof and public inputs.');
    process.exit(1);
}
function p256(n) {
    let nstr = n.toString(16);
    while (nstr.length < 64) nstr = "0"+nstr;
    nstr = `"0x${nstr}"`;
    return nstr;
}

function unstringifyBigInts(o) {
    if (typeof o == "string" && /^[0-9]+$/.test(o)) {
        return BigInt(o);
    } else if (typeof o == "string" && /^0x[0-9a-fA-F]+$/.test(o)) {
        return BigInt(o);
    } else if (Array.isArray(o)) {
        return o.map(unstringifyBigInts);
    } else if (typeof o == "object") {
        if (o === null) return null;
        const res = {};
        const keys = Object.keys(o);
        keys.forEach((k) => {
            res[k] = unstringifyBigInts(o[k]);
        });
        return res;
    } else {
        return o;
    }
}

async function groth16ExportSolidityCallData(proofString, pubString) {
    let _proof = JSON.parse(proofString);
    let _pub = JSON.parse(pubString);


    const proof = unstringifyBigInts(_proof);
    const pub = unstringifyBigInts(_pub);

    let inputs = "";
    for (let i=0; i<pub.length; i++) {
        if (inputs != "") inputs = inputs + ",";
        inputs = inputs + p256(pub[i]);
    }

    let S;
    S=`[${p256(proof.pi_a[0])}, ${p256(proof.pi_a[1])}],` +
        `[[${p256(proof.pi_b[0][1])}, ${p256(proof.pi_b[0][0])}],[${p256(proof.pi_b[1][1])}, ${p256(proof.pi_b[1][0])}]],` +
        `[${p256(proof.pi_c[0])}, ${p256(proof.pi_c[1])}],` +
        `[${inputs}]`;

    console.log(S);
}

groth16ExportSolidityCallData(program.opts().proof, program.opts().pub);