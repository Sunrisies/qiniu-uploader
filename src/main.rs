use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local};
use log::info;
use qiniu_sdk::upload::{
    AutoUploader, AutoUploaderObjectParams, UploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use qiniu_uploader::{TOKEN_EXPIRY_SECS, copy_file, load_config, setup_logging};
use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};

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
