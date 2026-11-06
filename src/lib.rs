pub mod capture;
pub mod formatter;
pub mod inventory;
pub mod workbench;

use sha2::{Digest, Sha256};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
