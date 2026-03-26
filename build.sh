#!/bin/bash
# 云厂商图片处理参数转换工具 - 编译脚本
# 支持 Linux/macOS 平台

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 显示帮助信息
show_help() {
    cat << EOF
云厂商图片处理参数转换工具 - 编译脚本

用法: ./build.sh [选项] [目标]

选项:
    -h, --help          显示此帮助信息
    -r, --release       编译 Release 版本（默认为 Debug）
    -v, --verbose       显示详细输出
    -c, --clean         编译前清理
    --cli-only          仅编译 CLI 工具
    --gui-only          仅编译 GUI 应用

目标:
    cli                 编译 CLI 工具（当前平台）
    cli-cross           交叉编译 CLI 工具（Windows + Linux）
    gui                 编译 GUI 应用（当前平台）
    gui-all             编译所有平台的 GUI 安装包
    all                 编译所有内容（默认）

示例:
    ./build.sh                      # 编译所有内容（Debug）
    ./build.sh -r cli               # 编译 CLI 工具（Release）
    ./build.sh -r cli-cross         # 交叉编译 CLI 工具
    ./build.sh -r gui               # 编译 GUI 应用（Release）
    ./build.sh -r gui-all           # 编译所有平台 GUI 安装包

EOF
}

# 检测系统
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS=Linux;;
        Darwin*)    OS=macOS;;
        MINGW*|MSYS*|CYGWIN*) OS=Windows;;
        *)          OS="Unknown";;
    esac
    echo "检测到操作系统: $OS"
}

# 检查依赖
check_dependencies() {
    print_info "检查依赖..."

    # 检查 Rust
    if ! command -v rustc &> /dev/null; then
        print_error "未安装 Rust，请访问 https://rustup.rs/ 安装"
        exit 1
    fi

    # 检查 Node.js (用于 GUI)
    if [ "$BUILD_GUI" = "true" ]; then
        if ! command -v node &> /dev/null; then
            print_warning "未安装 Node.js，GUI 编译将失败"
            print_warning "请访问 https://nodejs.org/ 安装 Node.js"
        fi
    fi

    # 检查 cargo-cross (用于交叉编译)
    if [ "$TARGET" = "cli-cross" ]; then
        if ! command -v cross &> /dev/null; then
            print_warning "未安装 cross，将使用 cargo 的 --target 参数"
            print_warning "建议安装: cargo install cross"
        fi
    fi

    # 检查 Tauri CLI (用于 GUI 编译)
    if [ "$BUILD_GUI" = "true" ]; then
        if ! cargo tauri --version &> /dev/null 2>&1; then
            print_warning "未安装 Tauri CLI，正在安装..."
            cargo install tauri-cli
        fi
    fi

    print_success "依赖检查完成"
}

# 编译 CLI 工具
build_cli() {
    local target_triple="$1"
    local cargo_cmd="cargo"

    if [ -n "$target_triple" ]; then
        print_info "交叉编译 CLI 工具 ($target_triple)..."
        if command -v cross &> /dev/null; then
            cargo_cmd="cross"
        else
            print_warning "使用 cargo 直接交叉编译，需要安装对应 target"
            rustup target add "$target_triple" 2>/dev/null || true
        fi
    else
        print_info "编译 CLI 工具 ($OS)..."
    fi

    local build_args=("build")
    if [ "$RELEASE" = "true" ]; then
        build_args+=("--release")
    fi

    if [ -n "$target_triple" ]; then
        build_args+=("--target" "$target_triple")
    fi

    if [ "$VERBOSE" = "true" ]; then
        build_args+=("--verbose")
    fi

    if [ "$CLEAN" = "true" ]; then
        print_info "清理之前的编译产物..."
        $cargo_cmd clean -p imgconv
    fi

    cd crates/cli
    $cargo_cmd "${build_args[@]}"
    cd ../..

    # 确定输出目录
    local output_dir="target/debug"
    if [ "$RELEASE" = "true" ]; then
        output_dir="target/release"
    fi
    if [ -n "$target_triple" ]; then
        output_dir="target/$target_triple/${output_dir#target/}"
    fi

    # 复制到 dist 目录
    mkdir -p dist/cli
    local exe_name="imgconv"
    if [ "$OS" = "Windows" ]; then
        exe_name="imgconv.exe"
    fi

    if [ -f "$output_dir/$exe_name" ]; then
        cp "$output_dir/$exe_name" "dist/cli/"
        print_success "CLI 工具编译完成: dist/cli/$exe_name"
    else
        print_error "编译产物未找到: $output_dir/$exe_name"
        return 1
    fi
}

# 交叉编译 CLI 工具（所有平台）
build_cli_cross() {
    print_info "开始交叉编译 CLI 工具..."

    local targets=(
        "x86_64-unknown-linux-gnu"      # Linux x64
        "x86_64-pc-windows-gnu"         # Windows x64 (MinGW)
        "x86_64-apple-darwin"           # macOS x64 (当前 macOS)
        "aarch64-unknown-linux-gnu"     # Linux ARM64
    )

    for target in "${targets[@]}"; do
        echo ""
        build_cli "$target"
    done

    print_success "所有平台的 CLI 工具编译完成"
}

# 编译 GUI 应用
build_gui() {
    print_info "编译 GUI 应用..."

    cd crates/tauri-app

    # 检查是否需要安装前端依赖
    if [ ! -d "src-ui/node_modules" ]; then
        print_info "安装前端依赖..."
        cd src-ui
        npm install
        cd ..
    fi

    local build_args=("build")
    if [ "$VERBOSE" = "true" ]; then
        build_args+=("--verbose")
    fi

    if [ "$CLEAN" = "true" ]; then
        print_info "清理之前的编译产物..."
        cargo tauri clean
    fi

    cargo tauri "${build_args[@]}"

    cd ../..

    print_success "GUI 应用编译完成"
    print_info "安装包位置: crates/tauri-app/src-tauri/target/release/bundle/"
}

# 编译所有平台的 GUI 应用
build_gui_all() {
    print_info "编译所有平台的 GUI 安装包..."

    cd crates/tauri-app

    # 构建 Linux
    print_info "构建 Linux 版本..."
    cargo tauri build --target x86_64-unknown-linux-gnu

    # 构建 Windows（需要交叉编译环境）
    # cargo tauri build --target x86_64-pc-windows-gnu

    # 构建 macOS（当前平台）
    print_info "构建 macOS 版本..."
    cargo tauri build --target x86_64-apple-darwin

    # 构建 macOS ARM64
    if [ "$(uname -m)" = "arm64" ]; then
        cargo tauri build --target aarch64-apple-darwin
    fi

    cd ../..

    print_success "所有平台的 GUI 应用编译完成"
}

# 主函数
main() {
    local TARGET=""
    local RELEASE="false"
    local VERBOSE="false"
    local CLEAN="false"
    local BUILD_CLI="true"
    local BUILD_GUI="true"

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -r|--release)
                RELEASE="true"
                shift
                ;;
            -v|--verbose)
                VERBOSE="true"
                shift
                ;;
            -c|--clean)
                CLEAN="true"
                shift
                ;;
            --cli-only)
                BUILD_GUI="false"
                shift
                ;;
            --gui-only)
                BUILD_CLI="false"
                shift
                ;;
            cli|cli-cross|gui|gui-all|all)
                TARGET="$1"
                shift
                ;;
            *)
                print_error "未知参数: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # 默认目标
    if [ -z "$TARGET" ]; then
        TARGET="all"
    fi

    echo "================================================"
    echo "  云厂商图片处理参数转换工具 - 编译脚本"
    echo "================================================"
    echo ""

    # 检测系统
    detect_os
    echo ""

    # 检查依赖
    check_dependencies
    echo ""

    # 创建输出目录
    mkdir -p dist

    # 根据目标执行编译
    case $TARGET in
        cli)
            if [ "$BUILD_CLI" = "true" ]; then
                build_cli
            fi
            ;;
        cli-cross)
            if [ "$BUILD_CLI" = "true" ]; then
                build_cli_cross
            fi
            ;;
        gui)
            if [ "$BUILD_GUI" = "true" ]; then
                build_gui
            fi
            ;;
        gui-all)
            if [ "$BUILD_GUI" = "true" ]; then
                build_gui_all
            fi
            ;;
        all)
            if [ "$BUILD_CLI" = "true" ]; then
                build_cli
                echo ""
            fi
            if [ "$BUILD_GUI" = "true" ]; then
                build_gui
            fi
            ;;
    esac

    echo ""
    print_success "编译完成！"
    echo ""
    echo "输出文件:"
    echo "  CLI 工具: dist/cli/"
    echo "  GUI 应用: crates/tauri-app/src-tauri/target/release/bundle/"
    echo ""
}

# 运行主函数
main "$@"
