#!/usr/bin/python3
import json
import os
import sys
import time
from pathlib import Path
from pprint import pformat

try:
    import requests  # pip install requests
except ModuleNotFoundError:
    print("'requests' module not installed. Goodbye.")
    sys.exit(1)

DEFAULT_FORGE_API_URL = "https://forge.sindri.app/api/v1/"


class ForgeApiError(Exception):
    """Custom Exception for Forge API Errors"""

    pass


class ForgeAPIUtil:
    """
    Utility class for contacting the Forge API.

    Parameters:
    - `api_key: str`: Your Forge API Key (required)
    - `api_url: str`: Forge API Url. (default=https://forge.sindri.app/api/v1/)
    - `verbose: bool`: Verbose print (default=True)

    ### Usage
    API authentication testing occurs upon calling the .run() method
    by using the `api_url` and `api_key` variables.

    ```python
    # Collect inputs for circuit compilation and proving
    circuit_name = "Circom multiplier2"
    circuit_type = "Circom C Groth16 bn254"
    circuit_code_tar_path = "circom/multiplier2.tar.gz"
    # Set proof input as a dict. Here we read from a JSON file.
    proof_input_dict = {}
    proof_input_file_path = "circom/multiplier2_input.json"
    with open(proof_input_file_path, "r") as f:
        proof_input_dict = json.load(f)

    # Run Forge
    api_key = ""  # Obtained elsewhere
    forge_api_util = ForgeAPIUtil(api_key)
    circuit_id = forge_api_util.create_circuit(circuit_name, circuit_type, circuit_code_tar_path)
    proof_id = forge_api_util.prove_circuit(circuit_id, proof_input_dict)
    valid = forge_api_util.verify_proof(proof_id)
    ```
    """

    def __init__(
        self,
        api_key: str,
        api_url: str = DEFAULT_FORGE_API_URL,
        verbose: bool = True,
    ):
        self.polling_interval_sec = 1  # polling interval for circuit compilation & proving
        self.max_polling_iterations = 1800
        self.verbose = verbose
        self.api_url = api_url
        self.headers_json = {
            "Accept": "application/json",
            "Authorization": f"Bearer {api_key}",
        }
        if self.verbose:
            print(f"Forge Api Url: {api_url}")
            print(f"Forge Api Key: {api_key}\n")

    def _hit_api(self, method: str, path: str, data=None, files=None) -> tuple[int, dict]:
        """
        Hit the Forge API.

        Return
        - int:  response status code
        - dict: response json

        Raises an Exception if:
        - response is None
        - cannot connect to the API
        - response cannot be JSON decoded
        - invalid API Key
        """

        # Initialize data if not provided
        if data is None:
            data = {}

        # Construct the full path to the API endpoint.
        full_path = os.path.join(self.api_url, path)
        try:
            if method == "POST":
                response = requests.post(
                    full_path, headers=self.headers_json, data=data, files=files
                )
            elif method == "GET":
                response = requests.get(full_path, headers=self.headers_json, params=data)
            elif method == "DELETE":
                response = requests.delete(full_path, headers=self.headers_json, data=data)
            else:
                raise ForgeApiError("Invalid request method")
        except requests.exceptions.ConnectionError:
            # Raise a clean exception and suppress the original exception's traceback.
            raise ForgeApiError("Unable to connect to the Forge API.") from None

        if response is None:
            raise ForgeApiError(
                f"No response received. method={method}, path={full_path},"
                f" data={data} headers={self.headers_json}, files={files}"
            )
        if response.status_code == 401:
            raise ForgeApiError("401 - Invalid API Key.")
        elif response.status_code == 404:
            raise ForgeApiError("404 - Not found.")
        else:
            # Decode JSON response
            try:
                response_json = response.json()
            except json.decoder.JSONDecodeError:
                raise ForgeApiError(
                    f"Unexpected Error. Unable to decode response as JSON."
                    f" status={response.status_code} response={response.text}"
                ) from None
        return response.status_code, response_json

    def create_circuit(
        self,
        circuit_name: str,
        circuit_type: str,
        circuit_code_tar_path: str,
    ) -> str:
        """
        Create a circuit, upload the circuit code tarfile,
        submit the compile request, poll until the circuit is Ready.

        Return:
        - str: circuit_id

        Raises Exception if
        - Invalid API Key
        - Unable to connect to the API
        - Failed circuit compilation
        """

        # Return value
        circuit_id = ""  # set later

        # 1. Create a circuit, obtain a circuit_id.
        if self.verbose:
            print("Circuit: Create")
            print(f"    circuit_name: {circuit_name}")
            print(f"    circuit_type: {circuit_type}")
            print(f"    targz_path:   {circuit_code_tar_path}")
        response_status_code, response_json = self._hit_api(
            "POST",
            "circuit/create",
            data={"circuit_name": circuit_name, "circuit_type": circuit_type},
        )
        if response_status_code != 201:
            raise ForgeApiError(
                f"Unable to create a new circuit."
                f" status={response_status_code} response={response_json}"
            )
        # Obtain circuit_id
        circuit_id = response_json.get("circuit_id", "")
        if self.verbose:
            print(f"    circuit_id:   {circuit_id}")

        # 2. Upload your local (compressed) circuit directory
        if self.verbose:
            print("Circuit: Upload files")
        files = {"files": open(circuit_code_tar_path, "rb")}
        response_status_code, response_json = self._hit_api(
            "POST",
            f"circuit/{circuit_id}/uploadfiles",
            files=files,
        )
        if response_status_code != 201:
            raise ForgeApiError(
                f"Unable to upload circuit files."
                f" status={response_status_code} response={response_json}"
            )

        # 3. Submit async compile command
        if self.verbose:
            print("Circuit: Compile")
        response_status_code, response_json = self._hit_api("POST", f"circuit/{circuit_id}/compile")
        if response_status_code != 201:
            raise ForgeApiError(
                f"Unable to compile circuit."
                f" status={response_status_code} response={response_json}"
            )

        # 4. Poll circuit detail until it has a status of Ready/Failed
        if self.verbose:
            print("Circuit: Poll until ready")
        for _ in range(self.max_polling_iterations):
            response_status_code, response_json = self._hit_api(
                "GET",
                f"circuit/{circuit_id}/detail",
                data={"include_verification_key": True},
            )
            if response_status_code != 200:
                raise ForgeApiError(
                    f"Failure to poll circuit detail."
                    f" status={response_status_code} response={response_json}"
                )

            circuit_status = response_json.get("status", "")
            if circuit_status == "Failed":
                raise ForgeApiError(
                    f"Circuit compilation failed."
                    f" status={response_status_code} response={response_json}"
                )
            if circuit_status == "Ready":
                break
            time.sleep(self.polling_interval_sec)
        else:
            raise ForgeApiError(
                f"Circuit compile polling timed out."
                f" status={response_status_code} response={response_json}"
            )

        if self.verbose:
            print(f"Circuit Detail: \n{pformat(response_json, indent=4)}\n\n")

        # Circuit compilation success!
        return circuit_id

    def prove_circuit(self, circuit_id: str, proof_input: dict) -> str:
        """
        Prove a circuit given a circuit_id and a proof_input.

        Return
        - str: proof_id

        Raises Exception if
        - Invalid API Key
        - Unable to connect to the API
        - Circuit does not exist
        - Circuit is not Ready
        """

        # Return values
        proof_id = ""

        # Convert the proof_input into a json string
        proof_input_json_str = json.dumps(proof_input)

        # 1. Submit a proof, obtain a proof_id.
        if self.verbose:
            print("Prove circuit")
        response_status_code, response_json = self._hit_api(
            "POST",
            f"circuit/{circuit_id}/prove",
            data={
                "proof_input": proof_input_json_str,
            },
        )
        if response_status_code != 201:
            raise ForgeApiError(
                f"Unable to prove circuit."
                f" status={response_status_code} response={response_json}"
            )
        # Obtain proof_id
        proof_id = response_json.get("proof_id", "")
        if self.verbose:
            print(f"    proof_id:     {proof_id}")

        # 2. Poll proof detail until it has a status of Ready/Failed
        if self.verbose:
            print("Proof: Poll until ready")
        for _ in range(self.max_polling_iterations):
            response_status_code, response_json = self._hit_api(
                "GET",
                f"proof/{proof_id}/detail",
                data={
                    "include_proof_input": True,
                    "include_public": True,
                    "include_verification_key": True,
                    "include_proof": True,
                },
            )
            if response_status_code != 200:
                raise ForgeApiError(
                    f"Failure to poll proof detail."
                    f" status={response_status_code} response={response_json}"
                )

            proof_status = response_json.get("status", "")
            if proof_status == "Failed":
                raise ForgeApiError(
                    f"Prove circuit failed."
                    f" status={response_status_code} response={response_json}"
                )
            if proof_status == "Ready":
                break
            time.sleep(self.polling_interval_sec)
        else:
            raise ForgeApiError(
                f"Prove circuit polling timed out."
                f" status={response_status_code} response={response_json}"
            )

        if self.verbose:
            print(f"Proof Detail: \n{pformat(response_json, indent=4)}\n\n")

        # Prove circuit success!
        return proof_id

    def verify_proof(self, proof_id: str) -> str:
        """
        Verify a proof given a proof_id.

        Return
        - bool: valid

        Raises Exception if
        - Invalid API Key
        - Unable to connect to the API
        - Proof does not exist
        - Proof is not Ready
        """

        # 3. Verify the proof
        if self.verbose:
            print("Verify proof")
        response_status_code, response_json = self._hit_api(
            "GET",
            f"proof/{proof_id}/verify",
        )
        if response_status_code != 200:
            raise ForgeApiError(
                f"Failure to verify the proof."
                f" status={response_status_code} response={response_json}"
            )

        valid = response_json.get("success", False)
        if self.verbose:
            print(f"    valid:      {valid}\n\n")

        # Proof verification computation successful
        return valid

    def get_all_circuits(self) -> list[dict]:
        """Get all circuits."""
        if self.verbose:
            print("Circuit: Get all")
        response_status_code, response_json = self._hit_api(
            "GET",
            "circuit/list",
            data={"include_verification_key": True},
        )
        if response_status_code != 200:
            raise ForgeApiError(
                f"Unable to fetch circuits."
                f" status={response_status_code} response={response_json}"
            )
        circuit_detail_list = response_json
        if self.verbose:
            print(f"{pformat(circuit_detail_list, indent=4)}\n\n")
        return circuit_detail_list  # type: ignore

    def get_all_proofs(self) -> list[dict]:
        """Get all proofs."""
        if self.verbose:
            print("Proof: Get all")
        response_status_code, response_json = self._hit_api(
            "GET",
            "proof/list",
            data={
                "include_proof_input": True,
                "include_public": True,
                "include_verification_key": True,
                "include_proof": True,
            },
        )
        if response_status_code != 200:
            raise ForgeApiError(
                f"Unable to fetch proofs."
                f" status={response_status_code} response={response_json}"
            )
        proof_detail_list = response_json
        if self.verbose:
            print(f"{pformat(proof_detail_list, indent=4)}\n\n")
        return proof_detail_list  # type: ignore

    def get_all_circuit_proofs(self, circuit_id: str) -> list[dict]:
        """Get all proofs for a circuit_id."""
        if self.verbose:
            print("Circuit: Get all proofs")
        response_status_code, response_json = self._hit_api(
            "GET",
            f"circuit/{circuit_id}/proofs",
            data={
                "include_proof_input": True,
                "include_public": True,
                "include_verification_key": True,
                "include_proof": True,
            },
        )
        if response_status_code != 200:
            raise ForgeApiError(
                f"Unable to fetch proofs for circuit_id={circuit_id}."
                f" status={response_status_code} response={response_json}"
            )
        proof_detail_list = response_json
        if self.verbose:
            print(f"{pformat(proof_detail_list, indent=4)}\n\n")
        return proof_detail_list  # type: ignore

    def get_proof(self, proof_id: str) -> dict:
        """Get proof by proof_id."""
        if self.verbose:
            print("Proof: Get proof detail")
        response_status_code, response_json = self._hit_api(
            "GET",
            f"proof/{proof_id}/detail",
            data={
                "include_proof_input": True,
                "include_public": True,
                "include_verification_key": True,
                "include_proof": True,
            },
        )
        if response_status_code != 200:
            raise ForgeApiError(
                f"Unable to fetch proof_id={proof_id}."
                f" status={response_status_code} response={response_json}"
            )
        proof_detail = response_json
        if self.verbose:
            print(f"{pformat(proof_detail, indent=4)}\n\n")
        return proof_detail

    def get_circuit(self, circuit_id: str) -> dict:
        """Get circuit by circuit_id."""
        if self.verbose:
            print("Circuit: Get circuit detail")
        response_status_code, response_json = self._hit_api(
            "GET",
            f"circuit/{circuit_id}/detail",
            data={"include_verification_key": True},
        )
        if response_status_code != 200:
            raise ForgeApiError(
                f"Unable to fetch circuit_id={circuit_id}."
                f" status={response_status_code} response={response_json}"
            )
        circuit_detail = response_json
        if self.verbose:
            print(f"{pformat(circuit_detail, indent=4)}\n\n")
        return circuit_detail


def print_sindri_logo():
    # https://ascii-generator.site/
    print(
        """
         Sindri Labs' Forge API
                  =-.
                  ***=
                 +****+
               .+******=
              .*****+***  =
              +***+--+**.-*:
             =***+----+*:**+
             **++=-:---+****
             +*-==:  :-=***-
              +--.    --**-
               ::      .=.
          """
    )


def load_api_key() -> str:
    """
    Try to obtain the API Key in the following order. If an option results in an
    empty string, try the next best option.
    1. From the string (hard-coded)
    2. From the `FORGE_API_KEY` environment variable
    3. From the `../API_KEY` file
    """
    api_key = ""
    if api_key == "":
        api_key = os.environ.get('FORGE_API_KEY', "")
    if api_key == "":
        this_directory_path = Path(__file__).parent.resolve()  # absolute path to this directory
        API_KEY_FILE_PATH = os.path.join(this_directory_path, "..", "API_KEY")
        api_key = ""
        with open(API_KEY_FILE_PATH, "r") as f:
            api_key = f.read()
    if api_key == "":
        raise ForgeApiError("Invalid API Key")
    return api_key


def get_api_url() -> str:
    """
    Get the Forge API Url
    - Read from the `FORGE_API_URL` environment variable. Default to
      https://forge.sindri.app/api/

    NOTE: `v1/` is appended to the Forge API Url (hard-coded).
    - Example: https://forge.sindri.app/api/ becomes https://forge.sindri.app/api/v1/
    """
    api_url = os.environ.get('FORGE_API_URL', "https://forge.sindri.app/api/")
    api_version = "v1"
    api_url = os.path.join(api_url, api_version, "")
    return api_url


if __name__ == "__main__":
    print_sindri_logo()
    # Toggle various examples by uncommenting/commenting the setup below input setup blocks.

    """Gnark poseidon"""
    circuit_name = "gnark poseidon"
    circuit_type = "Gnark"
    circuit_code_tar_path = "gnark/valid_circuits/poseidon.tar.gz"
    proof_input_dict = {}
    proof_input_file_path = "gnark/valid_circuits/poseidon_input.json"
    with open(proof_input_file_path, "r") as f:
        proof_input_dict = json.load(f)

    # """Circom C Groth16 bn254 multiplier2"""
    # circuit_name = "circom multiplier2"
    # circuit_type = "Circom C Groth16 bn254"
    # circuit_code_tar_path = "circom/valid_circuits/multiplier2.tar.gz"
    # proof_input_dict = {}
    # proof_input_file_path = "circom/valid_circuits/multiplier2_input.json"
    # with open(proof_input_file_path, "r") as f:
    #     proof_input_dict = json.load(f)

    # """Axiom Halo2 v0.3.0 quadratic"""
    # circuit_name = "axiom halo2 quadratic"
    # circuit_type = "Halo2 Axiom v0.3.0"
    # circuit_code_tar_path = "halo2/valid_circuits/quadratic_circuit.tar.gz"
    # proof_input_dict = {}
    # proof_input_file_path = "halo2/valid_circuits/quadratic_circuit_input.json"
    # with open(proof_input_file_path, "r") as f:
    #     proof_input_dict = json.load(f)

    """Set/load your Forge API Key and the Forge API Url"""
    api_key = load_api_key()
    api_url = get_api_url()

    """
    Run Forge!
        0. Create an instance of the ForgeAPIUtil class
        1. Create, upload, compile a circuit
        2. Prove the circuit
        3. Verify the proof
    """
    forge_api_util = ForgeAPIUtil(api_key, api_url=api_url)
    circuit_id = forge_api_util.create_circuit(circuit_name, circuit_type, circuit_code_tar_path)
    proof_id = forge_api_util.prove_circuit(circuit_id, proof_input_dict)
    valid = forge_api_util.verify_proof(proof_id)

    """
    The ForgeAPIUtil class contains utility methods for general interaction
    with the Forge API:
        - `get_all_circuits()` - get all circuits for a user
        - `get_all_proofs()` - get all proofs for a user
        - `get_all_circuit_proofs()` - get all proofs for a specific circuit
        - `get_circuit()` - get circuit detail
        - `get_proof()` - get proof detail

    Commented examples are below. Uncomment and utilize them as desired.
    """
    # circuit_id = "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE"
    # proof_id = "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE"
    # circuits = forge_api_util.get_all_circuits()
    # proofs = forge_api_util.get_all_proofs()
    # proofs = forge_api_util.get_all_circuit_proofs(circuit_id)
    # circuit = forge_api_util.get_proof(proof_id)
    # proof = forge_api_util.get_circuit(circuit_id)

    print("Done!\nUsing Sindri Labs' Forge API is EZ!\n")
