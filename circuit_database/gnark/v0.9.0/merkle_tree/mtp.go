package mtp

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/mimc"
)

// Circuit that checks a Merkle Proof
// This is a variation of the existing circuit in the gnark library https://github.com/Consensys/gnark/blob/master/std/accumulator/merkle/verify.go

type Circuit struct {
	Root          frontend.Variable    `gnark:",public"`
	ProofElements [5]frontend.Variable // private
	ProofIndex    frontend.Variable    // private
	Leaf          frontend.Variable    // private
}

func (circuit *Circuit) Define(api frontend.API) error {
	h, err := mimc.NewMiMC(api)
	if err != nil {
		return err
	}

	// Hash leaf
	h.Reset()
	h.Write(circuit.Leaf)
	hashed := h.Sum()

	depth := len(circuit.ProofElements)
	proofIndices := api.ToBinary(circuit.ProofIndex, depth)

	// Continuously hash with the proof elements
	for i := 0; i < len(circuit.ProofElements); i++ {
		element := circuit.ProofElements[i]
		// 0 = left, 1 = right
		index := proofIndices[i]

		d1 := api.Select(index, element, hashed)
		d2 := api.Select(index, hashed, element)

		h.Reset()
		h.Write(d1, d2)
		hashed = h.Sum()
	}

	// Verify calculates hash is equal to root
	api.AssertIsEqual(hashed, circuit.Root)
	return nil
}

// Common utility for reading JSON in from a file.
func ReadFromInputPath(pathInput string) (map[string]interface{}, error) {

	absPath, err := filepath.Abs(pathInput)
	if err != nil {
		fmt.Println("Error constructing absolute path:", err)
		return nil, err
	}

	file, err := os.Open(absPath)
	if err != nil {
		panic(err)
	}
	defer file.Close()

	var data map[string]interface{}
	err = json.NewDecoder(file).Decode(&data)
	if err != nil {
		panic(err)
	}

	return data, nil
}

// Construct a witness from input data in a JSON file.
func FromJson(pathInput string) witness.Witness {

	data, err := ReadFromInputPath(pathInput)
	if err != nil {
		panic(err)
	}
	duration := time.Duration(5) * time.Second
	time.Sleep(duration)

	root, _ := new(big.Int).SetString(data["Root"].(string), 10)
	proofIndex, _ := new(big.Int).SetString(data["ProofIndex"].(string), 10)
	leaf, _ := new(big.Int).SetString(data["Leaf"].(string), 10)
	str := strings.Trim(data["ProofElements"].(string), "[]")
	strValues := strings.Split(str, ",")
	bigIntList := [5]frontend.Variable{}
	for i, v := range strValues {
		num, _ := new(big.Int).SetString(strings.TrimSpace(v), 10)
		bigIntList[i] = num
	}
	assignment := Circuit{
		Root:          root,
		ProofElements: bigIntList,
		ProofIndex:    proofIndex,
		Leaf:          leaf,
	}
	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}
	return w
}
