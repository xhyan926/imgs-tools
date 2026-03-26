# 云厂商图片处理参数转换工具 - Makefile
# 支持 Linux/macOS/Windows (with MinGW/MSYS)

.PHONY: all help clean test cli gui cli-cross release

# 默认目标
all: cli

# 显示帮助信息
help:
	@echo "云厂商图片处理参数转换工具 - Makefile"
	@echo ""
	@echo "可用目标:"
	@echo "  make              - 编译 CLI 工具 (Debug)"
	@echo "  make cli          - 编译 CLI 工具 (Debug)"
	@echo "  make cli-release  - 编译 CLI 工具 (Release)"
	@echo "  make cli-cross    - 交叉编译 CLI 工具 (所有平台)"
	@echo "  make gui          - 编译 GUI 应用 (Debug)"
	@echo "  make gui-release  - 编译 GUI 应用 (Release)"
	@echo "  make release      - 编译所有内容 (Release)"
	@echo "  make test         - 运行所有测试"
	@echo "  make clean        - 清理编译产物"
	@echo "  make install      - 安装 CLI 工具到本地"
	@echo ""

# 清理编译产物
clean:
	@echo "清理编译产物..."
	@cargo clean
	@rm -rf dist/
	@rm -rf crates/tauri-app/src-ui/dist/
	@rm -rf crates/tauri-app/src-tauri/target/
	@echo "清理完成"

# 运行测试
test:
	@echo "运行所有测试..."
	@cargo test --workspace

# 运行测试并显示输出
test-verbose:
	@echo "运行所有测试（详细输出）..."
	@cargo test --workspace -- --nocapture

# 编译 CLI 工具 (Debug)
cli:
	@echo "编译 CLI 工具 (Debug)..."
	@cargo build -p imgconv
	@mkdir -p dist/cli
	@cp target/debug/imgconv dist/cli/ || cp target/debug/imgconv.exe dist/cli/
	@echo "CLI 工具编译完成: dist/cli/"

# 编译 CLI 工具 (Release)
cli-release:
	@echo "编译 CLI 工具 (Release)..."
	@cargo build -p imgconv --release
	@mkdir -p dist/cli
	@cp target/release/imgconv dist/cli/ || cp target/release/imgconv.exe dist/cli/
	@echo "CLI 工具编译完成: dist/cli/"

# 交叉编译 CLI 工具
cli-cross:
	@echo "交叉编译 CLI 工具..."
	@chmod +x build.sh
	@./build.sh --cli-only -r cli-cross

# 编译 GUI 应用 (Debug)
gui:
	@echo "编译 GUI 应用 (Debug)..."
	@cd crates/tauri-app && cargo tauri build --debug

# 编译 GUI 应用 (Release)
gui-release:
	@echo "编译 GUI 应用 (Release)..."
	@cd crates/tauri-app && cargo tauri build

# 编译所有内容 (Release)
release: cli-release gui-release
	@echo "所有内容编译完成 (Release)"

# 安装 CLI 工具到本地
install:
	@echo "安装 CLI 工具到本地..."
	@cargo install --path crates/cli
	@echo "CLI 工具安装完成"

# 开发模式：运行 CLI
run-cli:
	@cargo run -p imgconv --

# 开发模式：运行 GUI
run-gui:
	@cd crates/tauri-app && cargo tauri dev

# 检查代码
check:
	@echo "检查代码..."
	@cargo check --workspace

# 格式化代码
fmt:
	@echo "格式化代码..."
	@cargo fmt --all

# Lint 代码
lint:
	@echo "检查代码风格..."
	@cargo clippy --workspace -- -D warnings

# 更新依赖
update:
	@echo "更新依赖..."
	@cargo update

# 构建 Docker 镜像
docker-build:
	@echo "构建 Docker 镜像..."
	@docker build -t imgconv:latest .

# 运行 Docker 容器
docker-run:
	@echo "运行 Docker 容器..."
	@docker run -it --rm imgconv:latest

# 发布到 crates.io
publish-core:
	@echo "发布核心库到 crates.io..."
	@cd crates/core && cargo publish

publish-cli:
	@echo "发布 CLI 工具到 crates.io..."
	@cd crates/cli && cargo publish

# 创建发布包
package: release
	@echo "创建发布包..."
	@mkdir -p dist/release
	@tar -czf dist/release/imgconv-$(shell git describe --tags --always).tar.gz -C dist/cli imgconv
	@echo "发布包创建完成: dist/release/"
