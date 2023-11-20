const fs = require("fs");
const path = require("path");
const process = require("process");

// NOTE: Install dependencies with `npm i axios form-data`.
const axios = require("axios");
const FormData = require("form-data");

// NOTE: Provide your API key here.
const API_KEY = process.env.SINDRI_API_KEY || "";

const API_VERSION = "v1";
const API_URL = `https://forge.sindri.app/api/${API_VERSION}`;

const apiKeyQueryString = `?api_key=${API_KEY}`;
const headersJson = {
  Accept: "application/json",
  'Content-Type': 'application/json',
};

// Utility to poll a detail API endpoint until the status is `Ready` or `Failed`.
// Returns the response object of the final request or throws an error if the timeout is reached.
async function pollForStatus(endpoint, timeout = 20 * 60) {
  for (let i = 0; i < timeout; i++) {
    const response = await axios.get(API_URL + endpoint + apiKeyQueryString, {
      headers: headersJson,
      validateStatus: (status) => status === 200,
    });

    const status = response.data.status;
    if (["Ready", "Failed"].includes(status)) {
      console.log(`Poll exited after ${i} seconds with status: ${status}`);
      return response;
    }

    await new Promise((r) => setTimeout(r, 1000));
  }

  throw new Error(`Polling timed out after ${timeout} seconds.`);
}

async function main() {
  try {
    // Load the circuit's `tar.gz` file.
    const circuitFilePath = path.join(
      __dirname,
      "..",
      "circom",
      "multiplier2",
      "multiplier2.tar.gz",
    );
    const circuitFileBuffer = fs.readFileSync(circuitFilePath);
    const uploadFormData = new FormData();
    uploadFormData.append("files", circuitFileBuffer, {
      filename: "upload.tar.gz",
    });


    // Create a new circuit.
    console.log("1. Creating circuit...");
    const createResponse = await axios.post(
      API_URL + "/circuit" + apiKeyQueryString,
      {
        circuit_name: "multiplier2",
      },
      uploadFormData,
      {
        validateStatus: (status) => status === 201 },
    );
    const circuitId = createResponse.data.circuit_id;

    // await axios.post(
    //   API_URL + `/circuit/${circuitId}/uploadfiles` + apiKeyQueryString,
    //   uploadFormData,
    //   { validateStatus: (status) => status === 201 },
    // );

    // // Initiate circuit compilation.
    // await axios.post(
    //   API_URL + `/circuit/${circuitId}/compile` + apiKeyQueryString,
    //   { validateStatus: (status) => status === 201 },
    // );

    // Poll the circuit detail endpoint until the compilation status is `Ready` or `Failed`.
    const {
      data: { status: compileStatus },
    } = await pollForStatus(`/circuit/${circuitId}/detail`);

    // Check for compilation issues.
    if (compileStatus === "Failed") {
      throw new Error("Circuit compilation failed.");
    }
    console.log("Circuit compilation succeeded!");

    // Initiate proof generation.
    console.log("2. Proving circuit...");
    const proofInput = JSON.stringify({ a: "7", b: "42" });
    const proveResponse = await axios.post(
      API_URL + `/circuit/${circuitId}/prove` + apiKeyQueryString,
      { proof_input: proofInput },
      { headers: headersJson, validateStatus: (status) => status === 201 },
    );
    const proofId = proveResponse.data.proof_id;

    // Poll the proof detail endpoint until the compilation status is `Ready` or `Failed`.
    const proofDetailResponse = await pollForStatus(`/proof/${proofId}/detail`);

    // Check for proving issues.
    const proofDetailStatus = proveResponse.data.status;
    if (proofDetailStatus === "Failed") {
      throw new Error("Proving failed");
    }

    // Retrieve output from the proof.
    const publicOutput = proofDetailResponse.data.public[0];
    console.log(`Circuit proof output signal: ${publicOutput}`);
  } catch (error) {
    if (error instanceof Error) {
      console.error(error.message);
    } else {
      console.error("An unknown error occurred.");
    }
  }
}

if (require.main === module) {
  main();
}
