use anyhow::{Error, Result};
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

/// Stores provided token in the target location where it can be read.
pub async fn store_token(token: &String) -> Result<()> {
    let dirs = match directories::ProjectDirs::from("", "", "micron") {
        Some(dirs) => dirs,
        None => return Err(Error::msg("couldn't access default directory on system")),
    };

    let path = dirs.config_dir();
    create_dir_all(path).await.expect("failed creating dirs");

    let mut file = File::create(path.join("token"))
        .await
        .expect("failed creating token file");
    file.write_all(token.as_bytes()).await;

    Ok(())
}

pub async fn retrieve_token() -> Result<Uuid> {
    let dirs = match directories::ProjectDirs::from("", "", "micron") {
        Some(dirs) => dirs,
        None => return Err(Error::msg("couldn't access default directory on system")),
    };

    let path = dirs.config_dir();
    create_dir_all(path).await.expect("failed creating dirs");

    let mut file = File::open(path.join("token"))
        .await
        .map_err(|e| Error::msg("token not found"))?;
    let mut string = String::new();
    file.read_to_string(&mut string)
        .await
        .expect("failed reading from token file");
    let token = string.parse()?;
    Ok(token)
}
