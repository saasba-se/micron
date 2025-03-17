//! Data initialization procedures.
//!
//! Both the app config and the `/content` directory can contain entries
//! descibing items expected to exist after the application is started. This
//! module addresses the need for a streamlined way of converting relevant
//! data into initial application state.

use std::io::Read;

use crate::{db::Collectable, Config, Database, Error, ErrorKind, Image, Post, Result, User};

/// Initializes database state based on entries found at default locations.
// TODO: provide a config switch for re-initialization of existing items
pub fn initialize(config: &Config, db: &Database) -> Result<()> {
    users(config, db)?;
    posts(config, db)?;
    blog_posts(config, db)?;
    Ok(())
}

/// Initializes users from entries found in the configuration.
pub fn users(config: &Config, db: &Database) -> Result<()> {
    for user_ in &config.users {
        let mut user = user_.user.clone();
        if let Some(avatar_path) = &user_.avatar {
            let mut bytes = vec![];
            // println!("cwd: {:?}", std::env::current_dir());
            // println!("avatar_path: {avatar_path}");
            std::fs::File::open(avatar_path)?.read_to_end(&mut bytes);
            // let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
            //     .map_err(|e| ErrorKind::Other(e.to_string()).into())?;
            let image = Image::new(bytes);
            db.set(&image)?;
            user.avatar = image.id;
        } else {
            user.avatar = crate::user::new_avatar_image(&db)?;
        }

        // If the user already exists, update them with the information
        // in the config.
        if let Some(mut existing_user) = db
            .get_collection::<crate::User>()?
            .into_iter()
            .find(|u| u.email == user.email)
        {
            // TODO: implement merging strategy
            existing_user.is_admin = user.is_admin;

            db.set(&existing_user)?;
        } else {
            db.set(&user)?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostIntermediate {
    #[serde(flatten)]
    post: Post,
    author: String,
    image: String,
}

/// Initializes posts as found in the `content/posts` directory.
pub fn posts(config: &Config, db: &Database) -> Result<()> {
    posts_raw(Post::get_collection_name(), "content/posts", config, db)
}

/// Same as `posts` but searches the `content/blog` directory and stores the
/// items in the `blog_posts` collection.
pub fn blog_posts(config: &Config, db: &Database) -> Result<()> {
    posts_raw("blog_posts", "content/blog", config, db)
}

/// Initializes posts.
///
/// # Regenerated ids
///
/// Posts are re-created with each call to this function. This means they end
/// up having different unique ids on each initialization. Because of that it's
/// best to refer to them using slugs or titles instead.
pub fn posts_raw(
    collection_name: &str,
    content_dir: &str,
    config: &Config,
    db: &Database,
) -> Result<()> {
    // Clear the posts on each initialization
    // TODO: perhaps a config entry should be used to control this behavior
    db.clear_at(collection_name)?;

    let posts = match std::fs::read_dir(content_dir) {
        Ok(p) => p,
        Err(e) => {
            // If the directory doesn't exist we expect this function to just
            // return normally.
            return Ok(());
        }
    };
    for post in posts {
        let post = post?.path();
        if post.is_dir() {
            continue;
        };
        let post = std::fs::read_to_string(post)?;
        // TODO: don't require the frontmatter delineator to be present
        let frontmatter = post.splitn(3, "---").collect::<Vec<&str>>()[1];
        let mut post_: PostIntermediate = serde_yaml::from_str(frontmatter)?;
        // TODO convert markdown to html on post load
        // let mut markdown_opts = markdown::Options::gfm();
        // markdown_opts.parse.constructs.frontmatter = true;
        // let html = markdown::to_html_with_options(&post, &markdown_opts)
        //     .map_err(|e| anyhow::Error::msg(e.to_string()))?;
        post_.post.markdown = post;
        let users = db.get_collection::<User>()?;
        if let Some(user) = users.iter().find(|u| u.email == post_.author) {
            post_.post.owner = user.id;
        }
        // load and set post image
        let mut bytes = vec![];
        std::fs::File::open(post_.image)?.read_to_end(&mut bytes);
        let image = Image::new(bytes);
        db.set(&image)?;
        post_.post.image = image.id;

        db.set_at(collection_name, &post_.post)?;
    }
    Ok(())
}
