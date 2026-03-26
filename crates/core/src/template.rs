use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

/// 转换模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionTemplate {
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: Option<String>,
    /// 默认源厂商
    pub default_from: String,
    /// 默认目标厂商
    pub default_to: String,
    /// 默认转换模式
    pub default_mode: String,
    /// 预设参数（可选）
    pub preset_params: Option<String>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 转换历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionHistory {
    /// 记录 ID
    pub id: String,
    /// 源 URL
    pub source_url: String,
    /// 转换后的 URL
    pub converted_url: String,
    /// 源厂商
    pub from: String,
    /// 目标厂商
    pub to: String,
    /// 转换时间
    pub timestamp: String,
    /// 是否成功
    pub success: bool,
    /// 警告数量
    pub warning_count: usize,
    /// 忽略操作数量
    pub dropped_count: usize,
}

/// 模板管理器
pub struct TemplateManager {
    templates_dir: PathBuf,
    templates: HashMap<String, ConversionTemplate>,
}

impl TemplateManager {
    /// 创建新的模板管理器
    pub fn new(templates_dir: PathBuf) -> Result<Self, io::Error> {
        // 确保模板目录存在
        fs::create_dir_all(&templates_dir)?;

        let mut manager = Self {
            templates_dir,
            templates: HashMap::new(),
        };

        // 加载现有模板
        manager.load_templates()?;

        Ok(manager)
    }

    /// 获取默认模板目录
    pub fn default_dir() -> PathBuf {
        let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push(".imgs-tools");
        dir.push("templates");
        dir
    }

    /// 加载所有模板
    fn load_templates(&mut self) -> Result<(), io::Error> {
        self.templates.clear();

        if !self.templates_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.templates_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(template) = serde_json::from_str::<ConversionTemplate>(&content) {
                    let name = template.name.clone();
                    self.templates.insert(name, template);
                }
            }
        }

        Ok(())
    }

    /// 获取所有模板
    pub fn get_templates(&self) -> Vec<ConversionTemplate> {
        self.templates.values().cloned().collect()
    }

    /// 获取指定模板
    pub fn get_template(&self, name: &str) -> Option<&ConversionTemplate> {
        self.templates.get(name)
    }

    /// 保存模板
    pub fn save_template(&mut self, template: ConversionTemplate) -> Result<(), io::Error> {
        let filename = format!("{}.json", template.name);
        let path = self.templates_dir.join(filename);

        let content = serde_json::to_string_pretty(&template)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(&path, content)?;

        // 更新内存中的模板
        self.templates.insert(template.name.clone(), template);

        Ok(())
    }

    /// 删除模板
    pub fn delete_template(&mut self, name: &str) -> Result<(), io::Error> {
        let filename = format!("{}.json", name);
        let path = self.templates_dir.join(filename);

        if path.exists() {
            fs::remove_file(&path)?;
        }

        self.templates.remove(name);

        Ok(())
    }

    /// 创建默认模板
    pub fn create_default_templates(&mut self) -> Result<(), io::Error> {
        let templates = vec![
            ConversionTemplate {
                name: "阿里云转腾讯云".to_string(),
                description: Some("将阿里云 OSS 图片处理参数转换为腾讯云数据万象格式".to_string()),
                default_from: "aliyun".to_string(),
                default_to: "tencent".to_string(),
                default_mode: "lenient".to_string(),
                preset_params: None,
                created_at: chrono_timestamp(),
                updated_at: chrono_timestamp(),
            },
            ConversionTemplate {
                name: "通用质量压缩".to_string(),
                description: Some("降低图片质量以减小文件大小".to_string()),
                default_from: "aliyun".to_string(),
                default_to: "tencent".to_string(),
                default_mode: "lenient".to_string(),
                preset_params: Some("quality,q_80".to_string()),
                created_at: chrono_timestamp(),
                updated_at: chrono_timestamp(),
            },
        ];

        for template in templates {
            self.save_template(template)?;
        }

        Ok(())
    }
}

/// 历史记录管理器
pub struct HistoryManager {
    history_file: PathBuf,
    history: Vec<ConversionHistory>,
    max_entries: usize,
}

impl HistoryManager {
    /// 创建新的历史记录管理器
    pub fn new(history_file: PathBuf, max_entries: usize) -> Result<Self, io::Error> {
        // 确保目录存在
        if let Some(parent) = history_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut manager = Self {
            history_file,
            history: Vec::new(),
            max_entries,
        };

        manager.load_history()?;

        Ok(manager)
    }

    /// 获取默认历史文件路径
    pub fn default_file() -> PathBuf {
        let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push(".imgs-tools");
        dir.push("history.json");
        dir
    }

    /// 加载历史记录
    fn load_history(&mut self) -> Result<(), io::Error> {
        if !self.history_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.history_file)?;
        if let Ok(history) = serde_json::from_str::<Vec<ConversionHistory>>(&content) {
            self.history = history;
        }

        Ok(())
    }

    /// 保存历史记录
    fn save_history(&self) -> Result<(), io::Error> {
        let content = serde_json::to_string_pretty(&self.history)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fs::write(&self.history_file, content)
    }

    /// 添加历史记录
    pub fn add_record(&mut self, record: ConversionHistory) -> Result<(), io::Error> {
        self.history.push(record);

        // 限制历史记录数量
        if self.history.len() > self.max_entries {
            self.history = self.history.split_off(self.history.len() - self.max_entries);
        }

        self.save_history()
    }

    /// 获取所有历史记录
    pub fn get_history(&self) -> &[ConversionHistory] {
        &self.history
    }

    /// 获取最近的历史记录
    pub fn get_recent(&self, count: usize) -> &[ConversionHistory] {
        let start = if self.history.len() > count {
            self.history.len() - count
        } else {
            0
        };
        &self.history[start..]
    }

    /// 按厂商筛选历史记录
    pub fn filter_by_provider(&self, provider: &str) -> Vec<&ConversionHistory> {
        self.history
            .iter()
            .filter(|h| h.from == provider || h.to == provider)
            .collect()
    }

    /// 清空历史记录
    pub fn clear(&mut self) -> Result<(), io::Error> {
        self.history.clear();
        self.save_history()
    }

    /// 删除指定时间之前的记录
    pub fn delete_before(&mut self, timestamp: &str) -> Result<usize, io::Error> {
        let original_len = self.history.len();
        self.history.retain(|h| h.timestamp.as_str() > timestamp);
        let deleted = original_len - self.history.len();
        self.save_history()?;
        Ok(deleted)
    }
}

/// 生成时间戳字符串
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager() {
        let temp_dir = std::env::temp_dir().join("imgs-tools-test");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut manager = TemplateManager::new(temp_dir.clone()).unwrap();
        assert!(manager.get_templates().is_empty());

        manager.create_default_templates().unwrap();
        assert_eq!(manager.get_templates().len(), 2);

        let template = manager.get_template("阿里云转腾讯云");
        assert!(template.is_some());

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_history_manager() {
        let temp_file = std::env::temp_dir().join("imgs-tools-history-test.json");
        let _ = fs::remove_file(&temp_file);

        let mut manager = HistoryManager::new(temp_file.clone(), 100).unwrap();
        assert!(manager.get_history().is_empty());

        let record = ConversionHistory {
            id: "1".to_string(),
            source_url: "https://example.com/img.jpg".to_string(),
            converted_url: "https://example.com/img_processed.jpg".to_string(),
            from: "aliyun".to_string(),
            to: "tencent".to_string(),
            timestamp: chrono_timestamp(),
            success: true,
            warning_count: 0,
            dropped_count: 0,
        };

        manager.add_record(record.clone()).unwrap();
        assert_eq!(manager.get_history().len(), 1);

        fs::remove_file(temp_file).ok();
    }
}
