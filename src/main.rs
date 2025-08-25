// use crate::
use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local};
use log::{LevelFilter, info};
use qiniu_sdk::upload::{
    AutoUploader, AutoUploaderObjectParams, UploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use qiniu_uploader::copy_file;
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};
const LOG_FILE: &str = "log.log";
const CONFIG_FILE: &str = "config.json";
const TOKEN_EXPIRY_SECS: u64 = 3600;

fn setup_logging() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(fern::log_file(LOG_FILE)?)
        .apply()
        .context("Failed to initialize logging")?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct Config {
    access_key: String,
    secret_key: String,
    bucket_name: String,
    base_url: String,         // 七牛云存储的域名
    base_dir: Option<String>, // 上传文件的根目录
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
    // if cfg.base_dir.is_empty() {
    //     bail!("Base dir cannot be empty");
    // }
    Ok(cfg)
}

fn main() -> Result<()> {
    setup_logging()?;
    let cfg = load_config()?;
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
        .file_name(&file_name)
        .build();

    let response = uploader
        .upload_path(&file_path, params)
        .context("Failed to upload file")?;
    let key = response["key"].as_str().unwrap_or_default();
    let final_url = format!("{}/{}", &cfg.base_url, key);
    println!("{}", final_url); // 供外部脚本捕获

    // base_dir 存在才复制到本地
    let dest_path = cfg
        .base_dir
        .as_ref()
        .map(|dir| PathBuf::from(format!("{}\\{}", dir, file_name)));

    if let Some(path) = dest_path {
        match copy_file(&file_path, &path) {
            Ok(_) => info!("本地备份成功: {}", path.display()),
            Err(e) => info!("本地备份失败: {}", e),
        }
    } else {
        info!("未配置 base_dir，仅云端存储");
    }
    info!("上传成功，文件链接: {}", final_url); // 记录日志
    Ok(())
}
