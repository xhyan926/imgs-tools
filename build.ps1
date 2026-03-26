# 云厂商图片处理参数转换工具 - 编译脚本
# 支持 Windows 平台

param(
    [switch]$Help,
    [switch]$Release,
    [switch]$Verbose,
    [switch]$Clean,
    [switch]$CliOnly,
    [switch]$GuiOnly,
    [Parameter(Position=0)]
    [ValidateSet("cli", "cli-cross", "gui", "gui-all", "all", "")]
    $Target = ""
)

# 颜色输出函数
function Print-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Print-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Print-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Print-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# 显示帮助信息
function Show-Help {
    @"
云厂商图片处理参数转换工具 - 编译脚本

用法: .\build.ps1 [选项] [目标]

选项:
    -h, -Help            显示此帮助信息
    -r, -Release         编译 Release 版本（默认为 Debug）
    -v, -Verbose         显示详细输出
    -c, -Clean           编译前清理
    -CliOnly             仅编译 CLI 工具
    -GuiOnly             仅编译 GUI 应用

目标:
    cli                  编译 CLI 工具（当前平台）
    cli-cross            交叉编译 CLI 工具（Windows + Linux）
    gui                  编译 GUI 应用（当前平台）
    gui-all              编译所有平台的 GUI 安装包
    all                  编译所有内容（默认）

示例:
    .\build.ps1                      # 编译所有内容（Debug）
    .\build.ps1 -r cli               # 编译 CLI 工具（Release）
    .\build.ps1 -r cli-cross         # 交叉编译 CLI 工具
    .\build.ps1 -r gui               # 编译 GUI 应用（Release）
    .\build.ps1 -r gui-all           # 编译所有平台 GUI 安装包

"@
}

# 检查依赖
function Test-Dependencies {
    Print-Info "检查依赖..."

    # 检查 Rust
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Print-Error "未安装 Rust，请访问 https://rustup.rs/ 安装"
        exit 1
    }
    Print-Success "Rust 已安装: $(rustc --version)"

    # 检查 Node.js (用于 GUI)
    if ($script:BuildGui) {
        if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
            Print-Warning "未安装 Node.js，GUI 编译将失败"
            Print-Warning "请访问 https://nodejs.org/ 安装 Node.js"
        } else {
            Print-Success "Node.js 已安装: $(node --version)"
        }
    }

    # 检查 Tauri CLI (用于 GUI 编译)
    if ($script:BuildGui) {
        $tauriVersion = cargo tauri --version 2>$null
        if (-not $tauriVersion) {
            Print-Warning "未安装 Tauri CLI，正在安装..."
            cargo install tauri-cli
        } else {
            Print-Success "Tauri CLI 已安装: $tauriVersion"
        }
    }

    Print-Success "依赖检查完成"
}

# 编译 CLI 工具
function Build-Cli {
    param(
        [string]$TargetTriple
    )

    if ($TargetTriple) {
        Print-Info "交叉编译 CLI 工具 ($TargetTriple)..."
        rustup target add $TargetTriple 2>$null | Out-Null
    } else {
        Print-Info "编译 CLI 工具 (Windows)..."
    }

    $buildArgs = @("build")
    if ($Release) {
        $buildArgs += "--release"
    }

    if ($TargetTriple) {
        $buildArgs += "--target"
        $buildArgs += $TargetTriple
    }

    if ($Verbose) {
        $buildArgs += "--verbose"
    }

    if ($Clean) {
        Print-Info "清理之前的编译产物..."
        cargo clean -p imgconv
    }

    Push-Location crates\cli
    & cargo $buildArgs
    Pop-Location

    # 确定输出目录
    $outputDir = "target\debug"
    if ($Release) {
        $outputDir = "target\release"
    }
    if ($TargetTriple) {
        $outputDir = "target\$TargetTriple\release"
    }

    # 创建 dist 目录
    New-Item -ItemType Directory -Force -Path "dist\cli" | Out-Null

    # 复制可执行文件
    $exePath = "$outputDir\imgconv.exe"
    if (Test-Path $exePath) {
        Copy-Item $exePath "dist\cli\"
        Print-Success "CLI 工具编译完成: dist\cli\imgconv.exe"
    } else {
        Print-Error "编译产物未找到: $exePath"
        return $false
    }

    return $true
}

# 交叉编译 CLI 工具（所有平台）
function Build-CliCross {
    Print-Info "开始交叉编译 CLI 工具..."

    $targets = @(
        "x86_64-pc-windows-msvc",      # Windows x64 (当前)
        "x86_64-unknown-linux-gnu",    # Linux x64
        "aarch64-unknown-linux-gnu",   # Linux ARM64
        "x86_64-apple-darwin"          # macOS x64
    )

    foreach ($target in $targets) {
        Write-Host ""
        Build-Cli -TargetTriple $target
    }

    Print-Success "所有平台的 CLI 工具编译完成"
}

# 编译 GUI 应用
function Build-Gui {
    Print-Info "编译 GUI 应用..."

    Push-Location crates\tauri-app

    # 检查是否需要安装前端依赖
    if (-not (Test-Path "src-ui\node_modules")) {
        Print-Info "安装前端依赖..."
        Push-Location src-ui
        npm install
        Pop-Location
    }

    $buildArgs = @("build")
    if ($Verbose) {
        $buildArgs += "--verbose"
    }

    if ($Clean) {
        Print-Info "清理之前的编译产物..."
        cargo tauri clean
    }

    & cargo tauri $buildArgs

    Pop-Location

    Print-Success "GUI 应用编译完成"
    Print-Info "安装包位置: crates\tauri-app\src-tauri\target\release\bundle\"
}

# 编译所有平台的 GUI 应用
function Build-GuiAll {
    Print-Info "编译所有平台的 GUI 安装包..."

    Push-Location crates\tauri-app

    # 构建 Windows（当前平台）
    Print-Info "构建 Windows 版本..."
    cargo tauri build --target x86_64-pc-windows-msvc

    # 注意：交叉编译 GUI 应用需要额外的依赖和环境设置
    # Linux 和 macOS 的构建需要在对应平台上进行

    Pop-Location

    Print-Success "GUI 应用编译完成"
    Print-Warning "注意：完整的跨平台 GUI 构建需要在对应平台上进行"
    Print-Warning "GitHub Actions 可以用于自动化构建所有平台"
}

# 主函数
function Main {
    Write-Host "================================================"
    Write-Host "  云厂商图片处理参数转换工具 - 编译脚本"
    Write-Host "================================================"
    Write-Host ""

    # 默认值
    if (-not $Target) {
        $Target = "all"
    }

    $script:BuildCli = -not $GuiOnly
    $script:BuildGui = -not $CliOnly

    # 检查依赖
    Test-Dependencies
    Write-Host ""

    # 创建输出目录
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null

    # 根据目标执行编译
    switch ($Target) {
        "cli" {
            if ($BuildCli) {
                Build-Cli
            }
        }
        "cli-cross" {
            if ($BuildCli) {
                Build-CliCross
            }
        }
        "gui" {
            if ($BuildGui) {
                Build-Gui
            }
        }
        "gui-all" {
            if ($BuildGui) {
                Build-GuiAll
            }
        }
        "all" {
            if ($BuildCli) {
                Build-Cli
                Write-Host ""
            }
            if ($BuildGui) {
                Build-Gui
            }
        }
    }

    Write-Host ""
    Print-Success "编译完成！"
    Write-Host ""
    Write-Host "输出文件:"
    Write-Host "  CLI 工具: dist\cli\"
    Write-Host "  GUI 应用: crates\tauri-app\src-tauri\target\release\bundle\"
    Write-Host ""
}

# 处理帮助参数
if ($Help) {
    Show-Help
    exit 0
}

# 运行主函数
Main
