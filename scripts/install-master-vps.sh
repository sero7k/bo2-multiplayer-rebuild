#!/usr/bin/env bash
# bo2mp: install (or update) the online server -- iw4l-master, the relay and
# lobby list -- on a Linux VPS, run ON the VPS itself. Nothing is needed on
# your PC first, and the VPS needs no game files and no GPU.
#
#   curl -fsSL https://raw.githubusercontent.com/sero7k/bo2-multiplayer-rebuild/main/scripts/install-master-vps.sh | sudo bash
#
# Debian / Ubuntu (apt) or Fedora / Rocky / Alma (dnf). Run it again to
# update: it pulls, rebuilds and restarts, and keeps the certificates (so
# every player's ca.pem stays valid).
#
# What it does: installs git, a C toolchain and Rust (its own copy under
# /opt/iw4l, not touching anything else); builds iw4l-master; makes the
# certificates once under /etc/iw4l; installs the server's own systemd unit
# (iw4l-master print-unit) as the `iw4l` user; opens UDP 4433 when ufw or
# firewalld is on; then prints what to put in online.cfg and the ca.pem to
# hand to players. It records nothing on disk (docs/MASTER.md).
#
# Settings (environment): IW4L_REPO (git URL), IW4L_BRANCH (default main).
set -euo pipefail

REPO="${IW4L_REPO:-https://github.com/sero7k/bo2-multiplayer-rebuild.git}"
BRANCH="${IW4L_BRANCH:-main}"
ROOT=/opt/iw4l
SRC="$ROOT/src"
BIN_DIR=/usr/local/lib/iw4l
CERTS=/etc/iw4l
UNIT=/etc/systemd/system/iw4l-master.service
PORT=4433

say() { printf '\n== %s\n' "$*"; }

if [ "$(id -u)" -ne 0 ]; then
    echo "Run as root (sudo bash install-master-vps.sh)." >&2
    exit 1
fi

say "Packages"
if command -v apt-get >/dev/null; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq git curl ca-certificates build-essential pkg-config >/dev/null
elif command -v dnf >/dev/null; then
    dnf install -y -q git curl ca-certificates gcc gcc-c++ make pkgconf-pkg-config >/dev/null
else
    echo "Needs apt or dnf (Debian, Ubuntu, Fedora, Rocky, Alma)." >&2
    exit 1
fi

# A small VPS runs out of memory building: 2 GB of swap while it builds.
mem_kb=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
swap_kb=$(awk '/SwapTotal/ {print $2}' /proc/meminfo)
if [ "$mem_kb" -lt 3000000 ] && [ "$swap_kb" -lt 1000000 ] && [ ! -f "$ROOT/swapfile" ]; then
    say "Swap (this VPS has under 3 GB of memory)"
    mkdir -p "$ROOT"
    if fallocate -l 2G "$ROOT/swapfile" 2>/dev/null || dd if=/dev/zero of="$ROOT/swapfile" bs=1M count=2048 status=none; then
        chmod 600 "$ROOT/swapfile"
        mkswap -q "$ROOT/swapfile" && swapon "$ROOT/swapfile" || echo "(swap not enabled; building anyway)"
    fi
fi

say "Rust (its own copy under $ROOT)"
export RUSTUP_HOME="$ROOT/rustup" CARGO_HOME="$ROOT/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
if ! command -v cargo >/dev/null; then
    curl -fsSL https://sh.rustup.rs | sh -s -- -y -q --profile minimal --no-modify-path
fi
rustup -q update stable >/dev/null 2>&1 || true

say "Source ($REPO, $BRANCH)"
if [ -d "$SRC/.git" ]; then
    git -C "$SRC" fetch -q --depth 1 origin "$BRANCH"
    git -C "$SRC" reset -q --hard FETCH_HEAD
else
    rm -rf "$SRC"
    git clone -q --depth 1 --branch "$BRANCH" "$REPO" "$SRC"
fi
git -C "$SRC" log -1 --format='%h %s'

say "Building iw4l-master (a few minutes the first time)"
(cd "$SRC" && cargo build -q --release -p iw4l-master)
install -d "$BIN_DIR"
install -m 755 "$SRC/target/release/iw4l-master" "$BIN_DIR/iw4l-master"

say "User and certificates"
id iw4l >/dev/null 2>&1 || useradd --system --no-create-home --shell /usr/sbin/nologin iw4l
install -d -m 750 -g iw4l "$CERTS"
"$BIN_DIR/iw4l-master" gen-certs --out "$CERTS"
chown root:iw4l "$CERTS"/server.pem "$CERTS"/server.key "$CERTS"/ca.pem
chmod 640 "$CERTS"/server.key
chmod 644 "$CERTS"/server.pem "$CERTS"/ca.pem

say "Service"
"$BIN_DIR/iw4l-master" print-unit --channel prod --exec "$BIN_DIR/iw4l-master" \
    --cert "$CERTS/server.pem" --key "$CERTS/server.key" > "$UNIT"
systemctl daemon-reload
systemctl enable -q iw4l-master
systemctl restart iw4l-master
sleep 1
systemctl is-active --quiet iw4l-master && echo "iw4l-master is running" || {
    echo "iw4l-master did not start:" >&2
    journalctl -u iw4l-master -n 20 --no-pager >&2
    exit 1
}

say "Firewall (UDP $PORT)"
if command -v ufw >/dev/null && ufw status | grep -q "Status: active"; then
    ufw allow "$PORT/udp" >/dev/null && echo "ufw: opened $PORT/udp"
elif command -v firewall-cmd >/dev/null && firewall-cmd --state >/dev/null 2>&1; then
    firewall-cmd -q --permanent --add-port="$PORT/udp" && firewall-cmd -q --reload && echo "firewalld: opened $PORT/udp"
else
    echo "No ufw or firewalld running. If your provider has a cloud firewall, allow UDP $PORT there."
fi

ip=$(curl -fsS -4 --max-time 5 https://api.ipify.org 2>/dev/null || hostname -I | awk '{print $1}')
cat <<EOF

==================================================================
 Done. The online server is running on this VPS (UDP $PORT).

 1. On every player's PC, in online.cfg next to run-bo2.cmd:

      server=$ip:$PORT
      ca=online\\certs\\ca.pem
      name=iw4l-prod

 2. Save the block below as online\\certs\\ca.pem in each player's
    project folder (it is $CERTS/ca.pem on this VPS):

$(cat "$CERTS/ca.pem")

 Update later: run this script again. Logs: journalctl -u iw4l-master -f
==================================================================
EOF
