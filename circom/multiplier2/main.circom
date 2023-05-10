pragma circom 2.0.0;

/*
This circuit template checks that c is the multiplication of a and b.
Source (March 12, 2023): 
https://docs.circom.io/getting-started/writing-circuits/
https://docs.circom.io/getting-started/compiling-circuits/
*/  

template Multiplier2 () {  

   // Declaration of signals.  
   signal input a;  
   signal input b;  
   signal output c;  

   // Constraints.  
   c <== a * b;  
}

 component main = Multiplier2();