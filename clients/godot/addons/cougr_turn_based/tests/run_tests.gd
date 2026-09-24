extends SceneTree

func _init():
    var source_account = "GAVR3J6DOKXZQNNFR4B337FDF7764XU2NDRTSW66J6I3OAYK7H3SNTY4"
    var sequence = 12345
    var contract_id_hex = "1122334455667788990011223344556677889900112233445566778899001122"
    var player_address = "GBG664Y7G5YI6VHT2U3W37N46D7DABQMMG5XUOSB3Q2HHRB6JOT5A76H"
    var position = 4

    var bytes = SorobanMakeMoveTxBuilder.build_make_move_tx(source_account, sequence, contract_id_hex, player_address, position)
    var hex = bytes.hex_encode()
    print("XDR_HEX=", hex)
    quit(0)

