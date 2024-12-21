use git2::{Repository, Commit, DiffOptions};
use crate::{Result, HistorianError};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let repo = Repository::open(path)
            .map_err(|e| HistorianError::Git(e))?;
        Ok(Self { repo })
    }

    pub fn get_commit(&self, commit_id: &str) -> Result<Commit> {
        let obj = self.repo
            .revparse_single(commit_id)
            .map_err(|e| HistorianError::Git(e))?;
        
        obj.peel_to_commit()
            .map_err(|e| HistorianError::Git(e))
    }

    pub fn walk_commits(&self) -> Result<Vec<Commit>> {
        let mut revwalk = self.repo.revwalk()
            .map_err(|e| HistorianError::Git(e))?;
        
        revwalk.push_head()
            .map_err(|e| HistorianError::Git(e))?;
        
        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid.map_err(|e| HistorianError::Git(e))?;
            let commit = self.repo
                .find_commit(oid)
                .map_err(|e| HistorianError::Git(e))?;
            commits.push(commit);
        }
        
        Ok(commits)
    }

    pub fn get_diff(&self, commit: &Commit) -> Result<String> {
        let parent = commit.parent(0)
            .map_err(|e| HistorianError::Git(e))?;
        
        let tree = commit.tree()
            .map_err(|e| HistorianError::Git(e))?;
        let parent_tree = parent.tree()
            .map_err(|e| HistorianError::Git(e))?;
        
        let mut diff_opts = DiffOptions::new();
        let diff = self.repo
            .diff_tree_to_tree(
                Some(&parent_tree),
                Some(&tree),
                Some(&mut diff_opts),
            )
            .map_err(|e| HistorianError::Git(e))?;
        
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            use std::str;
            if let Ok(text) = str::from_utf8(line.content()) {
                diff_text.push_str(text);
            }
            true
        }).map_err(|e| HistorianError::Git(e))?;
        
        Ok(diff_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();
        
        // Initialize a test repository
        Repository::init(&path).unwrap();
        
        (dir, path)
    }

    #[test]
    fn test_open_repo() {
        let (_dir, path) = setup_test_repo();
        let result = GitRepo::open(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_walk_commits() {
        let (_dir, path) = setup_test_repo();
        let repo = GitRepo::open(&path).unwrap();
        let commits = repo.walk_commits();
        assert!(commits.is_ok());
    }
} 