// Internationalization support

use serde::{Deserialize, Serialize};

/// List of available languages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Language {
    English,
}
