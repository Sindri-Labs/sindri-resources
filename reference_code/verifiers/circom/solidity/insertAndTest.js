const fs = require("fs");
const { exec } = require("child_process");

// Insert calldata into groth16verifier.test.js
const calldata = fs.readFileSync("calldata.txt", "utf-8").trim().split(",");
const testFileContent = fs.readFileSync("./test/groth16verifier.test.js", "utf-8");
const updatedContent = testFileContent.replace(
  "// replace with calldata for your verifier",
  `const [pa, pb, pc, pub] = [${calldata}];`
);
fs.writeFileSync("./test/groth16verifier.test.js", updatedContent);

// Run npx hardhat test
exec("npx hardhat test", (error, stdout, stderr) => {
  if (error) {
    console.error(`Error: ${error.message}`);
    return;
  }
  if (stderr) {
    console.error(`Stderr: ${stderr}`);
    return;
  }
  console.log(`Stdout: ${stdout}`);
});
