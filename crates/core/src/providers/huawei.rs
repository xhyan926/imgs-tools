use crate::types::*;
use crate::providers::ProviderAdapter;
use std::collections::HashSet;

/// 华为云 OBS 图片处理参数前缀
const OBS_PROCESS_PREFIX: &str = "image/";

/// 华为云 OBS 图片处理适配器
#[derive(Debug, Clone)]
pub struct HuaweiProvider;

impl HuaweiProvider {
    pub fn new() -> Self {
        Self
    }

    // ==================== 解析方法 ====================

    /// 解析华为云缩放参数
    /// 格式: resize,w_100,h_100,m_lfit,p_50,l_100,s_100,color_FF0000,limit_0
    fn parse_resize(params: &str) -> Option<ResizeParams> {
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

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");

            match key {
                "w" => width = value.parse().ok(),
                "h" => height = value.parse().ok(),
                "m" => {
                    mode = match value {
                        "lfit" => ResizeMode::Fit,
                        "mfit" => ResizeMode::Fill,
                        "fill" => ResizeMode::Fill,
                        "pad" => ResizeMode::Pad,
                        "fixed" => ResizeMode::Fixed,
                        "ratio" => ResizeMode::Ratio,
                        _ => mode,
                    };
                }
                "limit" => limit = Some(value == "0"),
                "p" => percentage = value.parse().ok(),
                "l" => longest_side = value.parse().ok(),
                "s" => shortest_side = value.parse().ok(),
                "color" => fill_color = Some(value.to_string()),
                _ => {}
            }
        }

        // 至少需要有一个尺寸参数
        if width.is_none() && height.is_none() && percentage.is_none()
            && longest_side.is_none() && shortest_side.is_none() {
            return None;
        }

        Some(ResizeParams {
            width,
            height,
            mode,
            limit,
            ratio: percentage.map(|p| p as f32 / 100.0),
            percentage,
            longest_side,
            shortest_side,
            fill_color,
        })
    }

    /// 解析华为云裁剪参数
    /// 格式: crop,x_100,y_100,w_200,h_200,g_br
    fn parse_crop(params: &str) -> Option<CropParams> {
        let mut x = 0;
        let mut y = 0;
        let mut width = 0;
        let mut height = 0;
        let mut grid_position = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");

            match key {
                "x" => x = value.parse().unwrap_or(0),
                "y" => y = value.parse().unwrap_or(0),
                "w" => width = value.parse().unwrap_or(0),
                "h" => height = value.parse().unwrap_or(0),
                "g" => {
                    grid_position = match value {
                        "tl" => Some(GridPosition::TopLeft),
                        "top" => Some(GridPosition::Top),
                        "tr" => Some(GridPosition::TopRight),
                        "left" => Some(GridPosition::Left),
                        "center" => Some(GridPosition::Center),
                        "right" => Some(GridPosition::Right),
                        "bl" => Some(GridPosition::BottomLeft),
                        "bottom" => Some(GridPosition::Bottom),
                        "br" => Some(GridPosition::BottomRight),
                        _ => None,
                    };
                }
                _ => {}
            }
        }

        if width == 0 || height == 0 {
            return None;
        }

        Some(CropParams {
            x, y, width, height,
            circle: None,
            grid_position,
        })
    }

    /// 解析内切圆参数
    /// 格式: circle,r_100
    fn parse_circle(params: &str) -> Option<CropParams> {
        let mut radius = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");

            if key == "r" {
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
    fn parse_index_crop(params: &str) -> Option<IndexCropParams> {
        let mut x_length = None;
        let mut y_length = None;
        let mut index = 0;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");

            match key {
                "x" => x_length = value.parse().ok(),
                "y" => y_length = value.parse().ok(),
                "i" => index = value.parse().unwrap_or(0),
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

    /// 解析圆角裁剪参数
    /// 格式: rounded-corners,r_100 或 rounded-corners,rx_100,ry_200
    fn parse_rounded_corners(params: &str) -> Option<RoundedCornersParams> {
        let mut radius = None;
        let mut radius_x = None;
        let mut radius_y = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");

            match key {
                "r" => radius = value.parse().ok(),
                "rx" => radius_x = value.parse().ok(),
                "ry" => radius_y = value.parse().ok(),
                _ => {}
            }
        }

        Some(RoundedCornersParams {
            radius,
            radius_x,
            radius_y,
        })
    }

    /// 解析旋转参数
    /// 格式: rotate,90
    fn parse_rotate(params: &str) -> Option<RotateParams> {
        let angle = params.trim().parse().ok()?;
        Some(RotateParams {
            angle,
            auto: None,
            flip: None,
        })
    }

    /// 解析自动方向参数
    /// 格式: auto-orient,0 或 auto-orient,1
    fn parse_auto_orient(params: &str) -> Option<RotateParams> {
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

    /// 解析镜像翻转参数
    /// 格式: flip,horizontal 或 flip,vertical
    fn parse_flip(params: &str) -> Option<RotateParams> {
        let direction = match params.trim() {
            "horizontal" => FlipDirection::Horizontal,
            "vertical" => FlipDirection::Vertical,
            _ => return None,
        };
        Some(RotateParams {
            angle: 0,
            auto: None,
            flip: Some(direction),
        })
    }

    /// 解析质量参数
    /// 格式: quality,q_80 或 quality,Q_80
    fn parse_quality(params: &str) -> Option<QualityParams> {
        let mut relative = None;
        let mut value = None;
        let mut is_absolute = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let num = kv.next().unwrap_or("");

            match key {
                "q" => relative = num.parse().ok(),
                "Q" => {
                    value = num.parse().ok();
                    is_absolute = Some(true);
                }
                _ => {}
            }
        }

        if relative.is_none() && value.is_none() {
            return None;
        }

        Some(QualityParams {
            value,
            relative,
            is_absolute,
        })
    }

    /// 解析格式转换参数
    /// 格式: format,png
    fn parse_format(params: &str) -> Option<FormatParams> {
        let format = match params.trim() {
            "jpg" => ImageFormat::Jpg,
            "jpeg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            "webp" => ImageFormat::Webp,
            "bmp" => ImageFormat::Bmp,
            "gif" => ImageFormat::Gif,
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
    fn parse_interlace(params: &str) -> Option<FormatParams> {
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
    fn parse_blur(params: &str) -> Option<BlurParams> {
        let mut radius = None;
        let mut sigma = None;

        for part in params.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '_');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");

            match key {
                "r" => radius = value.parse().ok(),
                "s" => sigma = value.parse().ok(),
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
    fn parse_sharpen(params: &str) -> Option<SharpenParams> {
        let amount = params.trim().parse().ok()?;
        Some(SharpenParams { amount })
    }

    /// 解析亮度参数
    /// 格式: bright,50 或 bright,-50
    fn parse_brightness(params: &str) -> Option<BrightnessContrastParams> {
        let brightness = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness,
            contrast: 0,
        })
    }

    /// 解析对比度参数
    /// 格式: contrast,-50
    fn parse_contrast(params: &str) -> Option<BrightnessContrastParams> {
        let contrast = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness: 0,
            contrast,
        })
    }

    /// 解析灰度参数
    /// 格式: colorspace,gray
    fn parse_grayscale(params: &str) -> Option<bool> {
        if params.trim() == "gray" {
            Some(true)
        } else {
            None
        }
    }

    // ==================== 生成方法 ====================

    /// 生成缩放参数字符串
    fn generate_resize(resize: &ResizeParams) -> String {
        let mut parts = Vec::new();

        if let Some(p) = resize.percentage {
            parts.push(format!("p_{}", p));
        }

        if let Some(l) = resize.longest_side {
            parts.push(format!("l_{}", l));
        }

        if let Some(s) = resize.shortest_side {
            parts.push(format!("s_{}", s));
        }

        if let Some(w) = resize.width {
            parts.push(format!("w_{}", w));
        }

        if let Some(h) = resize.height {
            parts.push(format!("h_{}", h));
        }

        // 华为云OBS的缩放模式映射
        let mode = match resize.mode {
            ResizeMode::Fit => "lfit",
            ResizeMode::Fill => "fill",
            ResizeMode::Pad => "pad",
            ResizeMode::Fixed => "fixed",
            ResizeMode::Ratio => "ratio",
            ResizeMode::Crop => "mfit",
        };
        parts.push(format!("m_{}", mode));

        if let Some(false) = resize.limit {
            parts.push("limit_0".to_string());
        }

        if let Some(ref color) = resize.fill_color {
            parts.push(format!("color_{}", color));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("resize,{}", parts.join(","))
        }
    }

    /// 生成裁剪参数字符串
    fn generate_crop(crop: &CropParams) -> String {
        let mut parts = vec![
            format!("x_{}", crop.x),
            format!("y_{}", crop.y),
            format!("w_{}", crop.width),
            format!("h_{}", crop.height),
        ];

        if let Some(pos) = crop.grid_position {
            parts.push(format!("g_{}", pos));
        }

        format!("crop,{}", parts.join(","))
    }

    /// 生成内切圆参数字符串
    fn generate_circle(crop: &CropParams) -> String {
        if crop.circle != Some(true) {
            return String::new();
        }
        let radius = crop.width.min(crop.height) / 2;
        format!("circle,r_{}", radius)
    }

    /// 生成索引裁剪参数字符串
    fn generate_index_crop(index_crop: &IndexCropParams) -> String {
        if let Some(x) = index_crop.x_length {
            format!("indexcrop,x_{},i_{}", x, index_crop.index)
        } else if let Some(y) = index_crop.y_length {
            format!("indexcrop,y_{},i_{}", y, index_crop.index)
        } else {
            String::new()
        }
    }

    /// 生成圆角裁剪参数字符串
    fn generate_rounded_corners(rounded: &RoundedCornersParams) -> String {
        if let Some(r) = rounded.radius {
            format!("rounded-corners,r_{}", r)
        } else if let (Some(rx), Some(ry)) = (rounded.radius_x, rounded.radius_y) {
            format!("rounded-corners,rx_{},ry_{}", rx, ry)
        } else {
            String::new()
        }
    }

    /// 生成旋转参数字符串
    fn generate_rotate(rotate: &RotateParams) -> String {
        if let Some(flip) = rotate.flip {
            format!("flip,{}", flip)
        } else if rotate.angle > 0 {
            format!("rotate,{}", rotate.angle)
        } else if let Some(true) = rotate.auto {
            "auto-orient,1".to_string()
        } else {
            String::new()
        }
    }

    /// 生成质量参数字符串
    fn generate_quality(quality: &QualityParams) -> String {
        if let Some(true) = quality.is_absolute {
            if let Some(v) = quality.value {
                return format!("quality,Q_{}", v);
            }
        }

        if let Some(r) = quality.relative {
            format!("quality,q_{}", r)
        } else if let Some(v) = quality.value {
            format!("quality,q_{}", v)
        } else {
            String::new()
        }
    }

    /// 生成格式转换参数字符串
    fn generate_format(format: &FormatParams) -> String {
        let mut result = format!("format,{}", format.format);

        if let Some(true) = format.progressive {
            result = format!("{}/interlace,1", result);
        }

        result
    }

    /// 生成模糊参数字符串
    fn generate_blur(blur: &BlurParams) -> String {
        let sigma = blur.sigma.unwrap_or(50);
        format!("blur,r_{},s_{}", blur.radius, sigma)
    }

    /// 生成锐化参数字符串
    fn generate_sharpen(sharpen: &SharpenParams) -> String {
        format!("sharpen,{}", sharpen.amount)
    }

    /// 生成亮度参数字符串
    fn generate_brightness(brightness: i16) -> String {
        format!("bright,{}", brightness)
    }

    /// 生成对比度参数字符串
    fn generate_contrast(contrast: i16) -> String {
        format!("contrast,{}", contrast)
    }

    /// 生成灰度参数字符串
    fn generate_grayscale() -> String {
        "colorspace,gray".to_string()
    }
}

impl Default for HuaweiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for HuaweiProvider {
    fn parse(&self, url: &str) -> Result<ImageProcessParams, ParseError> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| ParseError::InvalidUrl(format!("{}: {}", url, e)))?;

        let query_pairs: std::collections::HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        // 查找 x-image-process 参数
        let process_param = query_pairs
            .get("x-image-process")
            .ok_or_else(|| ParseError::MissingParameter("x-image-process".to_string()))?;

        // 检查前缀
        let process_value = process_param.strip_prefix(OBS_PROCESS_PREFIX).ok_or_else(|| {
            ParseError::InvalidValue("x-image-process".to_string(), process_param.clone())
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
                "flip" => {
                    if let Some(rotate) = Self::parse_flip(op_params) {
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
                "colorspace" => {
                    if let Some(gray) = Self::parse_grayscale(op_params) {
                        result.grayscale = Some(gray);
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

        // 缩放
        if let Some(ref resize) = params.resize {
            let resize_str = Self::generate_resize(resize);
            if !resize_str.is_empty() {
                operations.push(resize_str);
            }
        }

        // 裁剪
        if let Some(ref crop) = params.crop {
            if crop.circle == Some(true) {
                let circle_str = Self::generate_circle(crop);
                if !circle_str.is_empty() {
                    operations.push(circle_str);
                }
            } else {
                operations.push(Self::generate_crop(crop));
            }
        }

        // 索引裁剪
        if let Some(ref index_crop) = params.index_crop {
            let ic_str = Self::generate_index_crop(index_crop);
            if !ic_str.is_empty() {
                operations.push(ic_str);
            }
        }

        // 圆角裁剪
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

        // 灰度
        if let Some(true) = params.grayscale {
            operations.push(Self::generate_grayscale());
        }

        if operations.is_empty() {
            return Err(GenerateError::EmptyParams);
        }

        let params_str = format!("{}{}", OBS_PROCESS_PREFIX, operations.join("/"));

        Ok(GenerateResult {
            params: params_str,
            dropped,
        })
    }

    fn validate(&self, params: &ImageProcessParams) -> Result<(), ValidationError> {
        if let Some(ref quality) = params.quality {
            let value = if let Some(true) = quality.is_absolute {
                quality.value
            } else if let Some(r) = quality.relative {
                Some(r)
            } else {
                quality.value
            };

            if let Some(v) = value {
                if v > 100 || v < 1 {
                    return Err(ValidationError::OutOfRange("quality value must be 1-100".to_string()));
                }
            }
        }

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

        if let Some(ref sharpen) = params.sharpen {
            if sharpen.amount < 50 || sharpen.amount > 399 {
                return Err(ValidationError::OutOfRange("sharpen amount must be 50-399".to_string()));
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
    fn test_parse_resize_with_percentage() {
        let provider = HuaweiProvider::new();
        let url = "https://example.com/image.jpg?x-image-process=image/resize,p_50";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.resize.is_some());
        let resize = params.resize.unwrap();
        assert_eq!(resize.percentage, Some(50));
        assert_eq!(resize.ratio, Some(0.5));
    }

    #[test]
    fn test_parse_crop_with_grid_position() {
        let provider = HuaweiProvider::new();
        let url = "https://example.com/image.jpg?x-image-process=image/crop,x_10,y_10,w_200,h_200,g_br";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.crop.is_some());
        let crop = params.crop.unwrap();
        assert_eq!(crop.x, 10);
        assert_eq!(crop.grid_position, Some(GridPosition::BottomRight));
    }

    #[test]
    fn test_parse_blur() {
        let provider = HuaweiProvider::new();
        let url = "https://example.com/image.jpg?x-image-process=image/blur,r_3,s_2";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.blur.is_some());
        let blur = params.blur.unwrap();
        assert_eq!(blur.radius, 3);
        assert_eq!(blur.sigma, Some(2));
    }

    #[test]
    fn test_parse_grayscale() {
        let provider = HuaweiProvider::new();
        let url = "https://example.com/image.jpg?x-image-process=image/colorspace,gray";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.grayscale, Some(true));
    }

    #[test]
    fn test_parse_flip() {
        let provider = HuaweiProvider::new();
        let url = "https://example.com/image.jpg?x-image-process=image/flip,horizontal";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.rotate.is_some());
        let rotate = params.rotate.unwrap();
        assert_eq!(rotate.flip, Some(FlipDirection::Horizontal));
    }

    #[test]
    fn test_generate_blur() {
        let blur = BlurParams {
            radius: 5,
            sigma: Some(10),
        };
        let result = HuaweiProvider::generate_blur(&blur);
        assert_eq!(result, "blur,r_5,s_10");
    }

    #[test]
    fn test_generate_sharpen() {
        let sharpen = SharpenParams { amount: 100 };
        let result = HuaweiProvider::generate_sharpen(&sharpen);
        assert_eq!(result, "sharpen,100");
    }

    #[test]
    fn test_generate_grayscale() {
        let result = HuaweiProvider::generate_grayscale();
        assert_eq!(result, "colorspace,gray");
    }

    #[test]
    fn test_quality_absolute() {
        let provider = HuaweiProvider::new();
        let url = "https://example.com/image.jpg?x-image-process=image/quality,Q_80";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.quality.is_some());
        let quality = params.quality.unwrap();
        assert_eq!(quality.value, Some(80));
        assert_eq!(quality.is_absolute, Some(true));
    }

    #[test]
    fn test_validate_blur_range() {
        let provider = HuaweiProvider::new();
        let params = ImageProcessParams {
            blur: Some(BlurParams {
                radius: 100, // 超出范围
                sigma: None,
            }),
            ..Default::default()
        };
        assert!(provider.validate(&params).is_err());
    }

    #[test]
    fn test_validate_sharpen_range() {
        let provider = HuaweiProvider::new();
        let params = ImageProcessParams {
            sharpen: Some(SharpenParams { amount: 10 }), // 低于50
            ..Default::default()
        };
        assert!(provider.validate(&params).is_err());
    }
}
