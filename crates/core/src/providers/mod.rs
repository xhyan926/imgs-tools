pub mod aliyun;
pub mod huawei;
pub mod qiniu;
pub mod tencent;
pub mod volcengine;

use std::collections::HashSet;

pub use aliyun::AliyunProvider;
pub use huawei::HuaweiProvider;
pub use qiniu::QiniuProvider;
pub use tencent::TencentProvider;
pub use volcengine::VolcengineProvider;

use crate::types::{
    GenerateError, GenerateResult, ImageProcessParams, Operation, ParseError, Provider,
    ValidationError,
};

/// 厂商适配器接口
pub trait ProviderAdapter: Send + Sync {
    /// 解析 URL 参数为中间格式
    fn parse(&self, url: &str) -> Result<ImageProcessParams, ParseError>;

    /// 从中间格式生成 URL 参数字符串
    /// 返回 (生成的参数字符串, 不兼容操作列表)
    fn generate(&self, params: &ImageProcessParams) -> Result<GenerateResult, GenerateError>;

    /// 验证参数是否支持
    fn validate(&self, params: &ImageProcessParams) -> Result<(), ValidationError>;

    /// 获取厂商名称
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 获取支持的操作列表
    fn supported_operations(&self) -> HashSet<Operation>;
}

/// 根据厂商枚举创建对应的适配器
pub fn create_adapter(provider: Provider) -> Box<dyn ProviderAdapter> {
    match provider {
        Provider::Aliyun => Box::new(AliyunProvider::new()),
        Provider::Tencent => Box::new(TencentProvider::new()),
        Provider::Huawei => Box::new(HuaweiProvider::new()),
        Provider::Qiniu => Box::new(QiniuProvider::new()),
        Provider::Volcengine => Box::new(VolcengineProvider::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_adapter() {
        let adapters = vec![
            (Provider::Aliyun, "AliyunProvider"),
            (Provider::Tencent, "TencentProvider"),
            (Provider::Huawei, "HuaweiProvider"),
            (Provider::Qiniu, "QiniuProvider"),
            (Provider::Volcengine, "VolcengineProvider"),
        ];

        for (provider, _name) in adapters {
            let adapter = create_adapter(provider);
            assert!(!adapter.supported_operations().is_empty());
        }
    }
}
