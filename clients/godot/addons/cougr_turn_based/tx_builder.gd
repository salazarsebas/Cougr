class_name SorobanMakeMoveTxBuilder

# Builds an unsigned TransactionEnvelope XDR for a Soroban invoke contract call
static func build_make_move_tx(source_account: String, sequence: int, contract_id_hex: String, player_address: String, position: int) -> PackedByteArray:
    var xdr = PackedByteArray()
    
    # EnvelopeType = ENVELOPE_TYPE_TX (2)
    xdr.append_array(pack_u32(2))
    
    # --- Transaction ---
    # sourceAccount (MuxedAccount discriminant KEY_TYPE_ED25519 = 0)
    xdr.append_array(pack_u32(0))
    xdr.append_array(decode_account_id(source_account))
    
    # fee (uint32) - set a placeholder fee of 100 stroops
    xdr.append_array(pack_u32(100))
    
    # seqNum (int64)
    xdr.append_array(pack_u64(sequence))
    
    # timeBounds (Option = 0) -> None
    xdr.append_array(pack_u32(0))
    
    # memo (Memo discriminant MEMO_NONE = 0)
    xdr.append_array(pack_u32(0))
    
    # operations (Array length = 1)
    xdr.append_array(pack_u32(1))
    
    # --- Operation[0] ---
    # sourceAccount (Option = 0) -> None
    xdr.append_array(pack_u32(0))
    
    # body (OperationBody discriminant INVOKE_HOST_FUNCTION = 24)
    xdr.append_array(pack_u32(24))
    
    # --- InvokeHostFunctionOp ---
    # hostFunction (HostFunction discriminant HOST_FUNCTION_TYPE_INVOKE_CONTRACT = 0)
    xdr.append_array(pack_u32(0))
    
    # --- InvokeContractArgs ---
    # contractAddress (SCAddress discriminant SC_ADDRESS_TYPE_CONTRACT = 1)
    xdr.append_array(pack_u32(1))
    xdr.append_array(hex_decode(contract_id_hex))
    
    # functionName (SCSymbol)
    xdr.append_array(pack_string("make_move"))
    
    # args (Array of SCVal length = 2)
    xdr.append_array(pack_u32(2))
    
    # arg[0]: SCVal (player: Address)
    # discriminant SCV_ADDRESS = 19
    xdr.append_array(pack_u32(19))
    # SCAddress discriminant SC_ADDRESS_TYPE_ACCOUNT = 0
    xdr.append_array(pack_u32(0))
    # AccountID (PublicKeyType KEY_TYPE_ED25519 = 0)
    xdr.append_array(pack_u32(0))
    xdr.append_array(decode_account_id(player_address))
    
    # arg[1]: SCVal (position: u32)
    # discriminant SCV_U32 = 4
    xdr.append_array(pack_u32(4))
    xdr.append_array(pack_u32(position))
    
    # auth (Array of SorobanAuthorizationEntry length = 0)
    xdr.append_array(pack_u32(0))
    
    # ext (TransactionExt discriminant = 0) 
    xdr.append_array(pack_u32(0))
    
    # signatures (Array of DecoratedSignature length = 0)
    xdr.append_array(pack_u32(0))
    
    return xdr

static func pack_u32(val: int) -> PackedByteArray:
    var res = PackedByteArray()
    res.append((val >> 24) & 0xFF)
    res.append((val >> 16) & 0xFF)
    res.append((val >> 8) & 0xFF)
    res.append(val & 0xFF)
    return res

static func pack_u64(val: int) -> PackedByteArray:
    var res = PackedByteArray()
    res.append((val >> 56) & 0xFF)
    res.append((val >> 48) & 0xFF)
    res.append((val >> 40) & 0xFF)
    res.append((val >> 32) & 0xFF)
    res.append((val >> 24) & 0xFF)
    res.append((val >> 16) & 0xFF)
    res.append((val >> 8) & 0xFF)
    res.append(val & 0xFF)
    return res

static func pack_string(val: String) -> PackedByteArray:
    var utf8 = val.to_utf8_buffer()
    var res = pack_u32(utf8.size())
    res.append_array(utf8)
    var padding = (4 - (utf8.size() % 4)) % 4
    for i in range(padding):
        res.append(0)
    return res

static func decode_account_id(encoded: String) -> PackedByteArray:
    var alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
    var decoded = PackedByteArray()
    var bits = 0
    var bit_count = 0
    for i in range(encoded.length()):
        var char = encoded[i]
        var val = alphabet.find(char)
        if val == -1:
            continue
        bits = (bits << 5) | val
        bit_count += 5
        if bit_count >= 8:
            bit_count -= 8
            decoded.append((bits >> bit_count) & 0xFF)
    if decoded.size() >= 33:
        return decoded.slice(1, 33)
    printerr("Invalid Stellar account ID: ", encoded)
    return PackedByteArray()

static func hex_decode(hex: String) -> PackedByteArray:
    var res = PackedByteArray()
    for i in range(0, hex.length(), 2):
        res.append(hex.substr(i, 2).hex_to_int())
    return res

