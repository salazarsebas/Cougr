class_name CougrCheckpointPayload

# Byte layout for the checkpoint contract's two state-changing methods, plus the
# read-only `get_state` invocation.
#
# The layout is not written down here from memory. Every function below is
# pinned by clients/godot/checkpoint_vectors, which rebuilds the same bytes
# through soroban_sdk from the same inputs, and
# addons/cougr_checkpoint/tests/run_tests.gd asserts the hex on this side. Change
# one side and the other side's CI fails.
#
# Functions return an empty PackedByteArray when an argument cannot be encoded.
# Callers must treat that as a failure rather than submitting a partial payload.

const Xdr := preload("res://addons/cougr_checkpoint/soroban_xdr.gd")

## `EnvelopeType` discriminant for a v1 transaction envelope.
const ENVELOPE_TYPE_TX := 2

## `OperationType` discriminant for an `invoke_host_function` operation.
const OP_INVOKE_HOST_FUNCTION := 24

## `HostFunctionType` discriminant for `invoke_contract`.
const HOST_FUNCTION_TYPE_INVOKE_CONTRACT := 0

## `SCAddressType` discriminants.
const SC_ADDRESS_TYPE_ACCOUNT := 0
const SC_ADDRESS_TYPE_CONTRACT := 1

## `SCValType` discriminants used by the checkpoint interface. `u64` covers the
## state hash, `u32` the tick index, score and status.
const SCV_U32 := 3
const SCV_U64 := 5
const SCV_ADDRESS := 18

## Inclusion fee in stroops, before the simulated resource fee is added.
const DEFAULT_INCLUSION_FEE := 100

## Contract methods this addon calls, from the checkpoint template's interface.
const COMMIT_FUNCTION := "commit_checkpoint"
const DISPUTE_FUNCTION := "dispute_checkpoint"
const STATE_FUNCTION := "get_state"


# --- Values ------------------------------------------------------------------


## Encodes a Stellar account address as an `SCVal::Address`.
static func scval_address(address: String) -> PackedByteArray:
	var key := Xdr.account_key(address)
	if key.is_empty():
		return PackedByteArray()
	var out := Xdr.u32(SCV_ADDRESS)
	out.append_array(Xdr.u32(SC_ADDRESS_TYPE_ACCOUNT))
	out.append_array(Xdr.u32(Xdr.KEY_TYPE_ED25519))
	out.append_array(key)
	return out


## Encodes a `u32` as an `SCVal`.
static func scval_u32(value: int) -> PackedByteArray:
	if value < 0 or value > 0xffffffff:
		push_error("Cougr: %d does not fit in a u32" % value)
		return PackedByteArray()
	var out := Xdr.u32(SCV_U32)
	out.append_array(Xdr.u32(value))
	return out


## Encodes a `u64` as an `SCVal`.
##
## GDScript integers are signed 64-bit, so a hash with the top bit set cannot be
## written as an int literal. Pass either an int, for values below 2^63, or a 16
## character hex string, which carries all 64 bits.
static func scval_u64(value) -> PackedByteArray:
	var encoded := u64_argument(value)
	if encoded.is_empty():
		return PackedByteArray()
	var out := Xdr.u32(SCV_U64)
	out.append_array(encoded)
	return out


## Encodes a `u64` argument from an int or a 16 character hex string.
static func u64_argument(value) -> PackedByteArray:
	match typeof(value):
		TYPE_INT:
			return Xdr.u64(value)
		TYPE_STRING:
			return Xdr.decode_hex(value, 8)
		_:
			push_error(
				"Cougr: a u64 argument must be an int or a 16 character hex string, got %s"
				% type_string(typeof(value))
			)
			return PackedByteArray()


# --- Invocations -------------------------------------------------------------


## Encodes `InvokeContractArgs`: the contract address, the method name, and the
## argument list.
static func invoke_args(contract_id: String, function_name: String, values: Array) -> PackedByteArray:
	var hash := Xdr.contract_hash(contract_id)
	if hash.is_empty():
		return PackedByteArray()
	var out := Xdr.u32(SC_ADDRESS_TYPE_CONTRACT)
	out.append_array(hash)
	out.append_array(Xdr.string(function_name))
	# Every argument is encoded before the count is written, so a rejected
	# argument cannot leave a short list behind.
	var encoded: Array = []
	for value in values:
		var scval: PackedByteArray = value
		if scval.is_empty():
			return PackedByteArray()
		encoded.append(scval)
	out.append_array(Xdr.u32(encoded.size()))
	for scval in encoded:
		out.append_array(scval)
	return out


## `commit_checkpoint(player, tick, state_hash, score)`, the batch checkpoint.
static func commit_checkpoint_args(
	contract_id: String,
	player: String,
	tick: int,
	state_hash,
	score: int
) -> PackedByteArray:
	return invoke_args(
		contract_id,
		COMMIT_FUNCTION,
		[
			scval_address(player),
			scval_u32(tick),
			scval_u64(state_hash),
			scval_u32(score),
		]
	)


## `dispute_checkpoint(challenger, tick, claimed_hash)`, the dispute raised
## inside the window. The contract only accepts a tick that matches the last
## committed one, and a hash that differs from it.
static func dispute_checkpoint_args(
	contract_id: String,
	challenger: String,
	tick: int,
	claimed_hash
) -> PackedByteArray:
	return invoke_args(
		contract_id,
		DISPUTE_FUNCTION,
		[
			scval_address(challenger),
			scval_u32(tick),
			scval_u64(claimed_hash),
		]
	)


## Encodes the `auth` list an `InvokeHostFunctionOp` carries. Entries arrive from
## `simulateTransaction` as base64 `SorobanAuthorizationEntry` values and are
## embedded unchanged.
static func encode_auth(entries: Array) -> PackedByteArray:
	var out := Xdr.u32(entries.size())
	for entry in entries:
		var raw := Marshalls.base64_to_raw(entry)
		if raw.is_empty():
			push_error("Cougr: an authorization entry from simulation was empty")
			return PackedByteArray()
		out.append_array(raw)
	return out


# --- Transactions ------------------------------------------------------------


## Encodes a v1 `Transaction` carrying a single `invoke_host_function`.
##
## `auth` and `soroban_data` are empty for the payload the addon signs by hand
## and filled from simulation before it submits, because the resource footprint
## is only known once the host has run the invocation.
static func invoke_transaction(
	source_key: PackedByteArray,
	sequence: int,
	args: PackedByteArray,
	auth := PackedByteArray(),
	soroban_data := PackedByteArray(),
	fee: int = DEFAULT_INCLUSION_FEE
) -> PackedByteArray:
	if source_key.size() != 32:
		push_error("Cougr: source account key must be 32 bytes, got %d" % source_key.size())
		return PackedByteArray()
	if sequence < 0:
		push_error("Cougr: sequence number must not be negative, got %d" % sequence)
		return PackedByteArray()
	if args.is_empty():
		push_error("Cougr: the invocation arguments were not encoded")
		return PackedByteArray()

	var out := Xdr.u32(Xdr.KEY_TYPE_ED25519)  # MuxedAccount: a plain ed25519 account
	out.append_array(source_key)
	out.append_array(Xdr.u32(fee))
	out.append_array(Xdr.u64(sequence))
	out.append_array(Xdr.u32(0))  # Preconditions: PRECOND_NONE
	out.append_array(Xdr.u32(0))  # Memo: MEMO_NONE
	out.append_array(Xdr.u32(1))  # one operation
	out.append_array(Xdr.u32(0))  # operation source account: absent
	out.append_array(Xdr.u32(OP_INVOKE_HOST_FUNCTION))
	out.append_array(Xdr.u32(HOST_FUNCTION_TYPE_INVOKE_CONTRACT))
	out.append_array(args)
	# An operation with no authorization entries still encodes a zero length
	# array, so an absent list is not the same as no bytes at all.
	out.append_array(Xdr.u32(0) if auth.is_empty() else auth)
	out.append_array(transaction_ext(soroban_data))
	return out


## Wraps a `Transaction` in an `ENVELOPE_TYPE_TX` envelope. Signatures are added
## by the signer, so the envelope this returns is unsigned.
static func envelope(transaction: PackedByteArray) -> PackedByteArray:
	if transaction.is_empty():
		push_error("Cougr: cannot wrap an empty transaction")
		return PackedByteArray()
	var out := Xdr.u32(ENVELOPE_TYPE_TX)
	out.append_array(transaction)
	out.append_array(Xdr.u32(0))  # no signatures yet
	return out


## Encodes `Transaction.ext`, which carries the Soroban resource data requested
## by simulation once it exists.
static func transaction_ext(soroban_data: PackedByteArray) -> PackedByteArray:
	if soroban_data.is_empty():
		return Xdr.u32(0)
	var out := Xdr.u32(1)
	out.append_array(soroban_data)
	return out


## The bytes a signer authorises: the network id, the envelope type, then the
## transaction. This is the preimage of the transaction hash, and it is what
## `stellar tx sign` hashes on the other side of the signer seam.
##
## `transaction` is the encoded `Transaction`, not the envelope: signatures are
## not part of what is signed.
static func signing_payload(network_passphrase: String, transaction: PackedByteArray) -> PackedByteArray:
	if transaction.is_empty():
		push_error("Cougr: cannot build a signing payload for an empty transaction")
		return PackedByteArray()
	var out := Xdr.sha256(network_passphrase.to_utf8_buffer())
	out.append_array(Xdr.u32(ENVELOPE_TYPE_TX))
	out.append_array(transaction)
	return out


# --- Pinned entry points -----------------------------------------------------


## The `commit_checkpoint` transaction for the vector inputs. Checked against
## clients/godot/checkpoint_vectors by tests/run_tests.gd.
static func build_commit_checkpoint_transaction(
	source_account: String,
	sequence: int,
	contract_id: String,
	player: String,
	tick: int,
	state_hash,
	score: int
) -> PackedByteArray:
	return invoke_transaction(
		Xdr.account_key(source_account),
		sequence,
		commit_checkpoint_args(contract_id, player, tick, state_hash, score)
	)


## The unsigned `commit_checkpoint` envelope, which is what the addon submits.
static func build_commit_checkpoint_tx(
	source_account: String,
	sequence: int,
	contract_id: String,
	player: String,
	tick: int,
	state_hash,
	score: int
) -> PackedByteArray:
	return envelope(
		build_commit_checkpoint_transaction(
			source_account, sequence, contract_id, player, tick, state_hash, score
		)
	)


## The `dispute_checkpoint` transaction for the vector inputs.
static func build_dispute_checkpoint_transaction(
	source_account: String,
	sequence: int,
	contract_id: String,
	challenger: String,
	tick: int,
	claimed_hash
) -> PackedByteArray:
	return invoke_transaction(
		Xdr.account_key(source_account),
		sequence,
		dispute_checkpoint_args(contract_id, challenger, tick, claimed_hash)
	)


## The unsigned `dispute_checkpoint` envelope.
static func build_dispute_checkpoint_tx(
	source_account: String,
	sequence: int,
	contract_id: String,
	challenger: String,
	tick: int,
	claimed_hash
) -> PackedByteArray:
	return envelope(
		build_dispute_checkpoint_transaction(
			source_account, sequence, contract_id, challenger, tick, claimed_hash
		)
	)
