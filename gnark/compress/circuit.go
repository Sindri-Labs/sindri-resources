package compress

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
	X [100]frontend.Variable
	Y [200]frontend.Variable `gnark:",public"`
}

func (circuit *Circuit) Define(api frontend.API) error {

	y_current_symbol := circuit.Y[0]
	y_multiplicity := circuit.Y[1]
	y_left := circuit.Y

	for i := 0; i < 100; i++ {
		//ensure equality at i-th position
		api.AssertIsEqual(circuit.X[i], y_current_symbol)

		// decrement multiplicity counter
		y_multiplicity = api.Sub(y_multiplicity, 1)

		// if counter is at zero, chomp two from compressed list
		for i := 0; i < 198; i++ {
			y_left[i] = api.Select(api.IsZero(y_multiplicity), y_left[i+2], y_left[i])
		}
		y_left[198] = api.Select(api.IsZero(y_multiplicity), frontend.Variable(0), y_left[198])
		y_left[199] = api.Select(api.IsZero(y_multiplicity), frontend.Variable(0), y_left[199])
		y_multiplicity = api.Select(api.IsZero(y_multiplicity), y_left[1], y_multiplicity)

		y_current_symbol = y_left[0]
	}

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

	// send original to list of integers
	chars := []rune(data["original"].(string))
	original := [100]frontend.Variable{}
	compressed := [200]frontend.Variable{}
	preImage := []byte(data["original"].(string))

	prev_x := int(chars[0])
	multiplicity := 0
	y_idx := 0

	for i := 0; i < 100; i++ {
		if i < len(chars) {
			original[i] = frontend.Variable(chars[i])

			if int(chars[i]) != prev_x {
				compressed[y_idx] = frontend.Variable(prev_x)
				y_idx = y_idx + 1
				compressed[y_idx] = frontend.Variable(multiplicity)
				y_idx = y_idx + 1

				multiplicity = 1
			} else {
				multiplicity = multiplicity + 1
			}
			prev_x = int(chars[i])
		} else if i == len(chars) {
			compressed[y_idx] = frontend.Variable(prev_x)
			y_idx = y_idx + 1
			compressed[y_idx] = frontend.Variable(multiplicity)
			y_idx = y_idx + 1

			original[i] = frontend.Variable(0)
			preImage = append(preImage, 0)
		} else { // pad original with zeros to length 100
			original[i] = frontend.Variable(0)
			preImage = append(preImage, 0)
		}
		// pad compressed array to length 200
		for i := y_idx; i < 200; i++ {
			compressed[i] = frontend.Variable(0)
		}
	}

	assignment := Circuit{
		X: original,
		Y: compressed,
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}
	return w
}
