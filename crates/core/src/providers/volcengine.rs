use crate::types::*;
use crate::providers::ProviderAdapter;
use std::collections::{HashMap, HashSet};

/// 火山引擎图片处理参数前缀
const VOLC_PROCESS_PREFIX: &str = "";

/// 火山引擎图片处理适配器
#[derive(Debug, Clone)]
pub struct VolcengineProvider;

impl VolcengineProvider {
    pub fn new() -> Self {
        Self
    }

    // ==================== 解析方法 ====================

    /// 解析火山引擎缩放参数
    /// 格式: resize,w_100,h_100,m_fit
    pub fn parse_resize(params: &str) -> Option<ResizeParams> {
        let mut width = None;
        let mut height = None;
        let mut mode = ResizeMode::Fit;
        let mut ratio = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut chars = part.chars();
            let prefix = chars.next()?;
            let value: String = chars.collect();

            // 去掉下划线前缀（格式：w_100）
            let value = value.strip_prefix('_').unwrap_or(&value);

            match prefix {
                'w' => width = value.parse().ok(),
                'h' => height = value.parse().ok(),
                'm' => {
                    mode = match value {
                        "fit" | "lfit" => ResizeMode::Fit,
                        "fill" => ResizeMode::Fill,
                        "fixed" => ResizeMode::Fixed,
                        "pad" => ResizeMode::Pad,
                        _ => ResizeMode::Fit,
                    };
                }
                'p' => {
                    // 百分比模式
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
                _ => {}
            }
        }

        Some(ResizeParams {
            width,
            height,
            mode,
            limit: None,
            ratio,
            percentage: None,
            longest_side: None,
            shortest_side: None,
            fill_color: None,
        })
    }

    /// 解析火山引擎裁剪参数
    /// 格式: crop,x_100,y_100,w_200,h_200 或 crop,x_100,y_100,w_200,h_200,g_center
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
                        "top" => Some(GridPosition::Top),
                        "ne" | "tr" => Some(GridPosition::TopRight),
                        "w" | "left" => Some(GridPosition::Left),
                        "center" => Some(GridPosition::Center),
                        "e" | "right" => Some(GridPosition::Right),
                        "sw" | "bl" => Some(GridPosition::BottomLeft),
                        "bottom" => Some(GridPosition::Bottom),
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

            if prefix == 'r' {
                let value = value.strip_prefix('_').unwrap_or(&value);
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
            grid_position: Some(GridPosition::Center),
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

            if prefix == 'r' {
                let value = value.strip_prefix('_').unwrap_or(&value);
                radius = value.parse::<u32>().ok();
            }
        }

        let r = radius?;
        Some(RoundedCornersParams {
            radius: Some(r),
            radius_x: None,
            radius_y: None,
        })
    }

    /// 解析索引切割参数
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

    /// 解析旋转参数
    /// 格式: rotate,90
    pub fn parse_rotate(params: &str) -> Option<RotateParams> {
        let angle = params.trim().parse().ok()?;
        Some(RotateParams {
            angle,
            auto: None,
            flip: None,
        })
    }

    /// 解析自适应方向参数
    /// 格式: auto-orient,1
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

    /// 解析质量参数
    /// 格式: quality,q_90
    pub fn parse_quality(params: &str) -> Option<QualityParams> {
        let params = params.trim();
        if let Some(rest) = params.strip_prefix("q_") {
            let value = rest.parse().ok()?;
            Some(QualityParams {
                value: Some(value),
                relative: None,
                is_absolute: None,
            })
        } else {
            None
        }
    }

    /// 解析格式转换参数
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
            format: ImageFormat::Jpg, // 渐进显示主要用于 JPEG
            progressive: Some(progressive),
        })
    }

    /// 解析高斯模糊参数
    /// 格式: blur,r_5,s_2
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
    pub fn parse_bright(params: &str) -> Option<BrightnessContrastParams> {
        let brightness = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness,
            contrast: 0,
        })
    }

    /// 解析对比度参数
    /// 格式: contrast,50
    pub fn parse_contrast(params: &str) -> Option<BrightnessContrastParams> {
        let contrast = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness: 0,
            contrast,
        })
    }

    /// 解析灰度图参数
    /// 格式: grayscale,1
    pub fn parse_grayscale(params: &str) -> Option<bool> {
        match params.trim() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    /// 解析去除元信息参数
    /// 格式: strip,1
    pub fn parse_strip(params: &str) -> Option<bool> {
        match params.trim() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    // ==================== 生成方法 ====================

    /// 生成缩放参数字符串
    fn generate_resize(params: &ResizeParams) -> String {
        let mut parts = Vec::new();

        match params.mode {
            ResizeMode::Ratio => {
                if let Some(ratio) = params.ratio {
                    parts.push(format!("p_{}", (ratio * 100.0) as u32));
                }
            }
            ResizeMode::Pad => {
                // 火山引擎不支持 Pad 模式，降级到 Fit
                if let Some(w) = params.width {
                    parts.push(format!("w_{}", w));
                }
                if let Some(h) = params.height {
                    parts.push(format!("h_{}", h));
                }
                parts.push("m_fit".to_string());
            }
            _ => {
                if let Some(w) = params.width {
                    parts.push(format!("w_{}", w));
                }
                if let Some(h) = params.height {
                    parts.push(format!("h_{}", h));
                }
                let mode_str = match params.mode {
                    ResizeMode::Fit => "m_fit",
                    ResizeMode::Fill => "m_fill",
                    ResizeMode::Fixed => "m_fixed",
                    ResizeMode::Crop => "m_fill",
                    ResizeMode::Pad => "m_fit",
                    ResizeMode::Ratio => "m_fit",
                };
                parts.push(mode_str.to_string());
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("resize,{}", parts.join(","))
        }
    }

    /// 生成裁剪参数字符串
    fn generate_crop(params: &CropParams) -> String {
        if params.circle == Some(true) {
            // 内切圆裁剪
            let radius = params.width.min(params.height) / 2;
            return format!("circle,r_{}", radius);
        }

        let mut parts = vec![
            format!("x_{}", params.x),
            format!("y_{}", params.y),
            format!("w_{}", params.width),
            format!("h_{}", params.height),
        ];

        if let Some(grid_pos) = params.grid_position {
            parts.push(format!("g_{}", Self::generate_gravity(grid_pos)));
        }

        format!("crop,{}", parts.join(","))
    }

    /// 生成九宫格位置字符串
    fn generate_gravity(pos: GridPosition) -> &'static str {
        match pos {
            GridPosition::TopLeft => "nw",
            GridPosition::Top => "top",
            GridPosition::TopRight => "ne",
            GridPosition::Left => "w",
            GridPosition::Center => "center",
            GridPosition::Right => "e",
            GridPosition::BottomLeft => "sw",
            GridPosition::Bottom => "bottom",
            GridPosition::BottomRight => "se",
        }
    }

    /// 生成旋转参数字符串
    fn generate_rotate(params: &RotateParams) -> String {
        format!("rotate,{}", params.angle)
    }

    /// 生成质量参数字符串
    fn generate_quality(params: &QualityParams) -> String {
        if let Some(value) = params.value {
            format!("quality,q_{}", value)
        } else {
            String::new()
        }
    }

    /// 生成格式转换参数字符串
    fn generate_format(params: &FormatParams) -> String {
        format!("format,{}", params.format)
    }

    /// 生成渐进显示参数字符串
    fn generate_interlace(params: &FormatParams) -> String {
        if params.progressive == Some(true) {
            "interlace,1".to_string()
        } else {
            String::new()
        }
    }

    /// 生成模糊参数字符串
    fn generate_blur(params: &BlurParams) -> String {
        if let Some(sigma) = params.sigma {
            format!("blur,r_{},s_{}", params.radius, sigma)
        } else {
            format!("blur,r_{}", params.radius)
        }
    }

    /// 生成锐化参数字符串
    fn generate_sharpen(params: &SharpenParams) -> String {
        format!("sharpen,{}", params.amount)
    }

    /// 生成亮度参数字符串
    fn generate_bright(params: &BrightnessContrastParams) -> String {
        format!("bright,{}", params.brightness)
    }

    /// 生成对比度参数字符串
    fn generate_contrast(params: &BrightnessContrastParams) -> String {
        format!("contrast,{}", params.contrast)
    }

    /// 生成灰度图参数字符串
    fn generate_grayscale(value: bool) -> String {
        format!("grayscale,{}", if value { 1 } else { 0 })
    }

    /// 生成圆角参数字符串
    fn generate_rounded_corners(params: &RoundedCornersParams) -> String {
        if let Some(radius) = params.radius {
            format!("rounded-corners,r_{}", radius)
        } else if let Some(radius_x) = params.radius_x {
            let radius_y = params.radius_y.unwrap_or(radius_x);
            format!("rounded-corners,r_{}x{}", radius_x, radius_y)
        } else {
            String::new()
        }
    }

    /// 生成索引切割参数字符串
    fn generate_index_crop(params: &IndexCropParams) -> String {
        if let Some(x_length) = params.x_length {
            format!("indexcrop,x_{},i_{}", x_length, params.index)
        } else if let Some(y_length) = params.y_length {
            format!("indexcrop,y_{},i_{}", y_length, params.index)
        } else {
            String::new()
        }
    }

    /// 生成自适应方向参数字符串
    fn generate_auto_orient(params: &RotateParams) -> String {
        if params.auto == Some(true) {
            "auto-orient,1".to_string()
        } else {
            String::new()
        }
    }

    /// 生成去除元信息参数字符串
    fn generate_strip(value: bool) -> String {
        if value {
            "strip,1".to_string()
        } else {
            String::new()
        }
    }
}

impl Default for VolcengineProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for VolcengineProvider {
    fn parse(&self, url: &str) -> Result<ImageProcessParams, ParseError> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| ParseError::InvalidUrl(format!("{}: {}", url, e)))?;

        let query_pairs: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

        // 查找 image_process 参数
        let process_param = query_pairs
            .get("image_process")
            .ok_or_else(|| ParseError::MissingParameter("image_process".to_string()))?;

        // 检查前缀
        let process_value = process_param.strip_prefix(VOLC_PROCESS_PREFIX).ok_or_else(|| {
            ParseError::InvalidValue("image_process".to_string(), process_param.clone())
        })?;

        let mut result = ImageProcessParams::default();

        // 解析各个操作（以 / 分隔）
        for operation in process_value.split('/') {
            let operation = operation.trim();
            if operation.is_empty() {
                continue;
            }

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
                "rounded-corners" => {
                    if let Some(rounded) = Self::parse_rounded_corners(op_params) {
                        result.rounded_corners = Some(rounded);
                    }
                }
                "indexcrop" => {
                    if let Some(ic) = Self::parse_index_crop(op_params) {
                        result.index_crop = Some(ic);
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
                        // 合并到已有的 format 或创建新的
                        if let Some(existing) = result.format.take() {
                            result.format = Some(FormatParams {
                                format: existing.format,
                                progressive: format.progressive,
                            });
                        } else {
                            result.format = Some(format);
                        }
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
                    if let Some(bright) = Self::parse_bright(op_params) {
                        let existing = result.brightness_contrast.take();
                        result.brightness_contrast = Some(BrightnessContrastParams {
                            brightness: bright.brightness,
                            contrast: existing.map(|e| e.contrast).unwrap_or(0),
                        });
                    }
                }
                "contrast" => {
                    if let Some(contrast) = Self::parse_contrast(op_params) {
                        let existing = result.brightness_contrast.take();
                        result.brightness_contrast = Some(BrightnessContrastParams {
                            brightness: existing.map(|e| e.brightness).unwrap_or(0),
                            contrast: contrast.contrast,
                        });
                    }
                }
                "grayscale" => {
                    if let Some(gray) = Self::parse_grayscale(op_params) {
                        result.grayscale = Some(gray);
                    }
                }
                "strip" => {
                    if let Some(strip) = Self::parse_strip(op_params) {
                        result.metadata = Some(Metadata {
                            remove_exif: Some(strip),
                        });
                    }
                }
                _ => {
                    result.extra.insert(op_name.to_string(), operation.to_string().into());
                }
            }
        }

        Ok(result)
    }

    fn generate(&self, params: &ImageProcessParams) -> Result<GenerateResult, GenerateError> {
        let mut operations = Vec::new();
        let mut dropped = Vec::new();

        if let Some(ref resize) = params.resize {
            let resize_str = Self::generate_resize(resize);
            if !resize_str.is_empty() {
                operations.push(resize_str);
            }

            // 检查是否是 Pad 模式，添加到 dropped
            if resize.mode == ResizeMode::Pad {
                dropped.push(DroppedOperation {
                    name: "resize_mode_pad".to_string(),
                    original_value: "pad".to_string(),
                    reason: "火山引擎不支持 Pad 模式，已降级到 Fit".to_string(),
                });
            }
        }

        if let Some(ref crop) = params.crop {
            operations.push(Self::generate_crop(crop));
        }

        if let Some(ref rounded) = params.rounded_corners {
            let rounded_str = Self::generate_rounded_corners(rounded);
            if !rounded_str.is_empty() {
                operations.push(rounded_str);
            }
        }

        if let Some(ref ic) = params.index_crop {
            let ic_str = Self::generate_index_crop(ic);
            if !ic_str.is_empty() {
                operations.push(ic_str);
            }
        }

        if let Some(ref rotate) = params.rotate {
            // 优先使用 auto-orient，否则使用 rotate
            if rotate.auto == Some(true) {
                operations.push(Self::generate_auto_orient(rotate));
            } else {
                operations.push(Self::generate_rotate(rotate));
            }
        }

        if let Some(ref quality) = params.quality {
            let quality_str = Self::generate_quality(quality);
            if !quality_str.is_empty() {
                operations.push(quality_str);
            }

            // 火山引擎不支持相对质量
            if quality.relative.is_some() {
                dropped.push(DroppedOperation {
                    name: "quality_relative".to_string(),
                    original_value: format!("Q{}", quality.relative.unwrap()),
                    reason: "火山引擎不支持相对质量，已忽略".to_string(),
                });
            }
        }

        if let Some(ref format) = params.format {
            operations.push(Self::generate_format(format));
            if format.progressive == Some(true) {
                operations.push(Self::generate_interlace(format));
            }
        } else if params.format.is_none() {
            // 检查是否有独立的 interlace 操作
            // 这里需要更复杂的逻辑来处理，暂时跳过
        }

        if let Some(ref blur) = params.blur {
            operations.push(Self::generate_blur(blur));
        }

        if let Some(ref sharpen) = params.sharpen {
            operations.push(Self::generate_sharpen(sharpen));
        }

        if let Some(ref bc) = params.brightness_contrast {
            if bc.brightness != 0 {
                operations.push(Self::generate_bright(bc));
            }
            if bc.contrast != 0 {
                operations.push(Self::generate_contrast(bc));
            }
        }

        if let Some(grayscale) = params.grayscale {
            operations.push(Self::generate_grayscale(grayscale));
        }

        if let Some(ref metadata) = params.metadata {
            if let Some(true) = metadata.remove_exif {
                operations.push(Self::generate_strip(true));
            }
        }

        if operations.is_empty() {
            return Err(GenerateError::EmptyParams);
        }

        let params_str = format!("{}{}", VOLC_PROCESS_PREFIX, operations.join("/"));

        Ok(GenerateResult {
            params: params_str,
            dropped,
        })
    }

    fn validate(&self, params: &ImageProcessParams) -> Result<(), ValidationError> {
        if let Some(ref quality) = params.quality {
            if let Some(value) = quality.value {
                if value > 100 || value < 1 {
                    return Err(ValidationError::OutOfRange("quality value must be 1-100".to_string()));
                }
            }
        }

        if let Some(ref blur) = params.blur {
            if blur.radius > 50 || blur.sigma.unwrap_or(0) > 50 {
                return Err(ValidationError::OutOfRange("blur radius and sigma must be <= 50".to_string()));
            }
        }

        if let Some(ref sharpen) = params.sharpen {
            if sharpen.amount > 100 {
                return Err(ValidationError::OutOfRange("sharpen amount must be <= 100".to_string()));
            }
        }

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
        ops.insert(Operation::Metadata);
        ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resize_basic() {
        let result = VolcengineProvider::parse_resize("w_100,h_200");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.width, Some(100));
        assert_eq!(resize.height, Some(200));
        assert_eq!(resize.mode, ResizeMode::Fit);
    }

    #[test]
    fn test_parse_resize_with_mode() {
        let result = VolcengineProvider::parse_resize("w_100,h_200,m_fill");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.mode, ResizeMode::Fill);
    }

    #[test]
    fn test_parse_crop_with_gravity() {
        let result = VolcengineProvider::parse_crop("x_10,y_10,w_200,h_200,g_center");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.grid_position, Some(GridPosition::Center));
    }

    #[test]
    fn test_parse_circle() {
        let result = VolcengineProvider::parse_circle("r_100");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.circle, Some(true));
        assert_eq!(crop.width, 200);
        assert_eq!(crop.height, 200);
    }

    #[test]
    fn test_parse_index_crop() {
        let result = VolcengineProvider::parse_index_crop("x_1000,i_0");
        assert!(result.is_some());
        let ic = result.unwrap();
        assert_eq!(ic.x_length, Some(1000));
        assert_eq!(ic.index, 0);
    }

    #[test]
    fn test_parse_blur() {
        let result = VolcengineProvider::parse_blur("r_3,s_2");
        assert!(result.is_some());
        let blur = result.unwrap();
        assert_eq!(blur.radius, 3);
        assert_eq!(blur.sigma, Some(2));
    }

    #[test]
    fn test_parse_sharpen() {
        let result = VolcengineProvider::parse_sharpen("100");
        assert!(result.is_some());
        let sharpen = result.unwrap();
        assert_eq!(sharpen.amount, 100);
    }

    #[test]
    fn test_full_parse_with_blur() {
        let provider = VolcengineProvider::new();
        let url = "https://example.com/image.jpg?image_process=resize,w_100/blur,r_3,s_2";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.resize.is_some());
        assert!(params.blur.is_some());
    }

    #[test]
    fn test_pad_mode_downgrade() {
        let provider = VolcengineProvider::new();
        let params = ImageProcessParams {
            resize: Some(ResizeParams {
                width: Some(100),
                height: Some(100),
                mode: ResizeMode::Pad,
                limit: None,
                ratio: None,
                percentage: None,
                longest_side: None,
                shortest_side: None,
                fill_color: None,
            }),
            ..Default::default()
        };

        let result = provider.generate(&params);
        assert!(result.is_ok());
        let gen_result = result.unwrap();
        // 应该有 dropped 操作
        assert!(!gen_result.dropped.is_empty());
        assert!(gen_result.dropped[0].name.contains("pad"));
    }
}
