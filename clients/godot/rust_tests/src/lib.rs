// Pins the exact TransactionEnvelope XDR emitted by the GDScript builder in
// clients/godot/addons/cougr_turn_based/tx_builder.gd. The Godot headless test
// (tests/run_tests.gd) asserts the same hex, so the two encoders cannot drift.

use soroban_sdk::xdr::{
    ContractId, HostFunction, InvokeContractArgs, InvokeHostFunctionOp, Limits, Memo,
    MuxedAccount, Operation, OperationBody, Preconditions, ScAddress, ScSymbol, ScVal,
    SequenceNumber, Transaction, TransactionEnvelope, TransactionExt, TransactionV1Envelope,
    Uint256, VecM, WriteXdr, AccountId, PublicKey, Hash,
};
use soroban_sdk::{Address, Env, IntoVal, TryFromVal, Val};

#[cfg(test)]
mod tests {
    use super::*;

    pub fn hex_decode(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
            .collect()
    }

    pub fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn key_array(hex: &str) -> [u8; 32] {
        let bytes = hex_decode(hex);
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        out
    }

    #[test]
    fn print_and_assert_xdr() {
        let env = Env::default();
        
        let source_account = "GAVR3J6DOKXZQNNFR4B337FDF7764XU2NDRTSW66J6I3OAYK7H3SNTY4";
        let player_address = "GBG664Y7G5YI6VHT2U3W37N46D7DABQMMG5XUOSB3Q2HHRB6JOT5A76H";
        let contract_id_hex = "1122334455667788990011223344556677889900112233445566778899001122";
        let sequence: i64 = 12345;
        let position: u32 = 4;
        let fee: u32 = 100;

        let player = Address::from_str(&env, player_address);
        let pos_val = ScVal::U32(position);
        let player_val = ScVal::try_from_val(&env, &player.into_val(&env)).unwrap();

        let contract_hash = Hash(key_array(contract_id_hex));

        let args = InvokeContractArgs {
            contract_address: ScAddress::Contract(ContractId(contract_hash)),
            function_name: ScSymbol::try_from("make_move").unwrap(),
            args: VecM::try_from(vec![player_val, pos_val]).unwrap(),
        };

        let operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::InvokeContract(args),
                auth: VecM::default(),
            }),
        };

        // We need to decode the source account ed25519 key for MuxedAccount
        // But since we don't have strkey in this crate, let's just do base32 decode
        let decoded_source = decode_stellar_account(source_account);
        let mut source_key = [0u8; 32];
        source_key.copy_from_slice(&decoded_source);

        let tx = TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: Transaction {
                source_account: MuxedAccount::Ed25519(Uint256(source_key)),
                fee,
                seq_num: SequenceNumber(sequence),
                cond: Preconditions::None,
                memo: Memo::None,
                operations: VecM::try_from(vec![operation]).unwrap(),
                ext: TransactionExt::V0,
            },
            signatures: VecM::default(),
        });

        let out = tx.to_xdr(Limits::none()).unwrap();
        let actual = hex_encode(&out);
        println!("ACTUAL_HEX={}", actual);
        
        let expected = "00000002000000002b1da7c372af9835a58f03bdfca32fffee5e9a68e3395bde4f91b7030af9f726000000640000000000003039000000000000000000000001000000000000001800000000000000011122334455667788990011223344556677889900112233445566778899001122000000096d616b655f6d6f7665000000000000020000001200000000000000004def731f37708f54f3d5376dfdbcf0fe30060c61bb7a3a41dc3473c43e4ba7d0000000300000004000000000000000000000000";
        assert_eq!(expected, actual);
    }

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
}
