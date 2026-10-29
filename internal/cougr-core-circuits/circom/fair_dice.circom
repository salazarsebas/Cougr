pragma circom 2.1.0;

include "poseidon.circom";
include "comparators.circom";
include "lib/modulo.circom";

// Fair dice: commit seed with nonce, roll = (seed mod sides) + 1.

template FairDice() {
    signal input seed_commitment;
    signal input roll_result;
    signal input sides;
    signal input nonce;

    signal input seed;

    component sidesLo = GreaterThan(16);
    sidesLo.in[0] <== sides;
    sidesLo.in[1] <== 1;
    sidesLo.out === 1;

    component commit = Poseidon(2);
    commit.inputs[0] <== seed;
    commit.inputs[1] <== nonce;
    seed_commitment === commit.out;

    component mod = ModuloBounded(32);
    mod.dividend <== seed;
    mod.divisor <== sides;

    roll_result === mod.remainder + 1;

    component rollHi = LessEqThan(16);
    rollHi.in[0] <== roll_result;
    rollHi.in[1] <== sides;
    rollHi.out === 1;

    component rollLo = GreaterEqThan(16);
    rollLo.in[0] <== roll_result;
    rollLo.in[1] <== 1;
    rollLo.out === 1;
}

component main {public [seed_commitment, roll_result, sides, nonce]} = FairDice();