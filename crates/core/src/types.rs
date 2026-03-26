use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 统一的图片处理参数中间格式
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ImageProcessParams {
    /// 缩放参数
    pub resize: Option<ResizeParams>,
    /// 裁剪参数
    pub crop: Option<CropParams>,
    /// 旋转参数
    pub rotate: Option<RotateParams>,
    /// 质量参数
    pub quality: Option<QualityParams>,
    /// 格式转换参数
    pub format: Option<FormatParams>,
    /// 元数据操作
    pub metadata: Option<Metadata>,
    /// 水印参数
    pub watermark: Option<WatermarkParams>,
    /// 模糊参数
    pub blur: Option<BlurParams>,
    /// 锐化参数
    pub sharpen: Option<SharpenParams>,
    /// 亮度/对比度参数
    pub brightness_contrast: Option<BrightnessContrastParams>,
    /// 灰度模式（华为云OBS特有）
    pub grayscale: Option<bool>,
    /// 索引裁剪参数（华为云OBS特有）
    pub index_crop: Option<IndexCropParams>,
    /// 圆角裁剪参数（华为云OBS特有）
    pub rounded_corners: Option<RoundedCornersParams>,
    /// 文件大小限制（七牛云特有，单位：字节）
    pub size_limit: Option<u32>,
    /// 忽略错误（七牛云特有，失败时返回原图）
    pub ignore_error: Option<bool>,
    /// 其他原始参数（用于保存无法映射的参数）
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 索引裁剪参数（华为云OBS特有）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexCropParams {
    /// 水平剪切的每块图片长度
    pub x_length: Option<u32>,
    /// 垂直剪切的每块图片长度
    pub y_length: Option<u32>,
    /// 选择第几块（从0开始）
    pub index: u32,
}

/// 圆角裁剪参数（华为云OBS特有）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoundedCornersParams {
    /// 圆角半径（水平和垂直相同）
    pub radius: Option<u32>,
    /// 圆角水平大小
    pub radius_x: Option<u32>,
    /// 圆角垂直大小
    pub radius_y: Option<u32>,
}

/// 缩放参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResizeParams {
    /// 目标宽度（像素）
    pub width: Option<u32>,
    /// 目标高度（像素）
    pub height: Option<u32>,
    /// 缩放模式
    #[serde(default)]
    pub mode: ResizeMode,
    /// 是否限制放大（仅缩小时处理）
    pub limit: Option<bool>,
    /// 比例缩放因子（如 0.5 表示缩小到 50%）
    pub ratio: Option<f32>,
    /// 百分比缩放 [1-1000]（华为云OBS特有）
    pub percentage: Option<u32>,
    /// 最长边（华为云OBS特有）
    pub longest_side: Option<u32>,
    /// 最短边（华为云OBS特有）
    pub shortest_side: Option<u32>,
    /// 填充颜色（华为云OBS pad模式用，格式：六位十六进制）
    pub fill_color: Option<String>,
}

/// 缩放模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResizeMode {
    /// 等比缩放，限制在指定矩形内（默认）
    #[default]
    Fit,
    /// 等比缩放，居中裁剪到指定尺寸
    Fill,
    /// 按指定坐标裁剪
    Crop,
    /// 缩放后填充背景色到指定尺寸
    Pad,
    /// 强制指定宽高（可能变形）
    Fixed,
    /// 按比例缩放（使用 ratio 字段）
    Ratio,
}

impl fmt::Display for ResizeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResizeMode::Fit => write!(f, "fit"),
            ResizeMode::Fill => write!(f, "fill"),
            ResizeMode::Crop => write!(f, "crop"),
            ResizeMode::Pad => write!(f, "pad"),
            ResizeMode::Fixed => write!(f, "fixed"),
            ResizeMode::Ratio => write!(f, "ratio"),
        }
    }
}

/// 裁剪参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CropParams {
    /// 裁剪起始 X 坐标
    pub x: u32,
    /// 裁剪起始 Y 坐标
    pub y: u32,
    /// 裁剪宽度
    pub width: u32,
    /// 裁剪高度
    pub height: u32,
    /// 是否基于图片中心点的圆形裁剪
    pub circle: Option<bool>,
    /// 九宫格位置（华为云OBS特有）
    pub grid_position: Option<GridPosition>,
}

/// 九宫格位置（华为云OBS裁剪/水印用）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GridPosition {
    /// 左上角
    TopLeft,
    /// 顶部居中
    Top,
    /// 右上角
    TopRight,
    /// 左侧居中
    Left,
    /// 正中心
    Center,
    /// 右侧居中
    Right,
    /// 左下角
    BottomLeft,
    /// 底部居中
    Bottom,
    /// 右下角
    BottomRight,
}

impl fmt::Display for GridPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridPosition::TopLeft => write!(f, "tl"),
            GridPosition::Top => write!(f, "top"),
            GridPosition::TopRight => write!(f, "tr"),
            GridPosition::Left => write!(f, "left"),
            GridPosition::Center => write!(f, "center"),
            GridPosition::Right => write!(f, "right"),
            GridPosition::BottomLeft => write!(f, "bl"),
            GridPosition::Bottom => write!(f, "bottom"),
            GridPosition::BottomRight => write!(f, "br"),
        }
    }
}

/// 旋转参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RotateParams {
    /// 旋转角度（0-360度）
    pub angle: u32,
    /// 是否自动旋转（根据 EXIF 方向）
    pub auto: Option<bool>,
    /// 镜像翻转（华为云OBS特有）
    pub flip: Option<FlipDirection>,
}

/// 镜像翻转方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlipDirection {
    /// 水平翻转
    Horizontal,
    /// 垂直翻转
    Vertical,
}

impl fmt::Display for FlipDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlipDirection::Horizontal => write!(f, "horizontal"),
            FlipDirection::Vertical => write!(f, "vertical"),
        }
    }
}

/// 质量参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityParams {
    /// 绝对质量值（1-100）
    pub value: Option<u8>,
    /// 相对质量（如原质量 * 80%）
    pub relative: Option<u8>,
    /// 是否使用绝对质量（华为云OBS: Q参数，默认使用相对质量q）
    pub is_absolute: Option<bool>,
}

/// 格式转换参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormatParams {
    /// 目标格式
    pub format: ImageFormat,
    /// 是否渐进式加载
    pub progressive: Option<bool>,
}

/// 图片格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpg,
    Jpeg,
    Png,
    Webp,
    Gif,
    Bmp,
    Tiff,
    Avif,
    Heic,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Jpg => write!(f, "jpg"),
            ImageFormat::Jpeg => write!(f, "jpeg"),
            ImageFormat::Png => write!(f, "png"),
            ImageFormat::Webp => write!(f, "webp"),
            ImageFormat::Gif => write!(f, "gif"),
            ImageFormat::Bmp => write!(f, "bmp"),
            ImageFormat::Tiff => write!(f, "tiff"),
            ImageFormat::Avif => write!(f, "avif"),
            ImageFormat::Heic => write!(f, "heic"),
        }
    }
}

/// 元数据操作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// 是否移除 EXIF
    pub remove_exif: Option<bool>,
}

/// 水印参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatermarkParams {
    /// 水印类型
    #[serde(rename = "type")]
    pub watermark_type: WatermarkType,
    /// 水印内容（文字或图片 URL）
    pub content: String,
    /// 透明度 (0-100)
    pub opacity: Option<u8>,
    /// 位置
    pub position: Option<WatermarkPosition>,
    /// X 偏移量（像素）
    pub x_offset: Option<i32>,
    /// Y 偏移量（像素）
    pub y_offset: Option<i32>,
}

/// 水印类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WatermarkType {
    /// 文字水印
    Text,
    /// 图片水印
    Image,
}

/// 水印位置
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WatermarkPosition {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

/// 模糊参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlurParams {
    /// 模糊半径（像素）
    pub radius: u32,
    /// 模糊强度 (0-100)
    pub sigma: Option<u8>,
}

/// 锐化参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharpenParams {
    /// 锐化强度 (华为云OBS: 50-399, 其他厂商: 0-100)
    pub amount: u16,
}

/// 亮度/对比度参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrightnessContrastParams {
    /// 亮度调整 (-100 到 100)
    pub brightness: i16,
    /// 对比度调整 (-100 到 100)
    pub contrast: i16,
}

/// 云厂商枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// 阿里云 OSS
    Aliyun,
    /// 腾讯云数据万象
    Tencent,
    /// 华为云 OBS
    Huawei,
    /// 七牛云
    Qiniu,
    /// 火山引擎
    Volcengine,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Aliyun => write!(f, "aliyun"),
            Provider::Tencent => write!(f, "tencent"),
            Provider::Huawei => write!(f, "huawei"),
            Provider::Qiniu => write!(f, "qiniu"),
            Provider::Volcengine => write!(f, "volcengine"),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aliyun" | "oss" => Ok(Provider::Aliyun),
            "tencent" | "ci" => Ok(Provider::Tencent),
            "huawei" | "obs" => Ok(Provider::Huawei),
            "qiniu" => Ok(Provider::Qiniu),
            "volcengine" | "volc" => Ok(Provider::Volcengine),
            _ => Err(ParseError::InvalidProvider(s.to_string())),
        }
    }
}

/// 转换模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConversionMode {
    /// 严格模式：遇到不兼容参数时报错
    Strict,
    /// 宽松模式：跳过不兼容参数，尽可能转换（默认）
    #[default]
    Lenient,
    /// 报告模式：返回转换结果和警告信息
    Report,
}

impl fmt::Display for ConversionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionMode::Strict => write!(f, "strict"),
            ConversionMode::Lenient => write!(f, "lenient"),
            ConversionMode::Report => write!(f, "report"),
        }
    }
}

impl std::str::FromStr for ConversionMode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "strict" => Ok(ConversionMode::Strict),
            "lenient" => Ok(ConversionMode::Lenient),
            "report" => Ok(ConversionMode::Report),
            _ => Err(ParseError::InvalidConversionMode(s.to_string())),
        }
    }
}

/// 转换结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConversionResult {
    /// 转换后的 URL
    pub url: String,
    /// 转换是否完全成功
    pub success: bool,
    /// 警告信息（不兼容的操作）
    pub warnings: Vec<Warning>,
    /// 被忽略的操作
    pub dropped: Vec<DroppedOperation>,
}

/// 警告信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct Warning {
    /// 操作名称
    pub operation: String,
    /// 原因
    pub reason: String,
    /// 建议
    pub suggestion: Option<String>,
}

/// 被忽略的操作
#[derive(Debug, Clone, serde::Serialize)]
pub struct DroppedOperation {
    /// 操作名称
    pub name: String,
    /// 原始值
    pub original_value: String,
    /// 原因
    pub reason: String,
}

/// 生成结果
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// 生成的参数字符串
    pub params: String,
    /// 被丢弃的操作
    pub dropped: Vec<DroppedOperation>,
}

/// 支持的操作类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    /// 缩放
    Resize,
    /// 裁剪
    Crop,
    /// 旋转
    Rotate,
    /// 质量
    Quality,
    /// 格式转换
    Format,
    /// 元数据处理
    Metadata,
    /// 渐进式加载
    Progressive,
    /// 水印
    Watermark,
    /// 模糊
    Blur,
    /// 锐化
    Sharpen,
    /// 亮度/对比度
    BrightnessContrast,
    /// 灰度
    Grayscale,
    /// 索引裁剪
    IndexCrop,
    /// 圆角裁剪
    RoundedCorners,
}

// ============== Error Types ==============

/// 解析错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Unknown provider: {0}")]
    InvalidProvider(String),

    #[error("Invalid conversion mode: {0}")]
    InvalidConversionMode(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid parameter value: {0} = {1}")]
    InvalidValue(String, String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Parse error: {0}")]
    Custom(String),
}

/// 生成错误
#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("No valid parameters to generate")]
    EmptyParams,

    #[error("Parameter conflict: {0}")]
    Conflict(String),

    #[error("Value out of range: {0} must be between {1} and {2}")]
    OutOfRange(String, u32, u32),

    #[error("Generate error: {0}")]
    Custom(String),
}

/// 转换错误
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(Provider),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Generate error: {0}")]
    GenerateError(#[from] GenerateError),

    #[error("Incompatible parameters detected in strict mode: {0}")]
    IncompatibleParameters(String),

    #[error("URL build error: {0}")]
    UrlBuildError(String),

    #[error("Conversion error: {0}")]
    Custom(String),
}

/// 验证错误
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Parameter not supported by provider: {0}")]
    UnsupportedParameter(String),

    #[error("Value out of range: {0}")]
    OutOfRange(String),

    #[error("Required parameter missing: {0}")]
    MissingRequired(String),

    #[error("Validation error: {0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_provider_from_str() {
        assert_eq!(Provider::from_str("aliyun").unwrap(), Provider::Aliyun);
        assert_eq!(Provider::from_str("oss").unwrap(), Provider::Aliyun);
        assert_eq!(Provider::from_str("tencent").unwrap(), Provider::Tencent);
        assert!(Provider::from_str("unknown").is_err());
    }

    #[test]
    fn test_resize_mode_default() {
        let mode = ResizeMode::default();
        assert_eq!(mode, ResizeMode::Fit);
    }
}
