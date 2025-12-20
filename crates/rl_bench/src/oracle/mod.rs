pub mod compare;
pub mod git_cli;
pub mod normalize;

#[derive(Debug)]
pub enum OracleError {
    GitNotFound,
    GitFailed {
        status: i32,
        stdout: String,
        stderr: String,
    },
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleError::GitNotFound => write!(f, "git command not found"),
            OracleError::GitFailed {
                status,
                stdout,
                stderr,
            } => {
                write!(
                    f,
                    "git command failed with status {}: stdout={}, stderr={}",
                    status, stdout, stderr
                )
            }
            OracleError::Io(e) => write!(f, "IO error: {}", e),
            OracleError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for OracleError {}

impl From<std::io::Error> for OracleError {
    fn from(e: std::io::Error) -> Self {
        OracleError::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for OracleError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        OracleError::Utf8(e)
    }
}
