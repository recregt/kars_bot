#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEFTHOOK_CONFIG="$ROOT_DIR/lefthook.yml"
HOOK_PRE_COMMIT="$ROOT_DIR/scripts/hook_pre_commit.sh"

is_ci() {
	[[ "${CI:-}" == "true" ]]
}

ensure_lefthook_config() {
	if [[ ! -f "$LEFTHOOK_CONFIG" ]]; then
		echo "lefthook config missing: $LEFTHOOK_CONFIG"
		return 1
	fi
	return 0
}

try_install_lefthook() {
	if command -v go >/dev/null 2>&1; then
		echo "Installing lefthook with go..."
		if go install github.com/evilmartians/lefthook@latest; then
			export PATH="$(go env GOPATH)/bin:$PATH"
			return 0
		fi
	fi

	if command -v cargo >/dev/null 2>&1; then
		echo "Installing lefthook with cargo..."
		if cargo install --locked lefthook; then
			local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
			export PATH="$cargo_home/bin:$PATH"
			return 0
		fi
	fi

	return 1
}

ensure_lefthook_binary() {
	if command -v lefthook >/dev/null 2>&1; then
		return 0
	fi

	local mode="${LEFTHOOK_AUTO_INSTALL:-prompt}"

	if [[ "$mode" == "never" ]]; then
		echo "lefthook is required but not installed."
		return 1
	fi

	if [[ "$mode" == "always" ]] || is_ci; then
		if try_install_lefthook; then
			return 0
		fi
		echo "lefthook auto-install failed (go/cargo)."
		return 1
	fi

	if [[ "$mode" == "prompt" && -t 0 ]]; then
		read -r -p "lefthook not found. Try auto-install with go/cargo? [Y/n] " answer
		case "${answer:-Y}" in
			[Yy]|[Yy][Ee][Ss]|"")
				if try_install_lefthook; then
					return 0
				fi
				echo "lefthook auto-install failed (go/cargo)."
				return 1
				;;
			*)
				echo "lefthook installation skipped by user."
				return 1
				;;
		esac
	fi

	echo "lefthook is required."
	echo "Install one of:"
	echo "  go install github.com/evilmartians/lefthook@latest"
	echo "  cargo install --locked lefthook"
	echo "  brew install lefthook"
	echo "  npm i -g @evilmartians/lefthook"
	return 1
}

if ! ensure_lefthook_config; then
	exit 1
fi

if ! ensure_lefthook_binary; then
	exit 1
fi

if [[ -f "$HOOK_PRE_COMMIT" ]]; then
	chmod +x "$HOOK_PRE_COMMIT"
elif ! is_ci; then
	echo "Missing hook script: $HOOK_PRE_COMMIT"
	exit 1
fi

lefthook install --force
echo "Lefthook installed successfully."
