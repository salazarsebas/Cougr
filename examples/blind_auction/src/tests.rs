use super::*;
use cougr_core::circuits::{sealed_bid, test_fixtures, CircuitId};
use cougr_core::test::GameHarness;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn init_auction_configures_sealed_bid_circuit() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, BlindAuction);
    let client = BlindAuctionClient::new(harness.env(), harness.contract_id());
    let auction_id = BytesN::from_array(harness.env(), &[9u8; 32]);

    let config = client.init_auction(&1000, &auction_id);
    assert_eq!(config.max_bid, 1000);

    let spec = sealed_bid(harness.env(), 1000).unwrap();
    assert_eq!(spec.circuit_id, CircuitId::SealedBid);
    assert_eq!(spec.layout.public_input_count(), 4);
}

#[test]
fn reveal_bid_accepts_pipeline_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, BlindAuction);
    let client = BlindAuctionClient::new(harness.env(), harness.contract_id());
    let bidder = Address::generate(harness.env());
    let public = test_fixtures::pipeline_public_inputs(harness.env(), CircuitId::SealedBid);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::SealedBid);

    let auction_id = BytesN::from_array(harness.env(), &public[0].bytes.to_array());
    let commit = BytesN::from_array(harness.env(), &public[1].bytes.to_array());
    client.init_auction(&1000, &auction_id);

    assert!(client.reveal_bid(&bidder, &commit, &50, &proof));
    let reveal = client.bid_reveal(&bidder);
    assert_eq!(reveal.revealed_bid, 50);
}

#[test]
fn reveal_bid_rejects_over_max_bid() {
    let env = Env::default();
    env.mock_all_auths();
    let harness = GameHarness::new(env, BlindAuction);
    let client = BlindAuctionClient::new(harness.env(), harness.contract_id());
    let bidder = Address::generate(harness.env());
    let public = test_fixtures::pipeline_public_inputs(harness.env(), CircuitId::SealedBid);
    let proof = test_fixtures::pipeline_proof(harness.env(), CircuitId::SealedBid);

    let auction_id = BytesN::from_array(harness.env(), &public[0].bytes.to_array());
    let commit = BytesN::from_array(harness.env(), &public[1].bytes.to_array());
    client.init_auction(&1000, &auction_id);

    assert!(!client.reveal_bid(&bidder, &commit, &1001, &proof));
}
