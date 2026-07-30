#!/usr/bin/env bash
# Shared PATH for Cougr Circom scripts (Bun-managed node_modules).

cougr_circuits_root() {
  local lib_dir
  lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cd "$lib_dir/.." && pwd
}

cougr_circuits_export_path() {
  local root
  root="$(cougr_circuits_root)"
  local repo_root
  repo_root="$(cd "$root/../.." && pwd)"
  export PATH="$repo_root/target/circom_bin/bin:$root/node_modules/.bin:$PATH"
}