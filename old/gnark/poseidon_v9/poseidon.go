package poseidon

import (
	"math/big"

	"github.com/consensys/gnark/frontend"
)

type Poseidon struct {
	api  frontend.API
	data []frontend.Variable
}

func Hash(api frontend.API, inputs ...frontend.Variable) frontend.Variable {
	h := NewPoseidon(api)
	h.Write(inputs...)
	return h.Sum()
}

func NewPoseidon(api frontend.API) Poseidon {
	return Poseidon{
		api:  api,
		data: []frontend.Variable{},
	}
}

func (h *Poseidon) Write(data ...frontend.Variable) {
	h.data = append(h.data, data...)
}

func (h *Poseidon) Reset() {
	h.data = []frontend.Variable{}
}

func (h *Poseidon) Sum() frontend.Variable {
	nInputs := len(h.data)
	// And rounded up to nearest integer that divides by t
	nRoundsPC := [16]int{56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68}
	t := nInputs + 1
	nRoundsF := 8
	nRoundsP := nRoundsPC[t-2]
	c := getConstant(C, t)
	s := getConstant(S, t)
	m := getConstant(M, t)
	p := getConstant(P, t)

	state := make([]frontend.Variable, t)
	for j := 0; j < t; j++ {
		if j == 0 {
			state[0] = 0
		} else {
			state[j] = h.data[j-1]
		}
	}
	state = h.ark(state, c, 0)

	for r := 0; r < nRoundsF/2-1; r++ {
		for j := 0; j < t; j++ {
			state[j] = h.sigma(state[j])
		}
		state = h.ark(state, c, (r+1)*t)
		state = h.mix(state, m)
	}

	for j := 0; j < t; j++ {
		state[j] = h.sigma(state[j])
	}
	state = h.ark(state, c, nRoundsF/2*t)
	state = h.mix(state, p)

	for r := 0; r < nRoundsP; r++ {

		state[0] = h.sigma(state[0])

		state[0] = h.api.Add(state[0], c[(nRoundsF/2+1)*t+r])
		newState0 := frontend.Variable(0)
		for j := 0; j < len(state); j++ {
			mul := h.api.Mul(s[(t*2-1)*r+j], state[j])
			newState0 = h.api.Add(newState0, mul)
		}

		for k := 1; k < t; k++ {
			state[k] = h.api.Add(state[k], h.api.Mul(state[0], s[(t*2-1)*r+t+k-1]))
		}
		state[0] = newState0
	}

	for r := 0; r < nRoundsF/2-1; r++ {
		for j := 0; j < t; j++ {
			state[j] = h.sigma(state[j])
		}
		state = h.ark(state, c, (nRoundsF/2+1)*t+nRoundsP+r*t)
		state = h.mix(state, m)
	}

	for j := 0; j < t; j++ {
		state[j] = h.sigma(state[j])
	}

	out := h.mixLast(state, m, 0)
	h.data = []frontend.Variable{}
	return out
}

func (h *Poseidon) sigma(in frontend.Variable) frontend.Variable {
	in2 := h.api.Mul(in, in)
	in4 := h.api.Mul(in2, in2)
	return h.api.Mul(in4, in)
}

func (h *Poseidon) ark(in []frontend.Variable, c []*big.Int, r int) []frontend.Variable {
	out := make([]frontend.Variable, len(in))
	for i, v := range in {
		out[i] = h.api.Add(v, c[i+r])
	}
	return out
}

func (h *Poseidon) mix(in []frontend.Variable, m [][]*big.Int) []frontend.Variable {
	t := len(in)
	out := make([]frontend.Variable, t)
	for i := 0; i < t; i++ {
		lc := frontend.Variable(0)
		for j := 0; j < t; j++ {
			lc = h.api.Add(lc, h.api.Mul(m[j][i], in[j]))
		}
		out[i] = lc
	}
	return out
}

func (h *Poseidon) mixLast(in []frontend.Variable, m [][]*big.Int, s int) frontend.Variable {
	t := len(in)
	out := frontend.Variable(0)
	for j := 0; j < t; j++ {
		out = h.api.Add(out, h.api.Mul(m[j][s], in[j]))
	}
	return out
}
