use dep::std::println;
use dep::fraction::Fraction;
use dep::fraction::ZERO;
use dep::fraction::{compareFraction, addFraction, multiplyFraction};

global NUM_NODES: u32 = 10;
global NUM_ENTRIES: u32 = 100; // NUM_NODES**2
global MAX_NUM_EDGES: u32 = 10;
global MAX_ITERATIONS: u32 = 100;

// Initial transformation from a short list of edges to a sparsely
// populated transition matrix.  This is unconstrained since
// we access variable elements of the transition matrix.
unconstrained fn edgelist_to_matrix(edgelist: [Field; MAX_NUM_EDGES]) -> [u32; NUM_ENTRIES] {
    let mut transition_matrix: [u32; NUM_ENTRIES] = [0; NUM_ENTRIES];
    // initialize graph
    for i in 0..MAX_NUM_EDGES {
        transition_matrix[edgelist[i]] = 1;
    }
    transition_matrix
}

fn normalize_transition_matrix(transition_matrix: [u32; NUM_ENTRIES]) -> [Fraction; NUM_ENTRIES] {
    let mut normalized: [Fraction; NUM_ENTRIES] = [ZERO; NUM_ENTRIES];

    for row in 0..NUM_NODES {
        let mut out_degree = 0;
        for column in 0..NUM_NODES { // iterate once to get total out degree of a node
            out_degree = out_degree + transition_matrix[row*NUM_NODES+column];
        }
        for column in 0..NUM_NODES { // normalize that row of the transition matrix
            normalized[row*NUM_NODES+column] = Fraction{sign: true, num: transition_matrix[row*NUM_NODES+column], den: out_degree};
        }
    }

    normalized
}

fn iterate(mat: [Fraction; NUM_ENTRIES], vec: [Fraction; NUM_NODES]) -> [Fraction; NUM_NODES] {

    let mut next_probabilities: [Fraction; NUM_NODES] = [ZERO; NUM_NODES];
    for row in 0..NUM_NODES {
        let mut dot_product = ZERO;
        for column in 0..NUM_NODES {
            let mut delta = multiplyFraction(mat[row*NUM_NODES+column],vec[column]);
            dot_product = addFraction(dot_product,delta);
        }
        next_probabilities[row] = dot_product;
    }
    
    next_probabilities
}

fn iterate_int(mat: [u32; NUM_ENTRIES], vec: [u32; NUM_NODES]) -> [u32; NUM_NODES] {

    let mut next_probabilities: [u32; NUM_NODES] = [0; NUM_NODES];
    for row in 0..NUM_NODES {
        let mut dot_product = 0;
        for column in 0..NUM_NODES {
            dot_product = dot_product+mat[row*NUM_NODES+column]*vec[column];
        }
        next_probabilities[row] = dot_product;
    }
    
    next_probabilities
}

// inefficient (but easy to understand) bubble sort
fn final_arg_sort(mut scores: [Fraction; NUM_NODES]) -> [u32; NUM_NODES] {

    //begin with [1,2, ..., N] ranking
    let mut args: [u32; NUM_NODES] = [0; NUM_NODES];
    for i in 0..NUM_NODES {
        args[i] = i;
    }

    for _ in 0..NUM_NODES {
        for i in 0..(NUM_NODES-1) {
            if (compareFraction(scores[i],scores[i+1]) == 1) { // LHS > RHS 
                let mut aux = args[i];
                args[i] = args[i+1];
                args[i+1] = aux;

                let mut score_aux = scores[i];
                scores[i] = scores[i+1];
                scores[i+1] = score_aux;
            }
        }

    }

    args
}

// inefficient (but easy to understand) bubble sort
fn final_arg_sort_int(mut scores: [u32; NUM_NODES]) -> [u32; NUM_NODES] {

    //begin with [1,2, ..., N] ranking
    let mut args: [u32; NUM_NODES] = [0; NUM_NODES];
    for i in 0..NUM_NODES {
        args[i] = i;
    }

    for _ in 0..NUM_NODES {
        for i in 0..(NUM_NODES-1) {
            if scores[i]>scores[i+1] { // LHS > RHS 
                let mut aux = args[i];
                args[i] = args[i+1];
                args[i+1] = aux;

                let mut score_aux = scores[i];
                scores[i] = scores[i+1];
                scores[i+1] = score_aux;
            }
        }

    }

    args
}


fn main(edgelist: [Field; MAX_NUM_EDGES]) -> pub [u32; NUM_NODES] { 

    let mut transition_matrix: [u32; NUM_ENTRIES] = edgelist_to_matrix(edgelist);
    //let mut normalized_matrix: [Fraction; NUM_ENTRIES] = normalize_transition_matrix(transition_matrix);

    //let mut probabilities: [Fraction; NUM_NODES] = [Fraction{sign: true, num: 1, den: NUM_NODES}; NUM_NODES];
    //probabilities = iterate(normalized_matrix, probabilities);
    //final_arg_sort(probabilities)

    let mut probabilities: [u32; NUM_NODES] = [1; NUM_NODES];
    for _ in 0..10 {
        probabilities = iterate_int(transition_matrix,probabilities);
    }
    final_arg_sort_int(probabilities)


}

#[test]
fn test_main_fcn() {
    let edgelist = [1, 17, 6, 5, 40, 23, 30, 12, 24, 28];

    let result = main(edgelist);
    println(result);
}
