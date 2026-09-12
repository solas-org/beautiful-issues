use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Ticket {
    name: String,
    description: String,
    category: String,
    error_name: String,
    error_message: String,
    attached_files: Vec<PathBuf>
}