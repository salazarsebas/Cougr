extends SceneTree

# Headless cross-check of the checkpoint addon against the vectors pinned in
# clients/godot/checkpoint_vectors, where the same two invocations are rebuilt
# through soroban_sdk. If the GDScript encoder and the SDK encoder drift apart,
# this exits non-zero and prints both hex strings.
#
# Run from clients/godot, with no editor UI:
#
#   godot --headless --path . --script res://addons/cougr_checkpoint/tests/run_tests.gd
#
# The Rust half of the check is `cargo test` in clients/godot/checkpoint_vectors.

const Payload := preload("res://addons/cougr_checkpoint/checkpoint_payload.gd")
const State := preload("res://addons/cougr_checkpoint/checkpoint_state.gd")
const Xdr := preload("res://addons/cougr_checkpoint/soroban_xdr.gd")

# --- Vector inputs, matching checkpoint_vectors/src/lib.rs -------------------

const SOURCE_ACCOUNT := "GAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQTCQKRMFYYDENBWHA5DYPSABOV"
const PLAYER_ACCOUNT := "GCAYFA4EQWDIPCEJRKFYZDMOR6IJDEUTSSKZNF4YTGNJXHE5T2P2BJE2"
const CHALLENGER_ACCOUNT := "GCQ2FI5EUWTKPKFJVKV2ZLNOV6YLDMVTWS23NN5YXG5LXPF5X274BAEF"
const CONTRACT_ADDRESS := "CDAMDQWDYTC4NR6IZHFMXTGNZ3H5BUOS2PKNLVWX3DM5VW643XPN6VTO"

const CONTRACT_ID_HEX := "c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf"
const SOURCE_KEY_HEX := "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
const PLAYER_KEY_HEX := "8182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0"
const CHALLENGER_KEY_HEX := "a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0"

const SEQUENCE := 12345
const TICK := 1024
const SCORE := 4200
const STATE_HASH := 0x0123456789abcdef
const STATE_HASH_HEX := "0123456789abcdef"
const CLAIMED_HASH_HEX := "fedcba9876543210"
const NETWORK_PASSPHRASE := "Test SDF Network ; September 2015"

# --- Pinned vectors, copied from checkpoint_vectors/src/lib.rs ---------------

const COMMIT_ARGS_HEX := "00000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000011636f6d6d69745f636865636b706f696e74000000000000040000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000300000400000000050123456789abcdef0000000300001068"
const COMMIT_ENVELOPE_HEX := "00000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000011636f6d6d69745f636865636b706f696e74000000000000040000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000300000400000000050123456789abcdef0000000300001068000000000000000000000000"
const COMMIT_SIGNING_PAYLOAD_HEX := "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd47200000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000011636f6d6d69745f636865636b706f696e74000000000000040000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000300000400000000050123456789abcdef00000003000010680000000000000000"
const DISPUTE_ARGS_HEX := "00000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000012646973707574655f636865636b706f696e74000000000003000000120000000000000000a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0000000030000040000000005fedcba9876543210"
const DISPUTE_ENVELOPE_HEX := "00000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000012646973707574655f636865636b706f696e74000000000003000000120000000000000000a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0000000030000040000000005fedcba9876543210000000000000000000000000"
const DISPUTE_SIGNING_PAYLOAD_HEX := "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd47200000002000000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2000000064000000000000303900000000000000000000000100000000000000180000000000000001c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf00000012646973707574655f636865636b706f696e74000000000003000000120000000000000000a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0000000030000040000000005fedcba98765432100000000000000000"

const MATCH_STATE_HEX := "0000001100000001000000050000000f000000096c6173745f68617368000000000000050123456789abcdef0000000f0000000a6c6173745f73636f7265000000000003000010680000000f000000096c6173745f7469636b00000000000003000004000000000f00000006706c6179657200000000001200000000000000008182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa00000000f0000000673746174757300000000000300000000"

const _EXPECTED_BUTTONS := ["ReadStateButton", "CommitButton", "DisputeButton"]

var _failures: Array = []


func _initialize() -> void:
	_check_addresses()
	_check_commit_checkpoint()
	_check_dispute_checkpoint()
	_check_u64_arguments()
	_check_post_simulation_rebuild()
	_check_match_state()
	_check_account_sequence()
	_check_invalid_input()
	_check_sample_scene()

	if _failures.is_empty():
		print("\ncheckpoint addon: every vector matches")
		quit(0)
		return

	for failure in _failures:
		printerr("FAILED %s" % failure)
	printerr("\ncheckpoint addon: %d check(s) failed" % _failures.size())
	quit(1)


# --- Checks ------------------------------------------------------------------


func _check_addresses() -> void:
	_check("source account key", Xdr.account_key(SOURCE_ACCOUNT).hex_encode(), SOURCE_KEY_HEX)
	_check("player account key", Xdr.account_key(PLAYER_ACCOUNT).hex_encode(), PLAYER_KEY_HEX)
	_check(
		"challenger account key", Xdr.account_key(CHALLENGER_ACCOUNT).hex_encode(), CHALLENGER_KEY_HEX
	)
	_check("contract id", Xdr.contract_hash(CONTRACT_ADDRESS).hex_encode(), CONTRACT_ID_HEX)

	# Encoding back to a strkey must reproduce the address the Rust side printed,
	# which is what the state decoder relies on to report a player address.
	_check(
		"player address round trip",
		Xdr.encode_account(Xdr.account_key(PLAYER_ACCOUNT)),
		PLAYER_ACCOUNT
	)
	_check(
		"contract address round trip",
		Xdr.encode_contract(Xdr.contract_hash(CONTRACT_ADDRESS)),
		CONTRACT_ADDRESS
	)


func _check_commit_checkpoint() -> void:
	_check(
		"commit_checkpoint args",
		Payload.commit_checkpoint_args(
			CONTRACT_ADDRESS, PLAYER_ACCOUNT, TICK, STATE_HASH, SCORE
		).hex_encode(),
		COMMIT_ARGS_HEX
	)
	_check(
		"commit_checkpoint envelope",
		Payload.build_commit_checkpoint_tx(
			SOURCE_ACCOUNT, SEQUENCE, CONTRACT_ADDRESS, PLAYER_ACCOUNT, TICK, STATE_HASH, SCORE
		).hex_encode(),
		COMMIT_ENVELOPE_HEX
	)
	_check(
		"commit_checkpoint signing payload",
		Payload.signing_payload(
			NETWORK_PASSPHRASE,
			Payload.build_commit_checkpoint_transaction(
				SOURCE_ACCOUNT, SEQUENCE, CONTRACT_ADDRESS, PLAYER_ACCOUNT, TICK, STATE_HASH, SCORE
			)
		).hex_encode(),
		COMMIT_SIGNING_PAYLOAD_HEX
	)


func _check_dispute_checkpoint() -> void:
	_check(
		"dispute_checkpoint args",
		Payload.dispute_checkpoint_args(
			CONTRACT_ADDRESS, CHALLENGER_ACCOUNT, TICK, CLAIMED_HASH_HEX
		).hex_encode(),
		DISPUTE_ARGS_HEX
	)
	_check(
		"dispute_checkpoint envelope",
		Payload.build_dispute_checkpoint_tx(
			SOURCE_ACCOUNT,
			SEQUENCE,
			CONTRACT_ADDRESS,
			CHALLENGER_ACCOUNT,
			TICK,
			CLAIMED_HASH_HEX
		).hex_encode(),
		DISPUTE_ENVELOPE_HEX
	)
	_check(
		"dispute_checkpoint signing payload",
		Payload.signing_payload(
			NETWORK_PASSPHRASE,
			Payload.build_dispute_checkpoint_transaction(
				SOURCE_ACCOUNT,
				SEQUENCE,
				CONTRACT_ADDRESS,
				CHALLENGER_ACCOUNT,
				TICK,
				CLAIMED_HASH_HEX
			)
		).hex_encode(),
		DISPUTE_SIGNING_PAYLOAD_HEX
	)


## The dispute vector passes a hash with the top bit set, which a GDScript int
## cannot hold, so it goes through the hex string form. The commit vector passes
## an int. Both forms have to produce identical bytes for the same value.
func _check_u64_arguments() -> void:
	_check(
		"u64 int and hex forms agree",
		Payload.build_commit_checkpoint_tx(
			SOURCE_ACCOUNT, SEQUENCE, CONTRACT_ADDRESS, PLAYER_ACCOUNT, TICK, STATE_HASH_HEX, SCORE
		).hex_encode(),
		COMMIT_ENVELOPE_HEX
	)


## Before it signs, the client rebuilds the transaction with the resource data
## simulation returned, which is the one shape the pinned vectors do not cover:
## they pin the pre-simulation transaction. This checks the extension flips from
## an empty `TransactionExt` to the one carrying that data.
func _check_post_simulation_rebuild() -> void:
	var key := Xdr.account_key(SOURCE_ACCOUNT)
	var args := Payload.commit_checkpoint_args(
		CONTRACT_ADDRESS, PLAYER_ACCOUNT, TICK, STATE_HASH, SCORE
	)
	var resource_data := _hex("deadbeef")
	var before := Payload.invoke_transaction(key, SEQUENCE, args)
	var rebuilt := Payload.invoke_transaction(key, SEQUENCE, args, Xdr.u32(0), resource_data, 200)

	_check(
		"resource data extends the transaction",
		rebuilt.size(),
		before.size() + resource_data.size()
	)
	_check(
		"transaction extension carries the resource data",
		rebuilt.slice(rebuilt.size() - 8, rebuilt.size()).hex_encode(),
		"00000001" + "deadbeef"
	)
	_check("empty auth is a zero length array", Payload.encode_auth([]).hex_encode(), "00000000")


func _check_match_state() -> void:
	var state := State.decode(_hex(MATCH_STATE_HEX))
	if state.is_empty():
		_add_failure("match state did not decode")
		return

	_check("match state player", state["player"], PLAYER_ACCOUNT)
	_check("match state last_tick", state["last_tick"], TICK)
	_check("match state last_hash", state["last_hash"], STATE_HASH_HEX)
	_check("match state last_score", state["last_score"], SCORE)
	_check("match state status", state["status"], State.STATUS_RUNNING)
	_check("match state status name", state["status_name"], "running")


## Pins the account entry offset the client reads a sequence number from. This
## guards the arithmetic only: the layout itself is fixed by the account entry
## definition, and the offset is checked against testnet in the client's comment.
func _check_account_sequence() -> void:
	var key := PackedByteArray()
	key.resize(32)
	var entry := Xdr.u32(Xdr.LEDGER_ENTRY_TYPE_ACCOUNT)
	entry.append_array(Xdr.u32(Xdr.KEY_TYPE_ED25519))
	entry.append_array(key)
	entry.append_array(Xdr.u64(455400000001))
	entry.append_array(Xdr.u64(1664866762883072))
	_check("account sequence offset", Xdr.account_sequence(entry), 1664866762883072)

	var ledger_key := Xdr.account_ledger_key(key)
	_check("account ledger key", ledger_key.hex_encode(), "0000000000000000" + "00".repeat(32))


## A rejected argument must leave the payload empty rather than produce a short
## or zeroed invocation. The push_error() lines below are the expected output.
func _check_invalid_input() -> void:
	print("# the two errors below are expected: they exercise the rejection paths")
	_check("invalid address is rejected", Xdr.account_key("not-an-address").is_empty(), true)
	_check("negative u32 is rejected", Payload.scval_u32(-1).is_empty(), true)
	_check(
		"non-struct match state is rejected", State.decode(_hex(COMMIT_ARGS_HEX)).is_empty(), true
	)


func _check_sample_scene() -> void:
	var scene: PackedScene = load("res://sample_checkpoint_scene.tscn")
	if scene == null:
		_add_failure("sample_checkpoint_scene.tscn did not load")
		return
	var instance := scene.instantiate()
	if instance == null:
		_add_failure("sample_checkpoint_scene.tscn did not instantiate")
		return
	# The scene is readied off-tree: the SceneTree is not live yet while this
	# script initialises, so the engine would not deliver _ready() here. This
	# still runs the scene's own construction code, which is the part that can
	# break when the contract wiring changes.
	instance._ready()
	for button in _EXPECTED_BUTTONS:
		if instance.find_child(button, true, false) == null:
			_add_failure("sample scene has no %s" % button)
	instance.free()
	if _failures.is_empty():
		print("ok    sample scene builds its controls")


# --- Reporting ---------------------------------------------------------------


func _check(label: String, actual, expected) -> void:
	if actual == expected:
		print("ok    %s = %s" % [label, _describe(actual)])
		return
	_add_failure("%s\n  expected: %s\n  actual:   %s" % [label, _describe(expected), _describe(actual)])


func _add_failure(message: String) -> void:
	_failures.append(message)


## Prints long hex on its own indented line so a diff is readable in CI logs.
func _describe(value) -> String:
	var text := str(value)
	if text.length() <= 72:
		return text
	return "\n  " + text


## The vectors are decoded with the addon's own hex decoder rather than
## `String.hex_decode`, so a malformed literal fails instead of decoding short.
func _hex(value: String) -> PackedByteArray:
	return Xdr.decode_hex(value, value.length() / 2)
