// Copyright 2020 ConsenSys AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package exponentiate

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/bits"
)

// Circuit y == x**e
// only the bitSize least significant bits of e are used
type Circuit struct {
	// tagging a variable is optional
	// default uses variable name and secret visibility.
	X frontend.Variable `gnark:",public"`
	Y frontend.Variable `gnark:",public"`

	E frontend.Variable
}

// Define declares the circuit's constraints
// y == x**e
func (circuit *Circuit) Define(api frontend.API) error {

	// number of bits of exponent
	const bitSize = 8

	// specify constraints
	output := frontend.Variable(1)
	bits := bits.ToBinary(api, circuit.E, bits.WithNbDigits(bitSize))

	for i := 0; i < len(bits); i++ {
		if i != 0 {
			output = api.Mul(output, output)
		}
		multiply := api.Mul(output, circuit.X)
		output = api.Select(bits[len(bits)-1-i], multiply, output)

	}

	api.AssertIsEqual(circuit.Y, output)

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
		E: frontend.Variable(data["E"].(string)),
		X: frontend.Variable(data["X"].(string)),
		Y: frontend.Variable(data["Y"].(string)),
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}
	return w
}
