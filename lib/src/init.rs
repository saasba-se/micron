//! Data initialization procedures.
//!
//! As the configuration can contain entries descibing items expected to exist
//! after the application is started, there is a need for a streamlined way of
//! converting relevant config into initial database state.

use std::io::Read;

use crate::{Config, Database, Error, ErrorKind, Image, Result};

/// Initializes database state based on relevant config entries.
pub fn initialize(config: &Config, db: &Database) -> Result<()> {
    for user_ in &config.users {
        let mut user = user_.user.clone();
        if let Some(avatar_path) = &user_.avatar {
            let mut bytes = vec![];
            println!("cwd: {:?}", std::env::current_dir());
            println!("avatar_path: {avatar_path}");
            std::fs::File::open(avatar_path)?.read_to_end(&mut bytes);
            // let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
            //     .map_err(|e| ErrorKind::Other(e.to_string()).into())?;
            let image = Image::new(bytes);
            db.set(&image)?;
            user.avatar = image.id;
        }
        if db
            .get_collection::<crate::User>()?
            .iter()
            .any(|u| u.email == user.email)
        {
            continue;
        }
        db.set(&user)?;
    }
    Ok(())
}
