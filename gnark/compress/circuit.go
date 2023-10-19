package compress

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"path/filepath"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"

	bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
)

type Circuit struct {
	X [100]frontend.Variable
	// W frontend.Variable
	Y [200]frontend.Variable `gnark:",public"`
	// Z frontend.Variable      `gnark:",public"`
}

func (circuit *Circuit) Define(api frontend.API) error {

	// first check hash of original
	// mimc, _ := mimc.NewMiMC(api)
	// mimc.Write(circuit.W)
	// api.AssertIsEqual(circuit.Z, mimc.Sum())

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

func mimcHash(data []byte) string {
	f := bn254.NewMiMC()
	hash := f.Sum(data)
	hashInt := big.NewInt(0).SetBytes(hash)
	return hashInt.String()
}

func FromJson(pathInput string) witness.Witness {

	data, err := ReadFromInputPath(pathInput)
	if err != nil {
		panic(err)
	}

	// send original to list of integers
	chars := []rune(data["original"].(string))
	XtoFE := [100]frontend.Variable{}
	for i := 0; i < 100; i++ {
		if i < len(chars) {
			XtoFE[i] = frontend.Variable(chars[i])
		} else { // pad with zeros
			XtoFE[i] = frontend.Variable(0)
		}
	}

	// send y to list of integers
	chars = []rune(data["compressed"].(string))
	YtoFE := [200]frontend.Variable{}
	for i := 0; i < 200; i++ {
		if i < len(chars) {
			if i%2 == 0 { // symbol
				YtoFE[i] = frontend.Variable(chars[i])
			} else { // multiplicity
				YtoFE[i] = frontend.Variable(chars[i] - 48)
			}
		} else { // pad with zeros
			YtoFE[i] = frontend.Variable(0)
		}
	}

	preImage := []byte(data["original"].(string))
	hash := mimcHash(preImage)
	fmt.Println(preImage)
	fmt.Println(hash)

	assignment := Circuit{
		X: XtoFE,
		//W: "16130099170765464552823636852555369511329944820189892919423002775646948828469",
		Y: YtoFE,
		//Z: "12886436712380113721405259596386800092738845035233065858332878701083870690753",
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}
	return w
}
