#!/usr/bin/env bash
# Set up a server of your own from this computer, and check it works.
#
#   packaging/server/deploy.sh ubuntu@203.0.113.5
#
# Needs SSH access to the server with sudo. Sends this computer's qurb when the
# server can run it -- an x86-64 machine with a C library at least as new as the
# one it was built against -- and otherwise the source, to build there. Then
# runs setup.sh there, and checks from here, across the internet, that the
# rendezvous service introduces two devices and that the relay carries a
# transfer. Run from the repository root after
#   cargo build --release -p qurb-cli -p service-capacity
set -euo pipefail
cd "$(dirname "$0")/../.."
SERVER=${1:?user@host of the server}
# A new server's host key is accepted the first time, since nobody is there to
# confirm it; one that changes afterwards is still refused.
SSH_OPTS=(-o StrictHostKeyChecking=accept-new -o ConnectTimeout=15)
say()  { printf '\n== %s\n' "$*"; }
fail() { echo "deploy: $*" >&2; exit 1; }

[[ -x target/release/qurb ]] || fail "build first: cargo build --release -p qurb-cli -p service-capacity"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp packaging/server/setup.sh packaging/server/qurb-signal-tls.service packaging/server/qurb-relay.service "$stage/"

say "looking at $SERVER"
arch=$(ssh "${SSH_OPTS[@]}" "$SERVER" uname -m)
theirs=$(ssh "${SSH_OPTS[@]}" "$SERVER" "ldd --version 2>&1 | head -1 | grep -o '[0-9]\+\.[0-9]\+\$'" || true)
needs=$(objdump -T target/release/qurb | grep -o 'GLIBC_[0-9.]*' | sed 's/GLIBC_//' | sort -V | tail -1)
echo "  $arch, C library ${theirs:-unknown}; this qurb needs ${needs}"

if [[ $arch == x86_64 && -n $theirs && $(printf '%s\n%s\n' "$needs" "$theirs" | sort -V | head -1) == "$needs" ]]; then
    say "sending this computer's qurb"
    cp target/release/qurb "$stage/qurb"
    strip "$stage/qurb"
    how=(--binary ./qurb)
else
    say "sending the source, to build there (it cannot run this computer's build)"
    git archive --format=tar -o "$stage/qurb-src.tar" HEAD
    how=(--source ./qurb-src.tar)
fi

remote=/tmp/qurb-setup
ssh "${SSH_OPTS[@]}" "$SERVER" "rm -rf $remote && mkdir -p $remote"
scp -q "${SSH_OPTS[@]}" "$stage"/* "$SERVER:$remote/"
ssh "${SSH_OPTS[@]}" "$SERVER" "sudo bash $remote/setup.sh ${how[*]}" 2>&1 | tee "$stage/setup.log"
ssh "${SSH_OPTS[@]}" "$SERVER" "rm -rf $remote"

url=$(grep -o "wss://[^ ]*#[0-9a-f]\{64\}" "$stage/setup.log" | tail -1 || true)
[[ -n $url ]] || fail "setup did not print the rendezvous address"
host=${url#wss://}; host=${host%%:*}

say "checking from here, across the internet"
for port in 9000 9001; do
    if timeout 5 bash -c "exec 3<>/dev/tcp/$host/$port" 2>/dev/null; then
        echo "  port $port answers"
    else
        echo "  port $port does not answer -- open it in the cloud's own firewall (see above)"
        fail "not reachable yet"
    fi
done
if [[ -x target/release/service-capacity ]]; then
    # Two devices introduced by the service, and a small transfer through the
    # relay: the two things the phone needs from it. Answered by pressing
    # Enter for it, at the points the load generator pauses to be measured.
    yes "" | target/release/service-capacity signal "$url" 2 1 | grep -E "CONNECTED|INTRODUCTIONS"
    yes "" | target/release/service-capacity relay "$host:9001" 1 8 | grep RELAYED
fi
say "the server works; the lines to put on your computer and phone are above"
