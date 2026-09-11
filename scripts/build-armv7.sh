#!/usr/bin/env bash
# 交叉编译 orbien / orbien-server 到 ARMv7 软浮点 (armv7-unknown-linux-musleabi)
# 面向刷了 FreshTomato 的 Netgear R7000 (Broadcom BCM4709, Cortex-A9)
#
# FreshTomato 的 Broadcom SDK 工具链是 arm-brcm-linux-uclibcgnueabi（软浮点 EABI），
# 硬浮点 hf 二进制在上面跑不了，所以目标选 musleabi 而非 musleabihf。
# 我们编的是静态 musl 二进制，不依赖固件自带 uClibc，但 ABI 必须匹配：软浮点对软浮点。
#
# 用法:
#   ./scripts/build-armv7.sh              # 只编译 client（默认）
#   ./scripts/build-armv7.sh server       # 只编译 server
#   ./scripts/build-armv7.sh all          # 两者都编译
#   ./scripts/build-armv7.sh all --upx    # 额外产出 UPX 压缩版
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 默认编译 soft 软浮点版本；如需编译 softfp，可设置 FLOAT_ABI=softfp 或传 --abi=softfp
FLOAT_ABI="${FLOAT_ABI:-soft}"
WHAT="${1:-client}"
shift || true
DO_UPX=0
for a in "$@"; do
  case "$a" in
    --upx) DO_UPX=1 ;;
    --abi=*) FLOAT_ABI="${a#*=}" ;;
    --float-abi=*) FLOAT_ABI="${a#*=}" ;;
    --soft) FLOAT_ABI="soft" ;;
    --softfp) FLOAT_ABI="softfp" ;;
  esac
done

case "$FLOAT_ABI" in
  soft)
    # 软浮点 EABI，对应 FreshTomato 的 gnueabi ABI
    TARGET="armv7-unknown-linux-musleabi"
    # musl.cc 没有 armv7l 的 musleabi 包，用 arm-linux-musleabi 配合 -march=armv7-a
    TRIPLE="arm-linux-musleabi"
    FLOAT_CFLAGS="-march=armv7-a -mtune=cortex-a9 -mfloat-abi=soft"
    ABI_SUFFIX="musleabi"
    ;;
  softfp)
    # 软浮点 ABI 但允许 VFP 使用；需要目标环境确实支持 VFP/softfp ABI
    TARGET="armv7-unknown-linux-musleabihf"
    TRIPLE="arm-linux-musleabihf"
    FLOAT_CFLAGS="-march=armv7-a -mtune=cortex-a9 -mfpu=vfpv3-d16 -mfloat-abi=softfp"
    ABI_SUFFIX="musleabihf"
    ;;
  *)
    echo "unknown float ABI: $FLOAT_ABI (soft|softfp)" >&2
    exit 1
    ;;
esac

XC_DIR="${XC_DIR:-/tmp/xc/${TRIPLE}-cross}"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"

# ---------- 1. 交叉工具链 ----------
if [[ ! -x "${XC_DIR}/bin/${TRIPLE}-gcc" ]]; then
  echo "==> 下载 ${TRIPLE} 交叉工具链"
  mkdir -p "$(dirname "$XC_DIR")"
  curl -fsSL -o /tmp/xc-armv7.tgz "https://musl.cc/${TRIPLE}-cross.tgz"
  tar xzf /tmp/xc-armv7.tgz -C "$(dirname "$XC_DIR")"
fi
export PATH="${XC_DIR}/bin:$PATH"

# 由 TARGET 推导 cargo/CC 等环境变量名，避免 TARGET 改动后漏改
T_UPPER="$(echo "$TARGET" | tr '-' '_' | tr 'a-z' 'A-Z')"
export "CARGO_TARGET_${T_UPPER}_LINKER=${TRIPLE}-gcc"
export "CC_${T_UPPER}=${TRIPLE}-gcc"
export "CXX_${T_UPPER}=${TRIPLE}-g++"
export "AR_${T_UPPER}=${TRIPLE}-ar"

# 软浮点配置：
#   -march=armv7-a -mtune=cortex-a9 : R7000 的 BCM4709 是 Cortex-A9
#   -mfloat-abi=soft    : 纯软浮点，不生成 VFP/NEON 指令
#   -mfloat-abi=softfp  : 允许使用 VFP，但 ABI 仍保持 softfp，需目标固件/内核支持
#   -mfpu=vfpv3-d16     : softfp 需要显式设置 FPU feature
CFLAGS_VAR="CFLAGS_${T_UPPER}"
export "${CFLAGS_VAR}=${FLOAT_CFLAGS}"
export "CXXFLAGS_${T_UPPER}=${FLOAT_CFLAGS}"

# 不要加 +neon / +vfp：目标默认不带这些 feature，且在 FreshTomato 上不可靠
export RUSTFLAGS="-C link-arg=-s"

# ---------- 2. Rust target ----------
if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo 未安装，请先: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
  exit 1
fi
rustup target add "$TARGET" >/dev/null 2>&1 || true

# ---------- 3. 编译 ----------
PKGS=()
case "$WHAT" in
  client|orbien)      PKGS=(-p orbien-client) ;;
  server|orbien-server) PKGS=(-p orbien-server) ;;
  all)                PKGS=(-p orbien-server -p orbien-client) ;;
  *) echo "unknown target: $WHAT (client|server|all)" >&2; exit 1 ;;
esac

echo "==> float-abi : $FLOAT_ABI"
echo "==> target : $TARGET"
echo "==> cflags : ${!CFLAGS_VAR}"
echo "==> cargo build --release --locked ${PKGS[*]} --target $TARGET"
cargo build --release --locked "${PKGS[@]}" --target "$TARGET"

BIN_DIR="target/${TARGET}/release"
OUT="dist/release"
mkdir -p "$OUT"

# ---------- 4. 打包 (与 CI 命名保持一致) ----------
# --upx 时同时产出未压缩版与 UPX 压缩版，原二进制保持不压缩
if [[ "$DO_UPX" -eq 1 ]] && ! command -v upx >/dev/null 2>&1; then
  echo "!! 未找到 upx，只产出未压缩版 (apt install upx-ucl)"
  DO_UPX=0
fi

pack_one() {
  local name="$1" bin="$2" conf="$3"
  [[ -f "$bin" ]] || return 0

  local -a variants=("")
  [[ "$DO_UPX" -eq 1 ]] && variants+=("upx")

  for v in "${variants[@]}"; do
    local src="$bin" suffix=""
    if [[ "$v" == "upx" ]]; then
      src="${bin}.upx"; suffix="_upx"
      cp "$bin" "$src"
      upx --lzma --best "$src" >/dev/null 2>&1 || true
    fi

    # 手动打包：pack-release.sh 不支持 musleabi 命名与 upx 变体
    local stage
    stage="$(mktemp -d)"
    cp "$src" "${stage}/${name}"
    chmod +x "${stage}/${name}"
    [[ -n "$conf" && -f "$conf" ]] && cp "$conf" "${stage}/"
    local archive="${OUT}/${name}_${VERSION}_linux_armv7_${ABI_SUFFIX}${suffix}.tar.gz"
    tar -C "$stage" -czf "$archive" .
    rm -rf "$stage"
    [[ "$v" == "upx" ]] && rm -f "$src"
    echo "wrote ${archive}"
  done
}

if [[ "$WHAT" == "all" || "$WHAT" == "server" || "$WHAT" == "orbien-server" ]]; then
  pack_one orbien-server "$BIN_DIR/orbien-server" conf/orbien-server.toml
fi
if [[ "$WHAT" == "all" || "$WHAT" == "client" || "$WHAT" == "orbien" ]]; then
  pack_one orbien "$BIN_DIR/orbien" conf/orbien.toml
fi

echo ""
echo "==> 产物:"
ls -lh "$BIN_DIR"/orbien "$BIN_DIR"/orbien-server 2>/dev/null || true
[[ -d "$OUT" ]] && ls -lh "$OUT"/*.tar.gz 2>/dev/null || true
