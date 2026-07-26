pragma circom 2.1.0;

include "poseidon.circom";
include "comparators.circom";

template DeckCommitment(n) {
    signal input salt;
    signal input deck[n];
    signal output root;

    component h0 = Poseidon(2);
    h0.inputs[0] <== salt;
    h0.inputs[1] <== deck[0];

    component chain[n - 1];
    for (var i = 0; i < n - 1; i++) {
        chain[i] = Poseidon(2);
        if (i == 0) {
            chain[i].inputs[0] <== h0.out;
            chain[i].inputs[1] <== deck[1];
        } else {
            chain[i].inputs[0] <== chain[i - 1].out;
            chain[i].inputs[1] <== deck[i + 1];
        }
    }
    root <== chain[n - 2].out;
}

template CardInDeck(n) {
    signal input card;
    signal input deck[n];

    component eq[n];
    signal hits[n];
    for (var j = 0; j < n; j++) {
        eq[j] = IsEqual();
        eq[j].in[0] <== card;
        eq[j].in[1] <== deck[j];
        hits[j] <== eq[j].out;
    }

    signal total[n + 1];
    total[0] <== 0;
    for (var k = 0; k < n; k++) {
        total[k + 1] <== total[k] + hits[k];
    }

    component has = GreaterThan(16);
    has.in[0] <== total[n];
    has.in[1] <== 0;
    has.out === 1;
}

template CardRange(maxCard) {
    signal input card;

    component lo = GreaterEqThan(16);
    lo.in[0] <== card;
    lo.in[1] <== 1;
    lo.out === 1;

    component hi = LessEqThan(16);
    hi.in[0] <== card;
    hi.in[1] <== maxCard;
    hi.out === 1;
}

template HandUnique(handSize) {
    signal input hand[handSize];

    var pairs = handSize * (handSize - 1) / 2;
    component eq[pairs];
    var idx = 0;
    for (var i = 0; i < handSize; i++) {
        for (var j = i + 1; j < handSize; j++) {
            eq[idx] = IsEqual();
            eq[idx].in[0] <== hand[i];
            eq[idx].in[1] <== hand[j];
            eq[idx].out === 0;
            idx++;
        }
    }
}