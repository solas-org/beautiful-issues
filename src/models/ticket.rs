use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    name: String,
    description: String,
    category: String,
    error_name: String,
    error_message: String
}