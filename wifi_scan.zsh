#!/usr/bin/env zsh

IFACE="wlan0"
SLEEP_SHORT=4
SLEEP_LONG=8

count_bssids() {
    echo "$1" \
      | grep -aEo '([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}' \
      | sort -u \
      | wc -l
}

run_scan() {
    local label="$1"
    shift
    local cmd=( "$@" )

    echo "[WiFi] $label"
    echo "  ${cmd[*]}"

    local output
    output=$("${cmd[@]}" 2>&1)

    local count
    count=$(count_bssids "$output")

    echo "  → Found $count unique BSSIDs"
    echo
}

run_trigger_dump() {
    local label="$1"
    shift
    local sleep_time="$1"
    shift
    local trigger_cmd=( "$@" )

    echo "[WiFi] $label"
    echo "  ${trigger_cmd[*]}"

    sudo iw dev $IFACE scan trigger flush >/dev/null 2>&1
    "${trigger_cmd[@]}" >/dev/null 2>&1
    sleep "$sleep_time"

    local output
    output=$(sudo iw dev $IFACE scan dump 2>&1)

    local count
    count=$(count_bssids "$output")

    echo "  → Found $count unique BSSIDs"
    echo
}

echo "==============================================="
echo " Wi-Fi scan comparison on interface: $IFACE"
echo "==============================================="
echo

# 1. Baseline immediate scan
run_scan "Baseline scan (immediate)" sudo iw dev $IFACE scan flush

sleep $SLEEP_SHORT

# 2. Triggered scan (recommended default)
run_trigger_dump "Triggered scan (6s wait)" 5 true

# 3. Long background-triggered scan
run_trigger_dump "Triggered scan (10s accumulation)" 50 true

echo "==============================================="
echo " Scan comparison complete"
echo "==============================================="
