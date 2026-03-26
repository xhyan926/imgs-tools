# 云厂商图片处理参数转换器

在不同云厂商间转换图片处理参数的工具，支持阿里云OSS、腾讯云数据万象、华为云OBS、七牛云、火山引擎五家厂商。

## 功能特性

- ✅ 支持 5 家主流云厂商的图片处理参数互相转换
- ✅ CLI 命令行工具，支持单个转换和批量转换
- ✅ GUI 桌面应用（基于 Tauri + React）
- ✅ 三种转换模式：严格模式、宽松模式、报告模式
- ✅ 自动处理不兼容参数，提供警告和建议
- ✅ JSON 和表格输出格式
- ✅ 支持丰富的图片处理操作：缩放、裁剪、旋转、质量、水印、模糊、锐化、亮度/对比度
- ✅ 模板管理和历史记录功能

## 项目结构

```
imgs-tools/
├── crates/
│   ├── core/         # 核心库
│   ├── cli/          # CLI 工具
│   └── tauri-app/    # GUI 应用（待实现）
├── tests/            # 测试文件
└── Cargo.toml        # Workspace 配置
```

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/your-repo/imgs-tools.git
cd imgs-tools

# 编译并安装 CLI 工具
cargo install --path crates/cli
```

## 使用方法

### CLI 工具 (imgconv)

#### 转换单个 URL

```bash
# 阿里云转腾讯云
imgconv convert "https://example.com/image.jpg?x-oss-process=image/resize,w_100,h_200" \
  --from aliyun --to tencent

# 指定输出格式
imgconv convert [url] --from aliyun --to huawei --output json
imgconv convert [url] --from aliyun --to huawei --output table
```

#### 转换模式选择

```bash
# 严格模式：遇到不兼容时报错
imgconv convert [url] --from aliyun --to volcengine --mode strict

# 宽松模式：忽略不兼容（默认）
imgconv convert [url] --from aliyun --to volcengine --mode lenient

# 报告模式：显示警告但继续
imgconv convert [url] --from aliyun --to volcengine --mode report
```

#### 批量转换

```bash
# 从文件读取 URL（每行一个 URL）
imgconv batch --input urls.txt --from aliyun --to qiniu

# 指定输出格式
imgconv batch --input urls.txt --from aliyun --to qiniu --output table
```

#### 验证 URL

```bash
imgconv validate "https://example.com/image.jpg?x-oss-process=image/resize,w_100" \
  --provider aliyun
```

#### 查看厂商支持的操作

```bash
imgconv features --provider volcengine
```

## 各云厂商参数格式对比

| 厂商 | URL参数前缀 | 参数分隔符 | 示例 |
|------|------------|-----------|------|
| 阿里云OSS | `x-oss-process=image/` | 逗号(,) | `resize,w_100,h_100` |
| 腾讯云CI | 无(直接参数) | 斜杠(/) | `imageMogr2/thumbnail/100x100` |
| 华为云OBS | `x-image-process=image/` | 逗号(,) | `resize,w_100,h_100` |
| 七牛云 | 无(直接参数) | 斜杠(/) | `imageView2/2/w/100/h/100` |
| 火山引擎 | `image_process=` | 逗号(,) | `resize,w_100,h_100` |

## 支持的操作

- **缩放**: 等比缩放、强制缩放、裁剪缩放、填充缩放
- **裁剪**: 指定坐标裁剪、圆形裁剪
- **旋转**: 指定角度旋转、自动旋转
- **质量**: 绝对质量、相对质量
- **格式转换**: JPG、PNG、WebP、GIF 等
- **水印**: 文字水印、图片水印
- **模糊**: 高斯模糊
- **锐化**: 图片锐化
- **亮度/对比度**: 亮度和对比度调整

## GUI 应用

### 运行 GUI 应用

```bash
cd crates/tauri-app
npm install
npm run tauri dev
```

### 构建 GUI 应用

```bash
cd crates/tauri-app
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

### GUI 功能

- 可视化转换界面
- 实时参数验证
- 功能对比表
- 一键复制转换结果

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定包的测试
cargo test --package imgs-tools-core
```

### 构建项目

```bash
# 构建所有包
cargo build --workspace

# 构建 CLI 工具
cargo build --package imgconv
```

## License

MIT
