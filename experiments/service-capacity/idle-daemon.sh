#!/usr/bin/env bash
# An idle laptop daemon whose only peer is offline: CPU and wakeups over three
# minutes. Throwaway: see README.md. Run from the repository root after
#   cargo build --release -p qurb-cli
#
# Needs a folder that is set up and paired, given as the first argument; it
# runs with HOME and the config directory pointed beside it, so nothing reaches
# the real folder registry. Uses port 9000 on loopback for a rendezvous service.
set -euo pipefail
cd "$(dirname "$0")/../.."
FOLDER=${1:?a set-up, paired folder}
SANDBOX=$(dirname "$FOLDER")/idle-home
mkdir -p "$SANDBOX/.config"
export HOME=$SANDBOX XDG_CONFIG_HOME=$SANDBOX/.config
target/release/qurb config "$FOLDER" signal=ws://127.0.0.1:9000 relay= >/dev/null
target/release/qurb signal 127.0.0.1:9000 >/dev/null 2>&1 & sig=$!
sleep 1
target/release/qurb run "$FOLDER" >/dev/null 2>&1 & d=$!
sleep 15   # past starting up
tick=$(getconf CLK_TCK)
wakes() { cat /proc/$1/task/*/status 2>/dev/null | awk '/^voluntary_ctxt_switches/ {s += $2} END {print s}'; }
ticks() { awk '{print $14+$15}' /proc/$1/stat; }
c0=$(ticks $d); w0=$(wakes $d); sc0=$(ticks $sig); sw0=$(wakes $sig)
sleep 180
c1=$(ticks $d); w1=$(wakes $d); sc1=$(ticks $sig); sw1=$(wakes $sig)
awk -v a=$c0 -v b=$c1 -v t=$tick 'BEGIN {printf "daemon cpu_s %.2f over 180 s (%.3f%% of one core)\n", (b-a)/t, (b-a)/t/180*100}'
echo "daemon wakeups, all threads: $((w1-w0)) ($(awk -v w=$((w1-w0)) 'BEGIN {printf "%.1f", w/180}') per second)"
echo "daemon threads $(ls /proc/$d/task | wc -l) rss_kib $(awk '/^VmRSS/ {print $2}' /proc/$d/status)"
awk -v a=$sc0 -v b=$sc1 -v t=$tick 'BEGIN {printf "rendezvous cpu_s %.2f\n", (b-a)/t}'
echo "rendezvous wakeups, all threads: $((sw1-sw0))"
kill $d $sig
