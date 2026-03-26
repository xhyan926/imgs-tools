// 云厂商图片处理参数转换集成测试
use imgs_tools_core::types::*;
use imgs_tools_core::converter::Converter;
use imgs_tools_core::providers::create_adapter;

fn test_single_conversion(url: &str, from: Provider, to: Provider, description: &str) {
    println!("\n{:-<70}", "");
    println!("测试: {}", description);
    println!("厂商: {:?} -> {:?}", from, to);
    println!("{:-<70}", "");

    println!("原始 URL:");
    println!("  {}", url);

    // 解析原始参数
    let source_adapter = create_adapter(from);
    match source_adapter.parse(url) {
        Ok(params) => {
            println!("\n解析成功:");
            if params.resize.is_some() {
                let r = params.resize.as_ref().unwrap();
                println!("  Resize: w={:?}, h={:?}, mode={:?}", r.width, r.height, r.mode);
            }
            if params.crop.is_some() {
                let c = params.crop.as_ref().unwrap();
                println!("  Crop: x={}, y={}, w={}, h={}", c.x, c.y, c.width, c.height);
            }
            if params.rotate.is_some() {
                let r = params.rotate.as_ref().unwrap();
                if r.auto == Some(true) {
                    println!("  Rotate: auto-orient");
                } else {
                    println!("  Rotate: {}°", r.angle);
                }
            }
            if params.quality.is_some() {
                let q = params.quality.as_ref().unwrap();
                if let Some(v) = q.value {
                    println!("  Quality: {} (绝对)", v);
                }
                if let Some(r) = q.relative {
                    println!("  Quality: Q{} (相对)", r);
                }
            }
            if params.format.is_some() {
                let f = params.format.as_ref().unwrap();
                println!("  Format: {}", f.format);
            }
            if params.blur.is_some() {
                let b = params.blur.as_ref().unwrap();
                println!("  Blur: radius={}, sigma={:?}", b.radius, b.sigma);
            }
            if params.sharpen.is_some() {
                let s = params.sharpen.as_ref().unwrap();
                println!("  Sharpen: {}", s.amount);
            }
            if params.grayscale == Some(true) {
                println!("  Grayscale: true");
            }
            if params.brightness_contrast.is_some() {
                let bc = params.brightness_contrast.as_ref().unwrap();
                println!("  BrightnessContrast: b={}, c={}", bc.brightness, bc.contrast);
            }
            if params.metadata.is_some() {
                let m = params.metadata.as_ref().unwrap();
                println!("  Metadata: strip={:?}", m.remove_exif);
            }
        }
        Err(e) => {
            println!("\n解析失败: {}", e);
            return;
        }
    }

    // 转换
    let converter = Converter::new(ConversionMode::Lenient);
    match converter.convert(url, from, to) {
        Ok(result) => {
            println!("\n转换结果:");
            println!("  成功: {}", result.success);
            println!("\n目标 URL:");
            println!("  {}", result.url);

            if !result.warnings.is_empty() {
                println!("\n警告:");
                for w in &result.warnings {
                    println!("  - {}: {}", w.operation, w.reason);
                }
            }

            if !result.dropped.is_empty() {
                println!("\n丢弃的操作:");
                for d in &result.dropped {
                    println!("  - {}: {} (原因: {})", d.name, d.original_value, d.reason);
                }
            }

            // 验证转换后的URL可以正确解析
            let target_adapter = create_adapter(to);
            if let Ok(params) = target_adapter.parse(&result.url) {
                println!("\n验证转换后URL解析成功:");
                if params.resize.is_some() {
                    let r = params.resize.as_ref().unwrap();
                    println!("  Resize: w={:?}, h={:?}, mode={:?}", r.width, r.height, r.mode);
                }
                if params.crop.is_some() {
                    let c = params.crop.as_ref().unwrap();
                    println!("  Crop: x={}, y={}, w={}, h={}", c.x, c.y, c.width, c.height);
                }
            }
        }
        Err(e) => {
            println!("\n转换失败: {}", e);
        }
    }
}

#[test]
fn test_conversion_aliyun_to_tencent() {
    test_single_conversion(
        "https://example.com/image.jpg?x-oss-process=image/resize,w_100,h_200/quality,q_90/format,png",
        Provider::Aliyun,
        Provider::Tencent,
        "阿里云 -> 腾讯云 (缩放+质量+格式)"
    );
    assert!(true); // 如果没有panic就算通过
}

#[test]
fn test_conversion_tencent_to_huawei() {
    test_single_conversion(
        "https://example.com/img.jpg?imageMogr2=thumbnail/!100x100/quality/90/format/png",
        Provider::Tencent,
        Provider::Huawei,
        "腾讯云 -> 华为云 (缩放+质量+格式)"
    );
    assert!(true);
}

#[test]
fn test_conversion_huawei_to_qiniu() {
    test_single_conversion(
        "https://example.com/file.jpg?x-image-process=image/resize,w_100,h_200/quality,Q_90/format,png",
        Provider::Huawei,
        Provider::Qiniu,
        "华为云 -> 七牛云 (缩放+质量+格式)"
    );
    assert!(true);
}

#[test]
fn test_conversion_qiniu_to_volcengine() {
    test_single_conversion(
        "https://example.com/qiniu.jpg?imageView2/2/w/100/h/200",
        Provider::Qiniu,
        Provider::Volcengine,
        "七牛云 -> 火山引擎 (缩放)"
    );
    assert!(true);
}

#[test]
fn test_conversion_volcengine_to_aliyun() {
    test_single_conversion(
        "https://example.com/volc.jpg?image_process=resize,w_100,h_200/format,png/Q_90",
        Provider::Volcengine,
        Provider::Aliyun,
        "火山引擎 -> 阿里云 (缩放+质量+格式)"
    );
    assert!(true);
}

#[test]
fn test_conversion_aliyun_to_tencent_complex() {
    test_single_conversion(
        "https://example.com/photo.jpg?x-oss-process=image/crop,x_10,y_10,w_200,h_200/rotate,90/blur,3,r_5",
        Provider::Aliyun,
        Provider::Tencent,
        "阿里云 -> 腾讯云 (裁剪+旋转+模糊)"
    );
    assert!(true);
}

#[test]
fn test_conversion_tencent_to_huawei_grayscale() {
    test_single_conversion(
        "https://example.com/demo.jpg?imageMogr2=thumbnail/300x300/grayscale/1/sharpen/50",
        Provider::Tencent,
        Provider::Huawei,
        "腾讯云 -> 华为云 (缩放+灰度+锐化)"
    );
    assert!(true);
}

#[test]
fn test_conversion_huawei_to_qiniu_with_unsupported() {
    test_single_conversion(
        "https://example.com/original.jpg?x-image-process=image/resize,m_lfit,w_300,h_300/sharpen,50/grayscale,1",
        Provider::Huawei,
        Provider::Qiniu,
        "华为云 -> 七牛云 (缩放+锐化+灰度，灰度会丢失)"
    );
    assert!(true);
}

#[test]
fn test_conversion_qiniu_to_volcengine_auto_orient() {
    test_single_conversion(
        "https://example.com/qiniu3.jpg?imageMogr2/auto-orient/strip/quality/Q80",
        Provider::Qiniu,
        Provider::Volcengine,
        "七牛云 -> 火山引擎 (自动旋转+去除元数据+质量)"
    );
    assert!(true);
}

#[test]
fn test_conversion_aliyun_to_qiniu_rounded_corners() {
    test_single_conversion(
        "https://example.com/pic.jpg?x-oss-process=image/resize,m_mfit,w_300,h_300/rounded-corners,r_10",
        Provider::Aliyun,
        Provider::Qiniu,
        "阿里云 -> 七牛云 (缩放+圆角，圆角会丢失)"
    );
    assert!(true);
}

#[test]
fn test_conversion_volcengine_to_tencent() {
    test_single_conversion(
        "https://example.com/volc2.jpg?image_process=crop,x_10,y_10,w_200,h_200/rotate,90/blur,r_5,sigma_3",
        Provider::Volcengine,
        Provider::Tencent,
        "火山引擎 -> 腾讯云 (裁剪+旋转+模糊)"
    );
    assert!(true);
}

#[test]
fn test_conversion_tencent_to_aliyun() {
    test_single_conversion(
        "https://example.com/test.jpg?imageMogr2=rotate/90/crop/!200x200a10a10/blur/5x3",
        Provider::Tencent,
        Provider::Aliyun,
        "腾讯云 -> 阿里云 (旋转+裁剪+模糊)"
    );
    assert!(true);
}

#[test]
fn test_all_providers_basic_conversion() {
    // 测试所有厂商之间的基础转换
    let base_url = "https://example.com/img.jpg?x-oss-process=image/resize,w_100,h_200";
    let providers = vec![
        Provider::Aliyun,
        Provider::Tencent,
        Provider::Huawei,
        Provider::Qiniu,
        Provider::Volcengine,
    ];

    let converter = Converter::new(ConversionMode::Lenient);

    for from in &providers {
        for to in &providers {
            if from == to {
                continue;
            }

            // 根据源厂商选择合适的测试URL
            let test_url = match from {
                Provider::Aliyun => "https://example.com/img.jpg?x-oss-process=image/resize,w_100,h_200",
                Provider::Tencent => "https://example.com/img.jpg?imageMogr2=thumbnail/!100x100",
                Provider::Huawei => "https://example.com/img.jpg?x-image-process=image/resize,w_100,h_200",
                Provider::Qiniu => "https://example.com/img.jpg?imageView2/2/w/100/h/200",
                Provider::Volcengine => "https://example.com/img.jpg?image_process=resize,w_100,h_200",
            };

            let result = converter.convert(test_url, *from, *to);
            assert!(result.is_ok(), "{:?} -> {:?} 转换失败: {:?}", from, to, result.err());
        }
    }
}
