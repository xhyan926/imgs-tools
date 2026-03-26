use crate::types::*;
use crate::providers::ProviderAdapter;
use std::collections::{HashMap, HashSet};

/// 阿里云 OSS 图片处理参数前缀
const OSS_PROCESS_PREFIX: &str = "image/";

/// 阿里云 OSS 图片处理适配器
#[derive(Debug, Clone)]
pub struct AliyunProvider;

impl AliyunProvider {
    pub fn new() -> Self {
        Self
    }

    // ==================== 解析方法 ====================

    /// 解析 OSS 缩放参数
    /// 格式: resize,w_100,h_100,m_lfit,l_100,s_100,p_50,color_FF0000,limit_0
    pub fn parse_resize(params: &str) -> Option<ResizeParams> {
        let mut width = None;
        let mut height = None;
        let mut mode = ResizeMode::Fit;
        let mut limit = None;
        let mut percentage = None;
        let mut longest_side = None;
        let mut shortest_side = None;
        let mut fill_color = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀（阿里云格式：w_100）
            let value = value.strip_prefix('_').unwrap_or(&value);

            match prefix {
                'w' => width = value.parse().ok(),
                'h' => height = value.parse().ok(),
                'm' => {
                    mode = match value {
                        "lfit" => ResizeMode::Fit,
                        "mfit" => ResizeMode::Fill,
                        "fill" => ResizeMode::Fill,
                        "pad" => ResizeMode::Pad,
                        "fixed" => ResizeMode::Fixed,
                        _ => return None,
                    };
                }
                'l' => {
                    if value == "0" {
                        // limit_0 表示不限制放大
                        limit = Some(false);
                    } else {
                        // l_100 表示最长边
                        longest_side = value.parse().ok();
                    }
                }
                's' => shortest_side = value.parse().ok(),
                'p' => {
                    // 百分比模式，转为比例
                    if let Ok(p) = value.parse::<f32>() {
                        return Some(ResizeParams {
                            width: None,
                            height: None,
                            mode: ResizeMode::Ratio,
                            limit: None,
                            ratio: Some(p / 100.0),
                            percentage: None,
                            longest_side: None,
                            shortest_side: None,
                            fill_color: None,
                        });
                    }
                }
                'c' => {
                    if value.starts_with("olor_") {
                        fill_color = Some(value[5..].to_string());
                    }
                }
                _ => {}
            }
        }

        Some(ResizeParams {
            width,
            height,
            mode,
            limit,
            ratio: None,
            percentage,
            longest_side,
            shortest_side,
            fill_color,
        })
    }

    /// 解析 OSS 裁剪参数
    /// 格式: crop,x_100,y_100,w_200,h_200,g_nw
    pub fn parse_crop(params: &str) -> Option<CropParams> {
        let mut x = None;
        let mut y = None;
        let mut width = None;
        let mut height = None;
        let mut grid_position = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀
            let value = value.strip_prefix('_').unwrap_or(&value);

            match prefix {
                'x' => x = value.parse().ok(),
                'y' => y = value.parse().ok(),
                'w' => width = value.parse().ok(),
                'h' => height = value.parse().ok(),
                'g' => {
                    grid_position = match value {
                        "nw" | "tl" => Some(GridPosition::TopLeft),
                        "north" | "top" => Some(GridPosition::Top),
                        "ne" | "tr" => Some(GridPosition::TopRight),
                        "west" | "left" => Some(GridPosition::Left),
                        "center" => Some(GridPosition::Center),
                        "east" | "right" => Some(GridPosition::Right),
                        "sw" | "bl" => Some(GridPosition::BottomLeft),
                        "south" | "bottom" => Some(GridPosition::Bottom),
                        "se" | "br" => Some(GridPosition::BottomRight),
                        _ => None,
                    };
                }
                _ => {}
            }
        }

        if let (Some(x), Some(y), Some(width), Some(height)) = (x, y, width, height) {
            Some(CropParams {
                x,
                y,
                width,
                height,
                circle: None,
                grid_position,
            })
        } else {
            None
        }
    }

    /// 解析内切圆参数
    /// 格式: circle,r_100
    pub fn parse_circle(params: &str) -> Option<CropParams> {
        let mut radius = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀
            let value = value.strip_prefix('_').unwrap_or(&value);

            if prefix == 'r' {
                radius = value.parse::<u32>().ok();
            }
        }

        let r = radius?;
        Some(CropParams {
            x: 0,
            y: 0,
            width: r * 2,
            height: r * 2,
            circle: Some(true),
            grid_position: None,
        })
    }

    /// 解析索引裁剪参数
    /// 格式: indexcrop,x_1000,i_0 或 indexcrop,y_1000,i_0
    pub fn parse_index_crop(params: &str) -> Option<IndexCropParams> {
        let mut x_length = None;
        let mut y_length = None;
        let mut index = 0;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀
            let value = value.strip_prefix('_').unwrap_or(&value);

            match prefix {
                'x' => x_length = value.parse().ok(),
                'y' => y_length = value.parse().ok(),
                'i' => index = value.parse().unwrap_or(0),
                _ => {}
            }
        }

        if x_length.is_none() && y_length.is_none() {
            return None;
        }

        Some(IndexCropParams {
            x_length,
            y_length,
            index,
        })
    }

    /// 解析圆角矩形参数
    /// 格式: rounded-corners,r_10
    pub fn parse_rounded_corners(params: &str) -> Option<RoundedCornersParams> {
        let mut radius = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀
            let value = value.strip_prefix('_').unwrap_or(&value);

            if prefix == 'r' {
                radius = value.parse().ok();
            }
        }

        let r = radius?;
        Some(RoundedCornersParams {
            radius: Some(r),
            radius_x: None,
            radius_y: None,
        })
    }

    /// 解析 OSS 旋转参数
    /// 格式: rotate,90
    pub fn parse_rotate(params: &str) -> Option<RotateParams> {
        let angle = params.trim().parse().ok()?;
        Some(RotateParams {
            angle,
            auto: None,
            flip: None,
        })
    }

    /// 解析自动方向参数
    /// 格式: auto-orient,0
    pub fn parse_auto_orient(params: &str) -> Option<RotateParams> {
        let value = params.trim();
        let auto = match value {
            "0" => false,
            "1" => true,
            _ => return None,
        };
        Some(RotateParams {
            angle: 0,
            auto: Some(auto),
            flip: None,
        })
    }

    /// 解析 OSS 质量参数
    /// 格式: quality,q_90
    /// 或: quality,Q_90 (相对质量)
    pub fn parse_quality(params: &str) -> Option<QualityParams> {
        let params = params.trim();
        if let Some(rest) = params.strip_prefix("q_") {
            let value = rest.parse().ok()?;
            Some(QualityParams {
                value: Some(value),
                relative: None,
                is_absolute: None,
            })
        } else if let Some(rest) = params.strip_prefix("Q_") {
            let relative = rest.parse().ok()?;
            Some(QualityParams {
                value: None,
                relative: Some(relative),
                is_absolute: None,
            })
        } else {
            None
        }
    }

    /// 解析 OSS 格式转换参数
    /// 格式: format,png
    pub fn parse_format(params: &str) -> Option<FormatParams> {
        let format_str = params.trim().to_lowercase();
        let format = match format_str.as_str() {
            "jpg" | "jpeg" => ImageFormat::Jpg,
            "png" => ImageFormat::Png,
            "webp" => ImageFormat::Webp,
            "gif" => ImageFormat::Gif,
            "bmp" => ImageFormat::Bmp,
            "tiff" => ImageFormat::Tiff,
            _ => return None,
        };
        Some(FormatParams {
            format,
            progressive: None,
        })
    }

    /// 解析渐进显示参数
    /// 格式: interlace,1
    pub fn parse_interlace(params: &str) -> Option<FormatParams> {
        let progressive = match params.trim() {
            "0" => false,
            "1" => true,
            _ => return None,
        };
        Some(FormatParams {
            format: ImageFormat::Jpg,
            progressive: Some(progressive),
        })
    }

    /// 解析模糊参数
    /// 格式: blur,r_3,s_2
    pub fn parse_blur(params: &str) -> Option<BlurParams> {
        let mut radius = None;
        let mut sigma = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀
            let value = value.strip_prefix('_').unwrap_or(&value);

            match prefix {
                'r' => radius = value.parse().ok(),
                's' => sigma = value.parse().ok(),
                _ => {}
            }
        }

        let r = radius?;
        Some(BlurParams {
            radius: r,
            sigma,
        })
    }

    /// 解析锐化参数
    /// 格式: sharpen,100
    pub fn parse_sharpen(params: &str) -> Option<SharpenParams> {
        let amount = params.trim().parse().ok()?;
        Some(SharpenParams { amount })
    }

    /// 解析亮度参数
    /// 格式: bright,50
    pub fn parse_brightness(params: &str) -> Option<BrightnessContrastParams> {
        let brightness = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness,
            contrast: 0,
        })
    }

    /// 解析对比度参数
    /// 格式: contrast,-50
    pub fn parse_contrast(params: &str) -> Option<BrightnessContrastParams> {
        let contrast = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness: 0,
            contrast,
        })
    }

    // ==================== 生成方法 ====================

    /// 生成 OSS 缩放参数字符串
    pub fn generate_resize(params: &ResizeParams) -> String {
        let mut parts = Vec::new();

        if let Some(l) = params.longest_side {
            parts.push(format!("l_{}", l));
        } else if let Some(s) = params.shortest_side {
            parts.push(format!("s_{}", s));
        }

        match params.mode {
            ResizeMode::Ratio => {
                if let Some(ratio) = params.ratio {
                    parts.push(format!("p_{}", (ratio * 100.0) as u32));
                }
                if let Some(p) = params.percentage {
                    parts.push(format!("p_{}", p));
                }
            }
            _ => {
                if let Some(w) = params.width {
                    parts.push(format!("w_{}", w));
                }
                if let Some(h) = params.height {
                    parts.push(format!("h_{}", h));
                }
                let mode_str = match params.mode {
                    ResizeMode::Fit => "m_lfit",
                    ResizeMode::Fill => "m_fill",
                    ResizeMode::Pad => "m_pad",
                    ResizeMode::Fixed => "m_fixed",
                    ResizeMode::Crop | ResizeMode::Ratio => "m_lfit",
                };
                parts.push(mode_str.to_string());
                if let Some(false) = params.limit {
                    parts.push("l_0".to_string());
                }
                if let Some(ref color) = params.fill_color {
                    parts.push(format!("color_{}", color));
                }
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("resize,{}", parts.join(","))
        }
    }

    /// 生成 OSS 裁剪参数字符串
    pub fn generate_crop(params: &CropParams) -> String {
        if params.circle == Some(true) {
            // 内切圆
            let radius = params.width.min(params.height) / 2;
            return format!("circle,r_{}", radius);
        }

        let mut parts = vec![
            format!("x_{}", params.x),
            format!("y_{}", params.y),
            format!("w_{}", params.width),
            format!("h_{}", params.height),
        ];

        if let Some(pos) = params.grid_position {
            let g_str = match pos {
                GridPosition::TopLeft => "nw",
                GridPosition::Top => "north",
                GridPosition::TopRight => "ne",
                GridPosition::Left => "west",
                GridPosition::Center => "center",
                GridPosition::Right => "east",
                GridPosition::BottomLeft => "sw",
                GridPosition::Bottom => "south",
                GridPosition::BottomRight => "se",
            };
            parts.push(format!("g_{}", g_str));
        }

        format!("crop,{}", parts.join(","))
    }

    /// 生成索引裁剪参数字符串
    pub fn generate_index_crop(index_crop: &IndexCropParams) -> String {
        if let Some(x) = index_crop.x_length {
            format!("indexcrop,x_{},i_{}", x, index_crop.index)
        } else if let Some(y) = index_crop.y_length {
            format!("indexcrop,y_{},i_{}", y, index_crop.index)
        } else {
            String::new()
        }
    }

    /// 生成圆角矩形参数字符串
    pub fn generate_rounded_corners(rounded: &RoundedCornersParams) -> String {
        if let Some(r) = rounded.radius {
            format!("rounded-corners,r_{}", r)
        } else {
            String::new()
        }
    }

    /// 生成 OSS 旋转参数字符串
    pub fn generate_rotate(params: &RotateParams) -> String {
        if let Some(flip) = params.flip {
            format!("flip,{}", match flip {
                FlipDirection::Horizontal => "h",
                FlipDirection::Vertical => "v",
            })
        } else if params.angle > 0 {
            format!("rotate,{}", params.angle)
        } else if let Some(true) = params.auto {
            "auto-orient,1".to_string()
        } else {
            String::new()
        }
    }

    /// 生成 OSS 质量参数字符串
    pub fn generate_quality(params: &QualityParams) -> String {
        if let Some(value) = params.value {
            format!("quality,q_{}", value)
        } else if let Some(relative) = params.relative {
            format!("quality,Q_{}", relative)
        } else {
            String::new()
        }
    }

    /// 生成 OSS 格式转换参数字符串
    pub fn generate_format(params: &FormatParams) -> String {
        let mut result = format!("format,{}", params.format);

        if let Some(true) = params.progressive {
            result = format!("{}/interlace,1", result);
        }

        result
    }

    /// 生成模糊参数字符串
    pub fn generate_blur(blur: &BlurParams) -> String {
        let sigma = blur.sigma.unwrap_or(50);
        format!("blur,r_{},s_{}", blur.radius, sigma)
    }

    /// 生成锐化参数字符串
    pub fn generate_sharpen(sharpen: &SharpenParams) -> String {
        format!("sharpen,{}", sharpen.amount)
    }

    /// 生成亮度参数字符串
    pub fn generate_brightness(brightness: i16) -> String {
        format!("bright,{}", brightness)
    }

    /// 生成对比度参数字符串
    pub fn generate_contrast(contrast: i16) -> String {
        format!("contrast,{}", contrast)
    }
}

impl Default for AliyunProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for AliyunProvider {
    fn parse(&self, url: &str) -> Result<ImageProcessParams, ParseError> {
        // 从 URL 中提取图片处理参数
        let parsed_url = url::Url::parse(url)
            .map_err(|e| ParseError::InvalidUrl(format!("{}: {}", url, e)))?;

        let query_pairs: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

        // 查找 x-oss-process 参数
        let process_param = query_pairs
            .get("x-oss-process")
            .ok_or_else(|| {
                ParseError::MissingParameter("x-oss-process".to_string())
            })?;

        // 检查前缀
        let process_value = process_param
            .strip_prefix(OSS_PROCESS_PREFIX)
            .ok_or_else(|| {
                ParseError::InvalidValue(
                    "x-oss-process".to_string(),
                    process_param.clone(),
                )
            })?;

        let mut result = ImageProcessParams::default();

        // 解析各个操作（以 / 分隔）
        for operation in process_value.split('/') {
            let operation = operation.trim();
            if operation.is_empty() {
                continue;
            }

            // 分割操作名和参数
            let mut parts = operation.splitn(2, ',');
            let op_name = parts.next().unwrap_or("");
            let op_params = parts.next().unwrap_or("");

            match op_name {
                "resize" => {
                    if let Some(resize) = Self::parse_resize(op_params) {
                        result.resize = Some(resize);
                    }
                }
                "crop" => {
                    if let Some(crop) = Self::parse_crop(op_params) {
                        result.crop = Some(crop);
                    }
                }
                "circle" => {
                    if let Some(crop) = Self::parse_circle(op_params) {
                        result.crop = Some(crop);
                    }
                }
                "indexcrop" => {
                    if let Some(index_crop) = Self::parse_index_crop(op_params) {
                        result.index_crop = Some(index_crop);
                    }
                }
                "rounded-corners" => {
                    if let Some(rounded) = Self::parse_rounded_corners(op_params) {
                        result.rounded_corners = Some(rounded);
                    }
                }
                "rotate" => {
                    if let Some(rotate) = Self::parse_rotate(op_params) {
                        result.rotate = Some(rotate);
                    }
                }
                "auto-orient" => {
                    if let Some(rotate) = Self::parse_auto_orient(op_params) {
                        result.rotate = Some(rotate);
                    }
                }
                "quality" => {
                    if let Some(quality) = Self::parse_quality(op_params) {
                        result.quality = Some(quality);
                    }
                }
                "format" => {
                    if let Some(format) = Self::parse_format(op_params) {
                        result.format = Some(format);
                    }
                }
                "interlace" => {
                    if let Some(format) = Self::parse_interlace(op_params) {
                        result.format = Some(format);
                    }
                }
                "blur" => {
                    if let Some(blur) = Self::parse_blur(op_params) {
                        result.blur = Some(blur);
                    }
                }
                "sharpen" => {
                    if let Some(sharpen) = Self::parse_sharpen(op_params) {
                        result.sharpen = Some(sharpen);
                    }
                }
                "bright" => {
                    if let Some(bc) = Self::parse_brightness(op_params) {
                        result.brightness_contrast = Some(bc);
                    }
                }
                "contrast" => {
                    if let Some(bc) = Self::parse_contrast(op_params) {
                        // 如果已经有brightness_contrast，合并
                        let existing = result.brightness_contrast.take().unwrap_or_else(|| {
                            BrightnessContrastParams {
                                brightness: 0,
                                contrast: 0,
                            }
                        });
                        result.brightness_contrast = Some(BrightnessContrastParams {
                            brightness: existing.brightness,
                            contrast: bc.contrast,
                        });
                    }
                }
                _ => {
                    // 保存未知操作到 extra
                    result.extra.insert(op_name.to_string(), operation.to_string().into());
                }
            }
        }

        Ok(result)
    }

    fn generate(&self, params: &ImageProcessParams) -> Result<GenerateResult, GenerateError> {
        let mut operations = Vec::new();
        let mut dropped = Vec::new();

        // 缩放
        if let Some(ref resize) = params.resize {
            let resize_str = Self::generate_resize(resize);
            if !resize_str.is_empty() {
                operations.push(resize_str);
            }
        }

        // 裁剪
        if let Some(ref crop) = params.crop {
            operations.push(Self::generate_crop(crop));
        }

        // 索引裁剪
        if let Some(ref index_crop) = params.index_crop {
            let ic_str = Self::generate_index_crop(index_crop);
            if !ic_str.is_empty() {
                operations.push(ic_str);
            }
        }

        // 圆角矩形
        if let Some(ref rounded) = params.rounded_corners {
            let rc_str = Self::generate_rounded_corners(rounded);
            if !rc_str.is_empty() {
                operations.push(rc_str);
            }
        }

        // 旋转
        if let Some(ref rotate) = params.rotate {
            let rotate_str = Self::generate_rotate(rotate);
            if !rotate_str.is_empty() {
                operations.push(rotate_str);
            }
        }

        // 质量
        if let Some(ref quality) = params.quality {
            let quality_str = Self::generate_quality(quality);
            if !quality_str.is_empty() {
                operations.push(quality_str);
            }
        }

        // 格式
        if let Some(ref format) = params.format {
            operations.push(Self::generate_format(format));
        }

        // 模糊
        if let Some(ref blur) = params.blur {
            operations.push(Self::generate_blur(blur));
        }

        // 锐化
        if let Some(ref sharpen) = params.sharpen {
            operations.push(Self::generate_sharpen(sharpen));
        }

        // 亮度/对比度
        if let Some(ref bc) = params.brightness_contrast {
            if bc.brightness != 0 {
                operations.push(Self::generate_brightness(bc.brightness));
            }
            if bc.contrast != 0 {
                operations.push(Self::generate_contrast(bc.contrast));
            }
        }

        if operations.is_empty() {
            return Err(GenerateError::EmptyParams);
        }

        let params_str = format!("{}{}", OSS_PROCESS_PREFIX, operations.join("/"));

        Ok(GenerateResult {
            params: params_str,
            dropped,
        })
    }

    fn validate(&self, params: &ImageProcessParams) -> Result<(), ValidationError> {
        // 验证质量参数范围
        if let Some(ref quality) = params.quality {
            let value = if let Some(v) = quality.value {
                Some(v)
            } else if let Some(r) = quality.relative {
                Some(r)
            } else {
                None
            };

            if let Some(v) = value {
                if v > 100 || v < 1 {
                    return Err(ValidationError::OutOfRange("quality value must be 1-100".to_string()));
                }
            }
        }

        // 验证缩放参数
        if let Some(ref resize) = params.resize {
            if resize.mode == ResizeMode::Ratio && resize.ratio.is_none() && resize.percentage.is_none() {
                return Err(ValidationError::MissingRequired("ratio is required for Ratio mode".to_string()));
            }
        }

        // 验证模糊参数
        if let Some(ref blur) = params.blur {
            if blur.radius < 1 || blur.radius > 50 {
                return Err(ValidationError::OutOfRange("blur radius must be 1-50".to_string()));
            }
            if let Some(sigma) = blur.sigma {
                if sigma < 1 || sigma > 50 {
                    return Err(ValidationError::OutOfRange("blur sigma must be 1-50".to_string()));
                }
            }
        }

        // 验证锐化参数
        if let Some(ref sharpen) = params.sharpen {
            if sharpen.amount < 1 || sharpen.amount > 399 {
                return Err(ValidationError::OutOfRange("sharpen amount must be 1-399".to_string()));
            }
        }

        // 验证亮度对比度参数
        if let Some(ref bc) = params.brightness_contrast {
            if bc.brightness < -100 || bc.brightness > 100 {
                return Err(ValidationError::OutOfRange("brightness must be -100 to 100".to_string()));
            }
            if bc.contrast < -100 || bc.contrast > 100 {
                return Err(ValidationError::OutOfRange("contrast must be -100 to 100".to_string()));
            }
        }

        Ok(())
    }

    fn supported_operations(&self) -> HashSet<Operation> {
        let mut ops = HashSet::new();
        ops.insert(Operation::Resize);
        ops.insert(Operation::Crop);
        ops.insert(Operation::Rotate);
        ops.insert(Operation::Quality);
        ops.insert(Operation::Format);
        ops.insert(Operation::Progressive);
        ops.insert(Operation::Blur);
        ops.insert(Operation::Sharpen);
        ops.insert(Operation::BrightnessContrast);
        ops.insert(Operation::Grayscale);
        ops.insert(Operation::RoundedCorners);
        ops.insert(Operation::IndexCrop);
        ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resize_with_longest_side() {
        let result = AliyunProvider::parse_resize("l_100");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.longest_side, Some(100));
    }

    #[test]
    fn test_parse_resize_with_shortest_side() {
        let result = AliyunProvider::parse_resize("s_100");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.shortest_side, Some(100));
    }

    #[test]
    fn test_parse_resize_with_color() {
        let result = AliyunProvider::parse_resize("w_100,h_100,m_pad,color_FF0000");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.fill_color, Some("FF0000".to_string()));
    }

    #[test]
    fn test_parse_crop_with_grid_position() {
        let result = AliyunProvider::parse_crop("x_10,y_10,w_200,h_200,g_se");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.grid_position, Some(GridPosition::BottomRight));
    }

    #[test]
    fn test_parse_circle() {
        let result = AliyunProvider::parse_circle("r_100");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.circle, Some(true));
        assert_eq!(crop.width, 200);
        assert_eq!(crop.height, 200);
    }

    #[test]
    fn test_parse_index_crop() {
        let result = AliyunProvider::parse_index_crop("x_1000,i_0");
        assert!(result.is_some());
        let ic = result.unwrap();
        assert_eq!(ic.x_length, Some(1000));
        assert_eq!(ic.index, 0);
    }

    #[test]
    fn test_parse_rounded_corners() {
        let result = AliyunProvider::parse_rounded_corners("r_10");
        assert!(result.is_some());
        let rounded = result.unwrap();
        assert_eq!(rounded.radius, Some(10));
    }

    #[test]
    fn test_parse_blur() {
        let result = AliyunProvider::parse_blur("r_3,s_2");
        assert!(result.is_some());
        let blur = result.unwrap();
        assert_eq!(blur.radius, 3);
        assert_eq!(blur.sigma, Some(2));
    }

    #[test]
    fn test_parse_sharpen() {
        let result = AliyunProvider::parse_sharpen("100");
        assert!(result.is_some());
        let sharpen = result.unwrap();
        assert_eq!(sharpen.amount, 100);
    }

    #[test]
    fn test_generate_resize_with_color() {
        let params = ResizeParams {
            width: Some(100),
            height: Some(100),
            mode: ResizeMode::Pad,
            limit: None,
            ratio: None,
            percentage: None,
            longest_side: None,
            shortest_side: None,
            fill_color: Some("FF0000".to_string()),
        };
        let result = AliyunProvider::generate_resize(&params);
        assert!(result.contains("color_FF0000"));
    }

    #[test]
    fn test_generate_crop_with_grid() {
        let params = CropParams {
            x: 10,
            y: 10,
            width: 200,
            height: 200,
            circle: None,
            grid_position: Some(GridPosition::TopRight),
        };
        let result = AliyunProvider::generate_crop(&params);
        assert!(result.contains("g_ne"));
    }

    #[test]
    fn test_generate_circle() {
        let params = CropParams {
            x: 0,
            y: 0,
            width: 200,
            height: 200,
            circle: Some(true),
            grid_position: None,
        };
        let result = AliyunProvider::generate_crop(&params);
        assert_eq!(result, "circle,r_100");
    }

    #[test]
    fn test_full_parse_with_blur() {
        let provider = AliyunProvider::new();
        let url = "https://example.com/image.jpg?x-oss-process=image/resize,w_100/blur,r_3,s_2";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.resize.is_some());
        assert!(params.blur.is_some());
    }

    #[test]
    fn test_validate_blur_range() {
        let provider = AliyunProvider::new();
        let params = ImageProcessParams {
            blur: Some(BlurParams {
                radius: 100, // 超出范围
                sigma: None,
            }),
            ..Default::default()
        };
        assert!(provider.validate(&params).is_err());
    }
}
