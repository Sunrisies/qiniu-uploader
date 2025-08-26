use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{env, fs};
pub const LOG_FILE: &str = "log.log";
pub const CONFIG_FILE: &str = "config.json";
pub const TOKEN_EXPIRY_SECS: u64 = 3600;
#[derive(Deserialize, Debug)]
pub struct Config {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub base_url: String,         // 七牛云存储的域名
    pub base_dir: Option<String>, // 上传文件的根目录
}

pub fn load_config() -> Result<Config> {
    let exe_dir = env::current_exe()
        .context("failed to get exe path")?
        .parent()
        .unwrap()
        .to_path_buf();
    let cfg_path = exe_dir.join(CONFIG_FILE);

    let cfg_str =
        fs::read_to_string(&cfg_path).with_context(|| format!("cannot read {:?}", cfg_path))?;
    let cfg: Config = serde_json::from_str(&cfg_str)?;

    // 验证配置
    if cfg.access_key.is_empty() {
        bail!("Access key cannot be empty");
    }
    if cfg.secret_key.is_empty() {
        bail!("Secret key cannot be empty");
    }
    if cfg.bucket_name.is_empty() {
        bail!("Bucket name cannot be empty");
    }
    if cfg.base_url.is_empty() {
        bail!("Base url cannot be empty");
    }
    Ok(cfg)
}
