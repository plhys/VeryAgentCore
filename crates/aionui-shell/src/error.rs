use aionui_common::AppError;

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("tool not installed: {0}")]
    ToolNotInstalled(String),

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ShellError> for AppError {
    fn from(err: ShellError) -> Self {
        match err {
            ShellError::FileNotFound(path) => {
                AppError::BadRequest(format!("file not found: {path}"))
            }
            ShellError::DirectoryNotFound(path) => {
                AppError::BadRequest(format!("directory not found: {path}"))
            }
            ShellError::InvalidUrl(msg) => {
                AppError::BadRequest(format!("invalid URL: {msg}"))
            }
            ShellError::ToolNotInstalled(tool) => {
                AppError::BadRequest(format!("tool not installed: {tool}"))
            }
            ShellError::CommandFailed(msg) => {
                AppError::Internal(format!("command failed: {msg}"))
            }
            ShellError::Io(e) => AppError::Internal(format!("IO error: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_not_found_maps_to_bad_request() {
        let err: AppError = ShellError::FileNotFound("/tmp/missing.txt".into()).into();
        assert!(matches!(err, AppError::BadRequest(msg) if msg.contains("/tmp/missing.txt")));
    }

    #[test]
    fn directory_not_found_maps_to_bad_request() {
        let err: AppError = ShellError::DirectoryNotFound("/tmp/nodir".into()).into();
        assert!(matches!(err, AppError::BadRequest(msg) if msg.contains("/tmp/nodir")));
    }

    #[test]
    fn invalid_url_maps_to_bad_request() {
        let err: AppError = ShellError::InvalidUrl("not a url".into()).into();
        assert!(matches!(err, AppError::BadRequest(msg) if msg.contains("not a url")));
    }

    #[test]
    fn tool_not_installed_maps_to_bad_request() {
        let err: AppError = ShellError::ToolNotInstalled("vscode".into()).into();
        assert!(matches!(err, AppError::BadRequest(msg) if msg.contains("vscode")));
    }

    #[test]
    fn command_failed_maps_to_internal() {
        let err: AppError = ShellError::CommandFailed("exit code 1".into()).into();
        assert!(matches!(err, AppError::Internal(msg) if msg.contains("exit code 1")));
    }

    #[test]
    fn io_error_maps_to_internal() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err: AppError = ShellError::Io(io_err).into();
        assert!(matches!(err, AppError::Internal(msg) if msg.contains("permission denied")));
    }

    #[test]
    fn display_messages() {
        assert_eq!(
            ShellError::FileNotFound("/a.txt".into()).to_string(),
            "file not found: /a.txt"
        );
        assert_eq!(
            ShellError::DirectoryNotFound("/dir".into()).to_string(),
            "directory not found: /dir"
        );
        assert_eq!(
            ShellError::InvalidUrl("bad".into()).to_string(),
            "invalid URL: bad"
        );
        assert_eq!(
            ShellError::ToolNotInstalled("code".into()).to_string(),
            "tool not installed: code"
        );
        assert_eq!(
            ShellError::CommandFailed("oops".into()).to_string(),
            "command failed: oops"
        );
    }
}
