class_name CougrCheckpointState

# Decodes the value `get_state` returns.
#
# `MatchState` is a `#[contracttype]` struct, so the host returns it as an
# `SCV_MAP` keyed by field name, with the keys sorted. The decoder is checked
# against a value built by soroban_sdk in clients/godot/checkpoint_vectors.

const Xdr := preload("res://addons/cougr_checkpoint/soroban_xdr.gd")

## `STATUS_RUNNING` and friends, from the checkpoint template's components.
const STATUS_RUNNING := 0
const STATUS_FINALISED := 1
const STATUS_DISPUTED := 2

## The fields `MatchState` carries, in the order the host sorts them.
const FIELDS := ["last_hash", "last_score", "last_tick", "player", "status"]

## Smallest possible match state: the map header plus one empty entry.
const _MINIMUM_BYTES := 12


## A bounds-checked cursor over an encoded `ScVal`.
##
## Nested classes cannot see the enclosing class's constants in GDScript, so the
## tags the cursor needs live here and are reached as `Cursor.SCV_MAP`.
class Cursor:
	const Xdr := preload("res://addons/cougr_checkpoint/soroban_xdr.gd")

	const SCV_BOOL := 0
	const SCV_VOID := 1
	const SCV_U32 := 3
	const SCV_I32 := 4
	const SCV_U64 := 5
	const SCV_I64 := 6
	const SCV_STRING := 14
	const SCV_SYMBOL := 15
	const SCV_MAP := 17
	const SCV_ADDRESS := 18

	const SC_ADDRESS_TYPE_ACCOUNT := 0
	const SC_ADDRESS_TYPE_CONTRACT := 1

	## Sentinel the cursor returns once the value it was reading is malformed.
	const INVALID := "<invalid>"

	var bytes: PackedByteArray
	var offset := 0


	func _init(source: PackedByteArray) -> void:
		bytes = source


	func remaining() -> int:
		return bytes.size() - offset


	func u32() -> int:
		if remaining() < 4:
			return -1
		var value := Xdr.u32_at(bytes, offset)
		offset += 4
		return value


	func s32() -> int:
		var value := u32()
		if value >= 0x80000000:
			return value - 0x100000000
		return value


	## Reads a `u64` and returns it as 16 hex characters. GDScript has no
	## unsigned 64-bit integer, so the hash keeps its exact bits as hex.
	func u64_hex() -> String:
		if remaining() < 8:
			return ""
		var value := bytes.slice(offset, offset + 8)
		offset += 8
		return value.hex_encode()


	func string() -> String:
		var length := u32()
		if length < 0 or remaining() < length:
			push_error("Cougr: truncated string in match state")
			return ""
		var value := bytes.slice(offset, offset + length).get_string_from_utf8()
		offset += length + (4 - (length % 4)) % 4
		return value


	func address() -> String:
		match u32():
			SC_ADDRESS_TYPE_ACCOUNT:
				var key_type := u32()
				if key_type != Xdr.KEY_TYPE_ED25519 or remaining() < 32:
					push_error("Cougr: malformed account address in match state")
					return ""
				var key := bytes.slice(offset, offset + 32)
				offset += 32
				return Xdr.encode_account(key)
			SC_ADDRESS_TYPE_CONTRACT:
				if remaining() < 32:
					push_error("Cougr: truncated contract address in match state")
					return ""
				var hash := bytes.slice(offset, offset + 32)
				offset += 32
				return Xdr.encode_contract(hash)
			_:
				push_error("Cougr: unknown SCAddress type in match state")
				return ""


	## Reads one `ScVal` and returns it as a GDScript value: an int for `u32`,
	## a string for symbols, addresses and `u64` hashes.
	func value() -> Variant:
		var tag := u32()
		match tag:
			SCV_BOOL:
				return u32() != 0
			SCV_VOID:
				return null
			SCV_U32:
				return u32()
			SCV_I32:
				return s32()
			SCV_U64, SCV_I64:
				return u64_hex()
			SCV_SYMBOL, SCV_STRING:
				return string()
			SCV_ADDRESS:
				return address()
			_:
				push_error("Cougr: unsupported ScVal tag %d in match state" % tag)
				return null


## Human readable name for a status value, for a UI or a log line.
static func status_name(status: int) -> String:
	match status:
		STATUS_RUNNING:
			return "running"
		STATUS_FINALISED:
			return "finalised"
		STATUS_DISPUTED:
			return "disputed"
		_:
			return "unknown"


## Decodes a `get_state` result. Returns an empty Dictionary if the value is not
## the `MatchState` struct, so callers can check `is_empty()`.
static func decode(scval_xdr: PackedByteArray) -> Dictionary:
	if scval_xdr.size() < _MINIMUM_BYTES:
		push_error(
			"Cougr: match state must be at least %d bytes, got %d"
			% [_MINIMUM_BYTES, scval_xdr.size()]
		)
		return {}

	var cursor := Cursor.new(scval_xdr)
	if cursor.u32() != Cursor.SCV_MAP:
		push_error(
			"Cougr: get_state did not return an SCV_MAP, which is how a contracttype struct encodes"
		)
		return {}
	if cursor.u32() != 1:
		push_error("Cougr: get_state returned an empty match state")
		return {}

	var entry_count := cursor.u32()
	if entry_count != FIELDS.size():
		push_error(
			"Cougr: match state has %d fields, MatchState has %d" % [entry_count, FIELDS.size()]
		)
		return {}

	var fields := {}
	for _entry in range(entry_count):
		var key = cursor.value()
		if typeof(key) != TYPE_STRING:
			push_error("Cougr: a match state key was not a symbol")
			return {}
		fields[key] = cursor.value()

	var problem := _validate(fields)
	if problem != "":
		push_error("Cougr: %s" % problem)
		return {}

	var status: int = fields["status"]
	return {
		"player": fields["player"],
		"last_tick": fields["last_tick"],
		"last_hash": fields["last_hash"],
		"last_score": fields["last_score"],
		"status": status,
		"status_name": status_name(status),
	}


static func _validate(fields: Dictionary) -> String:
	for field in FIELDS:
		if not fields.has(field):
			return "match state is missing the '%s' field" % field
	if typeof(fields["player"]) != TYPE_STRING or fields["player"] == "":
		return "match state field 'player' was not an address"
	if typeof(fields["last_hash"]) != TYPE_STRING or fields["last_hash"] == "":
		return "match state field 'last_hash' was not a u64"
	for field in ["last_score", "last_tick", "status"]:
		if typeof(fields[field]) != TYPE_INT:
			return "match state field '%s' was not a u32" % field
	return ""
