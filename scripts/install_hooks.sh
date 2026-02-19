#!/usr/bin/env bash
set -euo pipefail

if ! command -v lefthook >/dev/null 2>&1; then
	echo "lefthook is required."
	echo "Install one of:"
	echo "  brew install lefthook"
	echo "  npm i -g @evilmartians/lefthook"
	exit 1
fi

chmod +x scripts/hook_pre_commit.sh
lefthook install --force
echo "Lefthook installed successfully."
