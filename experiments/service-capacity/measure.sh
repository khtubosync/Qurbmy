#!/usr/bin/env bash
# Measure the rendezvous service and the relay under load. Throwaway: see
# README.md. Run from the repository root, after
#   cargo build --release -p qurb-cli -p service-capacity
#
#   experiments/service-capacity/measure.sh signal 1000 4000 9000
#   experiments/service-capacity/measure.sh signal-tls 1000 4000
#   experiments/service-capacity/measure.sh relay-idle 1000
#   experiments/service-capacity/measure.sh relay "1 512" "8 128"
set -euo pipefail
cd "$(dirname "$0")/../.."
QURB=target/release/qurb
LOAD=target/release/service-capacity
TICK=$(getconf CLK_TCK)

# Resident memory now, and at its peak, in KiB; CPU seconds used so far.
rss()  { awk '/^VmRSS/ {print $2}' "/proc/$1/status"; }
peak() { awk '/^VmHWM/ {print $2}' "/proc/$1/status"; }
cpu()  { awk -v t="$TICK" '{printf "%.2f", ($14 + $15) / t}' "/proc/$1/stat"; }

free_port() { python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])'; }

# Run the load, answering each MEASURE line after reading the service.
drive() {
    local service=$1; shift
    coproc LOADGEN { "$LOAD" "$@" 2>&1; }
    while IFS= read -r line <&"${LOADGEN[0]}"; do
        case "$line" in
            MEASURE_*)
                echo "  ${line#MEASURE_}: rss $(rss "$service") KiB  peak $(peak "$service") KiB  cpu $(cpu "$service") s"
                echo >&"${LOADGEN[1]}"
                ;;
            *) echo "  $line" ;;
        esac
    done
    wait "$LOADGEN_PID"
}

mode=$1; shift
case "$mode" in
    signal)
        for devices in "$@"; do
            port=$(free_port)
            "$QURB" signal "127.0.0.1:$port" >/dev/null 2>&1 &
            service=$!
            sleep 1
            echo "== rendezvous, $devices devices"
            echo "  START: rss $(rss $service) KiB  cpu $(cpu $service) s"
            drive "$service" signal "ws://127.0.0.1:$port" "$devices" 200
            kill "$service"; wait "$service" 2>/dev/null || true
        done
        ;;
    signal-tls)
        # The service presenting its own certificate, as on a server with no
        # domain name; devices pin its fingerprint from the URL it prints.
        for devices in "$@"; do
            port=$(free_port)
            log=$(mktemp)
            "$QURB" signal "127.0.0.1:$port" --tls --host 127.0.0.1 >"$log" 2>&1 &
            service=$!
            for _ in $(seq 1 50); do grep -q "wss://" "$log" && break; sleep 0.1; done
            url=$(grep -o "wss://[^ ]*#[0-9a-f]*" "$log" | head -1)
            echo "== rendezvous over TLS, $devices devices"
            echo "  START: rss $(rss $service) KiB  cpu $(cpu $service) s"
            drive "$service" signal "$url" "$devices" 200
            kill "$service"; wait "$service" 2>/dev/null || true
            rm -f "$log"
        done
        ;;
    relay-idle)
        for devices in "$@"; do
            port=$(free_port)
            "$QURB" relay "127.0.0.1:$port" >/dev/null 2>&1 &
            service=$!
            sleep 1
            echo "== relay, $devices idle devices"
            echo "  START: rss $(rss $service) KiB  cpu $(cpu $service) s"
            drive "$service" relay-idle "127.0.0.1:$port" "$devices"
            kill "$service"; wait "$service" 2>/dev/null || true
        done
        ;;
    relay)
        for run in "$@"; do
            read -r pairs mib <<<"$run"
            port=$(free_port)
            "$QURB" relay "127.0.0.1:$port" >/dev/null 2>&1 &
            service=$!
            sleep 1
            echo "== relay, $pairs pair(s) x $mib MiB"
            echo "  START: rss $(rss $service) KiB  cpu $(cpu $service) s"
            drive "$service" relay "127.0.0.1:$port" "$pairs" "$mib"
            kill "$service"; wait "$service" 2>/dev/null || true
        done
        ;;
    *) echo "signal <devices>... | relay \"<pairs> <mib>\"..." >&2; exit 1 ;;
esac
