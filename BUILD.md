# 构建指南

本文档介绍如何构建云厂商图片处理参数转换工具。

## 目录

- [环境要求](#环境要求)
- [快速开始](#快速开始)
- [构建选项](#构建选项)
- [平台特定说明](#平台特定说明)
- [CI/CD](#cicd)

## 环境要求

### 通用要求

- **Rust** 1.75 或更高版本
  - 安装: https://rustup.rs/

### CLI 工具要求

仅需要 Rust 工具链。

### GUI 应用要求

- **Node.js** 20 或更高版本
  - 安装: https://nodejs.org/
- **Tauri CLI**
  - 安装: `cargo install tauri-cli`

## 快速开始

### 使用脚本构建

#### Linux/macOS

```bash
# 编译所有内容 (Debug)
./build.sh

# 编译 Release 版本
./build.sh --release

# 仅编译 CLI 工具
./build.sh --cli-only --release cli

# 交叉编译 CLI 工具
./build.sh --cli-only --release cli-cross

# 编译 GUI 应用
./build.sh --gui-only --release gui
```

#### Windows (PowerShell)

```powershell
# 编译所有内容 (Debug)
.\build.ps1

# 编译 Release 版本
.\build.ps1 -Release

# 仅编译 CLI 工具
.\build.ps1 -CliOnly -Release cli

# 交叉编译 CLI 工具
.\build.ps1 -CliOnly -Release cli-cross

# 编译 GUI 应用
.\build.ps1 -GuiOnly -Release gui
```

### 使用 Makefile 构建

Linux/macOS:

```bash
# 编译 CLI 工具 (Debug)
make cli

# 编译 CLI 工具 (Release)
make cli-release

# 编译 GUI 应用 (Release)
make gui-release

# 编译所有内容 (Release)
make release

# 运行测试
make test

# 清理
make clean
```

## 构建选项

### 编译模式

| 模式 | 说明 | 优化 | 体积 |
|------|------|------|------|
| Debug | 调试版本 | 无优化 | 大 |
| Release | 发布版本 | 最大优化 | 小 |

### 构建目标

| 目标 | 说明 | 输出位置 |
|------|------|----------|
| cli | CLI 工具 (当前平台) | `dist/cli/` |
| cli-cross | CLI 工具 (所有平台) | `dist/cli/` |
| gui | GUI 应用 (当前平台) | `src-tauri/target/release/bundle/` |
| all | 所有内容 | 见上 |

### 其他选项

| 选项 | 说明 |
|------|------|
| `--clean` / `-c` | 编译前清理 |
| `--verbose` / `-v` | 显示详细输出 |
| `--cli-only` | 仅编译 CLI 工具 |
| `--gui-only` | 仅编译 GUI 应用 |

## 平台特定说明

### Linux

#### 依赖

```bash
# Ubuntu/Debian
sudo apt-get install build-essential libssl-dev pkg-config

# Fedora
sudo dnf install gcc make openssl-devel

# Arch Linux
sudo pacman -S base-devel openssl
```

#### 交叉编译

需要安装对应的交叉编译工具链:

```bash
# ARM64
sudo apt-get install gcc-aarch64-linux-gnu

# Windows (MinGW)
sudo apt-get install mingw-w64
```

### macOS

#### 依赖

macOS 需要安装 Xcode Command Line Tools:

```bash
xcode-select --install
```

#### 通用二进制

对于 Apple Silicon Mac，可以构建通用二进制:

```bash
# 构建 x86_64
cargo build --release --target x86_64-apple-darwin

# 构建 arm64
cargo build --release --target aarch64-apple-darwin

# 合并为通用二进制
lipo -create -output imgconv-universal \
  target/x86_64-apple-darwin/release/imgconv \
  target/aarch64-apple-darwin/release/imgconv
```

### Windows

#### 依赖

需要安装:

- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- 或 [Visual Studio 2022](https://visualstudio.microsoft.com/)

#### PowerShell 执行策略

如果 PowerShell 脚本无法运行，需要调整执行策略:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## CI/CD

项目使用 GitHub Actions 进行持续集成。

### 工作流

- **test**: 在所有平台上运行测试
- **build-cli**: 构建所有平台的 CLI 工具
- **build-gui**: 构建所有平台的 GUI 应用
- **release**: 创建 GitHub Release (标签推送时)

### 本地测试 CI

使用 [act](https://github.com/nektos/act) 在本地测试 GitHub Actions:

```bash
# 安装 act
brew install act  # macOS
# 或
cargo install act

# 运行工作流
act -j test
```

## 输出文件

### CLI 工具

构建完成后，可执行文件位于:

| 平台 | Debug | Release |
|------|-------|---------|
| Linux | `target/debug/imgconv` | `target/release/imgconv` |
| macOS | `target/debug/imgconv` | `target/release/imgconv` |
| Windows | `target/debug/imgconv.exe` | `target/release/imgconv.exe` |

### GUI 应用

构建完成后，安装包位于:

| 平台 | 位置 |
|------|------|
| Linux | `src-tauri/target/release/bundle/{deb,appimage}/` |
| macOS | `src-troui/target/release/bundle/{dmg,app}/` |
| Windows | `src-tauri/target/release/bundle/{msi,nsis}/` |

## 故障排除

### 编译错误

#### OpenSSL 错误

```
error: failed to run custom build command for `openssl-sys v0.x.x`
```

解决方案:

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# macOS
brew install openssl

# Windows
# 安装 OpenSSL for Windows 或使用 vcpkg
```

#### Tauri 构建错误

```
error: webhook failed
```

解决方案:

```bash
# 检查 Node.js 版本
node --version  # 应该是 20 或更高

# 重新安装前端依赖
cd crates/tauri-app/src-ui
rm -rf node_modules package-lock.json
npm install
```

### 交叉编译问题

#### 找不到目标

```
error: target not found: x86_64-unknown-linux-gnu
```

解决方案:

```bash
rustup target add x86_64-unknown-linux-gnu
```

#### 链接错误

交叉编译时可能需要安装链接器:

```bash
# Ubuntu/Debian
sudo apt-get install gcc-mingw-w64-x86-64  # Windows
sudo apt-get install gcc-aarch64-linux-gnu  # ARM64
```

## 性能优化

### 编译时间优化

```bash
# 使用增量编译
export CARGO_INCREMENTAL=1

# 使用更快的链接器
# Ubuntu/Debian
sudo apt-get install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# macOS
export RUSTFLAGS="-C link-arg=-ld64.lld"
```

### 二进制大小优化

```bash
# 使用 LTO
cargo build --release --lto

# 去除符号
strip target/release/imgconv

# 使用 upx 压缩 (可选)
upx --best --lzma target/release/imgconv
```
