use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct DevConfig{
    pub(crate) remotes: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub(crate) struct UserConfig {
    
}