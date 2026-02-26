set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

install-hooks:
  scripts/install_hooks.sh

sync:
    @echo ">>> Syncing with main branch (remote)..."
    git fetch --all --prune
    git checkout main
    git reset --hard origin/main
    git clean -fd
    @echo ">>> Sync complete!"
    @echo ""
    @# Yerel branch'leri listele (main hariç)
    @branches=$(git branch | grep -v "main" | sed 's/[* ] //'); \
    if [ -n "$$branches" ]; then \
        echo ">>> Local branches remaining (excluding main):"; \
        for b in $$branches; do echo "  - $$b"; done; \
        echo ""; \
        printf ">>> Do you want to clean these branches? [y/N]: " && read ans && \
        if [ "$$ans" = "y" ] || [ "$$ans" = "Y" ]; then \
            echo ">>> Cleaning branches..."; \
            git branch -D $$branches; \
            echo ">>> Cleaned!"; \
        else \
            echo ">>> Keeping branches."; \
        fi; \
    else \
        echo ">>> No extra branches to clean."; \
    fi
  
fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --locked --all-targets --all-features -- -D warnings

test:
  cargo test --locked

quality:
  just fmt-check
  just clippy
  just test

release-pr:
  gh workflow run release-plz.yml
