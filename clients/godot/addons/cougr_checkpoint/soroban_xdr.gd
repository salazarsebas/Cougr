class_name CougrSorobanXdr

# Encoding and decoding primitives for Stellar and Soroban XDR.
#
# Nothing here knows what a checkpoint payload looks like. Byte order for the
# invocations lives in checkpoint_payload.gd, and both are pinned by
# clients/godot/checkpoint_vectors, which rebuilds the same bytes through
# soroban_sdk.
#
# Every function that can fail reports through push_error() and returns an empty
# PackedByteArray, so a caller can treat "empty" as "did not encode" instead of
# submitting a half-built payload.

const BASE32_ALPHABET := "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"

## Strkey version byte of a `G...` ed25519 account address.
const STRKEY_VERSION_ACCOUNT := 6 << 3

## Strkey version byte of a `C...` contract address.
const STRKEY_VERSION_CONTRACT := 2 << 3

## XDR union discriminant for a ledger account entry.
const LEDGER_ENTRY_TYPE_ACCOUNT := 0

## XDR union discriminant for an ed25519 public key inside an account id.
const KEY_TYPE_ED25519 := 0

## Bytes in a strkey payload: version, 32 byte hash, 2 byte checksum.
const _STRKEY_LENGTH := 35

## Offset of the sequence number inside a serialised account entry: the account
## id (36 bytes) and the balance (8 bytes) precede it. Verified against the
## `getLedgerEntries` response for a testnet account.
const ACCOUNT_SEQUENCE_OFFSET := 48


# --- Encoders ----------------------------------------------------------------


## Encodes a `u32` big-endian, the order XDR uses.
##
## Godot ships `PackedByteArray.encode_u32`, but it writes little-endian on every
## platform Godot supports, so XDR priming is done by hand here.
static func u32(value: int) -> PackedByteArray:
	var out := PackedByteArray()
	for shift in [24, 16, 8, 0]:
		out.append((value >> shift) & 0xff)
	return out


## Encodes a 64-bit integer big-endian. GDScript integers are signed, so `value`
## must be non-negative; pass a hex string to scval_u64 in
## checkpoint_payload.gd when a value has the top bit set.
static func u64(value: int) -> PackedByteArray:
	if value < 0:
		push_error("Cougr: u64 value must not be negative, got %d" % value)
		return PackedByteArray()
	var out := PackedByteArray()
	for shift in [56, 48, 40, 32, 24, 16, 8, 0]:
		out.append((value >> shift) & 0xff)
	return out


## Reads a big-endian `u32`, or -1 when there are not four bytes left.
static func u32_at(bytes: PackedByteArray, offset: int) -> int:
	if offset < 0 or offset + 4 > bytes.size():
		return -1
	var value := 0
	for index in range(offset, offset + 4):
		value = (value << 8) | bytes[index]
	return value


## Reads a big-endian 64-bit integer, or -1 when there are not eight bytes left.
## Values with the top bit set come back negative, which is why hashes are
## carried as hex rather than as integers.
static func u64_at(bytes: PackedByteArray, offset: int) -> int:
	if offset < 0 or offset + 8 > bytes.size():
		return -1
	var value := 0
	for index in range(offset, offset + 8):
		value = (value << 8) | bytes[index]
	return value


## Encodes an XDR variable-length string: length, UTF-8 bytes, zero padding to a
## four byte boundary. Soroban symbols use the same layout.
static func string(value: String) -> PackedByteArray:
	var utf8 := value.to_utf8_buffer()
	var out := u32(utf8.size())
	out.append_array(utf8)
	var padding := (4 - (utf8.size() % 4)) % 4
	for _padding in range(padding):
		out.append(0)
	return out


## Encodes a strkey account address as the 32 byte ed25519 key an invocation
## carries.
static func account_key(address: String) -> PackedByteArray:
	return _decode_strkey(address, STRKEY_VERSION_ACCOUNT, "account")


## Encodes a strkey contract address as the 32 byte contract id an invocation
## carries.
static func contract_hash(address: String) -> PackedByteArray:
	return _decode_strkey(address, STRKEY_VERSION_CONTRACT, "contract")


## Encodes a 32 byte ed25519 key as a `G...` account address.
static func encode_account(key: PackedByteArray) -> String:
	return _encode_strkey(key, STRKEY_VERSION_ACCOUNT, "account")


## Encodes a 32 byte contract id as a `C...` contract address.
static func encode_contract(hash: PackedByteArray) -> String:
	return _encode_strkey(hash, STRKEY_VERSION_CONTRACT, "contract")


## Decodes a fixed number of bytes from a hex string, rejecting odd lengths and
## non-hex characters rather than silently encoding zeros.
static func decode_hex(value: String, byte_count: int) -> PackedByteArray:
	if value.length() != byte_count * 2:
		push_error(
			"Cougr: expected %d hex characters, got %d" % [byte_count * 2, value.length()]
		)
		return PackedByteArray()
	var out := PackedByteArray()
	for index in range(0, value.length(), 2):
		var pair := value.substr(index, 2)
		if not _is_hex(pair):
			push_error("Cougr: %s is not hexadecimal" % pair)
			return PackedByteArray()
		out.append(pair.hex_to_int())
	return out


## SHA-256, the digest Stellar uses for network ids and transaction signatures.
## Godot exposes MD5, SHA-1 and SHA-256 only, which is why the addon never hashes
## an ed25519 seed itself and leaves signing to the Stellar CLI.
static func sha256(bytes: PackedByteArray) -> PackedByteArray:
	var context := HashingContext.new()
	context.start(HashingContext.HASH_SHA256)
	context.update(bytes)
	return context.finish()


# --- Decoders ----------------------------------------------------------------


## Encodes the `LedgerKey` for an account, the key `getLedgerEntries` takes to
## read an account's sequence number.
static func account_ledger_key(key: PackedByteArray) -> PackedByteArray:
	if key.size() != 32:
		push_error("Cougr: an ed25519 key must be 32 bytes, got %d" % key.size())
		return PackedByteArray()
	var out := u32(LEDGER_ENTRY_TYPE_ACCOUNT)
	out.append_array(u32(KEY_TYPE_ED25519))
	out.append_array(key)
	return out


## Reads the account sequence number out of a `LedgerEntryData` an account key
## returns. The account entry layout is accountID (36 bytes), balance (8), then
## seqNum (8), so the sequence always starts at byte 48.
static func account_sequence(ledger_entry_xdr: PackedByteArray) -> int:
	if ledger_entry_xdr.size() < ACCOUNT_SEQUENCE_OFFSET + 8:
		push_error(
			"Cougr: an account ledger entry must be at least %d bytes, got %d"
			% [ACCOUNT_SEQUENCE_OFFSET + 8, ledger_entry_xdr.size()]
		)
		return -1
	if u32_at(ledger_entry_xdr, 0) != LEDGER_ENTRY_TYPE_ACCOUNT:
		push_error("Cougr: ledger entry is not an account entry")
		return -1
	return u64_at(ledger_entry_xdr, ACCOUNT_SEQUENCE_OFFSET)


# --- Internals ---------------------------------------------------------------


static func _encode_strkey(payload: PackedByteArray, version: int, kind: String) -> String:
	if payload.size() != 32:
		push_error("Cougr: a %s payload must be 32 bytes, got %d" % [kind, payload.size()])
		return ""
	var bytes := PackedByteArray([version])
	bytes.append_array(payload)
	var checksum := _crc16_xmodem(bytes)
	bytes.append(checksum & 0xff)
	bytes.append((checksum >> 8) & 0xff)
	return _base32_encode(bytes)


static func _decode_strkey(address: String, version: int, kind: String) -> PackedByteArray:
	var payload := _base32_decode(address)
	if payload.size() != _STRKEY_LENGTH:
		push_error(
			"Cougr: %s address %s must decode to %d bytes, got %d"
			% [kind, address, _STRKEY_LENGTH, payload.size()]
		)
		return PackedByteArray()
	if payload[0] != version:
		push_error(
			"Cougr: %s address %s has version byte %d, expected %d"
			% [kind, address, payload[0], version]
		)
		return PackedByteArray()
	var checksum := _crc16_xmodem(payload.slice(0, 33))
	if payload[33] != (checksum & 0xff) or payload[34] != ((checksum >> 8) & 0xff):
		push_error("Cougr: %s address %s failed its checksum" % [kind, address])
		return PackedByteArray()
	return payload.slice(1, 33)


static func _base32_encode(payload: PackedByteArray) -> String:
	var out := ""
	var bits := 0
	var bit_count := 0
	for byte in payload:
		bits = (bits << 8) | byte
		bit_count += 8
		while bit_count >= 5:
			bit_count -= 5
			out += BASE32_ALPHABET[(bits >> bit_count) & 0x1f]
	if bit_count > 0:
		out += BASE32_ALPHABET[(bits << (5 - bit_count)) & 0x1f]
	return out


static func _base32_decode(value: String) -> PackedByteArray:
	var out := PackedByteArray()
	var bits := 0
	var bit_count := 0
	for index in range(value.length()):
		var symbol := BASE32_ALPHABET.find(value[index])
		if symbol < 0:
			return PackedByteArray()
		bits = (bits << 5) | symbol
		bit_count += 5
		if bit_count >= 8:
			bit_count -= 8
			out.append((bits >> bit_count) & 0xff)
	return out


static func _crc16_xmodem(data: PackedByteArray) -> int:
	var crc := 0
	for byte in data:
		crc ^= byte << 8
		for _bit in range(8):
			if crc & 0x8000:
				crc = ((crc << 1) ^ 0x1021) & 0xffff
			else:
				crc = (crc << 1) & 0xffff
	return crc


static func _is_hex(pair: String) -> bool:
	for index in range(pair.length()):
		var code := pair.unicode_at(index)
		var digit := code >= 48 and code <= 57
		var lower := code >= 97 and code <= 102
		var upper := code >= 65 and code <= 70
		if not (digit or lower or upper):
			return false
	return true
