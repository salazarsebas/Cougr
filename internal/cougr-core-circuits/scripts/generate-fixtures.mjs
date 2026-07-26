#!/usr/bin/env node
/**
 * Generate fixtures/*.input.json with witnesses that satisfy the real Circom constraints.
 */
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { buildPoseidon } from "circomlibjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const FIXTURES = join(ROOT, "fixtures");

const poseidon = await buildPoseidon();
const F = poseidon.F;

function hash(inputs) {
  return F.toString(poseidon(inputs.map((x) => F.e(x))));
}

function deckRoot(salt, deck) {
  let acc = hash([salt, deck[0]]);
  for (let i = 1; i < deck.length; i++) {
    acc = hash([acc, deck[i]]);
  }
  return acc;
}

mkdirSync(FIXTURES, { recursive: true });

// hidden_cards (deck and hand sizes can be customized via env vars DECK_SIZE and HAND_SIZE)
const DECK_SIZE = Number(process.env.DECK_SIZE || "52");
const HAND_SIZE = Number(process.env.HAND_SIZE || "5");
const deck = Array.from({ length: DECK_SIZE }, (_, i) => i + 1);
const hand = Array.from({ length: HAND_SIZE }, (_, i) => i + 1);
const deckSalt = "12345";
const playerId = "2";
const handCommitment = hash([playerId, ...hand]);
const deckRootVal = deckRoot(deckSalt, deck);

writeFileSync(
  join(FIXTURES, "hidden_cards.input.json"),
  JSON.stringify(
    {
      deck_root: deckRootVal,
      hand_commitment: handCommitment,
      player_id: playerId,
      deck_size: String(DECK_SIZE),
      hand_size: String(HAND_SIZE),
      hand,
      deck,
      deck_salt: deckSalt,
    },
    null,
    2
  ) + "\n"
);

// fog_of_war
const mapRoot = "100";
const priorExplored = "10";
const tileX = "1";
const tileY = "2";
const nextExplored = hash([mapRoot, priorExplored, tileX, tileY]);

writeFileSync(
  join(FIXTURES, "fog_of_war.input.json"),
  JSON.stringify(
    {
      map_root: mapRoot,
      prior_explored_root: priorExplored,
      next_explored_root: nextExplored,
      origin_x: "0",
      origin_y: "0",
      tile_x: tileX,
      tile_y: tileY,
      visibility_radius: "3",
    },
    null,
    2
  ) + "\n"
);

// fair_dice: roll = (seed % sides) + 1
const seed = "17";
const sides = "6";
const nonce = "5";
const roll = String((BigInt(seed) % BigInt(sides)) + 1n);
const seedCommitment = hash([seed, nonce]);

writeFileSync(
  join(FIXTURES, "fair_dice.input.json"),
  JSON.stringify(
    {
      seed_commitment: seedCommitment,
      roll_result: roll,
      sides,
      nonce,
      seed,
    },
    null,
    2
  ) + "\n"
);

// sealed_bid
const auctionId = "100";
const revealedBid = "50";
const maxBid = "1000";
const bidSalt = "777";
const bidCommitment = hash([revealedBid, bidSalt, auctionId]);

writeFileSync(
  join(FIXTURES, "sealed_bid.input.json"),
  JSON.stringify(
    {
      auction_id: auctionId,
      bid_commitment: bidCommitment,
      revealed_bid: revealedBid,
      max_bid: maxBid,
      bid_salt: bidSalt,
    },
    null,
    2
  ) + "\n"
);

console.log("fixtures generated in", FIXTURES);