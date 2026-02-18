#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cmd_src="src/commands/command_def.rs"
cfg_src="src/config/schema.rs"
cmd_out="docs/reference/commands.md"
cfg_out="docs/reference/config.md"

mkdir -p "docs/reference"

generate_commands() {
  cat >"$cmd_out" <<'EOF'
# Command Reference

This file is generated from src/commands/command_def.rs.
Do not edit manually. Run scripts/generate_docs_reference.sh.

| Command | Has Args | Description |
|---|---|---|
EOF

  awk '
    function emit_row(name, has_args, description, cmd, args) {
      if (name == "") return
      cmd = tolower(name)
      args = (has_args ? "Yes" : "No")
      gsub(/\|/, "\\|", description)
      gsub(/`/, "\\`", description)
      printf("| `/%s` | %s | %s |\n", cmd, args, description)
    }

    {
      if (match($0, /#\[command\(description = "(.*)"\)\]/, m)) {
        desc = m[1]
        next
      }

      if ($0 ~ /^[[:space:]]*[A-Za-z0-9_]+(\(String\))?,[[:space:]]*$/) {
        line = $0
        gsub(/[[:space:]]/, "", line)
        sub(/,$/, "", line)

        has_arg = (index(line, "(") > 0)
        name = line
        if (has_arg) {
          split(line, parts, "(")
          name = parts[1]
        }

        emit_row(name, has_arg, desc)
        desc = ""
      }
    }
  ' "$cmd_src" >>"$cmd_out"
}

generate_config() {
  cat >"$cfg_out" <<'EOF'
# Config Reference

This file is generated from src/config/schema.rs.
Do not edit manually. Run scripts/generate_docs_reference.sh.

EOF

  awk '
    function print_struct_header(struct_name) {
      printf("## %s\n\n", struct_name)
      print "| Field | Type | Default | Aliases |"
      print "|---|---|---|---|"
    }

    {
      if (match($0, /^pub struct ([A-Za-z0-9_]+) \{/, m)) {
        in_struct = 1
        struct_name = m[1]
        printed_header = 0
        default_flag = "No"
        alias_value = "-"
        next
      }

      if (in_struct && $0 ~ /^}/) {
        print ""
        in_struct = 0
        struct_name = ""
        printed_header = 0
        next
      }

      if (!in_struct) next

      if ($0 ~ /#\[serde\(default/) {
        default_flag = "Yes"
      }

      if (match($0, /alias = "([^"]+)"/, m)) {
        alias_value = m[1]
      }

      if (match($0, /^[[:space:]]*pub ([A-Za-z0-9_]+): ([^,]+),/, f)) {
        if (!printed_header) {
          print_struct_header(struct_name)
          printed_header = 1
        }

        field_name = f[1]
        field_type = f[2]
        gsub(/\|/, "\\|", field_type)
        printf("| `%s` | `%s` | %s | %s |\n", field_name, field_type, default_flag, alias_value)

        default_flag = "No"
        alias_value = "-"
      }
    }
  ' "$cfg_src" >>"$cfg_out"
}

generate_commands
generate_config

echo "Generated: $cmd_out"
echo "Generated: $cfg_out"
