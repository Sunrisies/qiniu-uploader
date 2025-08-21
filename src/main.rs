use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local};
use log::{LevelFilter, Metadata, Record, info};
use qiniu_sdk::upload::{
    AutoUploader, AutoUploaderObjectParams, UploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use serde::Deserialize;
use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
    time::SystemTime,
};

/// 简单的文件日志器
struct FileLogger {
    file: std::sync::Mutex<std::fs::File>,
}
const LOG_FILE: &str = "uploader.log";
const CONFIG_FILE: &str = "config.json";
const TOKEN_EXPIRY_SECS: u64 = 3600;

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let now = SystemTime::now();

        let line = format!(
            "[{}] {} - {}\n",
            humantime::format_rfc3339_seconds(now),
            record.level(),
            record.args()
        );
        let _ = self.file.lock().unwrap().write_all(line.as_bytes());
    }

    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)?;
    log::set_boxed_logger(Box::new(FileLogger {
        file: std::sync::Mutex::new(file),
    }))
    .map(|()| log::set_max_level(LevelFilter::Info))?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Config {
    access_key: String,
    secret_key: String,
    bucket_name: String,
    base_url: String, // 七牛云存储的域名
}

fn load_config() -> Result<Config> {
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

fn main() -> Result<()> {
    let cfg = load_config()?;
    // 1. 初始化日志到文件
    init_logger().unwrap();
    // 获取待上传文件路径
    let file_path = env::args().nth(1).context("usage: uploader <file-path>")?;
    let file_path = PathBuf::from(&file_path);
    if !file_path.is_file() {
        bail!("{:?} is not a file", file_path);
    }
    let credential = Credential::new(&cfg.access_key, &cfg.secret_key);
    let upload_manager = UploadManager::builder(UploadTokenSigner::new_credential_provider(
        credential,
        &cfg.bucket_name,
        Duration::from_secs(TOKEN_EXPIRY_SECS),
    ))
    .build();

    let uploader: AutoUploader = upload_manager.auto_uploader();
    let now = Local::now();
    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_os_string().into_string().ok())
        .unwrap_or_else(|| "fallback.png".to_string());
    let dir = format!(
        "image/{}/{:02}/{:02}/{}",
        now.year(),
        now.month(),
        now.day(),
        file_name
    );
    println!("dir:{:?}", &dir);

    let params = AutoUploaderObjectParams::builder()
        .object_name(dir)
        .file_name(file_name)
        .build();

    let ps = uploader.upload_path(file_path, params)?;
    println!("ps:{:?}", ps);
    let key = ps["key"].as_str().unwrap_or_default();
    let final_url = format!("{}/{}", &cfg.base_url, key);
    println!("{}", final_url); // 供外部脚本捕获
    Ok(())
}
