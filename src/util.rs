use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

#[derive(Debug, Error)]
pub enum RangeParseError {
    #[error("Invalid range format: {0}")]
    InvalidFormat(String),
    #[error("Invalid start value: {0}")]
    InvalidStart(String),
    #[error("Invalid end value: {0}")]
    InvalidEnd(String),
}

impl<T: Clone> Clone for Range<T> {
    fn clone(&self) -> Self {
        Range {
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl<T> FromStr for Range<T>
where
    T: FromStr,
{
    type Err = RangeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("..").collect();
        if parts.len() != 2 {
            return Err(RangeParseError::InvalidFormat(s.to_string()));
        }

        let start = parts[0]
            .parse::<T>()
            .map_err(|_| RangeParseError::InvalidStart(parts[0].to_string()))?;
        let end = parts[1]
            .parse::<T>()
            .map_err(|_| RangeParseError::InvalidEnd(parts[1].to_string()))?;

        Ok(Range { start, end })
    }
}
