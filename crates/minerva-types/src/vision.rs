use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFrame {
    pub width: u32,
    pub height: u32,
    /// Raw RGBA pixel buffer. Early iterations may keep PNG bytes instead.
    pub data: Vec<u8>,
    pub captured_at: DateTime<Utc>,
}

impl ImageFrame {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            data: Vec::new(),
            captured_at: Utc::now(),
        }
    }
}
