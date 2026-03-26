use crate::types::*;
use crate::providers::ProviderAdapter;
use std::collections::HashSet;

/// 七牛云图片处理适配器
#[derive(Debug, Clone)]
pub struct QiniuProvider;

impl QiniuProvider {
    pub fn new() -> Self {
        Self
    }

    /// 解析七牛云缩放参数
    /// 格式: imageView2/2/w/100/h/100
    /// mode 0: 限定宽高，不放大
    /// mode 1: 限定宽高，自动裁剪
    /// mode 2: 限定宽高，等比缩放
    /// mode 3: 限定宽，高自适应
    /// mode 4: 限定高，宽自适应
    /// mode 5: 限定宽高，自动裁剪（与1类似）
    fn parse_resize(params: &str) -> Option<ResizeParams> {
        // 解析如 2/w/100/h/100 的格式
        let parts: Vec<&str> = params.split('/').collect();
        if parts.len() < 2 {
            return None;
        }

        let mode_str = parts.get(0)?;
        let mode = match mode_str.parse::<u32>().ok()? {
            0 => ResizeMode::Fit,
            1 | 5 => ResizeMode::Fill,
            2 => ResizeMode::Fit,
            3 | 4 => ResizeMode::Fit,
            _ => return None,
        };

        let mut width = None;
        let mut height = None;
        let mut fill_color = None;

        let mut iter = parts.iter().skip(1);
        while let Some(&part) = iter.next() {
            match part {
                "w" => {
                    if let Some(&value) = iter.next() {
                        width = value.parse().ok();
                    }
                }
                "h" => {
                    if let Some(&value) = iter.next() {
                        height = value.parse().ok();
                    }
                }
                "e" => {
                    // e 参数表示是否放大
                }
                _ => {}
            }
        }

        Some(ResizeParams {
            width,
            height,
            mode,
            limit: None,
            ratio: None,
            percentage: None,
            longest_side: None,
            shortest_side: None,
            fill_color,
        })
    }

    /// 解析七牛云裁剪参数
    /// 格式: imageMogr2/crop/!100x100a10a10 或 imageMogr2/crop/!300x300a30a100/gravity/NorthWest
    fn parse_crop(params: &str, gravity: Option<GridPosition>) -> Option<CropParams> {
        // 格式: !widthxheightaxoffsetXaoffsetY 或 widthxheightaxoffsetXaoffsetY
        let params = params.strip_prefix('!').unwrap_or(params);

        // 分离裁剪参数和可能的gravity参数
        // 先尝试按 x 和 a 分割
        let parts: Vec<&str> = params.split(['x', 'a']).collect();

        if parts.len() >= 4 {
            let width = parts[0].parse().ok()?;
            let height = parts[1].parse().ok()?;
            let x = parts[2].parse().ok()?;
            let y = parts[3].parse().ok()?;

            return Some(CropParams {
                x,
                y,
                width,
                height,
                circle: None,
                grid_position: gravity,
            });
        }
        None
    }

    /// 解析七牛云旋转参数
    /// 格式: imageMogr2/rotate/90 或 imageMogr2/auto-orient
    fn parse_rotate(params: &str) -> Option<RotateParams> {
        if params == "auto-orient" {
            return Some(RotateParams {
                angle: 0,
                auto: Some(true),
                flip: None,
            });
        }

        let angle = params.trim().parse().ok()?;
        Some(RotateParams {
            angle,
            auto: None,
            flip: None,
        })
    }

    /// 解析七牛云质量参数
    /// 格式: imageslim (自动质量压缩)
    /// 或: imageMogr2/quality/90 或 imageMogr2/quality/Q80
    fn parse_quality(params: &str) -> Option<QualityParams> {
        if params == "imageslim" {
            return Some(QualityParams {
                value: None,
                relative: Some(80),
                is_absolute: None,
            });
        }

        let params = params.trim();
        if let Some(rest) = params.strip_prefix("Q") {
            let relative = rest.parse().ok()?;
            Some(QualityParams {
                value: None,
                relative: Some(relative),
                is_absolute: None,
            })
        } else {
            let value = params.parse().ok()?;
            Some(QualityParams {
                value: Some(value),
                relative: None,
                is_absolute: None,
            })
        }
    }

    /// 解析七牛云格式转换参数
    /// 格式: imageMogr2/format/png
    fn parse_format(params: &str) -> Option<FormatParams> {
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

    /// 解析七牛云模糊参数
    /// 格式: imageMogr2/blur/5x3 (radiusxsigma)
    fn parse_blur(params: &str) -> Option<BlurParams> {
        let parts: Vec<&str> = params.split('x').collect();
        if parts.len() == 2 {
            let radius = parts[0].parse().ok()?;
            let sigma = parts[1].parse().ok();
            Some(BlurParams { radius, sigma })
        } else {
            None
        }
    }

    /// 解析七牛云锐化参数
    /// 格式: imageMogr2/sharpen/50
    fn parse_sharpen(params: &str) -> Option<SharpenParams> {
        let amount = params.trim().parse().ok()?;
        Some(SharpenParams { amount })
    }

    /// 解析七牛云重力锚点
    /// 格式: imageMogr2/gravity/NorthWest
    fn parse_gravity(params: &str) -> Option<GridPosition> {
        let gravity = match params.trim() {
            "NorthWest" => GridPosition::TopLeft,
            "North" => GridPosition::Top,
            "NorthEast" => GridPosition::TopRight,
            "West" => GridPosition::Left,
            "Center" => GridPosition::Center,
            "East" => GridPosition::Right,
            "SouthWest" => GridPosition::BottomLeft,
            "South" => GridPosition::Bottom,
            "SouthEast" => GridPosition::BottomRight,
            _ => return None,
        };
        Some(gravity)
    }

    /// 解析七牛云背景色
    /// 格式: imageMogr2/background/#FF0000
    fn parse_background(params: &str) -> Option<String> {
        let color = params.trim();
        if color.starts_with('#') {
            Some(color.to_string())
        } else {
            None
        }
    }

    /// 解析七牛云文件大小限制
    /// 格式: imageMogr2/size-limit/1024b
    fn parse_size_limit(params: &str) -> Option<u32> {
        let params = params.trim();
        if let Some(size_str) = params.strip_suffix('b') {
            size_str.trim().parse().ok()
        } else {
            params.parse().ok()
        }
    }

    /// 生成七牛云缩放参数字符串
    fn generate_resize(params: &ResizeParams) -> String {
        let mode = match params.mode {
            ResizeMode::Fit => "2",
            ResizeMode::Fill => "1",
            ResizeMode::Ratio => "2",
            _ => "2",
        };

        if params.mode == ResizeMode::Ratio {
            if let Some(ratio) = params.ratio {
                return format!("imageView2/{}/p/{}", mode, (ratio * 100.0) as u32);
            }
        }

        let mut parts = vec![mode.to_string()];
        if let Some(w) = params.width {
            parts.push("w".to_string());
            parts.push(w.to_string());
        }
        if let Some(h) = params.height {
            parts.push("h".to_string());
            parts.push(h.to_string());
        }

        format!("imageView2/{}", parts.join("/"))
    }

    /// 生成七牛云裁剪参数字符串
    fn generate_crop(params: &CropParams) -> String {
        let base = format!(
            "imageMogr2/crop/!{}x{}a{}a{}",
            params.width, params.height, params.x, params.y
        );

        // 如果有grid_position，添加gravity
        if let Some(gravity) = params.grid_position {
            let gravity_str = match gravity {
                GridPosition::TopLeft => "NorthWest",
                GridPosition::Top => "North",
                GridPosition::TopRight => "NorthEast",
                GridPosition::Left => "West",
                GridPosition::Center => "Center",
                GridPosition::Right => "East",
                GridPosition::BottomLeft => "SouthWest",
                GridPosition::Bottom => "South",
                GridPosition::BottomRight => "SouthEast",
            };
            return format!("{}/gravity/{}", base, gravity_str);
        }

        base
    }

    /// 生成七牛云旋转参数字符串
    fn generate_rotate(params: &RotateParams) -> String {
        if params.auto == Some(true) {
            "imageMogr2/auto-orient".to_string()
        } else {
            format!("imageMogr2/rotate/{}", params.angle)
        }
    }

    /// 生成七牛云质量参数字符串
    fn generate_quality(params: &QualityParams) -> String {
        if let Some(value) = params.value {
            format!("imageMogr2/quality/{}", value)
        } else if let Some(relative) = params.relative {
            format!("imageMogr2/quality/Q{}", relative)
        } else {
            String::new()
        }
    }

    /// 生成七牛云格式转换参数字符串
    fn generate_format(params: &FormatParams) -> String {
        let mut result = format!("imageMogr2/format/{}", params.format);

        // 如果有渐进式参数
        if params.progressive == Some(true) {
            result = format!("{}/interlace/1", result);
        }

        result
    }

    /// 生成七牛云模糊参数字符串
    fn generate_blur(params: &BlurParams) -> String {
        if let Some(sigma) = params.sigma {
            format!("imageMogr2/blur/{}x{}", params.radius, sigma)
        } else {
            format!("imageMogr2/blur/{}", params.radius)
        }
    }

    /// 生成七牛云锐化参数字符串
    fn generate_sharpen(params: &SharpenParams) -> String {
        format!("imageMogr2/sharpen/{}", params.amount)
    }

    /// 生成七牛云strip参数字符串
    fn generate_strip(metadata: &Metadata) -> String {
        if metadata.remove_exif == Some(true) {
            "imageMogr2/strip".to_string()
        } else {
            String::new()
        }
    }

    /// 生成七牛云背景色参数字符串
    fn generate_background(color: &str) -> String {
        format!("imageMogr2/background/{}", color)
    }

    /// 生成七牛云文件大小限制参数字符串
    fn generate_size_limit(size: u32) -> String {
        format!("imageMogr2/size-limit/{}b", size)
    }
}

impl Default for QiniuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for QiniuProvider {
    fn parse(&self, url: &str) -> Result<ImageProcessParams, ParseError> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| ParseError::InvalidUrl(format!("{}: {}", url, e)))?;

        let mut result = ImageProcessParams::default();
        let mut current_gravity: Option<GridPosition> = None;

        // 七牛云使用特殊的URL格式，如 ?imageView2/2/w/100 或 ?imageMogr2/auto-orient
        // 需要从query字符串中提取
        let query = parsed_url.query().unwrap_or("");

        // 检查 imageView2 参数（缩放）
        // 格式: ?imageView2/2/w/100/h/200
        if let Some(iv2_start) = query.find("imageView2/") {
            let remaining = &query[iv2_start..];
            // 找到imageView2参数段的结束位置（下一个参数或结尾）
            let end = remaining.find('&').unwrap_or(remaining.len());
            let iv2_param = &remaining["imageView2/".len()..end];
            if let Some(resize) = Self::parse_resize(iv2_param) {
                result.resize = Some(resize);
            }
        }

        // 检查 imageMogr2 参数（其他操作）
        // 格式: ?imageMogr2/auto-orient 或 ?imageMogr2/crop/!300x300a30a100
        if let Some(mogr_start) = query.find("imageMogr2/") {
            let remaining = &query[mogr_start..];
            let end = remaining.find('&').unwrap_or(remaining.len());
            let mogr2_content = &remaining["imageMogr2/".len()..end];

            let operations: Vec<&str> = mogr2_content.split('/').collect();
            let mut i = 0;

            while i < operations.len() {
                let operation = operations[i].trim();

                if operation.is_empty() {
                    i += 1;
                    continue;
                }

                match operation {
                    "thumbnail" => {
                        // thumbnail 格式: thumbnail/300x300
                        if i + 1 < operations.len() {
                            let size = operations[i + 1];
                            if let Some((w, h)) = Self::parse_thumbnail(size) {
                                result.resize = Some(ResizeParams {
                                    width: Some(w),
                                    height: Some(h),
                                    mode: ResizeMode::Fit,
                                    limit: None,
                                    ratio: None,
                                    percentage: None,
                                    longest_side: None,
                                    shortest_side: None,
                                    fill_color: None,
                                });
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "crop" => {
                        if i + 1 < operations.len() {
                            if let Some(crop) = Self::parse_crop(operations[i + 1], current_gravity) {
                                result.crop = Some(crop);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "auto-orient" => {
                        result.rotate = Some(RotateParams {
                            angle: 0,
                            auto: Some(true),
                            flip: None,
                        });
                        i += 1;
                    }
                    "rotate" => {
                        if i + 1 < operations.len() {
                            if let Some(rotate) = Self::parse_rotate(operations[i + 1]) {
                                result.rotate = Some(rotate);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "quality" => {
                        if i + 1 < operations.len() {
                            if let Some(quality) = Self::parse_quality(operations[i + 1]) {
                                result.quality = Some(quality);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "format" => {
                        if i + 1 < operations.len() {
                            if let Some(format) = Self::parse_format(operations[i + 1]) {
                                result.format = Some(format);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "strip" => {
                        result.metadata = Some(Metadata {
                            remove_exif: Some(true),
                        });
                        i += 1;
                    }
                    "gravity" => {
                        if i + 1 < operations.len() {
                            current_gravity = Self::parse_gravity(operations[i + 1]);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "blur" => {
                        if i + 1 < operations.len() {
                            if let Some(blur) = Self::parse_blur(operations[i + 1]) {
                                result.blur = Some(blur);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "sharpen" => {
                        if i + 1 < operations.len() {
                            if let Some(sharpen) = Self::parse_sharpen(operations[i + 1]) {
                                result.sharpen = Some(sharpen);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "background" => {
                        if i + 1 < operations.len() {
                            if let Some(color) = Self::parse_background(operations[i + 1]) {
                                if let Some(ref mut resize) = result.resize {
                                    resize.fill_color = Some(color);
                                } else {
                                    // 存储到extra中，因为没有resize
                                    result.extra.insert("background".to_string(), color.into());
                                }
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "interlace" => {
                        if let Some(ref mut format) = result.format {
                            format.progressive = Some(true);
                        }
                        i += 1;
                    }
                    "size-limit" => {
                        if i + 1 < operations.len() {
                            if let Some(size) = Self::parse_size_limit(operations[i + 1]) {
                                result.size_limit = Some(size);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "ignore-error" => {
                        result.ignore_error = Some(true);
                        i += 1;
                    }
                    _ => {
                        // 未知参数，存入extra
                        result.extra.insert(operation.to_string(), operation.to_string().into());
                        i += 1;
                    }
                }
            }
        }

        // 检查 imageslim 参数（自动压缩）
        // 格式: ?imageslim
        if query.contains("imageslim") {
            result.quality = Some(QualityParams {
                value: None,
                relative: Some(80),
                is_absolute: None,
            });
        }

        Ok(result)
    }

    fn generate(&self, params: &ImageProcessParams) -> Result<GenerateResult, GenerateError> {
        let mut query_parts = Vec::new();
        let mut dropped = Vec::new();

        // imageView2 用于缩放
        if let Some(ref resize) = params.resize {
            let resize_str = Self::generate_resize(resize);
            if !resize_str.is_empty() {
                query_parts.push(resize_str);
            }
        }

        let mut mogr2_ops = Vec::new();

        // 自动旋转
        if let Some(ref rotate) = params.rotate {
            if rotate.auto == Some(true) {
                mogr2_ops.push("auto-orient".to_string());
            } else {
                mogr2_ops.push(Self::generate_rotate(rotate));
            }
        }

        // 裁剪
        if let Some(ref crop) = params.crop {
            mogr2_ops.push(Self::generate_crop(crop));
        }

        // 去除元数据
        if let Some(ref metadata) = params.metadata {
            let strip_str = Self::generate_strip(metadata);
            if !strip_str.is_empty() {
                mogr2_ops.push(strip_str);
            }
        }

        // 质量
        if let Some(ref quality) = params.quality {
            let quality_str = Self::generate_quality(quality);
            if !quality_str.is_empty() {
                mogr2_ops.push(quality_str);
            }
        }

        // 格式转换
        if let Some(ref format) = params.format {
            mogr2_ops.push(Self::generate_format(format));
        }

        // 模糊
        if let Some(ref blur) = params.blur {
            mogr2_ops.push(Self::generate_blur(blur));
        }

        // 锐化
        if let Some(ref sharpen) = params.sharpen {
            mogr2_ops.push(Self::generate_sharpen(sharpen));
        }

        // 背景色
        if let Some(ref resize) = params.resize {
            if let Some(ref color) = resize.fill_color {
                mogr2_ops.push(Self::generate_background(color));
            }
        }

        // 文件大小限制
        if let Some(size) = params.size_limit {
            mogr2_ops.push(Self::generate_size_limit(size));
        }

        // 忽略错误
        if params.ignore_error == Some(true) {
            mogr2_ops.push("imageMogr2/ignore-error".to_string());
        }

        if !mogr2_ops.is_empty() {
            // 提取 imageMogr2 内部的操作
            let ops: Vec<String> = mogr2_ops
                .into_iter()
                .filter_map(|s| {
                    if s.starts_with("imageMogr2/") {
                        s.strip_prefix("imageMogr2/").map(|x| x.to_string())
                    } else {
                        Some(s)
                    }
                })
                .collect();
            query_parts.push(format!("imageMogr2/{}", ops.join("/")));
        }

        if query_parts.is_empty() {
            return Err(GenerateError::EmptyParams);
        }

        let params_str = query_parts.join("&");

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
        Ok(())
    }

    fn supported_operations(&self) -> HashSet<Operation> {
        let mut ops = HashSet::new();
        ops.insert(Operation::Resize);
        ops.insert(Operation::Crop);
        ops.insert(Operation::Rotate);
        ops.insert(Operation::Quality);
        ops.insert(Operation::Format);
        ops.insert(Operation::Metadata);
        ops.insert(Operation::Blur);
        ops.insert(Operation::Sharpen);
        ops.insert(Operation::Progressive);
        ops
    }
}

impl QiniuProvider {
    /// 解析thumbnail参数
    /// 格式: 300x300 或 300x 或 x300
    fn parse_thumbnail(params: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = params.split('x').collect();
        match parts.len() {
            2 => {
                let w = parts[0].parse().ok();
                let h = parts[1].parse().ok();
                if w.is_some() || h.is_some() {
                    Some((w.unwrap_or(0), h.unwrap_or(0)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resize_basic() {
        let result = QiniuProvider::parse_resize("2/w/100/h/200");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.width, Some(100));
        assert_eq!(resize.height, Some(200));
        assert_eq!(resize.mode, ResizeMode::Fit);
    }

    #[test]
    fn test_parse_resize_fill_mode() {
        let result = QiniuProvider::parse_resize("1/w/100/h/200");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.mode, ResizeMode::Fill);
    }

    #[test]
    fn test_parse_crop_with_gravity() {
        let gravity = GridPosition::TopLeft;
        let result = QiniuProvider::parse_crop("!300x300a30a100", Some(gravity));
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.width, 300);
        assert_eq!(crop.height, 300);
        assert_eq!(crop.x, 30);
        assert_eq!(crop.y, 100);
        assert_eq!(crop.grid_position, Some(GridPosition::TopLeft));
    }

    #[test]
    fn test_parse_blur() {
        let result = QiniuProvider::parse_blur("5x3");
        assert!(result.is_some());
        let blur = result.unwrap();
        assert_eq!(blur.radius, 5);
        assert_eq!(blur.sigma, Some(3));
    }

    #[test]
    fn test_parse_sharpen() {
        let result = QiniuProvider::parse_sharpen("50");
        assert!(result.is_some());
        let sharpen = result.unwrap();
        assert_eq!(sharpen.amount, 50);
    }

    #[test]
    fn test_parse_gravity() {
        let result = QiniuProvider::parse_gravity("NorthWest");
        assert_eq!(result, Some(GridPosition::TopLeft));

        let result = QiniuProvider::parse_gravity("Center");
        assert_eq!(result, Some(GridPosition::Center));
    }

    #[test]
    fn test_parse_size_limit() {
        let result = QiniuProvider::parse_size_limit("1024b");
        assert_eq!(result, Some(1024));

        let result = QiniuProvider::parse_size_limit("2048");
        assert_eq!(result, Some(2048));
    }

    #[test]
    fn test_generate_resize() {
        let params = ResizeParams {
            width: Some(100),
            height: Some(200),
            mode: ResizeMode::Fit,
            limit: None,
            ratio: None,
            percentage: None,
            longest_side: None,
            shortest_side: None,
            fill_color: None,
        };
        let result = QiniuProvider::generate_resize(&params);
        assert!(result.contains("imageView2"));
        assert!(result.contains("w"));
        assert!(result.contains("100"));
    }

    #[test]
    fn test_generate_crop_with_gravity() {
        let params = CropParams {
            x: 30,
            y: 100,
            width: 300,
            height: 300,
            circle: None,
            grid_position: Some(GridPosition::TopLeft),
        };
        let result = QiniuProvider::generate_crop(&params);
        assert!(result.contains("crop"));
        assert!(result.contains("gravity"));
        assert!(result.contains("NorthWest"));
    }

    #[test]
    fn test_generate_blur() {
        let params = BlurParams {
            radius: 5,
            sigma: Some(3),
        };
        let result = QiniuProvider::generate_blur(&params);
        assert!(result.contains("blur"));
        assert!(result.contains("5x3"));
    }

    #[test]
    fn test_generate_sharpen() {
        let params = SharpenParams { amount: 50 };
        let result = QiniuProvider::generate_sharpen(&params);
        assert!(result.contains("sharpen"));
        assert!(result.contains("50"));
    }

    #[test]
    fn test_parse_full_url_with_auto_orient() {
        let provider = QiniuProvider::new();
        let url = "https://example.com/image.jpg?imageMogr2/auto-orient";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.rotate.is_some());
        assert_eq!(params.rotate.as_ref().unwrap().auto, Some(true));
    }

    #[test]
    fn test_parse_full_url_with_strip() {
        let provider = QiniuProvider::new();
        let url = "https://example.com/image.jpg?imageMogr2/strip";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.metadata.is_some());
        assert_eq!(params.metadata.as_ref().unwrap().remove_exif, Some(true));
    }

    #[test]
    fn test_parse_full_url_with_blur() {
        let provider = QiniuProvider::new();
        let url = "https://example.com/image.jpg?imageMogr2/blur/5x3";
        let result = provider.parse(url);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.blur.is_some());
        assert_eq!(params.blur.as_ref().unwrap().radius, 5);
    }

    #[test]
    fn test_generate_with_strip() {
        let provider = QiniuProvider::new();
        let params = ImageProcessParams {
            metadata: Some(Metadata {
                remove_exif: Some(true),
            }),
            ..Default::default()
        };
        let result = provider.generate(&params);
        assert!(result.is_ok());
        let gen = result.unwrap();
        assert!(gen.params.contains("strip"));
    }

    #[test]
    fn test_generate_with_size_limit() {
        let provider = QiniuProvider::new();
        let params = ImageProcessParams {
            resize: Some(ResizeParams {
                width: Some(100),
                height: Some(100),
                mode: ResizeMode::Fit,
                limit: None,
                ratio: None,
                percentage: None,
                longest_side: None,
                shortest_side: None,
                fill_color: None,
            }),
            size_limit: Some(1024),
            ..Default::default()
        };
        let result = provider.generate(&params);
        assert!(result.is_ok());
        let gen = result.unwrap();
        assert!(gen.params.contains("size-limit"));
        assert!(gen.params.contains("1024b"));
    }

    #[test]
    fn test_generate_with_ignore_error() {
        let provider = QiniuProvider::new();
        let params = ImageProcessParams {
            resize: Some(ResizeParams {
                width: Some(100),
                height: Some(100),
                mode: ResizeMode::Fit,
                limit: None,
                ratio: None,
                percentage: None,
                longest_side: None,
                shortest_side: None,
                fill_color: None,
            }),
            ignore_error: Some(true),
            ..Default::default()
        };
        let result = provider.generate(&params);
        assert!(result.is_ok());
        let gen = result.unwrap();
        assert!(gen.params.contains("ignore-error"));
    }

    #[test]
    fn test_supported_operations() {
        let provider = QiniuProvider::new();
        let ops = provider.supported_operations();
        assert!(ops.contains(&Operation::Resize));
        assert!(ops.contains(&Operation::Crop));
        assert!(ops.contains(&Operation::Rotate));
        assert!(ops.contains(&Operation::Quality));
        assert!(ops.contains(&Operation::Format));
        assert!(ops.contains(&Operation::Metadata));
        assert!(ops.contains(&Operation::Blur));
        assert!(ops.contains(&Operation::Sharpen));
        assert!(ops.contains(&Operation::Progressive));
    }
}
