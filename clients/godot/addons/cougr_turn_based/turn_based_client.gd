class_name CougrTurnBasedClient
extends Node

const Xdr := preload("res://addons/cougr_turn_based/soroban_xdr.gd")
const CliSigner := preload("res://addons/cougr_turn_based/stellar_cli_signer.gd")
const TxBuilder := preload("res://addons/cougr_turn_based/tx_builder.gd")

const DEFAULT_RPC_URL := "https://soroban-testnet.stellar.org"
const TESTNET_PASSPHRASE := "Test SDF Network ; September 2015"
const CONFIRMATION_ATTEMPTS := 10
const CONFIRMATION_DELAY := 1.0

@export var contract_id := ""
@export var source_account := ""
@export var rpc_url := ""
@export var network_passphrase := ""
var signer = null
var _http: HTTPRequest
var _request_id := 0
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

func read_state() -> Dictionary:
    var problem = _contract_problem()
    if problem != "":
        return _failure(problem)
    
    # We construct a get_state invocation.
    var xdr = PackedByteArray()
    xdr.append_array(TxBuilder.pack_u32(2))
    xdr.append_array(TxBuilder.pack_u32(0))
    var dummy_key = PackedByteArray()
    dummy_key.resize(32)
    xdr.append_array(dummy_key)
    xdr.append_array(TxBuilder.pack_u32(100))
    xdr.append_array(TxBuilder.pack_u64(0))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(1))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(24))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(1))
    xdr.append_array(TxBuilder.hex_decode(contract_id))
    xdr.append_array(TxBuilder.pack_string("get_state"))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(0))
    xdr.append_array(TxBuilder.pack_u32(0))
    
    var simulation = await _simulate(xdr)
    if not simulation["success"]:
        return simulation
    return {"success": true, "result_scval": simulation["result_scval"]}

func make_move(position: int) -> Dictionary:
    var problem = _contract_problem()
    if problem != "":
        return _failure(problem)
    problem = _signing_problem()
    if problem != "":
        return _failure(problem)
    
    var source_key = Xdr.account_key(source_account)
    if source_key.is_empty():
        return _failure("invalid source account")
    var sequence = await _next_sequence(source_key)
    if sequence < 0:
        return _failure("could not read sequence")
    
    var base_tx = TxBuilder.build_make_move_tx(source_account, sequence, contract_id, source_account, position)
    var simulation = await _simulate(base_tx)
    if not simulation["success"]:
        return simulation
    
    var tx_with_auth = TxBuilder.build_make_move_tx(source_account, sequence, contract_id, source_account, position, simulation["auth_encoded"], simulation["transaction_data"], simulation["fee"])
    var signed = signer.sign(TxBuilder.pack_u32(2) + tx_with_auth.slice(4), network_passphrase)
    if signed.is_empty():
        return _failure("signer returned empty")
    return await _send(signed)

func _simulate(envelope_xdr: PackedByteArray) -> Dictionary:
    var response = await _rpc("simulateTransaction", {"transaction": Marshalls.raw_to_base64(envelope_xdr)})
    if not response["success"]:
        return response
    var result = response["result"]
    if result.has("error"):
        return _failure("simulation failed: " + result["error"])
    var results = result.get("results", [])
    if results.is_empty():
        return _failure("simulation returned no result")
    var first = results[0]
    var fee = int(result.get("minResourceFee", "0")) + 10000
    
    var auth_arr = first.get("auth", [])
    var auth_encoded = PackedByteArray()
    auth_encoded.append_array(TxBuilder.pack_u32(auth_arr.size()))
    for a in auth_arr:
        auth_encoded.append_array(Marshalls.base64_to_raw(a))
        
    return {
        "success": true,
        "transaction_data": Marshalls.base64_to_raw(result.get("transactionData", "")),
        "auth_encoded": auth_encoded,
        "fee": fee,
        "result_scval": Marshalls.base64_to_raw(first.get("xdr", ""))
    }

func _next_sequence(key: PackedByteArray) -> int:
    var ledger_key = Xdr.account_ledger_key(key)
    if ledger_key.is_empty(): return -1
    var response = await _rpc("getLedgerEntries", {"keys": [Marshalls.raw_to_base64(ledger_key)]})
    if not response["success"]: return -1
    var entries = response["result"].get("entries", [])
    if entries.is_empty(): return -1
    var entry = entries[0]
    if not entry.has("xdr"): return -1
    var sequence = Xdr.account_sequence(Marshalls.base64_to_raw(entry["xdr"]))
    if sequence < 0: return -1
    return sequence + 1

func _send(signed_envelope: PackedByteArray) -> Dictionary:
    var response = await _rpc("sendTransaction", {"transaction": Marshalls.raw_to_base64(signed_envelope)})
    if not response["success"]: return response
    var result = response["result"]
    var status = result.get("status", "")
    if status != "PENDING" and status != "DUPLICATE":
        return _failure("network rejected: " + JSON.stringify(result))
    return await _await_inclusion(result.get("hash", ""))

func _await_inclusion(hash: String) -> Dictionary:
    if hash.is_empty(): return _failure("no hash returned")
    for _attempt in range(CONFIRMATION_ATTEMPTS):
        var response = await _rpc("getTransaction", {"hash": hash})
        if response["success"]:
            var result = response["result"]
            match result.get("status", ""):
                "SUCCESS": return {"success": true, "hash": hash, "ledger": result.get("ledger", 0)}
                "FAILED": return _failure("transaction failed")
        if is_inside_tree():
            await get_tree().create_timer(CONFIRMATION_DELAY).timeout
    return _failure("timed out waiting for inclusion")

func _rpc(method: String, params: Dictionary) -> Dictionary:
    if _busy: return _failure("busy")
    _busy = true
    _request_id += 1
    var body = JSON.stringify({"jsonrpc": "2.0", "id": _request_id, "method": method, "params": params})
    var error = _http.request(rpc_url, PackedStringArray(["Content-Type: application/json"]), HTTPClient.METHOD_POST, body)
    if error != OK:
        _busy = false
        return _failure("request error")
    var response = await _http.request_completed
    _busy = false
    if response[0] != HTTPRequest.RESULT_SUCCESS: return _failure("request failed")
    var text = (response[3] as PackedByteArray).get_string_from_utf8()
    var parsed = JSON.parse_string(text)
    if typeof(parsed) != TYPE_DICTIONARY: return _failure("non-JSON")
    if parsed.has("error"): return _failure("RPC error")
    return {"success": true, "result": parsed.get("result", {})}

func _contract_problem() -> String:
    if contract_id.is_empty(): return "set COUGR_CONTRACT_ID"
    return ""

func _signing_problem() -> String:
    if source_account.is_empty(): return "set COUGR_SOURCE_ACCOUNT"
    if signer == null: return "no signer"
    return ""

func _environment(name: String, fallback: String) -> String:
    var value = OS.get_environment(name)
    if value.is_empty(): return fallback
    return value

func _failure(message: String) -> Dictionary:
    return {"success": false, "error": message}
