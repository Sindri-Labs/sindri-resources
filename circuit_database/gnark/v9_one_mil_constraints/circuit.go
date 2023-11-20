package cubic

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
)

type Circuit struct {
	X frontend.Variable `gnark:"x"`       // x  --> secret visibility (default)
	Y frontend.Variable `gnark:",public"` // Y  --> public visibility
}

// Define declares the circuit logic. The compiler then produces a list of constraints
// which must be satisfied (valid witness) in order to create a valid zk-SNARK
func (circuit *Circuit) Define(api frontend.API) error {
    
	N := 1000000
    out := make([]frontend.Variable, N)
    for i := 0; i < N; i++ {
        out[i] = api.Add(circuit.X, 0)
        api.AssertIsEqual(out[i], circuit.X)
    }
    summand := frontend.Variable(0)
    for i := 0; i < N; i++ {
        summand = api.Add(summand, out[i])
    }

	api.AssertIsEqual(circuit.Y, summand)
	return nil
}

func ReadFromInputPath(pathInput string) (map[string]interface{}, error) {

	// Construct the absolute path to the file
	//absPath := filepath.Join("../", pathInput)
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

	assignment := Circuit{
		X: frontend.Variable(data["x"].(string)),
		Y: frontend.Variable(data["y"].(string)),
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}
	return w
}
