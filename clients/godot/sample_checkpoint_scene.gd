extends Control

# Sample scene for the Cougr checkpoint addon. It commits a checkpoint and,
# separately, raises a dispute against it on Stellar testnet.
#
# Point it at your contract before running. Nothing here is a secret except the
# signing key, which stays outside the project:
#
#   export COUGR_RPC_URL=https://soroban-testnet.stellar.org
#   export COUGR_CONTRACT_ID=C...         # printed by `stellar contract deploy`
#   export COUGR_SOURCE_ACCOUNT=G...      # the player that commits checkpoints
#   export COUGR_SIGNING_KEY=cougr-testnet
#   godot --path clients/godot
#
# COUGR_SIGNING_KEY names a `stellar keys generate cougr-testnet` identity, or a
# testnet secret key starting with S. With it unset, Read state still works: a
# state read simulates the contract and needs no signature.
#
# The contract has to be a checkpoint contract, and the source account has to be
# the player that opened the match with `start_match`, or `commit_checkpoint`
# comes back rejected with `notplay`.

const Client := preload("res://addons/cougr_checkpoint/checkpoint_client.gd")

## Tick index of the first checkpoint this scene commits. The contract requires
## every commit to be strictly greater than the last one.
const FIRST_TICK := 1024

## Off-chain state hash committed at each checkpoint, as 16 hex characters.
const COMMITTED_HASH := "0123456789abcdef"

## Hash the dispute claims instead. It only has to differ from the committed one
## for the contract to accept the dispute.
const DISPUTED_HASH := "fedcba9876543210"

## Score committed alongside each checkpoint.
const COMMITTED_SCORE := 4200

var _client
var _log: RichTextLabel
var _lines: PackedStringArray = []
var _next_tick := FIRST_TICK
var _last_committed_tick := -1


func _ready() -> void:
	_client = Client.new()
	_client.name = "CougrCheckpointClient"
	add_child(_client)
	_build_ui()
	_report_configuration()


func _build_ui() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)

	var margin := MarginContainer.new()
	margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_%s" % side, 24)
	add_child(margin)

	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 10)
	margin.add_child(column)

	var title := Label.new()
	title.text = "Cougr checkpoint client"
	title.add_theme_font_size_override("font_size", 20)
	column.add_child(title)

	var hint := Label.new()
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.text = (
		"Set COUGR_CONTRACT_ID, COUGR_SOURCE_ACCOUNT and COUGR_SIGNING_KEY before running. "
		+ "Commit a checkpoint, then raise a dispute against the same tick with a different hash."
	)
	column.add_child(hint)

	var buttons := HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 8)
	column.add_child(buttons)
	_add_button(buttons, "ReadStateButton", "Read state", _on_read_state)
	_add_button(buttons, "CommitButton", "Commit checkpoint", _on_commit)
	_add_button(buttons, "DisputeButton", "Raise dispute", _on_dispute)

	_log = RichTextLabel.new()
	_log.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_log.scroll_following = true
	column.add_child(_log)


func _add_button(parent: Node, node_name: String, text: String, handler: Callable) -> void:
	var button := Button.new()
	button.name = node_name
	button.text = text
	button.pressed.connect(handler)
	parent.add_child(button)


func _report_configuration() -> void:
	_log_line("RPC            %s" % _client.rpc_url)
	_log_line("contract       %s" % _or_unset(_client.contract_id, "COUGR_CONTRACT_ID"))
	_log_line("source account %s" % _or_unset(_client.source_account, "COUGR_SOURCE_ACCOUNT"))
	_log_line("signing key    %s" % _or_unset(OS.get_environment("COUGR_SIGNING_KEY"), "COUGR_SIGNING_KEY"))
	_log_line("")


# --- Button handlers ---------------------------------------------------------


func _on_read_state() -> void:
	var result: Dictionary = await _client.read_state()
	if not result["success"]:
		_log_line("read_state failed: %s" % result["error"])
		return
	var state: Dictionary = result["state"]
	_log_line(
		"state player=%s tick=%d hash=%s score=%d status=%s"
		% [
			state["player"],
			state["last_tick"],
			state["last_hash"],
			state["last_score"],
			state["status_name"],
		]
	)


func _on_commit() -> void:
	var tick := _next_tick
	_log_line("commit_checkpoint tick=%d hash=%s score=%d" % [tick, COMMITTED_HASH, COMMITTED_SCORE])
	var result: Dictionary = await _client.commit_checkpoint(tick, COMMITTED_HASH, COMMITTED_SCORE)
	if not result["success"]:
		_log_line("  rejected: %s" % result["error"])
		return
	_log_line("  included in ledger %d as %s" % [result["ledger"], result["hash"]])
	_log_line("  signed payload %s..." % result["signing_payload"].substr(0, 64))
	_last_committed_tick = tick
	_next_tick += 1


func _on_dispute() -> void:
	if _last_committed_tick < 0:
		_log_line(
			"Commit a checkpoint first: the contract only accepts a dispute for the last committed tick, and only inside the dispute window."
		)
		return
	_log_line(
		"dispute_checkpoint tick=%d hash=%s" % [_last_committed_tick, DISPUTED_HASH]
	)
	var result: Dictionary = await _client.raise_dispute(_last_committed_tick, DISPUTED_HASH)
	if not result["success"]:
		_log_line("  rejected: %s" % result["error"])
		return
	_log_line("  included in ledger %d as %s" % [result["ledger"], result["hash"]])


# --- Logging -----------------------------------------------------------------


func _log_line(line: String) -> void:
	_lines.append(line)
	_log.text = "\n".join(_lines)
	print("[checkpoint sample] %s" % line)


func _or_unset(value: String, name: String) -> String:
	if value.is_empty():
		return "(unset: export %s)" % name
	return value
