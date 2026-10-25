#!/usr/bin/env bash
# 启动一个隔离的 zed 实例，指向 fake provider。
# 用法: ./run-zed.sh [ZED_BINARY] [DATA_DIR]
set -euo pipefail
ZED=${1:-/home/aa/repos/ide_ls/learn_ls/zed/target/debug/zed}
DATA=${2:-/tmp/opencode/zedtest/data}
PROJECT=${PROJECT:-/tmp/opencode/zedtest/project}

mkdir -p "$DATA/config" "$PROJECT"
# --user-data-dir 会把配置目录改成 <DATA>/config
cp "$(dirname "$0")/settings.json" "$DATA/config/settings.json"

systemctl --user stop zedtest.service 2>/dev/null || true
systemd-run --user --unit=zedtest --collect \
  --setenv=WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}" \
  --setenv=DISPLAY="${DISPLAY:-:0}" \
  --setenv=XDG_RUNTIME_DIR="/run/user/$(id -u)" \
  --setenv=DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$(id -u)/bus" \
  --setenv=STUB_API_KEY=stub \
  --setenv=XDG_CONFIG_HOME="$(dirname "$DATA")/config" \
  ${STUB_PROXY:+--setenv=ALL_PROXY=$STUB_PROXY --setenv=HTTP_PROXY=$STUB_PROXY --setenv=HTTPS_PROXY=$STUB_PROXY --setenv=NO_PROXY= --setenv=no_proxy=} \
  "$ZED" --user-data-dir "$DATA" "$PROJECT"
echo "zed started as unit zedtest (log: $DATA/logs/Zed.log)"
