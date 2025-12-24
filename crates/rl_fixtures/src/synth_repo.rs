use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub enum FixtureError {
    Io(std::io::Error),
    Git(String),
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureError::Io(e) => write!(f, "IO error: {}", e),
            FixtureError::Git(msg) => write!(f, "Git error: {}", msg),
        }
    }
}

impl std::error::Error for FixtureError {}

impl From<std::io::Error> for FixtureError {
    fn from(e: std::io::Error) -> Self {
        FixtureError::Io(e)
    }
}

pub struct SynthRepo {
    pub path: PathBuf,
}

impl SynthRepo {
    pub fn ensure(name: &str) -> Result<SynthRepo, FixtureError> {
        // Find workspace root by walking up to find Cargo.toml with [workspace]
        let workspace_root = Self::find_workspace_root()?;
        let base = workspace_root.join("target").join("rl_fixtures").join(name);
        let repo_path = base.join("repo");

        if repo_path.exists() {
            let git_dir = repo_path.join(".git");
            if git_dir.exists() {
                return Ok(SynthRepo { path: repo_path });
            }
        }

        fs::create_dir_all(&repo_path)?;

        let repo = SynthRepo { path: repo_path };
        repo.initialize()?;
        Ok(repo)
    }

    fn find_workspace_root() -> Result<PathBuf, FixtureError> {
        let mut current = std::env::current_dir()?;
        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                // Check if this is a workspace root by looking for [workspace]
                let content = fs::read_to_string(&cargo_toml)?;
                if content.contains("[workspace]") {
                    return Ok(current);
                }
            }
            if !current.pop() {
                break;
            }
        }
        Err(FixtureError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find workspace root",
        )))
    }

    fn initialize(&self) -> Result<(), FixtureError> {
        self.run_git(&["init"])?;
        self.run_git(&["config", "user.name", "Test User"])?;
        self.run_git(&["config", "user.email", "test@example.com"])?;

        self.create_c0()?;
        self.create_c1()?;
        self.create_c2()?;
        self.create_c3()?;

        Ok(())
    }

    fn create_c0(&self) -> Result<(), FixtureError> {
        let a_content = "line 1\nline 2\nline 3\nline 4\nline 5\n\
                         line 6\nline 7\nline 8\nline 9\nline 10\n\
                         line 11\nline 12\nline 13\n";
        self.write_file("a.txt", a_content)?;

        fs::create_dir_all(self.path.join("dir"))?;
        let b_content = "content of b\nline 2 of b\nline 3 of b\n";
        self.write_file("dir/b.txt", b_content)?;

        self.run_git(&["add", "."])?;
        self.run_git(&["commit", "-m", "C0: initial commit"])?;
        self.run_git(&["tag", "C0"])?;

        Ok(())
    }

    fn create_c1(&self) -> Result<(), FixtureError> {
        let a_content = "line 1\nline 2 modified\nline 3\nline 5\n\
                         line 6\nline 7\nline 8\nline 9\nline 10\n\
                         line 11\nline 12\nline 13\nnew line 14\nnew line 15\n";
        self.write_file("a.txt", a_content)?;

        let new_content = "this is a new file\nwith some content\n";
        self.write_file("new.txt", new_content)?;

        self.run_git(&["add", "."])?;
        self.run_git(&["commit", "-m", "C1: modify + add"])?;
        self.run_git(&["tag", "C1"])?;

        Ok(())
    }

    fn create_c2(&self) -> Result<(), FixtureError> {
        self.run_git(&["mv", "dir/b.txt", "dir/c.txt"])?;
        self.run_git(&["commit", "-m", "C2: rename"])?;
        self.run_git(&["tag", "C2"])?;

        Ok(())
    }

    fn create_c3(&self) -> Result<(), FixtureError> {
        self.run_git(&["rm", "new.txt"])?;

        let binary_data: Vec<u8> = (0u8..=255).cycle().take(512).collect();
        self.write_file_binary("bin.dat", &binary_data)?;

        self.run_git(&["add", "."])?;
        self.run_git(&["commit", "-m", "C3: delete + binary"])?;
        self.run_git(&["tag", "C3"])?;

        Ok(())
    }

    fn write_file(&self, rel_path: &str, content: &str) -> Result<(), FixtureError> {
        let full_path = self.path.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&full_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_file_binary(&self, rel_path: &str, content: &[u8]) -> Result<(), FixtureError> {
        let full_path = self.path.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&full_path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn run_git(&self, args: &[&str]) -> Result<(), FixtureError> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FixtureError::Git(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr
            )));
        }

        Ok(())
    }

    pub fn modify_working_tree(&self, rel_path: &str, append: &str) -> Result<(), FixtureError> {
        let full_path = self.path.join(rel_path);
        let mut file = fs::OpenOptions::new().append(true).open(&full_path)?;
        file.write_all(append.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synth_repo_creation() {
        let repo = SynthRepo::ensure("test_basic").expect("Failed to create synthetic repo");
        assert!(repo.path.exists(), "Repo path should exist");
        assert!(
            repo.path.join(".git").exists(),
            "Git directory should exist"
        );
        assert!(repo.path.join("a.txt").exists(), "a.txt should exist");
        assert!(
            repo.path.join("dir/c.txt").exists(),
            "dir/c.txt should exist (renamed in C2)"
        );
        assert!(repo.path.join("bin.dat").exists(), "bin.dat should exist");
        assert!(
            !repo.path.join("new.txt").exists(),
            "new.txt should not exist (deleted in C3)"
        );
    }
}
