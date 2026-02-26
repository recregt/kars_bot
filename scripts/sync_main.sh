#!/usr/bin/env bash
set -euo pipefail

echo ">>> Syncing with main branch (remote)..."
git fetch --all --prune
git checkout main
git reset --hard origin/main
git clean -fd

echo ">>> Sync complete!"
echo ""

branches=$(git for-each-ref refs/heads --format='%(refname:short)' | grep -v '^main$' || true)

if [[ -n "$branches" ]]; then
  echo ">>> Local branches remaining (excluding main):"
  while IFS= read -r branch; do
    [[ -n "$branch" ]] && echo "  - $branch"
  done <<< "$branches"
  echo ""

  answer="n"
  if [[ -t 0 ]]; then
    read -r -p ">>> Do you want to clean these branches? [y/N]: " answer
  else
    echo ">>> Non-interactive mode detected; keeping branches by default."
  fi

  if [[ "$answer" == "y" || "$answer" == "Y" ]]; then
    echo ">>> Cleaning branches..."
    while IFS= read -r branch; do
      [[ -n "$branch" ]] && git branch -D "$branch"
    done <<< "$branches"
    echo ">>> Cleaned!"
  else
    echo ">>> Keeping branches."
  fi
else
  echo ">>> No extra branches to clean."
fi
