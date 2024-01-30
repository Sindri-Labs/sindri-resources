const { expect } = require("chai");

describe("Groth16Verifier", function() {
    it("Should verify a valid proof", async function() {
        const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
        const verifier = await Groth16Verifier.deploy();

        // replace with calldata for your verifier

        expect(await verifier.verifyProof(pa, pb, pc, pub)).to.equal(true);

    });
});