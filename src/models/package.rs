use crate::commands::enums::Framework;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) framework: Framework,
    pub(crate) version: String,
    pub(crate) dependencies: Vec<String>,
}
