#!/bin/bash
# AUR 自动发布脚本 for ccuse
# 自动构建、打包并推送到 AUR

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# 配置
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
AUR_DIR="$PROJECT_DIR/.aur-build"
AUR_REMOTE="ssh://aur@aur.archlinux.org/ccuse.git"

# 清理函数
cleanup() {
    if [ -d "$AUR_DIR" ]; then
        info "清理临时文件 ..."
        rm -rf "$AUR_DIR"
    fi
    # 清理可能残留的 tarball
    rm -f "$PROJECT_DIR"/*.tar.gz 2>/dev/null || true
}

# 确保退出时清理
trap cleanup EXIT

# 从 Cargo.toml 读取版本
get_version() {
    grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/'
}

# 计算 sha256sum
calc_sha256() {
    sha256sum "$1" | cut -d' ' -f1
}

# 生成 PKGBUILD
generate_pkgbuild() {
    local version="$1"
    local sha256="$2"

    cat > "$AUR_DIR/PKGBUILD" << EOF
# Maintainer: wjsoj <wjs@wjsphy.top>

pkgname=ccuse
pkgver=${version}
pkgrel=1
pkgdesc="A CLI tool to manage and switch between Claude Code profiles"
arch=('x86_64' 'aarch64')
provides=('ccuse-bin')
conflicts=('ccuse-bin')
url="https://github.com/wjsoj/ccuse"
license=('MIT')
depends=('gcc-libs')
makedepends=('rust')
source=("\$pkgname-\$pkgver.tar.gz::https://github.com/wjsoj/ccuse/archive/refs/tags/v\$pkgver.tar.gz")
sha256sums=('${sha256}')

prepare() {
  cd "\$srcdir/\$pkgname-\$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "\$CARCH-unknown-linux-gnu"
}

build() {
  cd "\$srcdir/\$pkgname-\$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --all-features
}

check() {
  cd "\$srcdir/\$pkgname-\$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen --all-features
}

package() {
  cd "\$srcdir/\$pkgname-\$pkgver"
  install -Dm755 "target/release/ccuse" "\$pkgdir/usr/bin/ccuse"
  install -Dm644 "LICENSE" "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
  install -Dm644 "README.md" "\$pkgdir/usr/share/doc/\$pkgname/README.md"
}
EOF
}

# 生成 .SRCINFO
generate_srcinfo() {
    info "生成 .SRCINFO ..."
    cd "$AUR_DIR"
    makepkg --printsrcinfo > .SRCINFO
}

# 初始化或更新 AUR 仓库
setup_aur_repo() {
    info "准备 AUR 仓库 ..."
    rm -rf "$AUR_DIR"

    info "克隆 AUR 仓库 (首次会显示 empty repository 警告，这是正常的)..."
    info "URL: $AUR_REMOTE"
    git clone "$AUR_REMOTE" "$AUR_DIR" || {
        error "无法克隆 AUR 仓库，请确认 SSH 密钥已配置或包已存在"
    }
    info "AUR 仓库克隆完成"
}

# 提交并推送
commit_and_push() {
    local version="$1"

    cd "$AUR_DIR"

    # 添加所有需要的文件
    git add PKGBUILD .SRCINFO

    # 复制 LICENSE 文件到 AUR 目录
    if [ -f "$PROJECT_DIR/LICENSE" ]; then
        cp "$PROJECT_DIR/LICENSE" "$AUR_DIR/"
        git add LICENSE
    fi

    if git diff --cached --quiet; then
        warn "没有变更需要提交"
        return 0
    fi

    git commit -m "Update to v${version}"

    # 确保 branch 名为 master（AUR 只接受 master）
    local current_branch=$(git branch --show-current)
    if [ "$current_branch" != "master" ]; then
        info "重命名分支 $current_branch 为 master..."
        git branch -m master
    fi

    info "推送到 AUR (可能需要输入 SSH 密钥)..."
    git push origin master || error "推送失败，请检查 SSH 配置和网络连接"
    info "推送成功"
}

# 主流程
main() {
    cd "$PROJECT_DIR"

    VERSION=$(get_version)
    info "当前版本: $VERSION"

    # 检查是否有对应的 GitHub tag
    if ! git ls-remote --tags origin | grep -q "refs/tags/v${VERSION}"; then
        warn "GitHub 上没有 v${VERSION} tag"
        read -p "是否创建并推送 tag? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git tag -a "v${VERSION}" -m "Release v${VERSION}"
            git push origin "v${VERSION}"
            info "Tag v${VERSION} 已推送到 GitHub"
        else
            error "请先在 GitHub 创建 release tag"
        fi
    fi

    # 获取 GitHub release 的 sha256
    info "获取 GitHub release 的 sha256 ..."
    local tarball_url="https://github.com/wjsoj/ccuse/archive/refs/tags/v${VERSION}.tar.gz"
    local tmpfile=$(mktemp)
    info "下载 tarball: $tarball_url"
    curl -L "$tarball_url" -o "$tmpfile" || error "下载失败，请检查网络和 tag 是否存在"
    SHA256=$(calc_sha256 "$tmpfile")
    rm -f "$tmpfile"
    info "SHA256: $SHA256"

    # 设置 AUR 仓库
    setup_aur_repo

    # 生成 PKGBUILD
    info "生成 PKGBUILD ..."
    generate_pkgbuild "$VERSION" "$SHA256"

    # 生成 .SRCINFO
    generate_srcinfo

    # 提交并推送
    commit_and_push "$VERSION"

    echo ""
    echo -e "${GREEN}======================================${NC}"
    echo -e "${GREEN}✓ 完成! ccuse v${VERSION} 已发布到 AUR${NC}"
    echo -e "${GREEN}======================================${NC}"
    echo ""
}

main "$@"
