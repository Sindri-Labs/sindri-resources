// Code generated from Python2Noir
// Fixed number is 10
// scale: 255/39.95000076293945
// zero-point: 13
use dep::std::println;

fn main(inputs : [i32;6],) -> pub u3 {
	if inputs[2] <= 128 {
		if inputs[0] <= 147 {
			1
		} else {
			0
		}
	} else {
		if inputs[0] <= 102 {
			0
		} else {
			if inputs[1] <= 128 {
				2
			} else {
				if inputs[3] <= 128 {
					2
				} else {
					3
				}
			}
		}
	}
}


#[test]
fn test_main() {
    let inputs: [i32; 6] = [255, 0, 255, 255, 0, 255];
    let res = main(inputs);
    println(res);
    assert(res==2);

    // Uncomment to make test fail
    // main(1, 1);
}