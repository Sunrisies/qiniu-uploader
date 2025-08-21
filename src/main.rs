use qiniu_sdk::upload::{
    AutoUploader, AutoUploaderObjectParams, UploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Config {
    access_key: String,
    secret_key: String,
    bucket_name: String,
}

use std::time::Duration;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_key = "xxx";
    let secret_key = "xxx";
    let bucket_name = "xxx";
    let object_name = "\\12\\12\\image-20250821135211465.png";
    let credential = Credential::new(access_key, secret_key);
    let upload_manager = UploadManager::builder(UploadTokenSigner::new_credential_provider(
        credential,
        bucket_name,
        Duration::from_secs(3600),
    ))
    .build();
    let uploader: AutoUploader = upload_manager.auto_uploader();
    let params = AutoUploaderObjectParams::builder()
        .object_name(object_name)
        .file_name(object_name)
        .build();
    uploader.upload_path("C:\\Users\\hover\\AppData\\Roaming\\Typora\\typora-user-images\\image-20250821135211465.png", params)?;
    Ok(())
}
