use std::fs;
use std::path::Path;
use std::process::Stdio;
use tempfile::TempDir;

use aw_repo::{VcsRepo, VcsType, VcsError};

fn check_git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn setup_git_repo() -> Result<(TempDir, TempDir), Box<dyn std::error::Error>> {
    // Set environment variables globally for this test
    std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
    std::env::set_var("GIT_TERMINAL_PROMPT", "0");
    std::env::set_var("GIT_ASKPASS", "echo");
    std::env::set_var("SSH_ASKPASS", "echo");

    // Set HOME to a temporary directory to avoid accessing user git/ssh config
    let temp_home = TempDir::new()?;
    std::env::set_var("HOME", temp_home.path());

    let remote_dir = TempDir::new()?;
    let repo_dir = TempDir::new()?;

    // Initialize bare remote repository
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["init", "--bare"])
        .current_dir(&remote_dir);
    cmd.output().await?;

    // Initialize local repository
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["init", "-b", "main"])
        .current_dir(&repo_dir);
    cmd.output().await?;

    // Configure git
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_dir);
    cmd.output().await?;

    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["config", "user.name", "Test User"])
        .current_dir(&repo_dir);
    cmd.output().await?;

    // Create initial file and commit
    fs::write(repo_dir.path().join("README.md"), "Initial content")?;
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["add", "README.md"])
        .current_dir(&repo_dir);
    cmd.output().await?;

    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_dir);
    cmd.output().await?;

    // Don't add remote for now to avoid potential issues

    Ok((temp_home, repo_dir))
}

#[tokio::test]
async fn test_repository_detection() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();
    let repo_path = repo.path().join("some").join("nested").join("dir");
    fs::create_dir_all(&repo_path).unwrap();

    let vcs_repo = VcsRepo::new(&repo_path).await.unwrap();
    assert_eq!(vcs_repo.root(), repo.path());
    assert_eq!(vcs_repo.vcs_type(), VcsType::Git);
}

#[tokio::test]
async fn test_current_branch() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();

    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();
    let branch = vcs_repo.current_branch().await.unwrap();
    assert_eq!(branch, "main");
}

#[tokio::test]
async fn test_branch_validation() {
    // Valid branch names
    assert!(VcsRepo::valid_branch_name("feature-branch"));
    assert!(VcsRepo::valid_branch_name("bug_fix"));
    assert!(VcsRepo::valid_branch_name("v1.0.0"));
    assert!(VcsRepo::valid_branch_name("test_branch"));

    // Invalid branch names
    assert!(!VcsRepo::valid_branch_name("feature branch")); // space
    assert!(!VcsRepo::valid_branch_name("feature/branch")); // slash
    assert!(!VcsRepo::valid_branch_name("feature@branch")); // @ symbol
}

#[tokio::test]
async fn test_protected_branches() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();
    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();

    assert!(vcs_repo.is_protected_branch("main"));
    assert!(vcs_repo.is_protected_branch("master"));
    assert!(vcs_repo.is_protected_branch("trunk"));
    assert!(vcs_repo.is_protected_branch("default"));
    assert!(!vcs_repo.is_protected_branch("feature-branch"));
}

#[tokio::test]
async fn test_start_branch() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();
    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();

    // Test starting a new branch
    vcs_repo.start_branch("feature-test").await.unwrap();

    // Verify we're on the new branch
    let current_branch = vcs_repo.current_branch().await.unwrap();
    assert_eq!(current_branch, "feature-test");

    // Test that we can't start a protected branch
    let result = vcs_repo.start_branch("main").await;
    assert!(matches!(result, Err(VcsError::ProtectedBranch(_))));
}

#[tokio::test]
async fn test_commit_file() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();
    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();

    // Start a new branch
    vcs_repo.start_branch("test-commit").await.unwrap();

    // Create and commit a new file
    let test_file = repo.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();

    vcs_repo.commit_file("test.txt", "Add test file").await.unwrap();

    // Verify file was committed
    let status = vcs_repo.working_copy_status().await.unwrap();
    assert!(!status.contains("test.txt"));
}

#[tokio::test]
async fn test_branches() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();
    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();

    // Create a few branches
    vcs_repo.start_branch("branch1").await.unwrap();
    vcs_repo.checkout_branch("main").await.unwrap();
    vcs_repo.start_branch("branch2").await.unwrap();

    let branches = vcs_repo.branches().await.unwrap();
    assert!(branches.contains(&"main".to_string()));
    assert!(branches.contains(&"branch1".to_string()));
    assert!(branches.contains(&"branch2".to_string()));
}

#[tokio::test]
async fn test_default_remote_http_url() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_temp_home, repo) = setup_git_repo().await.unwrap();
    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();

    // Test HTTPS URL - add remote first
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["remote", "add", "origin", "https://github.com/user/repo.git"])
        .current_dir(repo.path());
    cmd.output().await.unwrap();

    let url = vcs_repo.default_remote_http_url().await.unwrap();
    assert_eq!(url, Some("https://github.com/user/repo.git".to_string()));

    // Test SSH URL conversion
    let mut cmd = tokio::process::Command::new("git");
    cmd.args(&["remote", "set-url", "origin", "git@github.com:user/repo.git"])
        .current_dir(repo.path());
    cmd.output().await.unwrap();

    let url = vcs_repo.default_remote_http_url().await.unwrap();
    assert_eq!(url, Some("https://github.com/user/repo.git".to_string()));
}

#[tokio::test]
async fn test_repository_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let result = VcsRepo::new(temp_dir.path()).await;
    assert!(matches!(result, Err(VcsError::RepositoryNotFound(_))));
}

#[tokio::test]
async fn test_invalid_branch_name() {
    if !check_git_available() {
        eprintln!("Git not available, skipping test");
        return;
    }

    let (_remote, repo) = setup_git_repo().await.unwrap();
    let vcs_repo = VcsRepo::new(repo.path()).await.unwrap();

    let result = vcs_repo.start_branch("invalid branch").await;
    assert!(matches!(result, Err(VcsError::InvalidBranchName(_))));
}
