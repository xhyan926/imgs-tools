use crate::types::*;
use crate::providers::ProviderAdapter;
use std::collections::{HashMap, HashSet};

/// 腾讯云数据万象图片处理适配器
#[derive(Debug, Clone)]
pub struct TencentProvider;

impl TencentProvider {
    pub fn new() -> Self {
        Self
    }

    // ==================== 解析方法 ====================

    /// 解析腾讯云缩放参数
    /// 格式: thumbnail/!50p (百分比宽高), thumbnail/!50px (百分比宽), thumbnail/!x50p (百分比高)
    ///       thumbnail/100x (指定宽), thumbnail/x200 (指定高), thumbnail/100x200 (指定宽高)
    ///       thumbnail/!100x200r (最小值), thumbnail/100x200> (仅缩小), thumbnail/100x200< (仅放大)
    ///       thumbnail/100x200! (强制), thumbnail/10000@ (按像素)
    pub fn parse_resize(params: &str) -> Option<ResizeParams> {
        let params = params.trim();
        if params.is_empty() {
            return None;
        }

        let mut width = None;
        let mut height = None;
        let mut mode = ResizeMode::Fit;
        let mut ratio = None;
        let mut limit = None; // 用于 > (仅缩小) 和 < (仅放大)
        let mut fill_color = None;

        // 检查是否有 ! 前缀（表示严格模式或百分比模式）
        let (has_bang, rest) = if params.starts_with('!') {
            (true, &params[1..])
        } else {
            (false, params)
        };

        // 检查是否有后缀修饰符
        let (rest, shrink_only, enlarge_only, force_mode, pixel_mode) = Self::parse_resize_suffixes(rest)?;

        // 解析尺寸部分
        if let Some(percent_end) = rest.find('p') {
            // 百分比模式
            let size_str = &rest[..percent_end];
            if size_str.contains('x') {
                // 宽高百分比: 50x50p 或 x50p 或 50xp
                if size_str.starts_with('x') {
                    // !x50p - 高百分比
                    if let Ok(h) = size_str[1..].parse::<u32>() {
                        ratio = Some(h as f32 / 100.0);
                        mode = ResizeMode::Fit;
                    }
                } else if size_str.ends_with('x') {
                    // !50xp - 宽百分比
                    if let Ok(w) = size_str[..size_str.len() - 1].parse::<u32>() {
                        ratio = Some(w as f32 / 100.0);
                        mode = ResizeMode::Fit;
                    }
                } else if let Some(x_pos) = size_str.find('x') {
                    // !50x50p - 宽高百分比
                    if let (Ok(w), Ok(h)) = (
                        size_str[..x_pos].parse::<u32>(),
                        size_str[x_pos + 1..].parse::<u32>()
                    ) {
                        width = Some(w);
                        height = Some(h);
                        mode = ResizeMode::Fit;
                        ratio = Some(w as f32 / 100.0);
                    }
                }
            } else {
                // !50p - 简单百分比
                if let Ok(p) = size_str.parse::<u32>() {
                    ratio = Some(p as f32 / 100.0);
                    mode = ResizeMode::Fit;
                }
            }
        } else if let Some(at_pos) = rest.find('@') {
            // 按像素总数缩放: 10000@
            if let Ok(area) = rest[..at_pos].parse::<u32>() {
                // 保存到 extra 中，因为标准 ResizeParams 没有这个字段
                mode = ResizeMode::Fit;
                // 计算近似宽高：假设原图是 4:3，宽 ≈ sqrt(area * 4/3)
                // 这里需要原图尺寸，暂时忽略
            }
        } else {
            // 普通尺寸模式: 100x200, 100x, x200
            if rest.contains('x') {
                let parts: Vec<&str> = rest.split('x').collect();
                if parts.len() >= 2 {
                    if !parts[0].is_empty() {
                        width = parts[0].parse().ok();
                    }
                    if !parts[1].is_empty() {
                        height = parts[1].parse().ok();
                    }
                }
            }
        }

        // 根据后缀设置模式
        if force_mode {
            mode = ResizeMode::Fixed;
        } else if has_bang && rest.contains('x') && rest.ends_with('r') {
            // !100x200r - 最小值模式
            mode = ResizeMode::Fill;
        } else if shrink_only {
            limit = Some(true); // 仅缩小
        } else if enlarge_only {
            limit = Some(false); // 仅放大
        }
        // 单百分比模式 (!50p) 保持 Fit 模式，但 ratio 字段已设置

        Some(ResizeParams {
            width,
            height,
            mode,
            limit,
            ratio,
            percentage: None,
            longest_side: None,
            shortest_side: None,
            fill_color,
        })
    }

    /// 解析缩放后缀修饰符
    fn parse_resize_suffixes(params: &str) -> Option<(&str, bool, bool, bool, bool)> {
        let mut rest = params;
        let mut shrink_only = false;
        let mut enlarge_only = false;
        let mut force_mode = false;
        let mut pixel_mode = false;

        // 检查后缀: > < ! @
        if rest.ends_with('>') {
            shrink_only = true;
            rest = &rest[..rest.len() - 1];
        } else if rest.ends_with('<') {
            enlarge_only = true;
            rest = &rest[..rest.len() - 1];
        } else if rest.ends_with('!') {
            force_mode = true;
            rest = &rest[..rest.len() - 1];
        } else if rest.ends_with('@') {
            pixel_mode = true;
            rest = &rest[..rest.len() - 1];
        }

        // 移除 r 后缀（最小值模式）
        if rest.ends_with('r') {
            rest = &rest[..rest.len() - 1];
        }

        Some((rest, shrink_only, enlarge_only, force_mode, pixel_mode))
    }

    /// 解析腾讯云自定义裁剪参数
    /// 格式: cut/600x600x100x10 或 cut/600x600/gravity/center
    pub fn parse_cut(params: &str) -> Option<CropParams> {
        let mut width = None;
        let mut height = None;
        let mut x = 0;
        let mut y = 0;
        let mut grid_position = None;

        let mut parts = params.split('/');
        let size_part = parts.next()?;

        // 解析尺寸和偏移: 600x600x100x10
        let size_parts: Vec<&str> = size_part.split('x').collect();
        if size_parts.len() >= 2 {
            width = size_parts[0].parse().ok();
            height = size_parts.get(1).and_then(|s| s.parse().ok());
            if size_parts.len() >= 4 {
                x = size_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                y = size_parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
        }

        // 解析 gravity 参数
        for part in parts {
            if let Some(rest) = part.strip_prefix("gravity/") {
                grid_position = Self::parse_gravity(rest);
            }
        }

        Some(CropParams {
            x,
            y,
            width: width.unwrap_or(0),
            height: height.unwrap_or(0),
            circle: None,
            grid_position,
        })
    }

    /// 解析腾讯云缩放裁剪参数
    /// 格式: crop/300x400 或 crop/300x400/gravity/center
    pub fn parse_crop(params: &str) -> Option<CropParams> {
        let mut width = None;
        let mut height = None;
        let mut grid_position = Some(GridPosition::Center); // 默认中心

        let mut parts = params.split('/');
        let size_part = parts.next()?;

        // 解析尺寸: 300x400
        if let Some(x_pos) = size_part.find('x') {
            width = size_part[..x_pos].parse().ok();
            height = size_part[x_pos + 1..].parse().ok();
        }

        // 解析 gravity 参数
        for part in parts {
            if let Some(rest) = part.strip_prefix("gravity/") {
                grid_position = Self::parse_gravity(rest);
            }
        }

        Some(CropParams {
            x: 0,
            y: 0,
            width: width.unwrap_or(0),
            height: height.unwrap_or(0),
            circle: None,
            grid_position,
        })
    }

    /// 解析九宫格位置
    fn parse_gravity(value: &str) -> Option<GridPosition> {
        match value {
            "northwest" | "nw" => Some(GridPosition::TopLeft),
            "north" | "n" => Some(GridPosition::Top),
            "northeast" | "ne" => Some(GridPosition::TopRight),
            "west" | "w" => Some(GridPosition::Left),
            "center" | "c" => Some(GridPosition::Center),
            "east" | "e" => Some(GridPosition::Right),
            "southwest" | "sw" => Some(GridPosition::BottomLeft),
            "south" | "s" => Some(GridPosition::Bottom),
            "southeast" | "se" => Some(GridPosition::BottomRight),
            _ => None,
        }
    }

    /// 解析内切圆裁剪参数
    /// 格式: iradius/200
    pub fn parse_iradius(params: &str) -> Option<CropParams> {
        let radius: u32 = params.trim().parse().ok()?;
        let diameter = radius * 2;
        Some(CropParams {
            x: 0,
            y: 0,
            width: diameter,
            height: diameter,
            circle: Some(true),
            grid_position: Some(GridPosition::Center),
        })
    }

    /// 解析圆角裁剪参数
    /// 格式: rradius/100
    pub fn parse_rradius(params: &str) -> Option<RoundedCornersParams> {
        let radius = params.trim().parse().ok()?;
        Some(RoundedCornersParams {
            radius: Some(radius),
            radius_x: None,
            radius_y: None,
        })
    }

    /// 解析人脸智能裁剪参数
    /// 格式: scrop/100x600
    pub fn parse_scrop(params: &str) -> Option<CropParams> {
        let mut width = None;
        let mut height = None;

        if let Some(x_pos) = params.find('x') {
            width = params[..x_pos].parse().ok();
            height = params[x_pos + 1..].parse().ok();
        }

        Some(CropParams {
            x: 0,
            y: 0,
            width: width.unwrap_or(0),
            height: height.unwrap_or(0),
            circle: None,
            grid_position: None,
        })
    }

    /// 解析自适应裁剪参数
    /// 格式: rcrop/50x100 (最小宽高比 1:2 到 1:1)
    pub fn parse_rcrop(params: &str) -> Option<CropParams> {
        // 自适应裁剪参数保存到 extra，因为标准类型不支持
        let mut parts = params.split('x');
        let _min_ratio: u32 = parts.next()?.parse().ok()?;
        let _max_ratio: u32 = parts.next()?.parse().ok()?;

        // 返回空的裁剪参数，实际参数保存在 extra
        Some(CropParams {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            circle: None,
            grid_position: None,
        })
    }

    /// 解析旋转参数
    /// 格式: rotate/90
    pub fn parse_rotate(params: &str) -> Option<RotateParams> {
        let angle = params.trim().parse().ok()?;
        Some(RotateParams {
            angle,
            auto: None,
            flip: None,
        })
    }

    /// 解析质量参数
    /// 格式: quality/90 或 quality/Q80 (相对质量)
    pub fn parse_quality(params: &str) -> Option<QualityParams> {
        let params = params.trim();
        if let Some(rest) = params.strip_prefix('Q') {
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

    /// 解析格式转换参数
    /// 格式: format/png 或 format/png/interlace/1
    pub fn parse_format(params: &str) -> Option<FormatParams> {
        let mut parts: Vec<&str> = params.split('/').collect();
        let format_str = parts.get(0)?.trim().to_lowercase();
        let format = match format_str.as_str() {
            "jpg" | "jpeg" => ImageFormat::Jpg,
            "png" => ImageFormat::Png,
            "webp" => ImageFormat::Webp,
            "gif" => ImageFormat::Gif,
            "bmp" => ImageFormat::Bmp,
            "tiff" => ImageFormat::Tiff,
            "tpg" => ImageFormat::Webp,
            _ => return None,
        };

        // 检查是否有 interlace 参数
        let mut progressive = None;
        if parts.len() >= 3 && parts.get(1) == Some(&"interlace") && parts.get(2) == Some(&"1") {
            progressive = Some(true);
        }

        Some(FormatParams {
            format,
            progressive,
        })
    }

    /// 解析高斯模糊参数
    /// 格式: blur/8x5
    pub fn parse_blur(params: &str) -> Option<BlurParams> {
        let parts: Vec<&str> = params.split('x').collect();
        if parts.len() >= 2 {
            let radius = parts[0].parse().ok()?;
            let sigma = parts[1].parse().ok()?;
            Some(BlurParams {
                radius,
                sigma: Some(sigma),
            })
        } else {
            let radius = params.parse().ok()?;
            Some(BlurParams {
                radius,
                sigma: None,
            })
        }
    }

    /// 解析亮度参数
    /// 格式: bright/70
    pub fn parse_bright(params: &str) -> Option<BrightnessContrastParams> {
        let brightness = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness,
            contrast: 0,
        })
    }

    /// 解析对比度参数
    /// 格式: contrast/-50
    pub fn parse_contrast(params: &str) -> Option<BrightnessContrastParams> {
        let contrast = params.trim().parse().ok()?;
        Some(BrightnessContrastParams {
            brightness: 0,
            contrast,
        })
    }

    /// 解析锐化参数
    /// 格式: sharpen/70
    pub fn parse_sharpen(params: &str) -> Option<SharpenParams> {
        let amount = params.trim().parse().ok()?;
        Some(SharpenParams { amount })
    }

    /// 解析灰度图参数
    /// 格式: grayscale/1
    pub fn parse_grayscale(params: &str) -> Option<bool> {
        match params.trim() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    // ==================== 生成方法 ====================

    /// 生成缩放参数字符串
    fn generate_resize(params: &ResizeParams) -> String {
        match params.mode {
            ResizeMode::Ratio => {
                if let Some(ratio) = params.ratio {
                    let percent = (ratio * 100.0) as u32;
                    format!("thumbnail/!{}p", percent)
                } else {
                    String::new()
                }
            }
            ResizeMode::Fixed => {
                format!(
                    "thumbnail/!{}x{}",
                    params.width.unwrap_or(0),
                    params.height.unwrap_or(0)
                )
            }
            ResizeMode::Fill => {
                format!(
                    "thumbnail/!{}x{}r",
                    params.width.unwrap_or(0),
                    params.height.unwrap_or(0)
                )
            }
            ResizeMode::Pad => {
                let mut result = format!(
                    "thumbnail/{}x{}",
                    params.width.unwrap_or(0),
                    params.height.unwrap_or(0)
                );
                result.push_str("/pad/1");
                if let Some(ref color) = params.fill_color {
                    result.push_str(&format!("/color/{}", color));
                }
                result
            }
            ResizeMode::Fit => {
                let w = params.width.unwrap_or(0);
                let h = params.height.unwrap_or(0);

                if let Some(limit) = params.limit {
                    if limit {
                        // 仅缩小
                        format!("thumbnail/{}x{}>", w, h)
                    } else {
                        // 仅放大
                        format!("thumbnail/{}x{}<", w, h)
                    }
                } else if w > 0 && h > 0 {
                    format!("thumbnail/{}x{}", w, h)
                } else if w > 0 {
                    format!("thumbnail/{}x", w)
                } else if h > 0 {
                    format!("thumbnail/x{}", h)
                } else {
                    String::new()
                }
            }
            ResizeMode::Crop => {
                format!(
                    "thumbnail/{}x{}",
                    params.width.unwrap_or(0),
                    params.height.unwrap_or(0)
                )
            }
        }
    }

    /// 生成裁剪参数字符串
    fn generate_crop(params: &CropParams) -> String {
        if params.circle == Some(true) {
            // 内切圆裁剪
            let radius = params.width.min(params.height) / 2;
            return format!("iradius/{}", radius);
        }

        let gravity = if let Some(grid_pos) = params.grid_position {
            format!("/gravity/{}", Self::generate_gravity(grid_pos))
        } else {
            String::new()
        };

        if params.x == 0 && params.y == 0 {
            // 缩放裁剪
            format!("crop/{}x{}{}", params.width, params.height, gravity)
        } else {
            // 自定义裁剪
            format!(
                "cut/{}x{}x{}x{}{}",
                params.width, params.height, params.x, params.y, gravity
            )
        }
    }

    /// 生成九宫格位置字符串
    fn generate_gravity(pos: GridPosition) -> &'static str {
        match pos {
            GridPosition::TopLeft => "northwest",
            GridPosition::Top => "north",
            GridPosition::TopRight => "northeast",
            GridPosition::Left => "west",
            GridPosition::Center => "center",
            GridPosition::Right => "east",
            GridPosition::BottomLeft => "southwest",
            GridPosition::Bottom => "south",
            GridPosition::BottomRight => "southeast",
        }
    }

    /// 生成旋转参数字符串
    fn generate_rotate(params: &RotateParams) -> String {
        format!("rotate/{}", params.angle)
    }

    /// 生成质量参数字符串
    fn generate_quality(params: &QualityParams) -> String {
        if let Some(value) = params.value {
            format!("quality/{}", value)
        } else if let Some(relative) = params.relative {
            format!("quality/Q{}", relative)
        } else {
            String::new()
        }
    }

    /// 生成格式转换参数字符串
    fn generate_format(params: &FormatParams) -> String {
        let mut result = format!("format/{}", params.format);
        if params.progressive == Some(true) {
            result.push_str("/interlace/1");
        }
        result
    }

    /// 生成模糊参数字符串
    fn generate_blur(params: &BlurParams) -> String {
        if let Some(sigma) = params.sigma {
            format!("blur/{}x{}", params.radius, sigma)
        } else {
            format!("blur/{}", params.radius)
        }
    }

    /// 生成锐化参数字符串
    fn generate_sharpen(params: &SharpenParams) -> String {
        format!("sharpen/{}", params.amount)
    }

    /// 生成亮度参数字符串
    fn generate_brightness(params: &BrightnessContrastParams) -> String {
        format!("bright/{}", params.brightness)
    }

    /// 生成对比度参数字符串
    fn generate_contrast(params: &BrightnessContrastParams) -> String {
        format!("contrast/{}", params.contrast)
    }

    /// 生成灰度图参数字符串
    fn generate_grayscale(value: bool) -> String {
        format!("grayscale/{}", if value { 1 } else { 0 })
    }

    /// 生成圆角参数字符串
    fn generate_rounded_corners(params: &RoundedCornersParams) -> String {
        if let Some(radius) = params.radius {
            format!("rradius/{}", radius)
        } else if let Some(radius_x) = params.radius_x {
            let radius_y = params.radius_y.unwrap_or(radius_x);
            format!("rradius/{}x{}", radius_x, radius_y)
        } else {
            String::new()
        }
    }
}

impl Default for TencentProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapter for TencentProvider {
    fn parse(&self, url: &str) -> Result<ImageProcessParams, ParseError> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| ParseError::InvalidUrl(format!("{}: {}", url, e)))?;

        let query_pairs: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

        let mut result = ImageProcessParams::default();

        // 检查 imageMogr2 参数
        if let Some(mogr2_param) = query_pairs.get("imageMogr2") {
            // 腾讯云格式: thumbnail/!50p/blur/8x5/grayscale/1
            // 每个操作名后跟参数，用 / 分隔
            let parts: Vec<&str> = mogr2_param.split('/').collect();
            let mut i = 0;

            while i < parts.len() {
                let operation = parts[i].trim();
                if operation.is_empty() {
                    i += 1;
                    continue;
                }

                match operation {
                    "thumbnail" => {
                        if i + 1 < parts.len() {
                            if let Some(resize) = Self::parse_resize(parts[i + 1]) {
                                result.resize = Some(resize);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "crop" => {
                        if i + 1 < parts.len() {
                            if let Some(crop) = Self::parse_crop(parts[i + 1]) {
                                result.crop = Some(crop);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "cut" => {
                        if i + 1 < parts.len() {
                            if let Some(crop) = Self::parse_cut(parts[i + 1]) {
                                result.crop = Some(crop);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "iradius" => {
                        if i + 1 < parts.len() {
                            if let Some(crop) = Self::parse_iradius(parts[i + 1]) {
                                result.crop = Some(crop);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "rradius" => {
                        if i + 1 < parts.len() {
                            if let Some(rounded) = Self::parse_rradius(parts[i + 1]) {
                                result.rounded_corners = Some(rounded);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "scrop" => {
                        if i + 1 < parts.len() {
                            if let Some(crop) = Self::parse_scrop(parts[i + 1]) {
                                result.crop = Some(crop);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "rcrop" => {
                        if i + 1 < parts.len() {
                            if let Some(crop) = Self::parse_rcrop(parts[i + 1]) {
                                result.crop = Some(crop);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "rotate" => {
                        if i + 1 < parts.len() {
                            if let Some(rotate) = Self::parse_rotate(parts[i + 1]) {
                                result.rotate = Some(rotate);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "quality" => {
                        if i + 1 < parts.len() {
                            if let Some(quality) = Self::parse_quality(parts[i + 1]) {
                                result.quality = Some(quality);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "format" => {
                        // format 操作可能带多个参数: format/png/interlace/1
                        if i + 1 < parts.len() {
                            // 收集 format 后面的所有参数，直到遇到下一个操作名
                            let mut format_parts = vec![parts[i + 1]];
                            let mut j = i + 2;
                            while j < parts.len() {
                                let next_part = parts[j].trim();
                                // 检查是否是已知操作名
                                match next_part {
                                    "thumbnail" | "crop" | "cut" | "iradius" | "rradius" | "scrop"
                                    | "rcrop" | "rotate" | "quality" | "blur" | "bright"
                                    | "contrast" | "sharpen" | "grayscale" => break,
                                    _ => {
                                        format_parts.push(next_part);
                                        j += 1;
                                    }
                                }
                            }
                            if let Some(format) = Self::parse_format(&format_parts.join("/")) {
                                result.format = Some(format);
                            }
                            i = j;
                        } else {
                            i += 1;
                        }
                    }
                    "blur" => {
                        if i + 1 < parts.len() {
                            if let Some(blur) = Self::parse_blur(parts[i + 1]) {
                                result.blur = Some(blur);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "bright" => {
                        if i + 1 < parts.len() {
                            if let Some(bright) = Self::parse_bright(parts[i + 1]) {
                                let existing = result.brightness_contrast.take();
                                result.brightness_contrast = Some(BrightnessContrastParams {
                                    brightness: bright.brightness,
                                    contrast: existing.map(|e| e.contrast).unwrap_or(0),
                                });
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "contrast" => {
                        if i + 1 < parts.len() {
                            if let Some(contrast) = Self::parse_contrast(parts[i + 1]) {
                                let existing = result.brightness_contrast.take();
                                result.brightness_contrast = Some(BrightnessContrastParams {
                                    brightness: existing.map(|e| e.brightness).unwrap_or(0),
                                    contrast: contrast.contrast,
                                });
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "sharpen" => {
                        if i + 1 < parts.len() {
                            if let Some(sharpen) = Self::parse_sharpen(parts[i + 1]) {
                                result.sharpen = Some(sharpen);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "grayscale" => {
                        if i + 1 < parts.len() {
                            if let Some(gray) = Self::parse_grayscale(parts[i + 1]) {
                                result.grayscale = Some(gray);
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    _ => {
                        result.extra.insert(operation.to_string(), operation.to_string().into());
                        i += 1;
                    }
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

        if let Some(ref rotate) = params.rotate {
            operations.push(Self::generate_rotate(rotate));
        }

        if let Some(ref quality) = params.quality {
            let quality_str = Self::generate_quality(quality);
            if !quality_str.is_empty() {
                operations.push(quality_str);
            }
        }

        if let Some(ref format) = params.format {
            operations.push(Self::generate_format(format));
        }

        if let Some(ref blur) = params.blur {
            operations.push(Self::generate_blur(blur));
        }

        if let Some(ref sharpen) = params.sharpen {
            operations.push(Self::generate_sharpen(sharpen));
        }

        if let Some(ref bc) = params.brightness_contrast {
            if bc.brightness != 0 {
                operations.push(Self::generate_brightness(bc));
            }
            if bc.contrast != 0 {
                operations.push(Self::generate_contrast(bc));
            }
        }

        if let Some(grayscale) = params.grayscale {
            operations.push(Self::generate_grayscale(grayscale));
        }

        if operations.is_empty() {
            return Err(GenerateError::EmptyParams);
        }

        let params_str = format!("imageMogr2/{}", operations.join("/"));

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
        ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resize_basic() {
        let result = TencentProvider::parse_resize("100x200");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.width, Some(100));
        assert_eq!(resize.height, Some(200));
        assert_eq!(resize.mode, ResizeMode::Fit);
    }

    #[test]
    fn test_parse_resize_width_only() {
        let result = TencentProvider::parse_resize("100x");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.width, Some(100));
        assert_eq!(resize.height, None);
    }

    #[test]
    fn test_parse_resize_height_only() {
        let result = TencentProvider::parse_resize("x200");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.width, None);
        assert_eq!(resize.height, Some(200));
    }

    #[test]
    fn test_parse_resize_percentage() {
        let result = TencentProvider::parse_resize("!50p");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.ratio, Some(0.5));
        assert_eq!(resize.mode, ResizeMode::Fit);
    }

    #[test]
    fn test_parse_resize_percentage_wh() {
        let result = TencentProvider::parse_resize("!50x50p");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.width, Some(50));
        assert_eq!(resize.height, Some(50));
    }

    #[test]
    fn test_parse_resize_shrink_only() {
        let result = TencentProvider::parse_resize("100x200>");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.limit, Some(true));
    }

    #[test]
    fn test_parse_resize_enlarge_only() {
        let result = TencentProvider::parse_resize("100x200<");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.limit, Some(false));
    }

    #[test]
    fn test_parse_resize_force() {
        let result = TencentProvider::parse_resize("100x200!");
        assert!(result.is_some());
        let resize = result.unwrap();
        assert_eq!(resize.mode, ResizeMode::Fixed);
    }

    #[test]
    fn test_parse_crop_with_gravity() {
        let result = TencentProvider::parse_crop("300x400/gravity/center");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.width, 300);
        assert_eq!(crop.height, 400);
        assert_eq!(crop.grid_position, Some(GridPosition::Center));
    }

    #[test]
    fn test_parse_cut() {
        let result = TencentProvider::parse_cut("600x600x100x10");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.width, 600);
        assert_eq!(crop.height, 600);
        assert_eq!(crop.x, 100);
        assert_eq!(crop.y, 10);
    }

    #[test]
    fn test_parse_iradius() {
        let result = TencentProvider::parse_iradius("200");
        assert!(result.is_some());
        let crop = result.unwrap();
        assert_eq!(crop.width, 400);
        assert_eq!(crop.height, 400);
        assert_eq!(crop.circle, Some(true));
    }

    #[test]
    fn test_parse_rradius() {
        let result = TencentProvider::parse_rradius("100");
        assert!(result.is_some());
        let rounded = result.unwrap();
        assert_eq!(rounded.radius, Some(100));
    }

    #[test]
    fn test_parse_blur() {
        let result = TencentProvider::parse_blur("8x5");
        assert!(result.is_some());
        let blur = result.unwrap();
        assert_eq!(blur.radius, 8);
        assert_eq!(blur.sigma, Some(5));
    }

    #[test]
    fn test_parse_format_with_interlace() {
        let result = TencentProvider::parse_format("png/interlace/1");
        assert!(result.is_some());
        let format = result.unwrap();
        assert_eq!(format.format, ImageFormat::Png);
        assert_eq!(format.progressive, Some(true));
    }

    #[test]
    fn test_generate_resize_percentage() {
        let params = ResizeParams {
            width: None,
            height: None,
            mode: ResizeMode::Ratio,
            limit: None,
            ratio: Some(0.5),
            percentage: None,
            longest_side: None,
            shortest_side: None,
            fill_color: None,
        };
        let result = TencentProvider::generate_resize(&params);
        assert_eq!(result, "thumbnail/!50p");
    }

    #[test]
    fn test_generate_crop_circle() {
        let params = CropParams {
            x: 0,
            y: 0,
            width: 400,
            height: 400,
            circle: Some(true),
            grid_position: Some(GridPosition::Center),
        };
        let result = TencentProvider::generate_crop(&params);
        assert_eq!(result, "iradius/200");
    }

    #[test]
    fn test_validate_blur_range() {
        let provider = TencentProvider::new();
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
    fn test_full_parse_with_multiple_operations() {
        let provider = TencentProvider::new();
        let url = "https://example.com/image.jpg?imageMogr2=thumbnail/!50p/blur/8x5/grayscale/1";

        // 先直接测试 parse_resize
        let resize_result = TencentProvider::parse_resize("!50p");
        eprintln!("parse_resize('!50p') = {:?}", resize_result);
        assert!(resize_result.is_some(), "parse_resize should return Some");

        let result = provider.parse(url);
        eprintln!("parse result = {:?}", result);
        assert!(result.is_ok());
        let params = result.unwrap();
        eprintln!("resize = {:?}", params.resize);
        eprintln!("blur = {:?}", params.blur);
        eprintln!("grayscale = {:?}", params.grayscale);
        assert!(params.resize.is_some());
        assert!(params.blur.is_some());
        assert_eq!(params.grayscale, Some(true));
    }
}
