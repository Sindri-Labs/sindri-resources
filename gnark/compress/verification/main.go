package main

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/sindri-labs/forge-sample-data/gnark/compress"
)

type ProofDetailResponse struct {
	Proof           Proof           `json:"proof"`
	VerificationKey VerificationKey `json:"verification_key"`
	PublicInputJson json.RawMessage `json:"public"`
}

type Proof struct {
	EncodedProof string `json:"proof"`
}

type VerificationKey struct {
	EncodedVerifyingKey string `json:"verifying_key"`
}

func exitOnError(err error, action string) {
	if err != nil {
		fmt.Printf("Error %s: %v\n", action, err)
		os.Exit(1)
	}
}

func main() {
	// Parse the necessary data from proof detail response JSON.
	var proofDetailResponse ProofDetailResponse
	if len(os.Args) < 2 {
		fmt.Println("Please provide the path to the proof detail JSON file as an argument.")
		return
	}
	filename := os.Args[1]
	data, err := os.ReadFile(filename)
	exitOnError(err, "reading file")
	err = json.Unmarshal(data, &proofDetailResponse)
	exitOnError(err, "unmarshalling JSON")

	// Load in the proof.
	decodedProof, err := base64.StdEncoding.DecodeString(proofDetailResponse.Proof.EncodedProof)
	exitOnError(err, "decoding proof")
	proof := groth16.NewProof(ecc.BN254)
	_, err = proof.ReadFrom(bytes.NewReader(decodedProof))
	exitOnError(err, "reading proof")

	// Load in the verifying key.
	decodedVerifyingKey, err := base64.StdEncoding.DecodeString(proofDetailResponse.VerificationKey.EncodedVerifyingKey)
	exitOnError(err, "decoding verifying key")
	verifyingKey := groth16.NewVerifyingKey(ecc.BN254)
	_, err = verifyingKey.ReadFrom(bytes.NewReader(decodedVerifyingKey))
	exitOnError(err, "reading verifying key")

	// Construct the witness based on the public inputs.
	schema, err := frontend.NewSchema(&compress.Circuit{})
	exitOnError(err, "constructing schema")
	publicWitness, err := witness.New(ecc.BN254.ScalarField())
	exitOnError(err, "constructing witness")
	err = publicWitness.FromJSON(schema, proofDetailResponse.PublicInputJson)
	exitOnError(err, "parsing public inputs")

	// Verify the proof.
	err = groth16.Verify(proof, verifyingKey, publicWitness)
	exitOnError(err, "verifying proof")
	fmt.Println("Proof verified successfully.")
}
