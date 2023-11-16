use dep::std::println;

//Quantization scale factor
global SCALE = 20000;

//PageRank hyperparameters
global DAMPING_SCALE = 5;
global DAMPING = 4;
global ITERATIONS: u32 = 10;

//Graph Size Restrictions
global NUM_NODES: u32 = 20;
global NUM_ENTRIES: u32 = 400; // NUM_NODES**2
global MAX_NUM_EDGES: u32 = 30;


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


fn normalize_transition_matrix(transition_matrix: [u32; NUM_ENTRIES]) -> [u32; NUM_ENTRIES] {
    let mut normalized: [u32; NUM_ENTRIES] = [0; NUM_ENTRIES];

    for row in 0..NUM_NODES {
        let mut out_degree = 0;
        for column in 0..NUM_NODES { // iterate once to get total out degree of a node
            out_degree = out_degree + transition_matrix[column*NUM_NODES+row];
        }
        for column in 0..NUM_NODES { // normalize that row of the transition matrix
            if (out_degree != 0) {
                normalized[column*NUM_NODES+row] = transition_matrix[column*NUM_NODES+row]* SCALE / out_degree;
            } else {
                normalized[column*NUM_NODES+row] = SCALE/NUM_NODES;
            }
            normalized[column*NUM_NODES+row] = normalized[column*NUM_NODES+row] * DAMPING / DAMPING_SCALE;
            normalized[column*NUM_NODES+row] = normalized[column*NUM_NODES+row] + SCALE * (DAMPING_SCALE - DAMPING) / DAMPING_SCALE / NUM_NODES;
        }
    }
    normalized
}


fn iterate(mat: [u32; NUM_ENTRIES], vec: [u32; NUM_NODES]) -> [u32; NUM_NODES] {

    let mut next_probabilities: [u32; NUM_NODES] = [0; NUM_NODES];
    for row in 0..NUM_NODES {
        let mut dot_product = 0;
        for column in 0..NUM_NODES {
            dot_product = dot_product+mat[row*NUM_NODES+column]*vec[column];
        }
        next_probabilities[row] = dot_product/SCALE;
    }
    
    next_probabilities
}

// inefficient (but easy to understand) bubble sort
fn arg_sort(mut scores: [u32; NUM_NODES]) -> [u32; NUM_NODES] {

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
    for _ in 0..ITERATIONS {
        probabilities = iterate(normalized_matrix,probabilities);
    }

    arg_sort(probabilities)

}

#[test]
fn test_end_probs() {
    let edgelist = [222, 331, 152, 294, 43, 270, 313, 278, 210, 383, 74, 22, 250, 317, 66, 169, 214, 385, 49, 337, 134, 5, 91, 1, 41, 299, 394, 160, 182, 299];
    let end_scores = [ 933,  855, 1622, 1290,  764,  422,  716,  969, 1511,  855,  977,  855,  683, 1021, 1839, 1409,  933,  422,  422, 1401];

    let mut transition_matrix: [u32; NUM_ENTRIES] = edgelist_to_matrix(edgelist);
    let mut normalized_matrix: [u32; NUM_ENTRIES] = normalize_transition_matrix(transition_matrix);
    let unit = SCALE/NUM_NODES;
    let mut probabilities: [u32; NUM_NODES] = [unit; NUM_NODES];
    for _ in 0..ITERATIONS {
        probabilities = iterate(normalized_matrix,probabilities);
    }

    for i in 1..NUM_NODES {
        assert(probabilities[i] == end_scores[i]);
    }

}


#[test]
fn test_bubble_sort() {
    let test = [7, 5, 1, 16, 11, 3, 9, 8, 4, 15, 2, 0, 14, 17, 12, 19, 6, 13, 18, 10];
    let result = [11, 2, 10, 5, 8, 1, 16, 0, 7, 6, 19, 4, 14, 17, 12, 9, 3, 13, 18, 15];
    let argsort = arg_sort(test);
    for i in 1..NUM_NODES {
        assert(result[i] == argsort[i]);
    }

    let test = [10, 7, 14, 4, 19, 11, 15, 17, 5, 0, 16, 6, 18, 12, 13, 1, 3, 2, 8, 9];
    let result = [9, 15, 17, 16, 3, 8, 11, 1, 18, 19, 0, 5, 13, 14, 2, 6, 10, 7, 12, 4];
    let argsort = arg_sort(test);
    for i in 1..NUM_NODES {
        assert(result[i] == argsort[i]);
    }

    let test = [8, 19, 0, 4, 15, 9, 17, 16, 10, 3, 13, 14, 2, 12, 5, 6, 11, 18, 1, 7];
    let result = [2, 18, 12, 9, 3, 14, 15, 19, 0, 5, 8, 16, 13, 10, 11, 4, 7, 6, 17, 1];
    let argsort = arg_sort(test);
    for i in 1..NUM_NODES {
        assert(result[i] == argsort[i]);
    }

}

