package poseidon

import (
	"math/big"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
	hash "github.com/iden3/go-iden3-crypto/poseidon"
)

type testPoseidonCiruit struct {
	Data frontend.Variable
	Hash frontend.Variable `gnark:",public"`
}

func (circuit *testPoseidonCiruit) Define(api frontend.API) error {
	h := Hash(api, circuit.Data)
	api.AssertIsEqual(h, circuit.Hash)
	return nil
}

func TestPoseidon(t *testing.T) {
	assert := test.NewAssert(t)

	var circuit, assignment testPoseidonCiruit

	input, _ := new(big.Int).SetString("297262668938251460872476410954775437897592223497", 10)
	assignment.Data = input
	assignment.Hash, _ = hash.Hash([]*big.Int{input})

	assert.SolvingSucceeded(&circuit, &assignment, test.WithCurves(ecc.BN254), test.WithBackends(backend.PLONK))
}
