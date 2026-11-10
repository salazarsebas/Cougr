pragma circom 2.1.0;

include "poseidon.circom";
include "lib/deck.circom";

template HiddenCards(DECK_SIZE, HAND_SIZE) {
    signal input deck_root;
    signal input hand_commitment;
    signal input player_id;
    signal input deck_size;
    signal input hand_size;

    signal input hand[HAND_SIZE];
    signal input deck[DECK_SIZE];
    signal input deck_salt;

    deck_size === DECK_SIZE;
    hand_size === HAND_SIZE;

    component handHash = Poseidon(HAND_SIZE + 1);
    handHash.inputs[0] <== player_id;
    for (var i = 0; i < HAND_SIZE; i++) {
        handHash.inputs[i + 1] <== hand[i];
    }
    hand_commitment === handHash.out;

    component deckCommit = DeckCommitment(DECK_SIZE);
    deckCommit.salt <== deck_salt;
    for (var d = 0; d < DECK_SIZE; d++) {
        deckCommit.deck[d] <== deck[d];
    }
    deck_root === deckCommit.root;

    component unique = HandUnique(HAND_SIZE);
    for (var u = 0; u < HAND_SIZE; u++) {
        unique.hand[u] <== hand[u];
    }

    component range[HAND_SIZE];
    component member[HAND_SIZE];
    for (var h = 0; h < HAND_SIZE; h++) {
        range[h] = CardRange(DECK_SIZE);
        range[h].card <== hand[h];

        member[h] = CardInDeck(DECK_SIZE);
        member[h].card <== hand[h];
        for (var j = 0; j < DECK_SIZE; j++) {
            member[h].deck[j] <== deck[j];
        }
    }
}

component main {public [deck_root, hand_commitment, player_id, deck_size, hand_size]} = HiddenCards(52, 5);