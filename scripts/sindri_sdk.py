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


class SindriSdk:
    """
    # Sindri SDK
    Utility class for interacting with the Sindri API.

    ### Dependencies:
    - `requests`: (`pip install requests`)

    ### Parameters:
    - `verbose_level: int`: Stdout print level (defeault=`1`, options=`[0,1,2]`)

    ### Usage
    ```python
    # Collect inputs for circuit compilation and proving
    circuit_name = "Circom multiplier2"
    circuit_upload_path = "circom/multiplier2.tar.gz"
    # Set proof input as a dict. Here we read from a JSON file.
    proof_input = {}
    proof_input_file_path = "circom/input.json"
    with open(proof_input_file_path, "r") as f:
        proof_input = json.load(f)

    # Run Sindri API
    # NOTE: API Key and API Url are obtained upon initialization of the SindriSdk class.
    sindri_sdk = SindriSdk()
    circuit_id = sindri_sdk.create_circuit(circuit_name, circuit_upload_path)
    proof_id = sindri_sdk.prove_circuit(circuit_id, proof_input)
    ```

    Methods for interacting with the Sindri API:
    - `create_circuit()` - create a circuit
    - `get_all_circuit_proofs()` - get all proofs for a specific circuit
    - `get_all_circuits()` - get all circuits
    - `get_all_proofs()` - get all proofs
    - `get_circuit()` - get circuit detail
    - `get_proof()` - get proof detail
    - `prove_circuit()` - prove a circuit

    Additional methods for configuring the SindriSdk:
    - `load_api_key()` - Load and return API Key:
    - `set_api_key()` - Configure SindriSdk instance with provided API Key
    - `set_api_url()` - Configure SindriSdk instance with provided API Url
    - `set_verbose_level()` - Configure SindriSdk instance with stdout verbosity level

    NOTE: By default, the SindriSdk tries to obtain the API Key in the following order.
    If an option results in an empty string, try the next best option.
    1. From the `SINDRI_API_KEY` environment variable
    1. From the `../API_KEY` file
    """

    class SindriApiError(Exception):
        """Custom Exception for Sindri API Errors"""

        pass

    DEFAULT_SINDRI_API_URL = "https://forge.sindri.app/api/"
    API_VERSION = "v1"

    def __init__(
        self,
        verbose_level: int = 2,
    ):
        self.polling_interval_sec: int = 1  # polling interval for circuit compilation & proving
        self.max_polling_iterations: int = 172800  # 2 days with polling interval 1 second
        self.perform_verify: bool = True
        self.set_verbose_level(verbose_level)

        self.api_key = self.load_api_key()
        self.api_url = self.get_api_url()
        self.headers_json = {
            "Accept": "application/json",
            "Authorization": f"Bearer {self.api_key}",
        }
        if self.verbose_level > 0:
            self._print_sindri_logo()
            print(f"Sindri Api Url: {self.api_url}")
            print(f"Sindri Api Key: {self.api_key}\n")

    def _get_verbose_1_circuit_detail(self, circuit_detail: dict) -> dict:
        """Return a slim circuit detail object for printing."""
        return {
            "circuit_id": circuit_detail.get("circuit_id", None),
            "circuit_name": circuit_detail.get("circuit_name", None),
            "circuit_type": circuit_detail.get("circuit_type", None),
            "compute_time": circuit_detail.get("compute_time", None),
            "date_created": circuit_detail.get("date_created", None),
            "status": circuit_detail.get("status", None),
        }

    def _get_verbose_1_proof_detail(self, proof_detail: dict) -> dict:
        """Return a slim proof detail object for printing."""
        return {
            "circuit_id": proof_detail.get("circuit_id", None),
            "circuit_name": proof_detail.get("circuit_name", None),
            "circuit_type": proof_detail.get("circuit_type", None),
            "compute_time": proof_detail.get("compute_time", None),
            "date_created": proof_detail.get("date_created", None),
            "proof_id": proof_detail.get("proof_id", None),
            "status": proof_detail.get("status", None),
        }

    def _hit_api(self, method: str, path: str, data=None, files=None) -> tuple[int, dict | list]:
        """
        Hit the Sindri API.

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
                raise SindriSdk.SindriApiError("Invalid request method")
        except requests.exceptions.ConnectionError:
            # Raise a clean exception and suppress the original exception's traceback.
            raise SindriSdk.SindriApiError(
                f"Unable to connect to the Sindri API. path={full_path}"
            ) from None

        if response is None:
            raise SindriSdk.SindriApiError(
                f"No response received. method={method}, path={full_path},"
                f" data={data} headers={self.headers_json}, files={files}"
            )
        if response.status_code == 401:
            raise SindriSdk.SindriApiError(f"401 - Invalid API Key. path={full_path}")
        elif response.status_code == 404:
            raise SindriSdk.SindriApiError(f"404 - Not found. path={full_path}")
        else:
            # Decode JSON response
            try:
                response_json = response.json()
            except json.decoder.JSONDecodeError:
                raise SindriSdk.SindriApiError(
                    f"Unexpected Error. Unable to decode response as JSON."
                    f" status={response.status_code} response={response.text}"
                ) from None
        return response.status_code, response_json

    def _print_sindri_logo(self):
        # https://ascii-generator.site/
        print(
            """
                  Sindri API
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

    def create_circuit(
        self,
        circuit_name: str,
        circuit_upload_path: str,
    ) -> str:
        """
        Create a circuit, upload the circuit code,
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

        if not os.path.exists(circuit_upload_path):
            raise ValueError(f"circuit_upload_path does not exist: {circuit_upload_path}")

        files = {"files": open(circuit_upload_path, "rb")}

        # 1. Create a circuit, obtain a circuit_id.
        if self.verbose_level > 0:
            print("Circuit: Create")
        if self.verbose_level > 1:
            print(f"    circuit_name: {circuit_name}")
            print(f"    upload_path:   {circuit_upload_path}")
        response_status_code, response_json = self._hit_api(
            "POST",
            "circuit",
            data={"circuit_name": circuit_name},
            files=files,
        )
        if response_status_code != 201:
            raise SindriSdk.SindriApiError(
                f"Unable to create a new circuit."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, dict):
            raise SindriSdk.SindriApiError("Received unexpected type for circuit detail response.")

        # Obtain circuit_id
        circuit_id = response_json.get("circuit_id", "")
        if self.verbose_level > 0:
            print(f"    circuit_id:   {circuit_id}")

        # 2. Poll circuit detail until it has a status of Ready/Failed
        if self.verbose_level > 0:
            print("Circuit: Poll until Ready/Failed")
        for _ in range(self.max_polling_iterations):
            response_status_code, response_json = self._hit_api(
                "GET",
                f"circuit/{circuit_id}/detail",
                data={"include_verification_key": False},
            )
            if response_status_code != 200:
                raise SindriSdk.SindriApiError(
                    f"Failure to poll circuit detail."
                    f" status={response_status_code} response={response_json}"
                )
            if not isinstance(response_json, dict):
                raise SindriSdk.SindriApiError(
                    "Received unexpected type for circuit detail response."
                )
            circuit_status = response_json.get("status", "")
            if circuit_status == "Failed":
                raise SindriSdk.SindriApiError(
                    f"Circuit compilation failed."
                    f" status={response_status_code} response={response_json}"
                )
            if circuit_status == "Ready":
                break
            time.sleep(self.polling_interval_sec)
        else:
            raise SindriSdk.SindriApiError(
                f"Circuit compile polling timed out."
                f" status={response_status_code} response={response_json}"
            )

        if self.verbose_level > 0:
            self.get_circuit(circuit_id)

        # Circuit compilation success!
        return circuit_id

    def get_all_circuit_proofs(self, circuit_id: str) -> list[dict]:
        """Get all proofs for a circuit_id."""
        if self.verbose_level > 0:
            print(f"Proof: Get all proofs for circuit_id: {circuit_id}")
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
            raise SindriSdk.SindriApiError(
                f"Unable to fetch proofs for circuit_id={circuit_id}."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, list):
            raise SindriSdk.SindriApiError("Received unexpected type for proof list response.")

        if self.verbose_level > 0:
            proof_detail_list = response_json.copy()
            if self.verbose_level == 1:
                proof_detail_list = []
                for proof_detail in response_json:
                    proof_detail_list.append(self._get_verbose_1_proof_detail(proof_detail))
            print(f"{pformat(proof_detail_list, indent=4)}\n")

        return response_json

    def get_all_circuits(self) -> list[dict]:
        """Get all circuits."""
        if self.verbose_level > 0:
            print("Circuit: Get all circuits")
        response_status_code, response_json = self._hit_api(
            "GET",
            "circuit/list",
            data={"include_verification_key": True},
        )
        if response_status_code != 200:
            raise SindriSdk.SindriApiError(
                f"Unable to fetch circuits."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, list):
            raise SindriSdk.SindriApiError("Received unexpected type for circuit list response.")

        if self.verbose_level > 0:
            circuit_detail_list = response_json.copy()
            if self.verbose_level == 1:
                circuit_detail_list = []
                for circuit_detail in response_json:
                    circuit_detail_list.append(self._get_verbose_1_circuit_detail(circuit_detail))
            print(f"{pformat(circuit_detail_list, indent=4)}\n")

        return response_json

    def get_all_proofs(self) -> list[dict]:
        """Get all proofs."""
        if self.verbose_level > 0:
            print("Proof: Get all proofs")
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
            raise SindriSdk.SindriApiError(
                f"Unable to fetch proofs."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, list):
            raise SindriSdk.SindriApiError("Received unexpected type for proof list response.")

        if self.verbose_level > 0:
            proof_detail_list = response_json.copy()
            if self.verbose_level == 1:
                proof_detail_list = []
                for proof_detail in response_json:
                    proof_detail_list.append(self._get_verbose_1_proof_detail(proof_detail))
            print(f"{pformat(proof_detail_list, indent=4)}\n")

        return response_json

    @classmethod
    def get_api_url(cls) -> str:
        """
        Get the Sindri API Url
        - Read from the `FORGE_API_URL` environment variable. Default to
        https://forge.sindri.app/api/

        NOTE: `v1/` is appended to the Sindri API Url (hard-coded).
        - Example: https://forge.sindri.app/api/ becomes https://forge.sindri.app/api/v1/
        """
        api_url = os.environ.get("FORGE_API_URL", cls.DEFAULT_SINDRI_API_URL)
        api_url = os.path.join(api_url, cls.API_VERSION, "")
        return api_url

    def get_circuit(self, circuit_id: str) -> dict:
        """Get circuit by circuit_id."""
        if self.verbose_level > 0:
            print(f"Circuit: Get circuit detail for circuit_id: {circuit_id}")
        response_status_code, response_json = self._hit_api(
            "GET",
            f"circuit/{circuit_id}/detail",
            data={"include_verification_key": True},
        )
        if response_status_code != 200:
            raise SindriSdk.SindriApiError(
                f"Unable to fetch circuit_id={circuit_id}."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, dict):
            raise SindriSdk.SindriApiError("Received unexpected type for circuit detail response.")

        if self.verbose_level > 0:
            circuit_detail = response_json.copy()
            if self.verbose_level == 1:
                circuit_detail = self._get_verbose_1_circuit_detail(circuit_detail)
            print(f"{pformat(circuit_detail, indent=4)}\n")

        return response_json

    def get_proof(self, proof_id: str, include_proof_input: bool = False) -> dict:
        """Get proof by proof_id."""
        if self.verbose_level > 0:
            print(f"Proof: Get proof detail for proof_id: {proof_id}")
        response_status_code, response_json = self._hit_api(
            "GET",
            f"proof/{proof_id}/detail",
            data={
                "include_proof_input": include_proof_input,
                "include_public": True,
                "include_verification_key": True,
                "include_proof": True,
            },
        )
        if response_status_code != 200:
            raise SindriSdk.SindriApiError(
                f"Unable to fetch proof_id={proof_id}."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, dict):
            raise SindriSdk.SindriApiError("Received unexpected type for proof detail response.")

        if self.verbose_level > 0:
            proof_detail = response_json.copy()
            if self.verbose_level == 1:
                proof_detail = self._get_verbose_1_proof_detail(proof_detail)
            print(f"{pformat(proof_detail, indent=4)}\n")

        return response_json

    @classmethod
    def load_api_key(cls) -> str:
        """
        Return the API Key as a string.

        Try to obtain the API Key in the following order. If an option results in an
        empty string, try the next best option.
        1. From the `SINDRI_API_KEY` environment variable
        1. From the `../API_KEY` file

        Raise SindriApiError if the result is an empty string.
        """
        api_key = ""
        if api_key == "":
            api_key = os.environ.get("SINDRI_API_KEY", "")
        if api_key == "":
            this_directory_path = Path(__file__).parent.resolve()  # absolute path to this directory
            API_KEY_FILE_PATH = os.path.join(this_directory_path, "..", "API_KEY")
            api_key = ""
            with open(API_KEY_FILE_PATH, "r") as f:
                api_key = f.read()
        if api_key == "":
            raise SindriSdk.SindriApiError("Invalid API Key")
        return api_key

    def prove_circuit(self, circuit_id: str, proof_input: str) -> str:
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

        # TODO: HANDLE the JSON/non-JSON
        # Convert the proof_input into a json string
        # proof_input_json_str = json.dumps(proof_input)

        # 1. Submit a proof, obtain a proof_id.
        if self.verbose_level > 0:
            print("Prove circuit")
        response_status_code, response_json = self._hit_api(
            "POST",
            f"circuit/{circuit_id}/prove",
            data={
                "proof_input": proof_input,
                "perform_verify": self.perform_verify,
            },
        )
        if response_status_code != 201:
            raise SindriSdk.SindriApiError(
                f"Unable to prove circuit."
                f" status={response_status_code} response={response_json}"
            )
        if not isinstance(response_json, dict):
            raise SindriSdk.SindriApiError("Received unexpected type for proof detail response.")

        # Obtain proof_id
        proof_id = response_json.get("proof_id", "")
        if self.verbose_level > 0:
            print(f"    proof_id:     {proof_id}")

        # 2. Poll proof detail until it has a status of Ready/Failed
        if self.verbose_level > 0:
            print("Proof: Poll until Ready/Failed")
        for _ in range(self.max_polling_iterations):
            response_status_code, response_json = self._hit_api(
                "GET",
                f"proof/{proof_id}/detail",
                data={
                    "include_proof_input": False,
                    "include_proof": False,
                    "include_public": False,
                    "include_verification_key": False,
                },
            )
            if response_status_code != 200:
                raise SindriSdk.SindriApiError(
                    f"Failure to poll proof detail."
                    f" status={response_status_code} response={response_json}"
                )
            if not isinstance(response_json, dict):
                raise SindriSdk.SindriApiError(
                    "Received unexpected type for proof detail response."
                )

            proof_status = response_json.get("status", "")
            if proof_status == "Failed":
                raise SindriSdk.SindriApiError(
                    f"Prove circuit failed."
                    f" status={response_status_code} response={response_json}"
                )
            if proof_status == "Ready":
                break
            time.sleep(self.polling_interval_sec)
        else:
            raise SindriSdk.SindriApiError(
                f"Prove circuit polling timed out."
                f" status={response_status_code} response={response_json}"
            )

        if self.verbose_level > 0:
            self.get_proof(proof_id)

        # Prove circuit success!
        return proof_id

    def set_api_key(self, api_key: str) -> None:
        """Set the API Key for the Sindri SDK instance."""
        self.api_key = api_key
        self.headers_json = {
            "Accept": "application/json",
            "Authorization": f"Bearer {self.api_key}",
        }
        if self.verbose_level > 0:
            print(f"Sindri Api Key: {self.api_key}")

    def set_api_url(self, api_url: str) -> None:
        """
        Set the API Url for the Sindri SDK instance.

        NOTE: `v1/` is appended to the Sindri API Url (hard-coded).
        - Example: https://forge.sindri.app/api/ becomes https://forge.sindri.app/api/v1/
        """
        self.api_url = os.path.join(api_url, self.API_VERSION, "")
        if self.verbose_level > 0:
            print(f"Sindri Api Url: {self.api_url}")

    def set_verbose_level(self, level: int) -> None:
        """
        Set verbose_level for stdout printing.

        Verbose level must be an int in `[0,1,2]`:
        - `0`: Do not print anything to stdout
        - `1`: Print only necesessary information from Circuit/Proof objects
        - `2`: Print everything

        This raises ValueError if `level` is invalid.
        """
        error_msg = "Invalid verbose_level. Must be an int in [0,1,2]."
        if not isinstance(level, int):
            raise ValueError(error_msg)
        elif level not in [0, 1, 2]:
            raise ValueError(error_msg)
        self.verbose_level = level
