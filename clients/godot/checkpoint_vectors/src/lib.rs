//! Byte-layout cross-check vectors for the Cougr checkpoint Godot addon.
//!
//! The addon in `clients/godot/addons/cougr_checkpoint` encodes Soroban
//! invocation payloads in GDScript, because a Godot project cannot take a
//! dependency on the Rust Soroban toolchain. This crate rebuilds the same two
//! invocations through `soroban_sdk`, from the same inputs, and pins the
//! resulting bytes.
//!
//! `clients/godot/addons/cougr_checkpoint/tests/run_tests.gd` asserts the
//! identical hex under a headless Godot run, so the GDScript encoder and the
//! Soroban SDK encoder cannot drift apart. The pinned values are the assertion
//! targets in both directions: change one side and CI on the other side fails.
//!
//! The vectors cover three layers per method, because each is signed on a
//! different boundary:
//!
//! 1. `InvokeContractArgs`, the argument list the contract receives,
//! 2. the full unsigned `TransactionEnvelope` the addon submits,
//! 3. the signing payload, the bytes the client authorises with its key.
//!
//! Run `cargo test -- --nocapture` to print every vector as hex.

use soroban_sdk::xdr::{
    ContractId, Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Limits, Memo,
    MuxedAccount, Operation, OperationBody, Preconditions, ScAddress, ScSymbol, ScVal,
    SequenceNumber, Transaction, TransactionEnvelope, TransactionExt, TransactionV1Envelope,
    Uint256, VecM, WriteXdr,
};
use soroban_sdk::{Address, Bytes, Env, IntoVal, Map, Symbol, TryFromVal, Val};

/// Ed25519 key of the transaction source account, used as the checkpoint
/// committer. Synthetic and sequential on purpose, so a byte shifted by one
/// position cannot cancel out against a neighbouring field.
pub const SOURCE_ACCOUNT_KEY_HEX: &str =
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

/// Ed25519 key of the player that commits checkpoints.
pub const PLAYER_KEY_HEX: &str = "8182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0";

/// Ed25519 key of the challenger that raises a dispute.
pub const CHALLENGER_KEY_HEX: &str =
    "a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0";

/// Contract id of the deployed checkpoint contract.
pub const CONTRACT_ID_HEX: &str =
    "c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf";

/// Account sequence number of the source account.
pub const SEQUENCE: i64 = 12345;

/// Inclusion fee in stroops, before the simulated resource fee is added.
pub const BASE_FEE: u32 = 100;

/// Tick index committed by the vector checkpoint.
pub const TICK: u32 = 1024;

/// Off-chain state hash committed by the vector checkpoint.
pub const STATE_HASH: u64 = 0x0123_4567_89ab_cdef;

/// Score committed alongside the vector checkpoint.
pub const SCORE: u32 = 4200;

/// Tick index challenged by the vector dispute. Matches [`TICK`], because the
/// contract rejects a dispute aimed at any other tick.
pub const DISPUTE_TICK: u32 = TICK;

/// Hash the challenger claims the engine produced at [`DISPUTE_TICK`].
pub const CLAIMED_HASH: u64 = 0xfedc_ba98_7654_3210;

/// Network passphrase the signing payload is bound to.
pub const NETWORK_PASSPHRASE: &str = "Test SDF Network ; September 2015";

/// The `STATUS_RUNNING` value the checkpoint template reports from `get_state`
/// while a match is open.
pub const STATUS_RUNNING: u32 = 0;

/// Soroban contract method committed by [`commit_checkpoint_envelope_hex`].
pub const COMMIT_FUNCTION: &str = "commit_checkpoint";

/// Soroban contract method committed by [`dispute_checkpoint_envelope_hex`].
pub const DISPUTE_FUNCTION: &str = "dispute_checkpoint";

/// The `ENVELOPE_TYPE_TX` discriminant, mixed into the signing payload.
pub const ENVELOPE_TYPE_TX: u32 = 2;

// --- Inputs ------------------------------------------------------------------

/// Hex decodes a fixed-length byte string, panicking on malformed input. These
/// are compile-time constants, so a failure here is a bug in the vector itself.
pub fn hex_decode(hex: &str) -> Vec<u8> {
    assert!(
        hex.len().is_multiple_of(2),
        "hex input must have an even length"
    );
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
        .collect()
}

/// Hex encodes bytes, lowercase, two characters per byte.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn key_array(hex: &str) -> [u8; 32] {
    let bytes = hex_decode(hex);
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

fn contract_hash() -> Hash {
    let mut out = [0u8; 32];
    out.copy_from_slice(&hex_decode(CONTRACT_ID_HEX));
    Hash(out)
}

/// Strkey account address for the vector source account.
pub fn source_account() -> String {
    strkey::encode_account(&key_array(SOURCE_ACCOUNT_KEY_HEX))
}

/// Strkey account address for the vector player.
pub fn player_address() -> String {
    strkey::encode_account(&key_array(PLAYER_KEY_HEX))
}

/// Strkey account address for the vector challenger.
pub fn challenger_address() -> String {
    strkey::encode_account(&key_array(CHALLENGER_KEY_HEX))
}

/// Strkey contract address for the vector contract id. This is the form a
/// deployer copies out of `stellar contract deploy`, so it is what the addon
/// takes as `COUGR_CONTRACT_ID`.
pub fn contract_address() -> String {
    strkey::encode_contract(&key_array(CONTRACT_ID_HEX))
}

// --- Argument lists ----------------------------------------------------------

/// Converts a native value into its `ScVal` through `soroban_sdk`, so the
/// discriminants in these vectors come from the SDK's own conversion table
/// rather than from an enum transcribed into this file.
fn scval<T>(env: &Env, value: T) -> ScVal
where
    T: IntoVal<Env, Val>,
{
    let val: Val = value.into_val(env);
    ScVal::try_from_val(env, &val).expect("value has an ScVal representation")
}

/// The `commit_checkpoint(player, tick, state_hash, score)` argument list.
pub fn commit_checkpoint_args(env: &Env) -> InvokeContractArgs {
    let player = Address::from_str(env, &player_address());
    invoke_args(
        COMMIT_FUNCTION,
        vec![
            scval(env, player),
            scval(env, TICK),
            scval(env, STATE_HASH),
            scval(env, SCORE),
        ],
    )
}

/// The `dispute_checkpoint(challenger, tick, claimed_hash)` argument list.
pub fn dispute_checkpoint_args(env: &Env) -> InvokeContractArgs {
    let challenger = Address::from_str(env, &challenger_address());
    invoke_args(
        DISPUTE_FUNCTION,
        vec![
            scval(env, challenger),
            scval(env, DISPUTE_TICK),
            scval(env, CLAIMED_HASH),
        ],
    )
}

fn invoke_args(function: &str, args: Vec<ScVal>) -> InvokeContractArgs {
    InvokeContractArgs {
        contract_address: ScAddress::Contract(ContractId(contract_hash())),
        function_name: ScSymbol::try_from(function).expect("symbol fits in SCSYMBOL_LIMIT"),
        args: VecM::try_from(args).expect("argument list within SCVEC_LIMIT"),
    }
}

// --- Envelopes ---------------------------------------------------------------

/// Builds the unsigned one-operation envelope around `args`. `auth` and `ext`
/// stay empty here; the addon fills both from transaction simulation before it
/// signs, which is why the vector pins the pre-simulation shape.
pub fn envelope(args: InvokeContractArgs) -> TransactionEnvelope {
    let operation = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(args),
            auth: VecM::default(),
        }),
    };

    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account: MuxedAccount::Ed25519(Uint256(key_array(SOURCE_ACCOUNT_KEY_HEX))),
            fee: BASE_FEE,
            seq_num: SequenceNumber(SEQUENCE),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: VecM::try_from(vec![operation]).expect("single operation"),
            ext: TransactionExt::V0,
        },
        signatures: VecM::default(),
    })
}

/// XDR of the `commit_checkpoint` argument list.
pub fn commit_checkpoint_args_xdr(env: &Env) -> Vec<u8> {
    commit_checkpoint_args(env)
        .to_xdr(Limits::none())
        .expect("encode arguments")
}

/// XDR of the `dispute_checkpoint` argument list.
pub fn dispute_checkpoint_args_xdr(env: &Env) -> Vec<u8> {
    dispute_checkpoint_args(env)
        .to_xdr(Limits::none())
        .expect("encode arguments")
}

/// XDR of the unsigned `commit_checkpoint` envelope.
pub fn commit_checkpoint_envelope(env: &Env) -> Vec<u8> {
    envelope(commit_checkpoint_args(env))
        .to_xdr(Limits::none())
        .expect("encode envelope")
}

/// XDR of the unsigned `dispute_checkpoint` envelope.
pub fn dispute_checkpoint_envelope(env: &Env) -> Vec<u8> {
    envelope(dispute_checkpoint_args(env))
        .to_xdr(Limits::none())
        .expect("encode envelope")
}

/// Hex of [`commit_checkpoint_args_xdr`].
pub fn commit_checkpoint_args_hex(env: &Env) -> String {
    hex_encode(&commit_checkpoint_args_xdr(env))
}

/// Hex of [`dispute_checkpoint_args_xdr`].
pub fn dispute_checkpoint_args_hex(env: &Env) -> String {
    hex_encode(&dispute_checkpoint_args_xdr(env))
}

/// Hex of [`commit_checkpoint_envelope`].
pub fn commit_checkpoint_envelope_hex(env: &Env) -> String {
    hex_encode(&commit_checkpoint_envelope(env))
}

/// Hex of [`dispute_checkpoint_envelope`].
pub fn dispute_checkpoint_envelope_hex(env: &Env) -> String {
    hex_encode(&dispute_checkpoint_envelope(env))
}

// --- Match state -------------------------------------------------------------

/// `get_state()` returns `MatchState`, which is a `#[contracttype]` struct and
/// therefore serialises as an `SCV_MAP` keyed by field name. Rebuilding it here
/// gives the addon's state decoder a vector produced by `soroban_sdk` itself.
pub fn match_state_scval(env: &Env) -> ScVal {
    let mut map = Map::<Symbol, ScVal>::new(env);
    map.set(Symbol::new(env, "last_hash"), scval(env, STATE_HASH));
    map.set(Symbol::new(env, "last_score"), scval(env, SCORE));
    map.set(Symbol::new(env, "last_tick"), scval(env, TICK));
    map.set(
        Symbol::new(env, "player"),
        scval(env, Address::from_str(env, &player_address())),
    );
    map.set(Symbol::new(env, "status"), scval(env, STATUS_RUNNING));
    ScVal::from(map)
}

/// Hex of the `ScVal` the RPC hands back in `results[0].xdr` for `get_state`.
pub fn match_state_hex(env: &Env) -> String {
    hex_encode(
        &match_state_scval(env)
            .to_xdr(Limits::none())
            .expect("encode match state"),
    )
}

// --- Signing payload ---------------------------------------------------------

/// The bytes a signer authorises for a v1 envelope:
/// `sha256(network_passphrase) || ENVELOPE_TYPE_TX || Transaction`, where the
/// hash is the network id and the envelope type is the four byte union
/// discriminant `stellar-core` serialises. The `Transaction` here is the one
/// carried by the envelope, not the whole envelope, so the empty signature list
/// is not part of the payload.
pub fn signing_payload(network_passphrase: &str, transaction_xdr: &[u8]) -> Vec<u8> {
    let env = Env::default();
    let passphrase = Bytes::from_slice(&env, network_passphrase.as_bytes());
    let network_id = env.crypto().sha256(&passphrase).to_array();

    let mut out = Vec::with_capacity(32 + 4 + transaction_xdr.len());
    out.extend_from_slice(&network_id);
    out.extend_from_slice(&ENVELOPE_TYPE_TX.to_be_bytes());
    out.extend_from_slice(transaction_xdr);
    out
}

/// XDR of just the `Transaction` inside a v1 envelope, which is what
/// [`signing_payload`] measures against.
fn transaction_xdr(envelope: &TransactionEnvelope) -> Vec<u8> {
    match envelope {
        TransactionEnvelope::Tx(v1) => v1.tx.to_xdr(Limits::none()).expect("encode transaction"),
        _ => unreachable!("the addon only builds ENVELOPE_TYPE_TX envelopes"),
    }
}

/// Hex of [`signing_payload`] for the unsigned `commit_checkpoint` envelope.
pub fn commit_checkpoint_signing_payload_hex(env: &Env) -> String {
    let envelope = envelope(commit_checkpoint_args(env));
    hex_encode(&signing_payload(
        NETWORK_PASSPHRASE,
        &transaction_xdr(&envelope),
    ))
}

/// Hex of [`signing_payload`] for the unsigned `dispute_checkpoint` envelope.
pub fn dispute_checkpoint_signing_payload_hex(env: &Env) -> String {
    let envelope = envelope(dispute_checkpoint_args(env));
    hex_encode(&signing_payload(
        NETWORK_PASSPHRASE,
        &transaction_xdr(&envelope),
    ))
}

// --- Strkey ------------------------------------------------------------------

/// Strkey (`G...` and `C...`) encoding, the inverse of the decoder the GDScript
/// addon uses to turn an address into the 32 bytes an invocation carries.
pub mod strkey {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    /// Version byte for an ed25519 public key.
    const VERSION_ACCOUNT_ID: u8 = 6 << 3;

    /// Version byte for a contract id (`C...`).
    const VERSION_CONTRACT: u8 = 2 << 3;

    /// Encodes a 32 byte ed25519 key as a `G...` account address.
    pub fn encode_account(key: &[u8; 32]) -> String {
        encode(VERSION_ACCOUNT_ID, key)
    }

    /// Encodes a 32 byte contract hash as a `C...` contract address.
    pub fn encode_contract(hash: &[u8; 32]) -> String {
        encode(VERSION_CONTRACT, hash)
    }

    /// Decodes a `G...` account address into its 32 byte ed25519 key, rejecting
    /// bad checksums, bad versions and bad lengths.
    pub fn decode_account(address: &str) -> Option<[u8; 32]> {
        decode(address, VERSION_ACCOUNT_ID)
    }

    /// Decodes a `C...` contract address into its 32 byte contract hash.
    pub fn decode_contract(address: &str) -> Option<[u8; 32]> {
        decode(address, VERSION_CONTRACT)
    }

    fn encode(version: u8, payload: &[u8; 32]) -> String {
        let mut bytes = Vec::with_capacity(35);
        bytes.push(version);
        bytes.extend_from_slice(payload);
        let checksum = crc16_xmodem(&bytes);
        bytes.extend_from_slice(&checksum.to_le_bytes());
        base32_encode(&bytes)
    }

    fn decode(address: &str, version: u8) -> Option<[u8; 32]> {
        let decoded = base32_decode(address)?;
        if decoded.len() != 35 || decoded[0] != version {
            return None;
        }
        let expected = crc16_xmodem(&decoded[..33]);
        if decoded[33..35] != expected.to_le_bytes() {
            return None;
        }
        let mut payload = [0u8; 32];
        payload.copy_from_slice(&decoded[1..33]);
        Some(payload)
    }

    fn base32_encode(data: &[u8]) -> String {
        let mut out = String::new();
        let mut bits: u32 = 0;
        let mut bit_count: u32 = 0;
        for byte in data {
            bits = (bits << 8) | u32::from(*byte);
            bit_count += 8;
            while bit_count >= 5 {
                bit_count -= 5;
                out.push(ALPHABET[((bits >> bit_count) & 0x1f) as usize] as char);
            }
        }
        if bit_count > 0 {
            out.push(ALPHABET[((bits << (5 - bit_count)) & 0x1f) as usize] as char);
        }
        out
    }

    fn base32_decode(value: &str) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut bits: u32 = 0;
        let mut bit_count: u32 = 0;
        for ch in value.bytes() {
            let index = ALPHABET.iter().position(|c| *c == ch)? as u32;
            bits = (bits << 5) | index;
            bit_count += 5;
            if bit_count >= 8 {
                bit_count -= 8;
                out.push(((bits >> bit_count) & 0xff) as u8);
            }
        }
        Some(out)
    }

    fn crc16_xmodem(data: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        for byte in data {
            crc ^= u16::from(*byte) << 8;
            for _ in 0..8 {
                crc = if crc & 0x8000 != 0 {
                    (crc << 1) ^ 0x1021
                } else {
                    crc << 1
                };
            }
        }
        crc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::xdr::{AccountId, PublicKey};

    /// Vectors pinned by `addons/cougr_checkpoint/tests/run_tests.gd`. Both
    /// sides assert these exact strings.
    const COMMIT_ARGS_HEX: &str = "00000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000011636f6d6d69745f636865636b706f696e74000000000000040000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000300000400000000050123456789abcdef0000000300001068";
    const DISPUTE_ARGS_HEX: &str = "00000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000012646973707574655f636865636b706f696e74000000000003000000120000000000000000a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0000000030000040000000005fedcba9876543210";
    const COMMIT_ENVELOPE_HEX: &str = "00000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000011636f6d6d69745f636865636b706f696e74000000000000040000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000300000400000000050123456789abcdef0000000300001068000000000000000000000000";
    const DISPUTE_ENVELOPE_HEX: &str = "00000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000012646973707574655f636865636b706f696e74000000000003000000120000000000000000a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0000000030000040000000005fedcba9876543210000000000000000000000000";
    const COMMIT_SIGNING_PAYLOAD_HEX: &str = "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd47200000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000011636f6d6d69745f636865636b706f696e74000000000000040000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000300000400000000050123456789abcdef00000003000010680000000000000000";
    const DISPUTE_SIGNING_PAYLOAD_HEX: &str = "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd47200000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000012646973707574655f636865636b706f696e74000000000003000000120000000000000000a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0000000030000040000000005fedcba98765432100000000000000000";

    /// Addresses pinned by the GDScript test. It decodes them with its own
    /// base32 and checksum code and must land on the same key hex.
    const SOURCE_ACCOUNT: &str = "GAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQTCQKRMFYYDENBWHA5DYPSABOV";
    const PLAYER_ACCOUNT: &str = "GCAYFA4EQWDIPCEJRKFYZDMOR6IJDEUTSSKZNF4YTGNJXHE5T2P2BJE2";
    const CHALLENGER_ACCOUNT: &str = "GCQ2FI5EUWTKPKFJVKV2ZLNOV6YLDMVTWS23NN5YXG5LXPF5X274BAEF";

    /// Contract address pinned by the GDScript test, and the `ScVal` a
    /// `get_state` simulation returns while a match is running.
    const CONTRACT_ADDRESS: &str = "CDAMDQWDYTC4NR6IZHFMXTGNZ3H5BUOS2PKNLVWX3DM5VW643XPN6VTO";
    const MATCH_STATE_HEX: &str = "0000001100000001000000050000000f000000096c6173745f68617368000000000000050123456789abcdef0000000f0000000a6c6173745f73636f7265000000000003000010680000000f000000096c6173745f7469636b00000000000003000004000000000f00000006706c6179657200000000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000f0000000673746174757300000000000300000000";

    /// The `ScAddress` a pinned account address has to decode to.
    fn sc_address(key: &[u8; 32]) -> ScAddress {
        ScAddress::Account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(*key))))
    }

    #[test]
    fn strkey_round_trips_the_well_known_zero_account() {
        // GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF is the
        // canonical encoding of the all-zero ed25519 key. If the encoder here
        // is wrong, every address in these vectors is wrong too.
        let zero = [0u8; 32];
        assert_eq!(
            strkey::encode_account(&zero),
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"
        );
        assert_eq!(
            strkey::decode_account(&strkey::encode_account(&zero)),
            Some(zero)
        );
    }

    #[test]
    fn vector_addresses_are_the_pinned_ones() {
        assert_eq!(source_account(), SOURCE_ACCOUNT);
        assert_eq!(player_address(), PLAYER_ACCOUNT);
        assert_eq!(challenger_address(), CHALLENGER_ACCOUNT);
    }

    #[test]
    fn vector_addresses_decode_back_to_the_pinned_keys() {
        assert_eq!(
            hex_encode(&strkey::decode_account(SOURCE_ACCOUNT).expect("valid source account")),
            SOURCE_ACCOUNT_KEY_HEX
        );
        assert_eq!(
            hex_encode(&strkey::decode_account(PLAYER_ACCOUNT).expect("valid player account")),
            PLAYER_KEY_HEX
        );
        assert_eq!(
            hex_encode(
                &strkey::decode_account(CHALLENGER_ACCOUNT).expect("valid challenger account")
            ),
            CHALLENGER_KEY_HEX
        );
    }

    /// The addon's contract address is a raw 32 byte contract id, not a strkey,
    /// so the only thing to check is that `soroban_sdk` agrees the bytes are a
    /// contract address.
    #[test]
    fn contract_address_is_a_contract_sc_address() {
        let env = Env::default();
        let args = commit_checkpoint_args(&env);
        assert_eq!(
            args.contract_address,
            ScAddress::Contract(ContractId(contract_hash()))
        );
    }

    /// `soroban_sdk` decodes the pinned address strings independently of the
    /// strkey code above. The two must agree, which is what lets the GDScript
    /// decoder be a substitute for the SDK one.
    #[test]
    fn soroban_sdk_agrees_with_the_local_strkey_decoder() {
        let env = Env::default();
        for (address, key_hex) in [
            (SOURCE_ACCOUNT, SOURCE_ACCOUNT_KEY_HEX),
            (PLAYER_ACCOUNT, PLAYER_KEY_HEX),
            (CHALLENGER_ACCOUNT, CHALLENGER_KEY_HEX),
        ] {
            let from_sdk = Address::from_str(&env, address);
            let decoded: ScVal = scval(&env, from_sdk);
            let expected = ScVal::Address(sc_address(&key_array(key_hex)));
            assert_eq!(
                decoded, expected,
                "address {address} decoded by soroban_sdk"
            );
            assert_eq!(
                strkey::decode_account(address).expect("valid strkey"),
                key_array(key_hex),
            );
        }
    }

    #[test]
    fn commit_checkpoint_vectors_are_pinned() {
        let env = Env::default();
        println!("source account  = {}", source_account());
        println!("player account  = {}", player_address());
        println!("challenger acct = {}", challenger_address());
        let args = commit_checkpoint_args_hex(&env);
        let envelope = commit_checkpoint_envelope_hex(&env);
        let payload = commit_checkpoint_signing_payload_hex(&env);
        println!("commit_checkpoint args         = {args}");
        println!("commit_checkpoint envelope     = {envelope}");
        println!("commit_checkpoint signing hash = {payload}");
        assert_eq!(args, COMMIT_ARGS_HEX, "commit_checkpoint argument list");
        assert_eq!(envelope, COMMIT_ENVELOPE_HEX, "commit_checkpoint envelope");
        assert_eq!(
            payload, COMMIT_SIGNING_PAYLOAD_HEX,
            "commit_checkpoint payload"
        );
    }

    #[test]
    fn dispute_checkpoint_vectors_are_pinned() {
        let env = Env::default();
        let args = dispute_checkpoint_args_hex(&env);
        let envelope = dispute_checkpoint_envelope_hex(&env);
        let payload = dispute_checkpoint_signing_payload_hex(&env);
        println!("dispute_checkpoint args         = {args}");
        println!("dispute_checkpoint envelope     = {envelope}");
        println!("dispute_checkpoint signing hash = {payload}");
        assert_eq!(args, DISPUTE_ARGS_HEX, "dispute_checkpoint argument list");
        assert_eq!(
            envelope, DISPUTE_ENVELOPE_HEX,
            "dispute_checkpoint envelope"
        );
        assert_eq!(
            payload, DISPUTE_SIGNING_PAYLOAD_HEX,
            "dispute_checkpoint payload"
        );
    }

    #[test]
    fn contract_address_is_the_pinned_one() {
        assert_eq!(contract_address(), CONTRACT_ADDRESS);
        assert_eq!(
            hex_encode(&strkey::decode_contract(CONTRACT_ADDRESS).expect("valid contract")),
            CONTRACT_ID_HEX
        );
        assert!(
            strkey::decode_account(CONTRACT_ADDRESS).is_none(),
            "a contract address must not decode as an account address"
        );
    }

    /// `soroban_sdk` decodes the pinned contract address to the same contract
    /// id the addon encodes, so the two decoders are interchangeable.
    #[test]
    fn soroban_sdk_agrees_on_the_contract_address() {
        let env = Env::default();
        assert_eq!(
            scval(&env, Address::from_str(&env, CONTRACT_ADDRESS)),
            ScVal::Address(ScAddress::Contract(ContractId(contract_hash())))
        );
    }

    /// `get_state` returns a struct, which is an `SCV_MAP` keyed by field name
    /// in ascending key order. The addon's decoder depends on both facts.
    #[test]
    fn match_state_vectors_are_pinned() {
        let env = Env::default();
        let hex = match_state_hex(&env);
        println!("match state = {hex}");
        assert_eq!(hex, MATCH_STATE_HEX, "match state ScVal");

        let ScVal::Map(Some(map)) = match_state_scval(&env) else {
            panic!("a contracttype struct encodes as a non-empty SCV_MAP");
        };
        let keys: Vec<String> = map
            .0
            .iter()
            .map(|entry| match &entry.key {
                ScVal::Symbol(symbol) => symbol.to_utf8_string().expect("ascii field name"),
                other => panic!("struct field keys are SCV_SYMBOL, got {other:?}"),
            })
            .collect();
        assert_eq!(
            keys,
            vec!["last_hash", "last_score", "last_tick", "player", "status"],
            "struct field keys are sorted and match MatchState"
        );
    }

    /// The dispute vector reuses the commit tick on purpose: the contract
    /// rejects a dispute whose tick does not match the last committed one, so a
    /// vector that drifted to another tick would not be submittable.
    #[test]
    fn dispute_targets_the_committed_tick() {
        assert_eq!(DISPUTE_TICK, TICK);
        assert_ne!(CLAIMED_HASH, STATE_HASH);
    }

    /// The pinned envelopes are the pre-simulation shape: one
    /// `invoke_host_function` operation, no auth entries, no Soroban resource
    /// extension. The addon fills the last two from simulation before signing,
    /// so a vector that already carried them would not match what it builds.
    #[test]
    fn vectors_are_the_pre_simulation_shape() {
        let env = Env::default();
        for envelope in [
            envelope(commit_checkpoint_args(&env)),
            envelope(dispute_checkpoint_args(&env)),
        ] {
            let TransactionEnvelope::Tx(v1) = envelope else {
                panic!("the addon only builds ENVELOPE_TYPE_TX envelopes");
            };
            assert_eq!(v1.tx.fee, BASE_FEE);
            assert_eq!(v1.tx.seq_num, SequenceNumber(SEQUENCE));
            assert_eq!(v1.tx.ext, TransactionExt::V0);
            assert!(v1.signatures.is_empty());
            assert_eq!(v1.tx.operations.len(), 1);
            let OperationBody::InvokeHostFunction(op) = &v1.tx.operations[0].body else {
                panic!("the addon only builds invoke_host_function operations");
            };
            assert!(op.auth.is_empty());
        }
    }
}
