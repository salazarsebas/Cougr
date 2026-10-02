extends SceneTree

# Compares the GDScript builder output against a vector pinned in Rust
# (clients/godot/rust_tests/src/lib.rs). Both sides must stay byte-identical.
func _init():
	var tx_builder = preload("res://addons/cougr_turn_based/tx_builder.gd")

	var source_account = "GAVR3J6DOKXZQNNFR4B337FDF7764XU2NDRTSW66J6I3OAYK7H3SNTY4"
	var sequence = 12345
	var contract_id_hex = "1122334455667788990011223344556677889900112233445566778899001122"
	var player_address = "GBG664Y7G5YI6VHT2U3W37N46D7DABQMMG5XUOSB3Q2HHRB6JOT5A76H"
	var position = 4
	var expected_hex = "00000002000000002b1da7c372af9835a58f03bdfca32fffee5e9a68e3395bde4f91b7030af9f726000000640000000000003039000000000000000000000001000000000000001800000000000000011122334455667788990011223344556677889900112233445566778899001122000000096d616b655f6d6f7665000000000000020000001300000000000000004def731f37708f54f3d5376dfdbcf0fe30060c61bb7a3a41dc3473c43e4ba7d00000000400000004000000000000000000000000"

	var bytes = tx_builder.build_make_move_tx(source_account, sequence, contract_id_hex, player_address, position)
	var hex = bytes.hex_encode()
	if hex != expected_hex:
		printerr("XDR mismatch")
		printerr("expected: ", expected_hex)
		printerr("actual:   ", hex)
		quit(1)
		return
	print("XDR_HEX=", hex)
	quit(0)

