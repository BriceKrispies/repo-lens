use std::path::PathBuf;
use std::process::Command;

use super::OracleError;

#[derive(Debug)]
pub struct GitCli {
    repo_path: PathBuf,
}

#[derive(Debug)]
pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

impl GitCli {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    pub fn run(&self, args: &[&str]) -> Result<GitOutput, OracleError> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&self.repo_path);
        cmd.args(args);

        let output = cmd.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let status = output.status.code().unwrap_or(-1);

        if status == 0 {
            Ok(GitOutput {
                stdout,
                stderr,
                status,
            })
        } else {
            Err(OracleError::GitFailed {
                status,
                stdout,
                stderr,
            })
        }
    }
}
