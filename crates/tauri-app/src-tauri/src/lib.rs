use serde::{Deserialize, Serialize};
use tauri::State;

use imgs_tools_core::{Converter, ConversionMode, Provider};

/// 应用状态
pub struct AppState {
    converter: Converter,
}

/// 转换请求
#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub url: String,
    pub from: String,
    pub to: String,
    pub mode: String,
}

/// 转换响应
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub url: String,
    pub success: bool,
    pub warnings: Vec<WarningData>,
    pub dropped: Vec<DroppedData>,
}

#[derive(Debug, Serialize)]
pub struct WarningData {
    pub operation: String,
    pub reason: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DroppedData {
    pub name: String,
    pub original_value: String,
    pub reason: String,
}

/// 验证请求
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub url: String,
    pub provider: String,
}

/// 验证响应
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub params: Option<ValidatedParams>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidatedParams {
    pub resize: Option<ResizeData>,
    pub crop: Option<CropData>,
    pub rotate: Option<RotateData>,
    pub quality: Option<QualityData>,
    pub format: Option<FormatData>,
}

#[derive(Debug, Serialize)]
pub struct ResizeData {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct CropData {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize)]
pub struct RotateData {
    pub angle: u32,
}

#[derive(Debug, Serialize)]
pub struct QualityData {
    pub value: Option<u8>,
    pub relative: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct FormatData {
    pub format: String,
}

/// 支持的操作响应
#[derive(Debug, Serialize)]
pub struct FeaturesResponse {
    pub provider: String,
    pub operations: Vec<OperationInfo>,
}

#[derive(Debug, Serialize)]
pub struct OperationInfo {
    pub name: String,
    pub supported: bool,
}

// Tauri 命令

#[tauri::command]
pub fn convert_url(
    request: ConvertRequest,
    state: State<AppState>,
) -> Result<ConvertResponse, String> {
    let from_provider = request
        .from
        .parse::<Provider>()
        .map_err(|e| format!("无效的源厂商: {}", e))?;

    let to_provider = request
        .to
        .parse::<Provider>()
        .map_err(|e| format!("无效的目标厂商: {}", e))?;

    let mode = request
        .mode
        .parse::<ConversionMode>()
        .unwrap_or(ConversionMode::Lenient);

    let result = state
        .converter
        .convert(&request.url, from_provider, to_provider)
        .map_err(|e| e.to_string())?;

    Ok(ConvertResponse {
        url: result.url,
        success: result.success,
        warnings: result
            .warnings
            .into_iter()
            .map(|w| WarningData {
                operation: w.operation,
                reason: w.reason,
                suggestion: w.suggestion,
            })
            .collect(),
        dropped: result
            .dropped
            .into_iter()
            .map(|d| DroppedData {
                name: d.name,
                original_value: d.original_value,
                reason: d.reason,
            })
            .collect(),
    })
}

#[tauri::command]
pub fn validate_url(request: ValidateRequest) -> Result<ValidateResponse, String> {
    let provider = request
        .provider
        .parse::<Provider>()
        .map_err(|e| format!("无效的厂商: {}", e))?;

    use imgs_tools_core::providers::create_adapter;

    let adapter = create_adapter(provider);

    match adapter.parse(&request.url) {
        Ok(params) => {
            let validated_params = ValidatedParams {
                resize: params.resize.map(|r| ResizeData {
                    width: r.width,
                    height: r.height,
                    mode: r.mode.to_string(),
                }),
                crop: params.crop.map(|c| CropData {
                    x: c.x,
                    y: c.y,
                    width: c.width,
                    height: c.height,
                }),
                rotate: params.rotate.map(|r| RotateData { angle: r.angle }),
                quality: params.quality.map(|q| QualityData {
                    value: q.value,
                    relative: q.relative,
                }),
                format: params.format.map(|f| FormatData {
                    format: f.format.to_string(),
                }),
            };

            Ok(ValidateResponse {
                valid: true,
                params: Some(validated_params),
                error: None,
            })
        }
        Err(e) => Ok(ValidateResponse {
            valid: false,
            params: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub fn get_features(provider: String) -> Result<FeaturesResponse, String> {
    let provider = provider
        .parse::<Provider>()
        .map_err(|e| format!("无效的厂商: {}", e))?;

    use imgs_tools_core::providers::create_adapter;
    use imgs_tools_core::Operation;

    let adapter = create_adapter(provider);
    let supported_ops = adapter.supported_operations();

    let all_ops = vec![
        ("缩放", Operation::Resize),
        ("裁剪", Operation::Crop),
        ("旋转", Operation::Rotate),
        ("质量", Operation::Quality),
        ("格式转换", Operation::Format),
        ("渐进式加载", Operation::Progressive),
    ];

    let operations = all_ops
        .into_iter()
        .map(|(name, op)| OperationInfo {
            name: name.to_string(),
            supported: supported_ops.contains(&op),
        })
        .collect();

    Ok(FeaturesResponse {
        provider: provider.to_string(),
        operations,
    })
}

#[tauri::command]
pub fn get_providers() -> Vec<String> {
    vec![
        "aliyun".to_string(),
        "tencent".to_string(),
        "huawei".to_string(),
        "qiniu".to_string(),
        "volcengine".to_string(),
    ]
}

#[tauri::command]
pub fn get_conversion_modes() -> Vec<ModeInfo> {
    vec![
        ModeInfo {
            name: "strict".to_string(),
            display_name: "严格模式".to_string(),
            description: "遇到不兼容参数时报错".to_string(),
        },
        ModeInfo {
            name: "lenient".to_string(),
            display_name: "宽松模式".to_string(),
            description: "跳过不兼容参数，尽可能转换".to_string(),
        },
        ModeInfo {
            name: "report".to_string(),
            display_name: "报告模式".to_string(),
            description: "返回转换结果和警告信息".to_string(),
        },
    ]
}

#[derive(Debug, Serialize)]
pub struct ModeInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
}

/// Tauri 初始化
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            converter: Converter::default_mode(),
        })
        .invoke_handler(tauri::generate_handler![
            convert_url,
            validate_url,
            get_features,
            get_providers,
            get_conversion_modes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_providers() {
        let providers = get_providers();
        assert_eq!(providers.len(), 5);
        assert!(providers.contains(&"aliyun".to_string()));
    }

    #[test]
    fn test_get_conversion_modes() {
        let modes = get_conversion_modes();
        assert_eq!(modes.len(), 3);
        assert_eq!(modes[0].name, "strict");
    }
}
