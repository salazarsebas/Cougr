class_name CougrCheckpointClient
extends Node

# Soroban RPC client for a deployed checkpoint contract.
#
# `read_state()` simulates `get_state`. `commit_checkpoint()` and
# `raise_dispute()` build the invocation, simulate it for its resource
# footprint, sign it, and submit it. Every method is a coroutine, so callers
# must await them:
#
#     var result = await client.read_state()
#
# Configuration is read from the environment in _ready() so a scene can be
# pointed at a contract without editing it:
#
#     COUGR_RPC_URL            Soroban RPC endpoint (defaults to testnet)
#     COUGR_CONTRACT_ID        the deployed contract, in `C...` form
#     COUGR_SOURCE_ACCOUNT     the account that signs, in `G...` form
#     COUGR_SIGNING_KEY        consumed by the signer, never by this client
#     COUGR_NETWORK_PASSPHRASE overrides the passphrase the payload is bound to
#
# The exported properties win over the environment when they are set, so a scene
# can hold its own contract id while the key still comes from outside the repo.

const Payload := preload("res://addons/cougr_checkpoint/checkpoint_payload.gd")
const State := preload("res://addons/cougr_checkpoint/checkpoint_state.gd")
const Xdr := preload("res://addons/cougr_checkpoint/soroban_xdr.gd")
const CliSigner := preload("res://addons/cougr_checkpoint/stellar_cli_signer.gd")

## Soroban RPC endpoint used when COUGR_RPC_URL is unset.
const DEFAULT_RPC_URL := "https://soroban-testnet.stellar.org"

## Passphrase of the testnet network the addon targets by default.
const TESTNET_PASSPHRASE := "Test SDF Network ; September 2015"

## How many ledgers to wait for a submitted transaction to be included.
const CONFIRMATION_ATTEMPTS := 10

## Seconds between confirmation polls.
const CONFIRMATION_DELAY := 1.0

## Contract id in `C...` form. Empty means "read COUGR_CONTRACT_ID".
@export var contract_id := ""

## Address that signs checkpoints, in `G...` form. It is the transaction source,
## so it is also the `player` the contract authorises on a commit and the
## `challenger` on a dispute.
@export var source_account := ""

## Soroban RPC endpoint.
@export var rpc_url := ""

## Network passphrase the signing payload is bound to.
@export var network_passphrase := ""

## Signer with a `sign(envelope, network_passphrase)` method. Defaults to the
## Stellar CLI signer.
var signer = null

var _http: HTTPRequest
var _request_id := 0

# Set while an RPC request is in flight, which is why two overlapping calls fail
# loudly rather than sharing one HTTPRequest.
var _busy := false


func _ready() -> void:
	if rpc_url.is_empty():
		rpc_url = _environment("COUGR_RPC_URL", DEFAULT_RPC_URL)
	if contract_id.is_empty():
		contract_id = OS.get_environment("COUGR_CONTRACT_ID")
	if source_account.is_empty():
		source_account = OS.get_environment("COUGR_SOURCE_ACCOUNT")
	if network_passphrase.is_empty():
		network_passphrase = _environment("COUGR_NETWORK_PASSPHRASE", TESTNET_PASSPHRASE)
	if signer == null:
		signer = CliSigner.new()
	_http = HTTPRequest.new()
	add_child(_http)


# --- Reads -------------------------------------------------------------------


## Reads the checkpoint match state.
##
## Read-only: it simulates `get_state` with a throwaway source account, so it
## needs no key, and a wallet does not have to be funded or configured yet.
func read_state() -> Dictionary:
	var problem := _contract_problem()
	if problem != "":
		return _failure(problem)

	var args := Payload.invoke_args(contract_id, Payload.STATE_FUNCTION, [])
	if args.is_empty():
		return _failure("could not encode the get_state invocation")

	var transaction := Payload.invoke_transaction(_throwaway_key(), 0, args)
	if transaction.is_empty():
		return _failure("could not encode the get_state transaction")

	var simulation := await _simulate(Payload.envelope(transaction))
	if not simulation["success"]:
		return simulation

	var state := State.decode(simulation["result_scval"])
	if state.is_empty():
		return _failure("could not decode the match state returned by get_state")

	return {"success": true, "state": state}


# --- Writes ------------------------------------------------------------------


## Commits a checkpoint: the tick index, the off-chain state hash, and the score
## the simulation reached at that tick.
##
## The tick must be greater than the last committed tick. `state_hash` is a
## `u64`, passed either as an int below 2^63 or as a 16 character hex string.
func commit_checkpoint(tick: int, state_hash, score: int) -> Dictionary:
	var args := Payload.commit_checkpoint_args(contract_id, source_account, tick, state_hash, score)
	return await _submit(args)


## Disputes the last committed checkpoint, inside the dispute window.
##
## The tick must match the last committed tick and `claimed_hash` must differ
## from the hash that was committed, otherwise the contract rejects the dispute.
func raise_dispute(tick: int, claimed_hash) -> Dictionary:
	var args := Payload.dispute_checkpoint_args(contract_id, source_account, tick, claimed_hash)
	return await _submit(args)


## Builds, simulates, signs, and submits one state-changing invocation.
##
## Returns `{"success": true, "hash": ..., "ledger": ...}` on inclusion, or
## `{"success": false, "error": ...}` with the reason. The signing payload is
## included on success as `signing_payload` so a caller can log exactly what the
## key authorised.
func _submit(args: PackedByteArray) -> Dictionary:
	var problem := _contract_problem()
	if problem != "":
		return _failure(problem)
	if args.is_empty():
		return _failure("could not encode the invocation; check the contract id and arguments")

	problem = _signing_problem()
	if problem != "":
		return _failure(problem)

	var source_key := Xdr.account_key(source_account)
	if source_key.is_empty():
		return _failure("invalid source account %s" % source_account)

	var sequence := await _next_sequence(source_key)
	if sequence < 0:
		return _failure(
			"could not read a sequence number for %s; the account has to exist on the network and be funded" % source_account
		)

	var simulation := await _simulate(
		Payload.envelope(Payload.invoke_transaction(source_key, sequence, args))
	)
	if not simulation["success"]:
		return simulation

	var auth := Payload.encode_auth(simulation["auth"])
	if auth.is_empty():
		return _failure("could not embed the authorization entries from simulation")

	# The resource footprint and the resource fee only exist after simulation, so
	# the transaction is rebuilt for signing rather than patched in place.
	var transaction := Payload.invoke_transaction(
		source_key, sequence, args, auth, simulation["transaction_data"], simulation["fee"]
	)
	if transaction.is_empty():
		return _failure("could not encode the transaction")

	var signed: PackedByteArray = signer.sign(Payload.envelope(transaction), network_passphrase)
	if signed.is_empty():
		return _failure("the signer did not return a signed envelope")

	var submitted := await _send(signed)
	if not submitted["success"]:
		return submitted

	return {
		"success": true,
		"hash": submitted["hash"],
		"ledger": submitted.get("ledger", 0),
		"signing_payload": Payload.signing_payload(network_passphrase, transaction).hex_encode(),
	}


# --- RPC ---------------------------------------------------------------------


## Simulates an envelope and returns everything a caller can need from it: the
## resource data for the submit path, and the `ScVal` for the read path.
func _simulate(envelope_xdr: PackedByteArray) -> Dictionary:
	var response := await _rpc("simulateTransaction", {
		"transaction": Marshalls.raw_to_base64(envelope_xdr),
	})
	if not response["success"]:
		return response

	var result: Dictionary = response["result"]
	if result.has("error"):
		return _failure("simulation failed: %s" % result["error"])

	var results: Array = result.get("results", [])
	if results.is_empty():
		return _failure("simulation returned no result for the invocation")

	var first: Dictionary = results[0]
	var resource_fee := int(result.get("minResourceFee", "0"))
	return {
		"success": true,
		"transaction_data": Marshalls.base64_to_raw(result.get("transactionData", "")),
		"auth": first.get("auth", []),
		"fee": resource_fee + Payload.DEFAULT_INCLUSION_FEE,
		"result_scval": Marshalls.base64_to_raw(first.get("xdr", "")),
	}


## Reads the next sequence number for an account from its ledger entry. The
## entry's `xdr` is the `LedgerEntryData`, whose account layout puts the sequence
## number at a fixed offset.
func _next_sequence(key: PackedByteArray) -> int:
	var ledger_key := Xdr.account_ledger_key(key)
	if ledger_key.is_empty():
		return -1

	var response := await _rpc("getLedgerEntries", {
		"keys": [Marshalls.raw_to_base64(ledger_key)],
	})
	if not response["success"]:
		return -1

	var entries: Array = response["result"].get("entries", [])
	if entries.is_empty():
		return -1

	var entry: Dictionary = entries[0]
	if not entry.has("xdr"):
		return -1

	var sequence := Xdr.account_sequence(Marshalls.base64_to_raw(entry["xdr"]))
	if sequence < 0:
		return -1
	return sequence + 1


## Submits a signed envelope and waits for the network to include it.
func _send(signed_envelope: PackedByteArray) -> Dictionary:
	var response := await _rpc("sendTransaction", {
		"transaction": Marshalls.raw_to_base64(signed_envelope),
	})
	if not response["success"]:
		return response

	var result: Dictionary = response["result"]
	var status: String = result.get("status", "")
	if status != "PENDING" and status != "DUPLICATE":
		return _failure("the network rejected the transaction: %s" % JSON.stringify(result))

	return await _await_inclusion(result.get("hash", ""))


func _await_inclusion(hash: String) -> Dictionary:
	if hash.is_empty():
		return _failure("the network accepted the transaction without returning a hash")

	for _attempt in range(CONFIRMATION_ATTEMPTS):
		var response := await _rpc("getTransaction", {"hash": hash})
		if response["success"]:
			var result: Dictionary = response["result"]
			match result.get("status", ""):
				"SUCCESS":
					return {"success": true, "hash": hash, "ledger": result.get("ledger", 0)}
				"FAILED":
					return _failure("transaction %s failed on chain: %s" % [hash, JSON.stringify(result)])
		if is_inside_tree():
			await get_tree().create_timer(CONFIRMATION_DELAY).timeout

	return _failure(
		"transaction %s was submitted but not included after %d polls; check it with `stellar tx hash` or a block explorer"
		% [hash, CONFIRMATION_ATTEMPTS]
	)


## One request at a time, because a single HTTPRequest cannot carry two. A
## caller that starts a second call before awaiting the first gets a clear
## failure instead of two requests racing on one connection.
func _rpc(method: String, params: Dictionary) -> Dictionary:
	if _busy:
		return _failure(
			"a Soroban RPC request is already in flight; await the previous call before starting another"
		)
	_busy = true

	_request_id += 1
	var body := JSON.stringify({
		"jsonrpc": "2.0",
		"id": _request_id,
		"method": method,
		"params": params,
	})
	var headers := PackedStringArray(["Content-Type: application/json"])
	var error := _http.request(rpc_url, headers, HTTPClient.METHOD_POST, body)
	if error != OK:
		_busy = false
		return _failure("cannot reach %s: %s" % [rpc_url, error_string(error)])

	var response: Array = await _http.request_completed
	_busy = false
	if response[0] != HTTPRequest.RESULT_SUCCESS:
		return _failure("RPC request to %s failed: result %d" % [rpc_url, response[0]])

	var text: String = (response[3] as PackedByteArray).get_string_from_utf8()
	var parsed = JSON.parse_string(text)
	if typeof(parsed) != TYPE_DICTIONARY:
		return _failure("RPC returned a non-JSON body: %s" % text)
	if parsed.has("error"):
		return _failure("RPC error from %s: %s" % [method, parsed["error"]])

	return {"success": true, "result": parsed.get("result", {})}


# --- Helpers -----------------------------------------------------------------


func _contract_problem() -> String:
	if contract_id.is_empty():
		return "set COUGR_CONTRACT_ID, or CougrCheckpointClient.contract_id, to the deployed checkpoint contract"
	return ""


func _signing_problem() -> String:
	if source_account.is_empty():
		return "set COUGR_SOURCE_ACCOUNT, or CougrCheckpointClient.source_account, to the account that signs checkpoints"
	if signer == null:
		return "no signer is configured"
	return ""


## A zero ed25519 key. Simulation does not require the source account to exist,
## which is what lets read_state() run before any wallet is set up.
func _throwaway_key() -> PackedByteArray:
	var key := PackedByteArray()
	key.resize(32)
	return key


func _environment(name: String, fallback: String) -> String:
	var value := OS.get_environment(name)
	if value.is_empty():
		return fallback
	return value


func _failure(message: String) -> Dictionary:
	return {"success": false, "error": message}
