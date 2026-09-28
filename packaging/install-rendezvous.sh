#!/usr/bin/env bash
# The rendezvous service on this computer, as a user service at login -- with
# push, when there is a Firebase key to wake phones with.
#
#   ./packaging/install-rendezvous.sh
#   ./packaging/install-rendezvous.sh --uninstall
#
# Builds qurb with the push feature and installs it as qurb-rendezvous: a name
# of its own, because a plain `cargo build` and packaging/install.sh put a qurb
# without push at ~/.local/bin/qurb, and a service started with --push refuses
# to run on one of those. Push is on when ~/.config/qurb/firebase.json exists:
# a service-account key from the Firebase project the Android app was built
# with. Publishing the service is separate -- `tailscale funnel --bg 9000`.
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
UNITS="$CONFIG/systemd/user"
KEY="$CONFIG/qurb/firebase.json"

if [[ ${1:-} == --uninstall ]]; then
    systemctl --user disable --now qurb-rendezvous 2>/dev/null || true
    rm -f "$UNITS/qurb-rendezvous.service" "$BIN/qurb-rendezvous" "$CONFIG/qurb/rendezvous.env"
    systemctl --user daemon-reload
    echo "Removed the rendezvous service."
    exit 0
fi

cargo build --release -p qurb-cli --features push
mkdir -p "$BIN" "$UNITS" "$CONFIG/qurb"
install -m755 target/release/qurb "$BIN/qurb-rendezvous"
install -m644 packaging/qurb-rendezvous.service "$UNITS/qurb-rendezvous.service"

if [[ -f $KEY ]]; then
    echo "PUSH=--push $KEY" >"$CONFIG/qurb/rendezvous.env"
    push="on, with $KEY"
else
    : >"$CONFIG/qurb/rendezvous.env"
    push="off -- put a Firebase service-account key at $KEY and run this again"
fi

systemctl --user daemon-reload
systemctl --user enable qurb-rendezvous >/dev/null 2>&1
systemctl --user restart qurb-rendezvous

echo "The rendezvous service is running on 127.0.0.1:9000, and starts at login."
echo "Push: $push"
