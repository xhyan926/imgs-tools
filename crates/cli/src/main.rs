use clap::{Parser, Subcommand};
use imgs_tools_core::{ConversionMode, Converter, Provider};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// 云厂商图片处理参数转换器
#[derive(Parser, Debug)]
#[command(name = "imgconv")]
#[command(about = "在不同云厂商间转换图片处理参数", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 转换单个 URL 的图片处理参数
    Convert {
        /// 源 URL
        url: String,

        /// 源云厂商
        #[arg(short = 'f', long = "from")]
        from: Provider,

        /// 目标云厂商
        #[arg(short = 't', long = "to")]
        to: Provider,

        /// 转换模式
        #[arg(short = 'm', long = "mode", default_value = "lenient")]
        mode: ConversionMode,

        /// 输出格式
        #[arg(short = 'o', long = "output", default_value = "text")]
        output: OutputFormat,
    },

    /// 批量转换 URL（从文件读取）
    Batch {
        /// 输入文件路径（每行一个 URL）
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// 源云厂商
        #[arg(short = 'f', long = "from")]
        from: Provider,

        /// 目标云厂商
        #[arg(short = 't', long = "to")]
        to: Provider,

        /// 转换模式
        #[arg(short = 'm', long = "mode", default_value = "lenient")]
        mode: ConversionMode,

        /// 输出格式
        #[arg(short = 'o', long = "output", default_value = "text")]
        output: OutputFormat,
    },

    /// 验证 URL 格式
    Validate {
        /// 要验证的 URL
        url: String,

        /// 云厂商
        #[arg(short = 'p', long = "provider")]
        provider: Provider,
    },

    /// 查看厂商支持的操作
    Features {
        /// 云厂商
        #[arg(short = 'p', long = "provider")]
        provider: Provider,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Table,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            url,
            from,
            to,
            mode,
            output,
        } => {
            let converter = Converter::new(mode);
            match converter.convert(&url, from, to) {
                Ok(result) => print_conversion_result(&result, output),
                Err(e) => eprintln!("转换失败: {}", e),
            }
        }

        Commands::Batch {
            input,
            from,
            to,
            mode,
            output,
        } => {
            let urls = read_urls_from_file(&input);
            let converter = Converter::new(mode);
            let results = converter.convert_batch(&urls, from, to);
            print_batch_results(&results, output);
        }

        Commands::Validate { url, provider } => {
            validate_url(&url, provider);
        }

        Commands::Features { provider } => {
            show_features(provider);
        }
    }
}

fn read_urls_from_file(path: &PathBuf) -> Vec<String> {
    let file = fs::File::open(path).unwrap_or_else(|e| {
        eprintln!("无法打开文件 {}: {}", path.display(), e);
        std::process::exit(1);
    });

    BufReader::new(file)
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn print_conversion_result(result: &imgs_tools_core::ConversionResult, output: OutputFormat) {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        OutputFormat::Table => {
            print_conversion_table(result);
        }
        OutputFormat::Text => {
            println!("转换后的 URL:");
            println!("  {}", result.url);
            println!();

            if result.success {
                println!("状态: 成功");
            } else {
                println!("状态: 部分成功");
            }

            if !result.warnings.is_empty() {
                println!();
                println!("警告:");
                for warning in &result.warnings {
                    println!("  - {}: {}", warning.operation, warning.reason);
                    if let Some(suggestion) = &warning.suggestion {
                        println!("    建议: {}", suggestion);
                    }
                }
            }

            if !result.dropped.is_empty() {
                println!();
                println!("忽略的操作:");
                for dropped in &result.dropped {
                    println!("  - {}: {}", dropped.name, dropped.reason);
                    println!("    原始值: {}", dropped.original_value);
                }
            }
        }
    }
}

fn print_conversion_table(result: &imgs_tools_core::ConversionResult) {
    use tabled::{settings::style::Style, Table, Tabled};

    #[derive(Tabled)]
    struct ConversionRow {
        #[tabled(rename = "字段")]
        field: String,
        #[tabled(rename = "值")]
        value: String,
    }

    let mut rows = vec![
        ConversionRow {
            field: "URL".to_string(),
            value: result.url.clone(),
        },
        ConversionRow {
            field: "状态".to_string(),
            value: if result.success {
                "成功".to_string()
            } else {
                "部分成功".to_string()
            },
        },
    ];

    if !result.warnings.is_empty() {
        let warnings = result
            .warnings
            .iter()
            .map(|w| format!("{}: {}", w.operation, w.reason))
            .collect::<Vec<_>>()
            .join("; ");
        rows.push(ConversionRow {
            field: "警告".to_string(),
            value: warnings,
        });
    }

    if !result.dropped.is_empty() {
        let dropped = result
            .dropped
            .iter()
            .map(|d| format!("{}: {}", d.name, d.reason))
            .collect::<Vec<_>>()
            .join("; ");
        rows.push(ConversionRow {
            field: "忽略的操作".to_string(),
            value: dropped,
        });
    }

    let table = Table::new(rows).with(Style::modern()).to_string();
    println!("{}", table);
}

fn print_batch_results(results: &[imgs_tools_core::ConversionResult], output: OutputFormat) {
    use tabled::{settings::style::Style, Table, Tabled};

    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(results).unwrap());
        }
        OutputFormat::Table => {
            #[derive(Tabled)]
            struct BatchRow<'a> {
                #[tabled(rename = "索引")]
                index: usize,
                #[tabled(rename = "URL")]
                url: String,
                #[tabled(rename = "状态")]
                status: &'a str,
                #[tabled(rename = "警告数")]
                warnings: usize,
                #[tabled(rename = "忽略数")]
                dropped: usize,
            }

            let rows: Vec<BatchRow> = results
                .iter()
                .enumerate()
                .map(|(i, result)| BatchRow {
                    index: i + 1,
                    url: result.url.chars().take(50).collect::<String>(),
                    status: if result.success { "成功" } else { "失败" },
                    warnings: result.warnings.len(),
                    dropped: result.dropped.len(),
                })
                .collect();

            let table = Table::new(rows).with(Style::modern()).to_string();
            println!("{}", table);
        }
        OutputFormat::Text => {
            println!("批量转换结果:");
            println!();
            for (i, result) in results.iter().enumerate() {
                println!("[{}] {}", i + 1, result.url);
                if !result.success {
                    println!("    状态: 失败");
                }
                if !result.warnings.is_empty() {
                    println!("    警告: {} 个", result.warnings.len());
                }
                if !result.dropped.is_empty() {
                    println!("    忽略: {} 个操作", result.dropped.len());
                }
                println!();
            }
        }
    }
}

fn validate_url(url: &str, provider: Provider) {
    use imgs_tools_core::providers::create_adapter;

    let adapter = create_adapter(provider);
    match adapter.parse(url) {
        Ok(params) => {
            println!("URL 格式有效");
            println!();

            println!("解析结果:");
            if let Some(resize) = params.resize {
                println!("  缩放:");
                println!("    宽度: {:?}", resize.width);
                println!("    高度: {:?}", resize.height);
                println!("    模式: {:?}", resize.mode);
            }
            if let Some(crop) = params.crop {
                println!("  裁剪:");
                println!("    位置: ({}, {})", crop.x, crop.y);
                println!("    尺寸: {}x{}", crop.width, crop.height);
            }
            if let Some(rotate) = params.rotate {
                println!("  旋转: {} 度", rotate.angle);
            }
            if let Some(quality) = params.quality {
                println!("  质量: {:?}", quality.value);
            }
            if let Some(format) = params.format {
                println!("  格式: {}", format.format);
            }
        }
        Err(e) => {
            eprintln!("URL 验证失败: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_features(provider: Provider) {
    use imgs_tools_core::providers::create_adapter;

    let adapter = create_adapter(provider);
    let operations = adapter.supported_operations();

    println!("{} 支持的操作:", provider);
    println!();

    use imgs_tools_core::Operation;
    let ops = vec![
        ("缩放", Operation::Resize),
        ("裁剪", Operation::Crop),
        ("旋转", Operation::Rotate),
        ("质量", Operation::Quality),
        ("格式转换", Operation::Format),
        ("渐进式加载", Operation::Progressive),
    ];

    for (name, op) in ops {
        let supported = operations.contains(&op);
        println!("  [{}] {}", if supported { "✓" } else { " " }, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("table").unwrap(), OutputFormat::Table);
        assert!(OutputFormat::from_str("invalid").is_err());
    }
}
