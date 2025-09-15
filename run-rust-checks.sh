#!/usr/bin/env bash
set -euo pipefail

REPO="$HOME/nexus-cli"
echo "Repo: $REPO"
cd "$REPO"

echo
echo "1) Searching for Cargo.toml files (this may show workspace members and crate crates)..."
mapfile -t TOMLS < <(find . -type f -name Cargo.toml 2>/dev/null | sed 's|^\./||')

if [ ${#TOMLS[@]} -eq 0 ]; then
  echo "No Cargo.toml files found anywhere inside $REPO."
  echo "If the Rust crate(s) are in a different folder, run this script from that path or tell me where they are."
  exit 1
fi

echo "Found ${#TOMLS[@]} Cargo.toml file(s):"
for t in "${TOMLS[@]}"; do
  echo " - $t"
done

echo
echo "2) For each crate we will run: cargo fmt, cargo build --release, cargo test"
read -p "Proceed to run these checks for each Cargo.toml? (y/N) " yn
if [[ "$yn" != "y" && "$yn" != "Y" ]]; then
  echo "Aborting. You can re-run the script and answer 'y' to continue."
  exit 0
fi

# Ensure rust env loaded
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

# run commands per manifest
FAILS=0
LOGDIR="/tmp/nexus-rust-logs-$(date +%s)"
mkdir -p "$LOGDIR"

for manifest in "${TOMLS[@]}"; do
  echo
  echo "==== Processing: $manifest ===="
  mpath="$REPO/$manifest"
  mdir=$(dirname "$mpath")
  echo "Manifest path: $mpath"
  echo "Crate dir: $mdir"
  echo

  echo "-> cargo fmt --manifest-path $mpath"
  if ! cargo fmt --manifest-path "$mpath" 2>&1 | tee "$LOGDIR/fmt-$(basename "$mdir").log"; then
    echo "cargo fmt failed for $manifest (see $LOGDIR/fmt-$(basename "$mdir").log)"
    FAILS=$((FAILS+1))
  fi

  echo "-> cargo build --manifest-path $mpath --release"
  if ! cargo build --manifest-path "$mpath" --release 2>&1 | tee "$LOGDIR/build-$(basename "$mdir").log"; then
    echo "cargo build failed for $manifest (see $LOGDIR/build-$(basename "$mdir").log)"
    FAILS=$((FAILS+1))
    # continue to next crate rather than aborting
    continue
  fi

  echo "-> cargo test --manifest-path $mpath"
  if ! cargo test --manifest-path "$mpath" 2>&1 | tee "$LOGDIR/test-$(basename "$mdir").log"; then
    echo "cargo test failed for $manifest (see $LOGDIR/test-$(basename "$mdir").log)"
    FAILS=$((FAILS+1))
  fi

done

echo
echo "Finished checks. Logs are in: $LOGDIR"
if [ "$FAILS" -eq 0 ]; then
  echo "All crates formatted, built, and tested successfully."
else
  echo "Some steps failed for one or more crates ($FAILS failures)."
  echo "Open the logs in $LOGDIR to inspect the first failing output."
  echo "If you paste the top ~80 lines from the failing log(s) here, I will give exact fixes."
fi
