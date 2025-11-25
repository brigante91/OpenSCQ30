#!/bin/bash
# Backend script for OpenSCQ30 Plasma Widget
# This script queries the OpenSCQ30 CLI for device status and outputs JSON

set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Assume the project root is 2 levels up from backend/
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Check for custom CLI path from environment variable (set by widget config)
CUSTOM_CLI_PATH="${OPENSCQ30_CLI_PATH:-}"

# Try multiple possible locations for the CLI
CLI_PATHS=(
    "$CUSTOM_CLI_PATH"
    "$PROJECT_ROOT/target/release/openscq30"
    "$PROJECT_ROOT/target/debug/openscq30"
    "/usr/local/bin/openscq30"
    "/usr/bin/openscq30"
    "$HOME/.local/bin/openscq30"
    "$(command -v openscq30 2>/dev/null || echo '')"
)

# Find the first available CLI
CLI_PATH=""
for path in "${CLI_PATHS[@]}"; do
    if [ -n "$path" ] && [ -f "$path" ] && [ -x "$path" ]; then
        CLI_PATH="$path"
        break
    fi
done

if [ -z "$CLI_PATH" ]; then
    echo '{"connection_status":"disconnected","device_name":"CLI not found","error":"openscq30 CLI not found in any standard location. Please compile it or set OPENSCQ30_CLI_PATH environment variable."}' >&2
    exit 0
fi

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/openscq30"
DB_PATH="$CONFIG_DIR/database.sqlite"
OUTPUT_FILE="${XDG_RUNTIME_DIR:-/tmp}/openscq30-widget-status.json"

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo '{"connection_status":"disconnected","device_name":"No devices paired","error":"Database not found"}'
    exit 0
fi

# Check if sqlite3 is available
if ! command -v sqlite3 &> /dev/null; then
    echo '{"connection_status":"disconnected","device_name":"sqlite3 not found","error":"sqlite3 command not available"}' >&2
    exit 0
fi

# Try to get the first paired device with its model name
PAIRED_INFO=$(sqlite3 "$DB_PATH" "SELECT mac_address, model FROM paired_devices LIMIT 1" 2>/dev/null || echo "")

if [ -z "$PAIRED_INFO" ]; then
    echo '{"connection_status":"disconnected","device_name":"No devices paired"}'
    exit 0
fi

# Parse MAC address and model
PAIRED_DEVICE=$(echo "$PAIRED_INFO" | cut -d'|' -f1)
DEVICE_MODEL=$(echo "$PAIRED_INFO" | cut -d'|' -f2)

# Get device settings (this will attempt connection)
# Use timeout to avoid hanging
DEVICE_INFO=$(
    timeout 10 "$CLI_PATH" device -a "$PAIRED_DEVICE" setting \
        -g BatteryLevel \
        -g BatteryLevelLeft \
        -g BatteryLevelRight \
        -g IsCharging \
        -g IsChargingLeft \
        -g IsChargingRight \
        -g CaseBatteryLevel \
        --json 2>&1 || echo '{"error":"Connection failed"}'
)

# Check if we got an error
if [ -z "$DEVICE_INFO" ] || echo "$DEVICE_INFO" | grep -qi "error\|failed\|timeout"; then
    echo "{\"connection_status\":\"disconnected\",\"device_name\":\"$DEVICE_MODEL\",\"error\":\"Connection failed\"}"
    exit 0
fi

# Save raw output for debugging
echo "$DEVICE_INFO" > "$OUTPUT_FILE"

# Parse JSON output - try with jq first (more reliable)
if command -v jq &> /dev/null; then
    # Parse the JSON array returned by the CLI
    # Format: [{"settingId":"BatteryLevel","value":...}, ...]
    jq -c --arg model "$DEVICE_MODEL" '
        if type == "array" and length > 0 then
            # Convert array to object keyed by settingId
            reduce .[] as $item ({}; . + {
                ($item.settingId): $item.value
            }) |
            # Extract battery values - handle different formats
            (if .BatteryLevelLeft or .BatteryLevelRight then "dual" else "single" end) as $battery_type |
            {
                connection_status: "connected",
                device_name: $model,
                battery_type: $battery_type,
                battery_level: (
                    if $battery_type == "single" then
                        (.BatteryLevel // "0") | 
                        if type == "string" then
                            (split("/")[0] | split("(")[0] | tonumber? // 0)
                        else (. | tonumber? // 0) end
                    else 0 end
                ),
                battery_left: (
                    if $battery_type == "dual" then
                        (.BatteryLevelLeft // "0") |
                        if type == "string" then
                            (split("/")[0] | split("(")[0] | tonumber? // 0)
                        else (. | tonumber? // 0) end
                    else 0 end
                ),
                battery_right: (
                    if $battery_type == "dual" then
                        (.BatteryLevelRight // "0") |
                        if type == "string" then
                            (split("/")[0] | split("(")[0] | tonumber? // 0)
                        else (. | tonumber? // 0) end
                    else 0 end
                ),
                is_charging: (
                    (.IsCharging // "false") |
                    if type == "string" then
                        (test("true|yes|charging"; "i"))
                    else (. == true) end
                ),
                is_charging_left: (
                    if $battery_type == "dual" then
                        ((.IsChargingLeft // "false") |
                        if type == "string" then
                            (test("true|yes|charging"; "i"))
                        else (. == true) end)
                    else false end
                ),
                is_charging_right: (
                    if $battery_type == "dual" then
                        ((.IsChargingRight // "false") |
                        if type == "string" then
                            (test("true|yes|charging"; "i"))
                        else (. == true) end)
                    else false end
                ),
                case_battery: (
                    (.CaseBatteryLevel // "-1") |
                    if type == "string" then
                        if . == "-1" or . == "" then -1
                        else (split("/")[0] | split("(")[0] | tonumber? // -1) end
                    else (. | tonumber? // -1) end
                )
            }
        else
            {
                connection_status: "disconnected",
                device_name: $model,
                error: "Invalid response format"
            }
        end
    ' "$OUTPUT_FILE" 2>/dev/null || echo "{\"connection_status\":\"disconnected\",\"device_name\":\"$DEVICE_MODEL\",\"error\":\"JSON parse error\"}"
else
    # Fallback without jq - try basic parsing with grep/sed
    # This is less reliable but better than nothing
    BATTERY_LEVEL=$(echo "$DEVICE_INFO" | grep -o '"BatteryLevel"[^}]*' | grep -o '"[0-9]*"' | head -1 | tr -d '"' || echo "0")
    BATTERY_LEFT=$(echo "$DEVICE_INFO" | grep -o '"BatteryLevelLeft"[^}]*' | grep -o '"[0-9]*"' | head -1 | tr -d '"' || echo "0")
    BATTERY_RIGHT=$(echo "$DEVICE_INFO" | grep -o '"BatteryLevelRight"[^}]*' | grep -o '"[0-9]*"' | head -1 | tr -d '"' || echo "0")
    
    if [ -n "$BATTERY_LEFT" ] && [ "$BATTERY_LEFT" != "0" ]; then
        BATTERY_TYPE="dual"
    else
        BATTERY_TYPE="single"
    fi
    
    echo "{\"connection_status\":\"connected\",\"device_name\":\"$DEVICE_MODEL\",\"battery_type\":\"$BATTERY_TYPE\",\"battery_level\":${BATTERY_LEVEL:-0},\"battery_left\":${BATTERY_LEFT:-0},\"battery_right\":${BATTERY_RIGHT:-0},\"is_charging\":false,\"is_charging_left\":false,\"is_charging_right\":false,\"case_battery\":-1}"
fi

