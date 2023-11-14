use dep::std::println;

global SCALE = 10000;
global DAMPING_SCALE = 5;
global DAMPING = 4;

global NUM_NODES: u32 = 10;
global NUM_ENTRIES: u32 = 100; // NUM_NODES**2
global MAX_NUM_EDGES: u32 = 10;
global MAX_ITERATIONS: u32 = 100;


// implementing our own simple quantizer:
//    N = number of nodes
//    v = np.ones(N) / N
//    M_hat = (d * M + (1 - d) / N)
//    for i in range(num_iterations):
//        v = M_hat @ v

// M is the normalized transition matrix - this is the only issue
// we need to implement a rounding for SCALE / degree to initialize
// entries. As long as N divides SCALE I think we're okay?

fn normalize_transition_matrix(transition_matrix: [u32; NUM_ENTRIES]) -> [u32; NUM_ENTRIES] {
    let mut normalized: [u32; NUM_ENTRIES] = [0; NUM_ENTRIES];

    for row in 0..NUM_NODES {
        let mut out_degree = 0;
        for column in 0..NUM_NODES { // iterate once to get total out degree of a node
            out_degree = out_degree + transition_matrix[row*NUM_NODES+column];
        }
        for column in 0..NUM_NODES { // normalize that row of the transition matrix
            if (out_degree != 0) {
                normalized[row*NUM_NODES+column] = transition_matrix[row*NUM_NODES+column]* SCALE / out_degree;
            } else {
                normalized[row*NUM_NODES+column] = 0;
            }
            normalized[row*NUM_NODES+column] = normalized[row*NUM_NODES+column] * DAMPING / DAMPING_SCALE;
            normalized[row*NUM_NODES+column] = normalized[row*NUM_NODES+column] + SCALE * (DAMPING_SCALE - DAMPING) / DAMPING_SCALE / NUM_NODES;
        }
    }
    normalized
}






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
    let mut normalized_matrix: [u32; NUM_ENTRIES] = normalize_transition_matrix(transition_matrix);


    let unit = SCALE/NUM_NODES;
    let mut probabilities: [u32; NUM_NODES] = [unit; NUM_NODES];
    for _ in 0..10 {
        probabilities = iterate_int(normalized_matrix,probabilities);
    }
    final_arg_sort_int(probabilities)

}

#[test]
fn test_main_fcn() {
    let edgelist = [1, 17, 6, 5, 40, 23, 30, 12, 24, 28];

    let result = main(edgelist);
    println(result);
}
