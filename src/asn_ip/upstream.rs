use git2::Repository;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const REPO_URL: &str = "https://github.com/ipverse/asn-ip";
const ASN_IP_SUBDIR: &str = "ipverse-mcp/asn-ip";

#[derive(Error, Debug)]
pub enum UpstreamError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Home directory not found")]
    HomeDirNotFound,
}

pub struct Upstream {
    repo_path: PathBuf,
}

impl Upstream {
    /// Creates a new Upstream instance
    pub fn new() -> Result<Self, UpstreamError> {
        let state_dir = dirs::state_dir().ok_or(UpstreamError::HomeDirNotFound)?;
        let repo_path = state_dir.join(ASN_IP_SUBDIR);
        Ok(Self { repo_path })
    }

    /// Creates or initializes the local repository
    pub fn provision(&self) -> Result<Repository, UpstreamError> {
        // Create parent directories if they don't exist
        if !self.repo_path.exists() {
            fs::create_dir_all(&self.repo_path)?;
        }

        // Check if repo already exists
        match Repository::open(&self.repo_path) {
            Ok(repo) => Ok(repo),
            Err(_) => {
                // Clone the repository
                let repo = Repository::clone(REPO_URL, &self.repo_path)?;
                Ok(repo)
            }
        }
    }

    /// Updates the local repository and returns a list of changed files
    pub fn update(&self) -> Result<Vec<PathBuf>, UpstreamError> {
        let repo = self.provision()?;
        let mut remote = repo.find_remote("origin")?;

        // Fetch latest changes
        remote.fetch(&["main"], None, None)?;

        let fetch_head = match repo.find_reference("FETCH_HEAD") {
            Ok(reference) => reference,
            Err(e) => {
                let fetch_head_path = repo.path().join("FETCH_HEAD");
                // FETCH_HEAD file should exist after a fetch
                if !fetch_head_path.exists() {
                    return Err(UpstreamError::IoError(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "expected FETCH_HEAD file not found",
                    )));
                }
                // If FETCH_HEAD is empty, there were no changes to fetch
                if fetch_head_path
                    .metadata()
                    .map_err(UpstreamError::IoError)?
                    .len()
                    == 0
                {
                    return Ok(vec![]);
                }
                // Something else went wrong
                return Err(UpstreamError::GitError(e));
            }
        };
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        // Get the current commit
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        // Compare the two commits to find changes
        let old_tree = head_commit.tree()?;
        let new_tree = repo.find_commit(fetch_commit.id())?.tree()?;

        let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?;

        let mut changed_files = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(new_file) = delta.new_file().path() {
                    changed_files.push(self.repo_path.join(new_file));
                }
                true
            },
            None,
            None,
            None,
        )?;

        // Now perform the merge
        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();

        repo.merge(&[&fetch_commit], None, Some(&mut checkout_opts))?;

        Ok(changed_files)
    }

    /// Get the local repository path
    pub fn get_repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Get the path to an ASN's aggregated.json file
    pub fn get_asn_file_path(&self, asn: u32) -> PathBuf {
        self.repo_path.join(format!("as/{}/aggregated.json", asn))
    }
}
