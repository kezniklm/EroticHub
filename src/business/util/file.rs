use std::path::Path;
use tokio::fs::create_dir_all;

pub async fn create_dir_if_not_exist(path: String) -> anyhow::Result<()> {
    let path = Path::new(path.as_str());
    if path.exists() {
        return Ok(());
    }
    create_dir_all(path).await?;
    Ok(())
}
