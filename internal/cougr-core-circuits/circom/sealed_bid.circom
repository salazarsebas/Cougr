pragma circom 2.1.0;

include "poseidon.circom";
include "comparators.circom";

// Sealed-bid opening: bid_commitment = Poseidon(revealed_bid, bid_salt, auction_id).

template SealedBid() {
    signal input auction_id;
    signal input bid_commitment;
    signal input revealed_bid;
    signal input max_bid;

    signal input bid_salt;

    component lo = GreaterEqThan(32);
    lo.in[0] <== revealed_bid;
    lo.in[1] <== 1;
    lo.out === 1;

    component hi = LessEqThan(32);
    hi.in[0] <== revealed_bid;
    hi.in[1] <== max_bid;
    hi.out === 1;

    component commit = Poseidon(3);
    commit.inputs[0] <== revealed_bid;
    commit.inputs[1] <== bid_salt;
    commit.inputs[2] <== auction_id;
    bid_commitment === commit.out;
}

component main {public [auction_id, bid_commitment, revealed_bid, max_bid]} = SealedBid();