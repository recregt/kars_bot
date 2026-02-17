#!/usr/bin/env bash
set -euo pipefail

git config core.hooksPath .githooks
chmod +x .githooks/*
echo "Git hooks path set to .githooks"
echo "Installed hooks:"
ls -1 .githooks
