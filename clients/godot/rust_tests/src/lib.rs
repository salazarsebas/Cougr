// Pins the exact TransactionEnvelope XDR emitted by the GDScript builder in
// clients/godot/addons/cougr_turn_based/tx_builder.gd. The Godot headless test
// (tests/run_tests.gd) asserts the same hex, so the two encoders cannot drift.
// Vector: envelope type TX, ed25519 source, fee 100, seq 12345, no time bounds,
// memo none, one INVOKE_HOST_FUNCTION op calling make_move(player, position=4),
// empty auth and signatures. Account IDs are the 32-byte ed25519 keys decoded
// from the same Stellar test addresses the GDScript side uses.
#[cfg(test)]
mod tests {
    #[test]
    fn test_xdr_matches_gdscript() {
        // Rebuild the same envelope the GDScript builder must emit, field by
        // field, independently of GDScript.
        let expected = "00000002000000002b1da7c372af9835a58f03bdfca32fffee5e9a68e3395bde4f91b7030af9f726000000640000000000003039000000000000000000000001000000000000001800000000000000011122334455667788990011223344556677889900112233445566778899001122000000096d616b655f6d6f7665000000000000020000001300000000000000004def731f37708f54f3d5376dfdbcf0fe30060c61bb7a3a41dc3473c43e4ba7d00000000400000004000000000000000000000000";
        let source_account = "GAVR3J6DOKXZQNNFR4B337FDF7764XU2NDRTSW66J6I3OAYK7H3SNTY4";
        let player_address = "GBG664Y7G5YI6VHT2U3W37N46D7DABQMMG5XUOSB3Q2HHRB6JOT5A76H";
        let contract_id_hex = "1122334455667788990011223344556677889900112233445566778899001122";
        let sequence: u64 = 12345;
        let position: u32 = 4;

        let decoded = decode_stellar_account(source_account);
        let player = decode_stellar_account(player_address);
        let contract = hex_decode(contract_id_hex);

        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(&2u32.to_be_bytes()); // envelope type ENVELOPE_TYPE_TX
        out.extend_from_slice(&0u32.to_be_bytes()); // muxed account KEY_TYPE_ED25519
        out.extend_from_slice(&decoded); // source account ed25519 key
        out.extend_from_slice(&100u32.to_be_bytes()); // fee
        out.extend_from_slice(&sequence.to_be_bytes()); // seqNum
        out.extend_from_slice(&0u32.to_be_bytes()); // timeBounds none
        out.extend_from_slice(&0u32.to_be_bytes()); // memo MEMO_NONE
        out.extend_from_slice(&1u32.to_be_bytes()); // operations length
        out.extend_from_slice(&0u32.to_be_bytes()); // op sourceAccount none
        out.extend_from_slice(&24u32.to_be_bytes()); // INVOKE_HOST_FUNCTION
        out.extend_from_slice(&0u32.to_be_bytes()); // HOST_FUNCTION_TYPE_INVOKE_CONTRACT
        out.extend_from_slice(&1u32.to_be_bytes()); // SC_ADDRESS_TYPE_CONTRACT
        out.extend_from_slice(&contract); // contract id
        out.extend_from_slice(&9u32.to_be_bytes()); // "make_move" length
        out.extend_from_slice(b"make_move");
        out.extend_from_slice(&[0u8; 3]); // SCSymbol padding
        out.extend_from_slice(&2u32.to_be_bytes()); // args length
        out.extend_from_slice(&19u32.to_be_bytes()); // SCV_ADDRESS
        out.extend_from_slice(&0u32.to_be_bytes()); // SC_ADDRESS_TYPE_ACCOUNT
        out.extend_from_slice(&0u32.to_be_bytes()); // KEY_TYPE_ED25519
        out.extend_from_slice(&player); // player ed25519 key
        out.extend_from_slice(&4u32.to_be_bytes()); // SCV_U32
        out.extend_from_slice(&position.to_be_bytes()); // position
        out.extend_from_slice(&0u32.to_be_bytes()); // auth empty
        out.extend_from_slice(&0u32.to_be_bytes()); // ext v0
        out.extend_from_slice(&0u32.to_be_bytes()); // signatures empty

        let actual: String = out.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(expected, actual);
    }

    // Same base32 decode (minus 1 version byte + 2 CRC bytes) as
    // tx_builder.gd's decode_account_id, so both sides agree on the input.
    fn decode_stellar_account(encoded: &str) -> Vec<u8> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut bits: u32 = 0;
        let mut bit_count: u32 = 0;
        let mut decoded: Vec<u8> = Vec::new();
        for ch in encoded.bytes() {
            let val = match ALPHABET.iter().position(|&a| a == ch) {
                Some(v) => v as u32,
                None => continue,
            };
            bits = (bits << 5) | val;
            bit_count += 5;
            if bit_count >= 8 {
                bit_count -= 8;
                decoded.push(((bits >> bit_count) & 0xFF) as u8);
            }
        }
        decoded[1..33].to_vec()
    }

    fn hex_decode(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
            .collect()
    }
}
