use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

pub const MAX_PAGE_SIZE: u32 = 1000;
pub const MAX_WINDOW_SIZE: u32 = 10000;
pub const MAX_DIFF_BYTES: u64 = 10 * 1024 * 1024; // 10MB
pub const MAX_DIFF_HUNKS: u32 = 10000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize(NonZeroU32);

impl PageSize {
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

impl TryFrom<u32> for PageSize {
    type Error = BoundsError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(BoundsError::TooSmall);
        }
        if value > MAX_PAGE_SIZE {
            return Err(BoundsError::TooLarge);
        }
        Ok(PageSize(NonZeroU32::new(value).unwrap()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize(NonZeroU32);

impl WindowSize {
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

impl TryFrom<u32> for WindowSize {
    type Error = BoundsError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(BoundsError::TooSmall);
        }
        if value > MAX_WINDOW_SIZE {
            return Err(BoundsError::TooLarge);
        }
        Ok(WindowSize(NonZeroU32::new(value).unwrap()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxBytes(u64);

impl MaxBytes {
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for MaxBytes {
    type Error = BoundsError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(BoundsError::TooSmall);
        }
        if value > MAX_DIFF_BYTES {
            return Err(BoundsError::TooLarge);
        }
        Ok(MaxBytes(value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxHunks(u32);

impl MaxHunks {
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for MaxHunks {
    type Error = BoundsError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(BoundsError::TooSmall);
        }
        if value > MAX_DIFF_HUNKS {
            return Err(BoundsError::TooLarge);
        }
        Ok(MaxHunks(value))
    }
}

#[derive(Debug, Clone)]
pub enum BoundsError {
    TooSmall,
    TooLarge,
}

impl std::fmt::Display for BoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundsError::TooSmall => write!(f, "value too small"),
            BoundsError::TooLarge => write!(f, "value too large"),
        }
    }
}

impl std::error::Error for BoundsError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor(String);

impl Cursor {
    pub fn initial() -> Self {
        Cursor(String::new())
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

impl From<String> for Cursor {
    fn from(data: String) -> Self {
        Cursor(data)
    }
}

impl AsRef<str> for Cursor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
