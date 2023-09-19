// use std::error::Error;
use std::fmt;

pub struct PrimitiveError {
    source: Box<dyn std::error::Error>,
}

impl fmt::Debug for PrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for PrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl<T> From<T> for PrimitiveError
where
    T: std::error::Error + 'static,
{
    fn from(value: T) -> Self {
        Self {
            source: Box::from(value),
        }
    }
}
