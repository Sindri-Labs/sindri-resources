import subprocess

snarkjs_verifier_args = [
    "npx",
    "snarkjs",
    "groth16",
    "verify",
    "verification_key.json",
    "public.json",
    "proof.json"
]
out = subprocess.Popen(snarkjs_verifier_args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
stdout, _ = out.communicate()

if out.returncode == 0:
    print("Verification successful")
    print(f"Verifier output: {stdout}")
else:
    print("Verification unsuccessful")
    print(f"Verifier output: {stdout}")
