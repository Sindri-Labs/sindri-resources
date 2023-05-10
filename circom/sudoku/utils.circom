pragma circom 2.0.8;

include "circomlib/comparators.circom";
include "circomlib/gates.circom";

/**
 * Check if the puzzle is valid.
 */
template IsValidPuzzle() {
    signal input puzzle[81];
    signal output result;
    var i,j,k = 0;

    component getNumberGroupForRow[9];
    component getNumberGroupForColumn[9];
    component getNumberGroupForBox[9];
    component validRowCheck[9];
    component validColumnCheck[9];
    component validBoxCheck[9];
    component allCheck = MultiAND(27);

    //
    // Check if 9 rows are all valid.
    //
    for (i = 0; i < 9; i++) {
        getNumberGroupForRow[i] = GetNumberGroupForRow(i);
        for (j = 0; j < 81; j++) {
            getNumberGroupForRow[i].board[j] <== puzzle[j];
        }

        validRowCheck[i] = IsValidPuzzleNumberGroup();
        for (j = 0; j < 9; j++) {
            validRowCheck[i].numberGroup[j] <== getNumberGroupForRow[i].numberGroup[j];
        }
        
        allCheck.in[k] <== validRowCheck[i].result;
        k++;
    }

    //
    // Check if 9 columns are all valid.
    //
    for (i = 0; i < 9; i++) {
        getNumberGroupForColumn[i] = GetNumberGroupForColumn(i);
        for (j = 0; j < 81; j++) {
            getNumberGroupForColumn[i].board[j] <== puzzle[j];
        }

        validColumnCheck[i] = IsValidPuzzleNumberGroup();
        for (j = 0; j < 9; j++) {
            validColumnCheck[i].numberGroup[j] <== getNumberGroupForColumn[i].numberGroup[j];
        }
        
        allCheck.in[k] <== validColumnCheck[i].result;
        k++;
    }

    //
    // Check if 9 boxes are all valid.
    //
    for (i = 0; i < 9; i++) {
        getNumberGroupForBox[i] = GetNumberGroupForBox(i);
        for (j = 0; j < 81; j++) {
            getNumberGroupForBox[i].board[j] <== puzzle[j];
        }

        validBoxCheck[i] = IsValidPuzzleNumberGroup();
        for (j = 0; j < 9; j++) {
            validBoxCheck[i].numberGroup[j] <== getNumberGroupForBox[i].numberGroup[j];
        }
        
        allCheck.in[k] <== validBoxCheck[i].result;
        k++;
    }
    k === 27;

    result <== allCheck.out;
}

/**
 * Check if the solution is complete and correct.
 */
template IsValidSolution() {
    signal input solution[81];
    signal output result;
    var i,j,k = 0;

    component getNumberGroupForRow[9];
    component getNumberGroupForColumn[9];
    component getNumberGroupForBox[9];
    component validRowCheck[9];
    component validColumnCheck[9];
    component validBoxCheck[9];
    component allCheck = MultiAND(27);

    //
    // Check if 9 rows are all valid.
    //
    for (i = 0; i < 9; i++) {
        getNumberGroupForRow[i] = GetNumberGroupForRow(i);
        for (j = 0; j < 81; j++) {
            getNumberGroupForRow[i].board[j] <== solution[j];
        }

        validRowCheck[i] = IsValidSolutionNumberGroup();
        for (j = 0; j < 9; j++) {
            validRowCheck[i].numberGroup[j] <== getNumberGroupForRow[i].numberGroup[j];
        }
        
        allCheck.in[k] <== validRowCheck[i].result;
        k++;
    }

    //
    // Check if 9 columns are all valid.
    //
    for (i = 0; i < 9; i++) {
        getNumberGroupForColumn[i] = GetNumberGroupForColumn(i);
        for (j = 0; j < 81; j++) {
            getNumberGroupForColumn[i].board[j] <== solution[j];
        }

        validColumnCheck[i] = IsValidSolutionNumberGroup();
        for (j = 0; j < 9; j++) {
            validColumnCheck[i].numberGroup[j] <== getNumberGroupForColumn[i].numberGroup[j];
        }
        
        allCheck.in[k] <== validColumnCheck[i].result;
        k++;
    }

    //
    // Check if 9 boxes are all valid.
    //
    for (i = 0; i < 9; i++) {
        getNumberGroupForBox[i] = GetNumberGroupForBox(i);
        for (j = 0; j < 81; j++) {
            getNumberGroupForBox[i].board[j] <== solution[j];
        }

        validBoxCheck[i] = IsValidSolutionNumberGroup();
        for (j = 0; j < 9; j++) {
            validBoxCheck[i].numberGroup[j] <== getNumberGroupForBox[i].numberGroup[j];
        }
        
        allCheck.in[k] <== validBoxCheck[i].result;
        k++;
    }
    k === 27;

    result <== allCheck.out;
}

/**
 * Check if the number is in range [from, to].(including from, to)
 */
template IsNumberInRange(from, to) {
    signal input number;
    signal output result;

    component greater = GreaterEqThan(4);
    component less = LessEqThan(4);

    greater.in[0] <== number;
    greater.in[1] <== from;

    less.in[0] <== number;
    less.in[1] <== to;

    component and = AND();
    and.a <== greater.out;
    and.b <== less.out;

    result <== and.out;
}

/**
 * Check if the number group is solved state.
 * All numbers are in range of [1~9].
 * No two numbers are duplicated in group.
 *
 * example: [1, 3, 2, 9, 7, 5, 6, 4, 8]
 */
template IsValidSolutionNumberGroup() {
    signal input numberGroup[9];
    signal output result;
    var i,j;

    //
    // Constraint 1: All numbers are in range of [1~9].
    //
    component allNumbersAreInRange = MultiAND(9);
    component isNumberInRange[9];
    for (i = 0; i < 9; i++) {
        isNumberInRange[i] = IsNumberInRange(1, 9);
        isNumberInRange[i].number <== numberGroup[i];
        allNumbersAreInRange.in[i] <== isNumberInRange[i].result;
    }

    //
    // Constraint 2: No two numbers are duplicated in group.
    //
    component allNumbersAreDifferentCheck = MultiAND(36);
    component equalCheck[36];
    var k = 0;
    for (i = 0; i < 8; i ++) {
        for (j = i + 1; j < 9; j++) {
            equalCheck[k] = IsEqual();
            equalCheck[k].in[0] <== numberGroup[i];
            equalCheck[k].in[1] <== numberGroup[j];
            allNumbersAreDifferentCheck.in[k] <== 1 - equalCheck[k].out;
            k++;
        }
    }
    k === 36;

    component and = AND();
    and.a <== allNumbersAreInRange.out;
    and.b <== allNumbersAreDifferentCheck.out;

    result <== and.out;
}

/**
 * Check if the number group is valid for puzzle.
 * All numbers are in range of [0~9]. 0 means empty slot.
 * No two numbers are duplicated in group if they are not zero.
 *
 * example: [1, 0, 2, 0, 0, 5, 6, 0, 8]
 */
template IsValidPuzzleNumberGroup() {
    signal input numberGroup[9];
    signal output result;
    var i,j;

    //
    // Constraint 1: All numbers are in range of [0~9].
    //
    component allNumbersAreInRange = MultiAND(9);
    component isNumberInRange[9];
    for (i = 0; i < 9; i++) {
        isNumberInRange[i] = IsNumberInRange(0, 9);
        isNumberInRange[i].number <== numberGroup[i];
        allNumbersAreInRange.in[i] <== isNumberInRange[i].result;
    }

    //
    // Constraint 2: No two numbers are duplicated in group if they are not zero.
    //
    component allNumbersAreDifferentCheck = MultiAND(36);
    component equalCheck[36];
    var k = 0;
    for (i = 0; i < 8; i ++) {
        for (j = i + 1; j < 9; j++) {
            equalCheck[k] = IsEqual();
            equalCheck[k].in[0] <== numberGroup[i];
            equalCheck[k].in[1] <== numberGroup[j];
            allNumbersAreDifferentCheck.in[k] <-- (numberGroup[i] == 0 || numberGroup[j] == 0) ? 1 : (1 - equalCheck[k].out);
            k++;
        }
    }
    k === 36;

    component and = AND();
    and.a <== allNumbersAreInRange.out;
    and.b <== allNumbersAreDifferentCheck.out;

    result <== and.out;
}

/**
 * Check if the solution is matched with the puzzle.
 * example: solution: [1, 3, 2, 9, 7, 5, 6, 4, 8, ...], puzzle: [1, 0, 0, 9, 0, 5, 0, 4, 8, ...]
 */
template IsValidSolutionOfPuzzle() {
    signal input solution[81];
    signal input puzzle[81];
    signal output result;

    component allNumbersAreEqualCheck = MultiAND(81);
    component equalCheck[81];
    for (var i = 0; i < 81; i ++) {
        equalCheck[i] = IsEqual();
        equalCheck[i].in[0] <== solution[i];
        equalCheck[i].in[1] <== puzzle[i];
        allNumbersAreEqualCheck.in[i] <-- (puzzle[i] == 0) ? 1 : equalCheck[i].out;
    }

    result <== allNumbersAreEqualCheck.out;
}

/**
 * Get 9 numbers of the row.
 */
template GetNumberGroupForRow(index) {
    signal input board[81];
    signal output numberGroup[9];

    for (var i = 0; i < 9; i++) {
        numberGroup[i] <== board[index * 9 + i];
    }
}

/**
 * Get 9 numbers of the column.
 */
template GetNumberGroupForColumn(index) {
    signal input board[81];
    signal output numberGroup[9];

    for (var i = 0; i < 9; i++) {
        numberGroup[i] <== board[index + i * 9];
    }
}

/**
 * Get 9 numbers of the box.
 */
template GetNumberGroupForBox(index) {
    signal input board[81];
    signal output numberGroup[9];
    var boxStarts[9] = [0, 3, 6, 27, 30, 33, 54, 57, 60];

    for (var i = 0; i < 9; i++) {
        var position = boxStarts[index] + (i \ 3) * 9 + (i % 3);
        numberGroup[i] <== board[position];
    }
}