

# Signs an unsigned envelope by handing it to `stellar tx sign`.
#
# Godot 4 has no ed25519 support and its hashing contexts stop at SHA-256, so
# the addon does not implement ed25519 itself. The Stellar CLI already holds the
# identity a deployer used to publish the contract, which keeps the testnet key
# out of the Godot process, out of the project directory, and out of git: the
# addon only ever reads the name of that identity from the environment.
#
# This is the default signer. Assign any object with a matching
# `sign(envelope, network_passphrase)` method to
# CougrCheckpointClient.signer to sign somewhere else, such as a hardware wallet
# or a remote signer.

const DEFAULT_BINARY := "stellar"

## Environment variable naming the CLI identity, secret key, or seed phrase to
## sign with. The value is expanded by the shell, so this process never builds a
## Godot argument out of it and the key stays out of the project.
const DEFAULT_KEY_ENV := "COUGR_SIGNING_KEY"

const _REQUEST_PATH := "user://cougr_unsigned.xdr"
const _RESPONSE_PATH := "user://cougr_signed.xdr"

## Stellar CLI binary name, or an absolute path when it is not on PATH.
var binary := DEFAULT_BINARY

## Environment variable holding the key or identity for `--sign-with-key`.
var key_env := DEFAULT_KEY_ENV


## Returns the signed envelope, or an empty PackedByteArray when signing fails.
##
## Callers must treat an empty result as a failure. Errors are reported through
## push_error() with the CLI's own message, because a wrong key, a missing
## binary, and an unknown identity all surface the same way here.
func sign(envelope_xdr: PackedByteArray, network_passphrase: String) -> PackedByteArray:
	if envelope_xdr.is_empty():
		push_error("Cougr: cannot sign an empty envelope")
		return PackedByteArray()
	if OS.get_environment(key_env).is_empty():
		push_error(
			"Cougr: set %s to the Stellar CLI identity or testnet secret key to sign with" % key_env
		)
		return PackedByteArray()
	if not _store(_REQUEST_PATH, Marshalls.raw_to_base64(envelope_xdr)):
		return PackedByteArray()

	var command := '"%s" tx sign --network-passphrase \'%s\' --sign-with-key "$%s" < "%s" > "%s"' % [
		binary,
		network_passphrase,
		key_env,
		ProjectSettings.globalize_path(_REQUEST_PATH),
		ProjectSettings.globalize_path(_RESPONSE_PATH),
	]
	var output := []
	var exit_code := OS.execute("/bin/sh", ["-c", command], output, true)
	if exit_code != 0:
		push_error(
			"Cougr: `%s tx sign` failed with exit code %d: %s"
			% [binary, exit_code, "\n".join(output)]
		)
		return PackedByteArray()

	var signed := Marshalls.base64_to_raw(_load(_RESPONSE_PATH))
	if signed.is_empty():
		push_error("Cougr: `%s tx sign` wrote no envelope to its output" % binary)
	return signed


func _store(path: String, contents: String) -> bool:
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		push_error("Cougr: cannot write %s: %s" % [path, error_string(FileAccess.get_open_error())])
		return false
	file.store_string(contents)
	file.close()
	return true


func _load(path: String) -> String:
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_error("Cougr: cannot read %s: %s" % [path, error_string(FileAccess.get_open_error())])
		return ""
	var contents := file.get_as_text()
	file.close()
	return contents.strip_edges()
