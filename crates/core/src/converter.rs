use crate::types::*;
use crate::providers::create_adapter;
use std::collections::{HashMap, HashSet};

/// 厂商功能支持矩阵
pub struct FeatureMatrix {
    /// 各厂商支持的操作
    supports: HashMap<Provider, HashSet<Operation>>,
}

impl FeatureMatrix {
    pub fn new() -> Self {
        let mut supports = HashMap::new();

        // 所有厂商都支持基本操作
        let common_ops = {
            let mut ops = HashSet::new();
            ops.insert(Operation::Resize);
            ops.insert(Operation::Crop);
            ops.insert(Operation::Rotate);
            ops.insert(Operation::Quality);
            ops.insert(Operation::Format);
            ops
        };

        for provider in [
            Provider::Aliyun,
            Provider::Tencent,
            Provider::Huawei,
            Provider::Qiniu,
            Provider::Volcengine,
        ] {
            supports.insert(provider, common_ops.clone());
        }

        Self { supports }
    }

    /// 检查目标厂商是否支持某操作
    pub fn is_supported(&self, provider: Provider, operation: &Operation) -> bool {
        self.supports
            .get(&provider)
            .map(|ops| ops.contains(operation))
            .unwrap_or(false)
    }

    /// 获取替代操作（暂不实现，返回 None）
    pub fn get_alternative(&self, _provider: Provider, _operation: &Operation) -> Option<Operation> {
        None
    }
}

impl Default for FeatureMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// 缩放模式降级映射
/// 当目标厂商不支持某模式时，按此顺序降级
pub fn get_resize_mode_fallback(mode: ResizeMode, target: Provider) -> Option<ResizeMode> {
    match (mode, target) {
        // Pad 模式在火山引擎中降级为 Fit
        (ResizeMode::Pad, Provider::Volcengine) => Some(ResizeMode::Fit),
        // Ratio 模式在七牛云中降级为 Fit
        (ResizeMode::Ratio, Provider::Qiniu) => Some(ResizeMode::Fit),
        // 其他情况保持不变，在 generate 时处理
        _ => Some(mode),
    }
}

/// 参数转换器
pub struct Converter {
    mode: ConversionMode,
    feature_matrix: FeatureMatrix,
}

impl Converter {
    /// 创建新的转换器
    pub fn new(mode: ConversionMode) -> Self {
        Self {
            mode,
            feature_matrix: FeatureMatrix::new(),
        }
    }

    /// 使用默认模式（Lenient）创建转换器
    pub fn default_mode() -> Self {
        Self::new(ConversionMode::Lenient)
    }

    /// 设置转换模式
    pub fn with_mode(mut self, mode: ConversionMode) -> Self {
        self.mode = mode;
        self
    }

    /// 厂商 A 转厂商 B
    pub fn convert(
        &self,
        url: &str,
        from: Provider,
        to: Provider,
    ) -> Result<ConversionResult, ConversionError> {
        // 1. 解析源 URL
        let source_adapter = create_adapter(from);
        let params = source_adapter.parse(url)?;

        // 2. 检查不兼容操作
        let warnings = self.check_compatibility(&params, from, to)?;

        // 3. 应用降级策略
        let adapted = self.adapt_params(params, from, to)?;

        // 4. 生成目标 URL
        let target_adapter = create_adapter(to);
        let result = target_adapter.generate(&adapted)?;

        // 5. 构建转换结果
        let conversion_warnings: Vec<Warning> = warnings
            .into_iter()
            .map(|w| Warning {
                operation: w,
                reason: "目标厂商不完全支持此操作".to_string(),
                suggestion: None,
            })
            .collect();

        let dropped_from_generation = result.dropped;

        Ok(ConversionResult {
            url: self.build_url(url, &result.params)?,
            success: dropped_from_generation.is_empty() && conversion_warnings.is_empty(),
            warnings: conversion_warnings,
            dropped: dropped_from_generation,
        })
    }

    /// 检查兼容性
    fn check_compatibility(
        &self,
        params: &ImageProcessParams,
        _from: Provider,
        to: Provider,
    ) -> Result<Vec<String>, ConversionError> {
        let mut warnings = Vec::new();

        // 检查缩放模式兼容性
        if let Some(ref resize) = params.resize {
            if let Some(fallback) = get_resize_mode_fallback(resize.mode, to) {
                if fallback != resize.mode {
                    let msg = format!(
                        "缩放模式 {:?} 在 {:?} 中不支持，将降级为 {:?}",
                        resize.mode, to, fallback
                    );
                    match self.mode {
                        ConversionMode::Strict => {
                            return Err(ConversionError::IncompatibleParameters(msg));
                        }
                        ConversionMode::Report | ConversionMode::Lenient => {
                            warnings.push(msg);
                        }
                    }
                }
            }
        }

        // 检查 Pad 模式在火山引擎中的兼容性
        if let Some(ref resize) = params.resize {
            if resize.mode == ResizeMode::Pad && to == Provider::Volcengine {
                let msg = "目标厂商不支持 Pad 填充模式".to_string();
                match self.mode {
                    ConversionMode::Strict => {
                        return Err(ConversionError::IncompatibleParameters(msg));
                    }
                    ConversionMode::Report | ConversionMode::Lenient => {
                        warnings.push(msg);
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// 参数适配（降级处理）
    fn adapt_params(
        &self,
        mut params: ImageProcessParams,
        _from: Provider,
        to: Provider,
    ) -> Result<ImageProcessParams, ConversionError> {
        // 处理缩放模式降级
        if let Some(ref mut resize) = params.resize {
            if let Some(fallback_mode) = get_resize_mode_fallback(resize.mode, to) {
                resize.mode = fallback_mode;
            }
        }

        Ok(params)
    }

    /// 构建 URL
    fn build_url(&self, original_url: &str, params: &str) -> Result<String, ConversionError> {
        let parsed = url::Url::parse(original_url)
            .map_err(|e| ConversionError::UrlBuildError(e.to_string()))?;

        // 移除原有的图片处理参数
        let mut query_pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(k, _)| {
                !matches!(
                    k.as_ref(),
                    "x-oss-process"
                        | "imageMogr2"
                        | "imageView2"
                        | "imageslim"
                        | "x-image-process"
                        | "image_process"
                )
            })
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        // 添加新的参数
        // 简单处理：根据参数格式判断参数名
        let param_name = if params.starts_with("x-oss-process=") {
            "x-oss-process"
        } else if params.starts_with("imageMogr2") {
            "imageMogr2"
        } else if params.starts_with("imageView2") {
            "imageView2"
        } else if params.starts_with("x-image-process=") {
            "x-image-process"
        } else if params.starts_with("image_process=") {
            "image_process"
        } else {
            // 默认情况，参数可能已经包含参数名，直接使用
            query_pairs.push((params.to_string(), String::new()));
            return Ok(parsed.to_string());
        };

        // 提取参数值
        let param_value = params
            .strip_prefix(param_name)
            .and_then(|s| s.strip_prefix('='))
            .unwrap_or(params);

        query_pairs.push((param_name.to_string(), param_value.to_string()));

        // 构建新 URL
        let new_url = url::Url::parse_with_params(
            &format!(
                "{}://{}{}",
                parsed.scheme(),
                parsed.host().unwrap().to_string(),
                parsed.path()
            ),
            &query_pairs,
        )
        .map_err(|e| ConversionError::UrlBuildError(e.to_string()))?;

        Ok(new_url.to_string())
    }

    /// 批量转换
    pub fn convert_batch(
        &self,
        urls: &[String],
        from: Provider,
        to: Provider,
    ) -> Vec<ConversionResult> {
        urls.iter()
            .map(|url| {
                self.convert(url, from, to).unwrap_or_else(|e| ConversionResult {
                    url: format!("[转换失败] {}", e),
                    success: false,
                    warnings: vec![],
                    dropped: vec![],
                })
            })
            .collect()
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::default_mode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_mode_fallback() {
        // Pad 模式在火山引擎中应降级为 Fit
        assert_eq!(
            get_resize_mode_fallback(ResizeMode::Pad, Provider::Volcengine),
            Some(ResizeMode::Fit)
        );

        // Ratio 模式在七牛云中应降级为 Fit
        assert_eq!(
            get_resize_mode_fallback(ResizeMode::Ratio, Provider::Qiniu),
            Some(ResizeMode::Fit)
        );
    }

    #[test]
    fn test_converter_create() {
        let converter = Converter::default_mode();
        assert_eq!(converter.mode, ConversionMode::Lenient);
    }

    #[test]
    fn test_feature_matrix() {
        let matrix = FeatureMatrix::new();
        assert!(matrix.is_supported(Provider::Aliyun, &Operation::Resize));
        assert!(matrix.is_supported(Provider::Tencent, &Operation::Crop));
    }

    #[test]
    fn test_convert_aliyun_to_tencent() {
        let converter = Converter::default_mode();
        let url = "https://example.com/image.jpg?x-oss-process=image/resize,w_100,h_200/quality,q_90";

        let result = converter.convert(url, Provider::Aliyun, Provider::Tencent);
        assert!(result.is_ok());

        let conversion_result = result.unwrap();
        assert!(!conversion_result.url.is_empty());
        assert!(conversion_result.url.contains("imageMogr2"));
    }

    #[test]
    fn test_convert_with_strict_mode() {
        let converter = Converter::new(ConversionMode::Strict);
        // 测试严格模式下不兼容参数的处理
        let url = "https://example.com/image.jpg?x-oss-process=image/resize,w_100,h_200";

        // 这个应该成功，因为 resize 是所有厂商都支持的
        let result = converter.convert(url, Provider::Aliyun, Provider::Tencent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_url() {
        let converter = Converter::default_mode();
        let original_url = "https://example.com/image.jpg?other=param&x-oss-process=image/resize,w_100";
        let params = "x-image-process=image/resize,w_100";

        let result = converter.build_url(original_url, params);
        assert!(result.is_ok());

        let new_url = result.unwrap();
        assert!(!new_url.contains("x-oss-process"));
        assert!(new_url.contains("x-image-process"));
        assert!(new_url.contains("other=param"));
    }
}
