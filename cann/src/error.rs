use cann_sys::aclError;

#[derive(Debug)]
pub struct Error {
    pub code: aclError,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CANN error ({}): {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<aclError> for Error {
    fn from(code: aclError) -> Self {
        let message = match code {
            cann_sys::ACL_ERROR_INVALID_FILE => "Invalid file".to_string(),
            _ => format!("Unknown CANN error code: {}", code),
        };
        Error { code, message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cann_sys::ACL_ERROR_INVALID_FILE;

    #[test]
    fn test_error_from_invalid_file() {
        let e = Error::from(ACL_ERROR_INVALID_FILE);
        assert_eq!(e.code, ACL_ERROR_INVALID_FILE);
        assert_eq!(e.message, "Invalid file");
    }

    #[test]
    fn test_error_display_does_not_panic() {
        let e = Error::from(ACL_ERROR_INVALID_FILE);
        let s = format!("{}", e);
        assert!(s.contains("Invalid file"));
    }

    #[test]
    fn test_error_from_unknown_code() {
        let e = Error::from(999_999);
        assert_eq!(e.code, 999_999);
        assert!(e.message.contains("999999"));
    }
}
