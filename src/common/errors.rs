use crate::binder::BindError;

#[derive(Debug)]
pub enum BustubError {
    BindError(BindError),
    Message(String),
}

impl From<BindError> for BustubError {
    fn from(e: BindError) -> Self {
        Self::BindError(e)
    }
}

impl From<String> for BustubError {
    fn from(e: String) -> Self {
        Self::Message(e)
    }
}

impl std::fmt::Display for BustubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BustubError::Message(e) => write!(f, "{}", e),
            BustubError::BindError(e) => write!(f, "{}", e),
        }
    }
}
