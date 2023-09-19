use std::{error, fmt};

pub enum ErrorKind {}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for ErrorKind {}

pub struct RuntimeError {
    context: Option<ErrorContext>,
    source: Box<dyn std::error::Error>,
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(context) = &self.context {
            write!(
                f,
                "context: {:?}\nsource: {}",
                context,
                self.source.to_string()
            )?;
        } else {
            write!(f, "source: {}", self.source)?;
        }
        Ok(())
    }
}

impl<T> From<T> for RuntimeError
where
    T: error::Error + Send + 'static,
{
    #[track_caller]
    fn from(value: T) -> Self {
        Self {
            context: None,
            source: Box::new(value),
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext;
