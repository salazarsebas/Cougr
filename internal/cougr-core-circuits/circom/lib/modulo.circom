pragma circom 2.1.0;

include "comparators.circom";

// Constrain dividend = quotient * divisor + remainder with 0 <= remainder < divisor.
template ModuloBounded(bits) {
    signal input dividend;
    signal input divisor;
    signal output remainder;

    component divPos = GreaterThan(bits);
    divPos.in[0] <== divisor;
    divPos.in[1] <== 1;
    divPos.out === 1;

    signal quotient;
    quotient <-- dividend \ divisor;
    remainder <-- dividend % divisor;

    dividend === quotient * divisor + remainder;

    component lt = LessThan(bits);
    lt.in[0] <== remainder;
    lt.in[1] <== divisor;
    lt.out === 1;
}