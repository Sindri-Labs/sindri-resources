const fs = require('fs');
const program = require('commander');
const ejs = require('ejs');
const path = require("path");
const process = require("process");

program
    .option('-f, --file <path>', 'Path to the zkey file')
    .option('-s, --string <string>', 'Verification key as a string')
    .option('-o, --output <path>', 'Output path for the verifier contract')
    .parse(process.argv);

if (program.opts().file && program.opts().string) {
    console.error('Error: Please provide either a file or a string, not both.');
    process.exit(1);
}

async function fileExists(file) {
    return fs.promises.access(file, fs.constants.F_OK)
        .then(() => true)
        .catch(() => false);
}

async function generateSolidityVerifier(input, isFile, outputPath) {

    const tempFilePath = isFile ? input : 'verification_key.json';
    let verifierFilePath = outputPath || 'verifier.sol';

    let verificationKey = isFile ? JSON.parse(fs.readFileSync(tempFilePath, 'utf8')) : JSON.parse(input);

    const templates = {};
    if (await fileExists(path.join(process.cwd(), "templates"))) {
        templates.groth16 = await fs.promises.readFile(path.join(process.cwd(), "templates", "verifier_groth16.sol.ejs"), "utf8");
        templates.plonk = await fs.promises.readFile(path.join(process.cwd(), "templates", "verifier_plonk.sol.ejs"), "utf8");
        templates.fflonk = await fs.promises.readFile(path.join(process.cwd(), "templates", "verifier_fflonk.sol.ejs"), "utf8");
    } else {
        console.log((path.join(__dirname, "templates")));
        throw new Error("Templates not found");
    }

    let template = templates[verificationKey.protocol];

    let verifier =  ejs.render(template, verificationKey);

    if (!outputPath) {
        console.log(verifier);
    } else {
        console.log('Solidity verifier generated successfully to file: ', outputPath);
    }

    if (!outputPath) {
        if (!(await fileExists(path.join(process.cwd(), "contracts")))) {
            fs.mkdirSync(path.join(process.cwd(), "contracts"));
        }
        verifierFilePath = path.join(process.cwd(), "contracts", "verifier.sol");
    }

    fs.writeFileSync(verifierFilePath, verifier);
}

const input = program.opts().file || program.opts().string;
const isFile = Boolean(program.opts().file);
generateSolidityVerifier(input, isFile, program.opts().output);