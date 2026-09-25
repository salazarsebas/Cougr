//! Cross-language passkey vectors for `packages/sdk-passkey`.
//!
//! The contract is the spec: `verify_secp256r1` hashes its `message` with
//! SHA-256 and hands the digest to `env.crypto().secp256r1_verify`. A WebAuthn
//! assertion therefore has to be mapped to `authenticatorData ||
//! sha256(clientDataJSON)` as the message, ASN.1 DER to 64 raw bytes `r || s`
//! as the signature, and the COSE EC2 key to SEC-1 uncompressed
//! `0x04 || x || y` as `Secp256r1Key.public_key`.
//!
//! These tests pin the values in `packages/sdk-passkey/vectors/passkey-vectors.json`,
//! which the TypeScript mapper asserts, so the two sides cannot drift.

#![cfg(feature = "testutils")]

use cougr_core::accounts::passkey::{Secp256r1Key, Secp256r1Storage};
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Bytes, BytesN, Env,
};

/// Committed vector file the TypeScript package also reads.
const VECTORS: &str = include_str!("../packages/sdk-passkey/vectors/passkey-vectors.json");

const RP_ID: &str = "cougr.test";
const AUTHENTICATOR_DATA: &str =
    "eaa0b50ba841022d6923e590d437995d33b44578283e596a28141f89a6ba61e00500000001";
const CLIENT_DATA_JSON: &str = concat!(
    r#"{"type":"webauthn.get","challenge":"#,
    r#""AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8","#,
    r#""origin":"https://cougr.test"}"#,
);
const CLIENT_DATA_HASH: &str = "dc242e782390c6cd5f5626eabd52486db5e916cd7905c388ccf6d75838ba0797";
const MESSAGE_HEX: &str = concat!(
    "eaa0b50ba841022d6923e590d437995d33b44578283e596a28141f89a6ba61e00500000001",
    "dc242e782390c6cd5f5626eabd52486db5e916cd7905c388ccf6d75838ba0797",
);

const DER_SIGNATURE_ONE: &str = "3045022100f12323232323232323232323232323232323232323232323232323232323232302200a45454545454545454545454545454545454545454545454545454545454545";
const RAW_SIGNATURE_ONE: &str = "f1232323232323232323232323232323232323232323232323232323232323230a45454545454545454545454545454545454545454545454545454545454545";
const DER_SIGNATURE_TWO: &str = "30440220010202020202020202020202020202020202020202020202020202020202020202207f11111111111111111111111111111111111111111111111111111111111111";
const RAW_SIGNATURE_TWO: &str = "01020202020202020202020202020202020202020202020202020202020202027f11111111111111111111111111111111111111111111111111111111111111";

const REGISTRATION_ATTESTATION: &str = "a363666d74646e6f6e656761747453746d74a06861757468446174615894eaa0b50ba841022d6923e590d437995d33b44578283e596a28141f89a6ba61e04100000000000000000000000000000000000000000010000102030405060708090a0b0c0d0e0fa501020326200121582011111111111111111111111111111111111111111111111111111111111111112258202222222222222222222222222222222222222222222222222222222222222222";
const REGISTRATION_AUTH_DATA: &str = "eaa0b50ba841022d6923e590d437995d33b44578283e596a28141f89a6ba61e04100000000000000000000000000000000000000000010000102030405060708090a0b0c0d0e0fa501020326200121582011111111111111111111111111111111111111111111111111111111111111112258202222222222222222222222222222222222222222222222222222222222222222";
const REGISTRATION_COSE_KEY: &str = "a501020326200121582011111111111111111111111111111111111111111111111111111111111111112258202222222222222222222222222222222222222222222222222222222222222222";
const REGISTRATION_CREDENTIAL_ID: &str = "000102030405060708090a0b0c0d0e0f";
const REGISTRATION_PUBLIC_KEY: &str = "0411111111111111111111111111111111111111111111111111111111111111112222222222222222222222222222222222222222222222222222222222222222";

#[contract]
pub struct PasskeyVectorContract;

#[contractimpl]
impl PasskeyVectorContract {}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> std::vec::Vec<u8> {
    assert!(text.len() % 2 == 0, "hex constants must be byte aligned");
    (0..text.len() / 2)
        .map(|i| {
            let hi = (text.as_bytes()[i * 2] as char).to_digit(16).expect("hex") as u8;
            let lo = (text.as_bytes()[i * 2 + 1] as char)
                .to_digit(16)
                .expect("hex") as u8;
            (hi << 4) | lo
        })
        .collect()
}

fn read_length(der: &[u8], offset: usize) -> Result<(usize, usize), &'static str> {
    let first = *der.get(offset).ok_or("truncated")?;
    if first & 0x80 == 0 {
        return Ok((first as usize, offset + 1));
    }
    let count = (first & 0x7f) as usize;
    if count == 0 || count > 2 {
        return Err("unsupported length encoding");
    }
    let mut length = 0usize;
    for byte in &der[offset + 1..offset + 1 + count] {
        length = (length << 8) | *byte as usize;
    }
    Ok((length, offset + 1 + count))
}

/// The DER to `r || s` mapping `packages/sdk-passkey/src/der.ts` performs.
fn der_to_raw(der: &[u8]) -> Result<[u8; 64], &'static str> {
    if der.len() < 8 || der[0] != 0x30 {
        return Err("missing SEQUENCE tag");
    }
    let (sequence_length, mut cursor) = read_length(der, 1)?;
    if cursor + sequence_length > der.len() {
        return Err("inconsistent length");
    }

    let mut raw = [0u8; 64];
    for coordinate in 0..2 {
        if der[cursor] != 0x02 {
            return Err("missing INTEGER");
        }
        let (length, next) = read_length(der, cursor + 1)?;
        let end = next + length;
        if end > der.len() {
            return Err("truncated integer");
        }
        let mut start = next;
        if length > 1 && der[start] == 0x00 {
            start += 1;
        }
        let size = end - start;
        if size == 0 || size > 32 {
            return Err("integer out of range");
        }
        raw[coordinate * 32 + (32 - size)..coordinate * 32 + 32].copy_from_slice(&der[start..end]);
        cursor = end;
    }
    Ok(raw)
}

/// The authData byte string inside the pinned attestation object.
fn registration_auth_data() -> std::vec::Vec<u8> {
    let attestation = unhex(REGISTRATION_ATTESTATION);
    let label = b"authData";
    let start = attestation
        .windows(label.len())
        .position(|window| window == label)
        .expect("the attestation object must contain an authData label")
        + label.len();
    assert_eq!(
        attestation[start], 0x58,
        "authData uses a 1 byte length prefix"
    );
    let length = attestation[start + 1] as usize;
    attestation[start + 2..start + 2 + length].to_vec()
}

#[test]
fn assertion_message_matches_the_typescript_mapper() {
    let env = Env::default();

    let client_data_hash = env
        .crypto()
        .sha256(&Bytes::from_slice(&env, CLIENT_DATA_JSON.as_bytes()))
        .to_array();
    assert_eq!(hex(&client_data_hash), CLIENT_DATA_HASH);

    let mut message = Bytes::from_slice(&env, &unhex(AUTHENTICATOR_DATA));
    message.append(&Bytes::from_array(&env, &client_data_hash));
    let emitted = hex(&message.to_alloc_vec());

    println!("cougr sdk-passkey message vector: {emitted}");
    assert_eq!(emitted, MESSAGE_HEX);
    assert!(
        VECTORS.contains(MESSAGE_HEX),
        "packages/sdk-passkey/vectors/passkey-vectors.json is stale"
    );
}

#[test]
fn authenticator_data_header_is_bound_to_the_rp_id() {
    let env = Env::default();
    let auth_data = unhex(AUTHENTICATOR_DATA);

    let rp_id_hash = env
        .crypto()
        .sha256(&Bytes::from_slice(&env, RP_ID.as_bytes()))
        .to_array();
    assert_eq!(hex(&auth_data[..32]), hex(&rp_id_hash));
    assert_eq!(auth_data[32], 0x05, "user present and user verified");
    assert_eq!(
        u32::from_be_bytes([auth_data[33], auth_data[34], auth_data[35], auth_data[36]]),
        1
    );
}

#[test]
fn der_signatures_become_raw_coordinates() {
    for (der, raw) in [
        (DER_SIGNATURE_ONE, RAW_SIGNATURE_ONE),
        (DER_SIGNATURE_TWO, RAW_SIGNATURE_TWO),
    ] {
        let emitted = hex(&der_to_raw(&unhex(der)).expect("vector signature must decode"));
        println!("cougr sdk-passkey raw signature vector: {emitted}");
        assert_eq!(emitted, raw);
        assert!(VECTORS.contains(raw), "the raw signature vector is stale");
    }

    let truncated = &DER_SIGNATURE_ONE[..DER_SIGNATURE_ONE.len() - 4];
    assert!(der_to_raw(&unhex(truncated)).is_err());
    assert!(der_to_raw(&[]).is_err());
    assert!(der_to_raw(&[0x00u8; 8]).is_err());
}

#[test]
fn registration_auth_data_carries_the_pinned_cose_key() {
    let auth_data = registration_auth_data();
    assert_eq!(hex(&auth_data), REGISTRATION_AUTH_DATA);

    assert_eq!(
        auth_data[32], 0x41,
        "user present with attested credential data"
    );
    assert_eq!(
        hex(&auth_data[33..37]),
        "00000000",
        "registration sign count"
    );
    assert_eq!(hex(&auth_data[37..53]), "00".repeat(16));

    let credential_id_length = u16::from_be_bytes([auth_data[53], auth_data[54]]) as usize;
    assert_eq!(credential_id_length, 16);
    assert_eq!(hex(&auth_data[55..71]), REGISTRATION_CREDENTIAL_ID);

    let cose = &auth_data[71..];
    assert_eq!(hex(cose), REGISTRATION_COSE_KEY);
    assert_eq!(cose[0], 0xa5, "COSE key map with five entries");
    assert_eq!(hex(&cose[1..5]), "01020326");
    assert_eq!(hex(&cose[5..10]), "2001215820");
    assert_eq!(hex(&cose[42..45]), "225820");

    let x = &cose[10..42];
    let y = &cose[45..77];
    let uncompressed = format!("04{}{}", hex(x), hex(y));
    assert_eq!(uncompressed, REGISTRATION_PUBLIC_KEY);
    assert!(VECTORS.contains(REGISTRATION_PUBLIC_KEY));
}

#[test]
fn the_mapped_key_round_trips_through_secp256r1_storage() {
    let env = Env::default();
    let contract_id = env.register(PasskeyVectorContract, ());
    let player = Address::generate(&env);

    let mut public_key = [0u8; 65];
    public_key.copy_from_slice(&unhex(REGISTRATION_PUBLIC_KEY));
    assert_eq!(public_key[0], 0x04);

    let key = Secp256r1Key {
        public_key: BytesN::from_array(&env, &public_key),
        label: symbol_short!("passkey1"),
        registered_at: 1_735_689_600,
    };

    env.as_contract(&contract_id, || {
        Secp256r1Storage::store(&env, &player, &key);

        let stored = Secp256r1Storage::load_all(&env, &player);
        assert_eq!(stored.len(), 1);
        let loaded = stored.get(0).expect("the stored key");
        assert_eq!(loaded.public_key.to_array(), public_key);
        assert_eq!(loaded.registered_at, 1_735_689_600);
        assert!(
            Secp256r1Storage::find_by_label(&env, &player, &symbol_short!("passkey1")).is_some()
        );
    });
}
