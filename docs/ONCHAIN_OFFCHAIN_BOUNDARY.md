# On-Chain / Off-Chain Boundary Guide

## Purpose

Deciding what belongs on-chain and what belongs off-chain is the first design decision a team
makes when building on Cougr, and the one most likely to produce a design that cannot ship. The
two failure modes are symmetric:

- putting too much on-chain, producing a contract that is correct in a unit test and too expensive
  or too slow to run as a real game
- putting too much off-chain, producing a game that is described as on-chain but whose outcome is
  actually decided by a server the players have to trust

This guide gives a repeatable way to make that call for each piece of state and logic in a
specific game, and then works through three shipped examples to show the call being made in
practice.

It does not restate Cougr's performance model or its privacy guarantees. Those live in
[PERFORMANCE.md](./PERFORMANCE.md) and [PRIVACY_MODEL.md](./PRIVACY_MODEL.md), and this guide links
into them rather than duplicating them.

## How to read the examples

Every claim in the worked examples below is drawn from the current source of the example it
describes, not from an idealized version of it. Where an example takes a shortcut, the shortcut is
named. That is deliberate: a boundary guide that only shows the clean case is not usable for the
decision it is supposed to help with.

## The framework

Take one piece of game state or one rule at a time and ask these five questions in order. The
first question that produces a clear answer usually settles the placement.

### 1. Does a player need to verify this without trusting anyone?

If a player has to be able to check the result themselves, with no trusted operator in the loop,
the check belongs on-chain. This is the only question that can force state on-chain on its own,
and it applies to a smaller share of game state than most teams initially assume.

Typical yes: win conditions, balances and prize distribution, anything a player could be cheated
out of. Typical no: cosmetic state, replay history, lobby membership, matchmaking.

### 2. What specific cheat does this prevent?

Name the cheat. If you cannot name a concrete way a player profits from tampering with a piece of
state, that state does not need the chain's tamper resistance, and paying for it on-chain buys
nothing.

"Someone could change it" is not a cheat. "The defender could claim a miss on a cell that actually
holds a ship, and never lose" is a cheat, and it tells you exactly what the contract has to
enforce.

### 3. Can a commitment stand in for the data?

This is the question that resolves most of the apparent conflicts between questions 1 and 2. The
chain often does not need the data, only a binding promise about it, so that a later reveal can be
checked against the promise.

A hash commitment, a Merkle root, or a proof is a fixed-size on-chain stand-in for state that
stays off-chain until it is needed. Cougr's `privacy::stable` surface (Stable) provides the
commit-reveal and Merkle primitives for this; see [PRIVACY_MODEL.md](./PRIVACY_MODEL.md) for what
each tier promises. When this substitution works, it is almost always the right answer: the
verification property from question 1 is preserved while the storage cost collapses to 32 bytes.

### 4. What does it cost in Soroban's resource dimensions?

Soroban meters CPU instructions, ledger entry reads and writes, read and write bytes, and
transaction size, each against its own limit. A design can be well inside the fee budget and still
fail because it crosses one dimension's ceiling. This is different from a single gas number, and
it is the reason a cost intuition carried over from an EVM chain does not transfer cleanly.

The current limits and the fee model are documented upstream in
[Fees, resource limits, and metering](https://developers.stellar.org/docs/learn/fundamentals/fees-resource-limits-metering).
Read them there rather than from a copy in this repository, because they change with protocol
versions.

Two Cougr-specific points affect the arithmetic:

- The choice between `SimpleWorld` and `ArchetypeWorld`, and between table and sparse component
  storage, changes how much of the world a query touches per invocation. The decision heuristics
  are in [PERFORMANCE.md](./PERFORMANCE.md).
- `src/incremental/` dirty-tracking means unchanged component data is not rewritten on every tick,
  so the cost of on-chain state is closer to the cost of the state you actually mutate than to the
  total size of the world. This lowers the penalty for keeping state on-chain, but it does not
  remove it.

Per-system resource reporting in `GameHarness`, which would let you answer this question from a
test rather than by reasoning, is planned work rather than something you can use today. It is
tracked as Phase 2 of the [roadmap](./strategy/13-roadmap.md) and analyzed in
[04-onchain-gaming-research.md](./strategy/04-onchain-gaming-research.md). Until it lands, treat
the resource question as a design-time estimate to be confirmed against a real network.

### 5. Who submits the transaction, and how often?

Every on-chain state change is a transaction that someone signs, pays for, and waits a ledger
close for. State that changes many times per second cannot be advanced by a transaction per
change, regardless of how cheap each one is.

If the answer is "many times per second", the state is not going on-chain in its raw form. Either
the game becomes turn-based, or the frequent state moves off-chain and the chain holds periodic
checkpoints, commitments, or a final result.

### Summary

| Answer pattern | Placement |
|---|---|
| Player must verify it, and it is small | On-chain, directly |
| Player must verify it, but it is large or secret | Off-chain, with an on-chain commitment or proof |
| No nameable cheat, and no verification need | Off-chain |
| Changes faster than one transaction per change | Off-chain, with on-chain checkpoints or a final result |
| Needed only to render or explain the game | Off-chain, ideally rebuilt from on-chain events |

## Worked example: Battleship

Source: [`examples/battleship`](../examples/battleship). Turn-based, two players, hidden board
layout. This is the reference case for question 3.

### What the contract holds

From `src/lib.rs`, `GameState` in instance storage holds, per player, a `commitment` and a
`merkle_root` (both `BytesN<32>`), an `AttackGrid` of resolved cells, a `ShipStatus` remaining
count, the `TurnState`, and the `winner`. `BoardCommitment` is registered as a table-storage
component through `impl_component!`.

Nothing in that list is the board. The contract never learns where the ships are, and never needs
to.

### What stays off the chain

Ship placement, the salt, the full board array, and the Merkle tree built over it are all client
side. The player computes a commitment over the board and salt, builds a Merkle tree whose leaves
are the individual cells, and sends only the two 32-byte roots on-chain via `commit_board`.
Generating the per-cell proof at reveal time is also client-side work.

### Why the split holds

Run question 2 against it. The cheat is a defender who answers "miss" for a cell that holds a
ship. `reveal_cell` closes it: the defender must supply an `OnChainMerkleProof` for the exact
coordinate the attacker named, the leaf is recomputed on-chain by `leaf_hash` from the coordinate
and the claimed value, and `Sha256MerkleProofVerifier` checks it against the `merkle_root`
recorded during setup. A defender who lies about a cell cannot produce a proof that verifies
against a root they committed to before the attack was made.

The board itself never had to be on-chain for that property to hold. A 10x10 board reduces to 32
bytes of commitment plus one proof per attacked cell, and the proofs are paid for only on the
cells actually attacked rather than on all one hundred up front.

### What the split does not buy

The contract verifies that each revealed cell matches the committed board. It does not verify that
the committed board is a legal fleet. `TOTAL_SHIP_CELLS` is a constant of 17, and `ShipStatus`
starts both players there by assumption; nothing checks that the committed Merkle tree actually
contains 17 cells with value 1, or that they form valid ship shapes. A player who commits a board
with fewer ship cells than the rules require never reaches zero remaining and therefore cannot
lose.

That is the honest state of the example, and it is a useful illustration of the general point: a
commitment binds a player to whatever they committed, not to the rules. Enforcing fleet legality
needs either a validity proof supplied at commit time or a full reveal and check at the end of the
game, and each of those is a separate boundary decision with its own cost.

## Worked example: Snake

Source: [`examples/snake`](../examples/snake). Single player, real-time in its original arcade
form. This is the reference case for question 5, and the example where the honest framing matters
most.

### What is genuinely on-chain

More than a reader might expect. The entire simulation runs in the contract: `SimpleWorld` in
persistent storage holds the snake head, the segments, and the food as entities with `Position`
and direction components, and `update_tick` builds a `GameApp`, runs `move_snake` in `Update`,
then `self_collision` and `food_collision` in `PostUpdate`, and writes the world back. Score,
game-over state, and every segment position are on-chain and independently checkable. There is no
off-chain simulator that the chain trusts.

### What is approximated

The game is not real-time. Each tick is a separate contract invocation, and each direction change
is another one, so the game advances exactly as fast as someone submits transactions and ledgers
close. The README states this plainly: rendering and real-time scheduling are out of scope, and
callers drive ticks through contract invocations. A client can render at sixty frames per second,
but it is rendering interpolation between on-chain ticks, not the game state itself.

This is the distinction the guide exists to make. Snake is not a real-time game that was made
on-chain. It is a turn-based game with a single-player arcade presentation, and calling it
anything else would set an expectation the network cannot meet.

### Two further shortcuts worth naming

Food placement is derived from the tick counter: `spawn_food` computes candidate positions from
`tick` and an attempt counter with fixed multipliers. That is deterministic and reproducible,
which is what an example needs, and it is fully predictable to anyone who can read the contract.
Question 2 applied to a single-player example with no stake gives "no cheat worth preventing", so
the shortcut is appropriate here. It would not be appropriate the moment food placement affects a
prize, at which point the placement needs `circuits::FairDiceBuilder` (Experimental) or a
commit-reveal scheme rather than a tick hash.

There is also no authorization anywhere in the example: `require_auth` does not appear in
`examples/snake/src/`, so any account can call `change_direction` or `update_tick` on any game.
For a single-player reference with one game per contract instance this is a deliberate
simplification, and it is one of the first things a real deployment would have to change.

### The general shape for real-time games

When a game genuinely needs sub-second simulation, the boundary moves rather than disappearing:
simulate off-chain, and put on-chain only what question 1 demands. In practice that is the entry
stake, periodic checkpoints, and a final score commitment, with the disputed cases resolved by
replaying a committed input log against the same deterministic systems. Cougr's per-tick
determinism is what makes that replay possible, but the architecture around it is multiplayer
synchronization, which is a distinct topic from the boundary itself and is not covered here.

## Worked example: Blind Auction

Source: [`examples/blind_auction`](../examples/blind_auction). Sealed-bid auction, hidden bid
values. This is the reference case for a proof standing in for data rather than a hash.

### The split

The commit phase is off-chain. The README says so directly: bidders record hash commitments of
their bids off-chain or in standard storage, and the contract's only entry points are
`init_auction`, `reveal_bid`, and `bid_reveal`.

On reveal, the bidder submits the commitment, the claimed bid value, and a `Groth16Proof`.
`reveal_bid` loads the auction config, rebuilds the circuit spec through
`circuits::sealed_bid(&env, max_bid)`, and calls `verify_bid_reveal` against the auction ID, the
commitment, and the revealed value. The reveal is recorded only if verification returns true.

### Why a proof rather than a plain hash

A plain hash commitment proves the revealed bid is the committed bid. It proves nothing about the
bid's relationship to the auction's rules. The `sealed_bid` circuit binds the reveal to the
auction ID and checks the value against `max_bid` as part of the same verification, so a bidder
cannot reuse a commitment across auctions or reveal a bid outside the allowed range. That is the
extra property the proof buys over a hash, and it is the criterion for choosing one over the
other.

[`examples/hidden_hand`](../examples/hidden_hand) applies the same pattern to card deals through
`circuits::hidden_cards`, verifying a hand commitment against a deck root without the contract
learning the hand.

### The maturity caveat, stated plainly

Groth16 verification and the prebuilt circuits are **Experimental** in
[PRIVACY_MODEL.md](./PRIVACY_MODEL.md), and they are excluded from the 1.0 stable privacy
contract. The circuits ship with test-only proving keys and there is no production trusted setup
today, which is tracked as a Phase 3 roadmap item. Commitments, commit-reveal, and Merkle
inclusion are **Stable** by contrast.

The practical consequence for a boundary decision: if hidden information can be handled with a
commitment and a Merkle proof, as Battleship does, that path is Stable today. Reach for a circuit
when you need a property a commitment cannot express, and plan for the trusted-setup gap before
mainnet.

### What the example does not do

Winner computation is not in the contract. `blind_auction` verifies that each revealed bid is
valid and stores it; comparing bids and settling the auction is left out. A production auction
would have to decide where that comparison happens, and it is a good exercise for the framework
above: the comparison is small, needs no secret input once reveals are on-chain, and directly
determines who gets paid, so questions 1 and 2 both point on-chain.

## What Soroban cannot support

Stated directly, so that nothing above is read as a promise it does not make:

- **Frame-rate gameplay on-chain is not achievable.** State advances one transaction per ledger
  close. Any game loop that needs to advance faster than that runs off-chain, and the chain holds
  commitments, checkpoints, or results.
- **There is no on-chain clock a game can tick against.** Contracts execute when someone invokes
  them. A game that must advance on its own needs an off-chain caller, and that caller is part of
  the trust model whether or not the design acknowledges it.
- **Simultaneous action is not free.** Two players acting in the same instant are two transactions
  whose order is decided by the network. If simultaneity matters to fairness, it needs
  commit-reveal, not a hope about ordering.
- **Large per-tick state is limited by resource ceilings, not just by fees.** A world that fits in
  a test can exceed a read-bytes or ledger-entry limit on-chain. Incremental persistence reduces
  this pressure; it does not remove the ceiling.
- **Nothing on-chain hides data.** Contract state is public. Values are hidden by not putting them
  on-chain and committing to them instead, which is what the Battleship and Blind Auction examples
  do.

None of this makes on-chain games impractical. It makes the boundary the design decision rather
than an implementation detail, which is why it is worth making explicitly and early.

## A short checklist

For each piece of state or rule in your game:

1. Name the cheat it prevents. No cheat means no reason for it to be on-chain.
2. If a player must verify it, check whether a commitment or proof can carry the property instead
   of the raw data.
3. Count the transactions its normal use implies. More than one per player action is a warning.
4. Estimate its cost across Soroban's resource dimensions, not as a single number, and confirm
   against a real network before committing to the design.
5. Write down what the placement does not protect, the way the Battleship fleet-legality gap is
   written down above. That note is what a future reader needs most.

## Related documents

| Document | What it covers that this guide does not |
|---|---|
| [PERFORMANCE.md](./PERFORMANCE.md) | Backend and storage choice, query cost model, benchmark interpretation |
| [PRIVACY_MODEL.md](./PRIVACY_MODEL.md) | What each privacy tier promises, and the Experimental boundary around Groth16 |
| [PATTERNS.md](./PATTERNS.md) | Which Cougr module answers a given gameplay problem |
| [strategy/04-onchain-gaming-research.md](./strategy/04-onchain-gaming-research.md) | Why this decision ranks as the highest-impact friction point |
| [strategy/13-roadmap.md](./strategy/13-roadmap.md) | Phase 2 resource-cost reporting in `GameHarness` |
