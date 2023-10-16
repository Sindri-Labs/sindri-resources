package poseidon

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	hash "github.com/iden3/go-iden3-crypto/poseidon"
)

type PoseidonCircuit struct {
	Data frontend.Variable
	Hash frontend.Variable `gnark:",public"`
}

func (circuit *PoseidonCircuit) Define(api frontend.API) error {
	h := Hash(api, circuit.Data)
	api.AssertIsEqual(h, circuit.Hash)
	return nil
}

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

func FromJson(pathInput string) witness.Witness {

	data, err := ReadFromInputPath(pathInput)
	if err != nil {
		panic(err)
	}

	var assignment PoseidonCircuit

	input, _ := new(big.Int).SetString(data["PreImage"].(string), 10)
	assignment.Data = input
	assignment.Hash, _ = hash.Hash([]*big.Int{input})

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}
	return w

}
