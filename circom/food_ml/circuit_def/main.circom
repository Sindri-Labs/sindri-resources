pragma circom 2.0.0;

include "../node_modules/circomlib-ml/circuits/ReLU.circom";
include "../node_modules/circomlib-ml/circuits/Dense.circom";
include "../node_modules/circomlib-ml/circuits/ArgMax.circom";

template Model() {
signal input in[374];
signal input dense_14_weights[374][64];
signal input dense_14_bias[64];
signal input dense_15_weights[64][11];
signal input dense_15_bias[11];
signal output out[1];

component dense_14 = Dense(374, 64);
component re_lu[64];
for (var i0 = 0; i0 < 64; i0++) {
    re_lu[i0] = ReLU();
}
component dense_15 = Dense(64, 11);
component softmax_8 = ArgMax(11);

for (var i0 = 0; i0 < 374; i0++) {
    dense_14.in[i0] <== in[i0];
}
for (var i0 = 0; i0 < 374; i0++) {
    for (var i1 = 0; i1 < 64; i1++) {
        dense_14.weights[i0][i1] <== dense_14_weights[i0][i1];
}}
for (var i0 = 0; i0 < 64; i0++) {
    dense_14.bias[i0] <== dense_14_bias[i0];
}
for (var i0 = 0; i0 < 64; i0++) {
    re_lu[i0].in <== dense_14.out[i0];
}
for (var i0 = 0; i0 < 64; i0++) {
    dense_15.in[i0] <== re_lu[i0].out;
}
for (var i0 = 0; i0 < 64; i0++) {
    for (var i1 = 0; i1 < 11; i1++) {
        dense_15.weights[i0][i1] <== dense_15_weights[i0][i1];
}}
for (var i0 = 0; i0 < 11; i0++) {
    dense_15.bias[i0] <== dense_15_bias[i0];
}
for (var i0 = 0; i0 < 11; i0++) {
    softmax_8.in[i0] <== dense_15.out[i0];
}
out[0] <== softmax_8.out;

}

component main = Model();
