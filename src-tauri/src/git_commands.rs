use git2::{Oid, Repository, Signature, Sort};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Debug)]
pub struct RepoInfo {
    pub path: String,
    pub branch: String,
    pub commit_count: usize,
}

#[derive(Serialize, Clone)]
pub struct CommitSummary {
    pub oid: String,
    pub short_message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: i64,
}

#[derive(Serialize, Clone)]
pub struct CommitDetail {
    pub oid: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: i64,
    pub author_offset: i32,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: i64,
    pub committer_offset: i32,
    pub parent_oids: Vec<String>,
    pub is_merge: bool,
}

#[derive(Serialize, Clone)]
pub struct RewriteProgress {
    pub current: usize,
    pub total: usize,
}

#[derive(Serialize, Clone)]
pub struct RewriteResult {
    pub old_oid: String,
    pub new_oid: String,
    pub commits_rewritten: usize,
}

const BACKUP_REF_PREFIX: &str = "refs/git-history-editor/pre-rewrite/";

fn backup_ref_name(branch_shorthand: &str) -> String {
    format!("{}{}", BACKUP_REF_PREFIX, branch_shorthand)
}

fn open_repo(path: &str) -> Result<Repository, String> {
    Repository::open(path).map_err(|e| {
        if e.message().contains("not a git repository") || e.message().contains("does not point to a valid git repository") {
            format!("'{}' is not a Git repository. Select a folder that contains a .git directory.", path)
        } else {
            format!("Failed to open repository: {}", e)
        }
    })
}

#[tauri::command]
pub fn open_repository(path: String) -> Result<RepoInfo, String> {
    let repo = open_repo(&path)?;

    let head = repo.head().ok();

    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD (detached)".to_string());

    let commit_count = if head.is_some() {
        let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
        match revwalk.push_head() {
            Ok(_) => revwalk.count(),
            Err(_) => 0,
        }
    } else {
        0
    };

    Ok(RepoInfo {
        path,
        branch,
        commit_count,
    })
}

#[tauri::command]
pub fn get_commits(path: String, offset: usize, limit: usize) -> Result<Vec<CommitSummary>, String> {
    let repo = open_repo(&path)?;

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    if revwalk.push_head().is_err() {
        return Ok(vec![]);
    }
    revwalk
        .set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
        .map_err(|e| e.to_string())?;

    let commits: Vec<CommitSummary> = revwalk
        .skip(offset)
        .take(limit)
        .filter_map(|oid| oid.ok())
        .filter_map(|oid| {
            let commit = repo.find_commit(oid).ok()?;
            let message = commit.message().unwrap_or("").to_string();
            let short_message = message.lines().next().unwrap_or("").to_string();
            let short_message = if short_message.len() > 72 {
                format!("{}...", &short_message[..69])
            } else {
                short_message
            };

            let author = commit.author();
            let author_name = author.name().unwrap_or("").to_string();
            let author_email = author.email().unwrap_or("").to_string();
            let author_date = author.when().seconds();

            Some(CommitSummary {
                oid: oid.to_string(),
                short_message,
                author_name,
                author_email,
                author_date,
            })
        })
        .collect();

    Ok(commits)
}

#[tauri::command]
pub fn get_commit_detail(path: String, oid: String) -> Result<CommitDetail, String> {
    let repo = open_repo(&path)?;
    let obj_oid = Oid::from_str(&oid).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(obj_oid).map_err(|e| e.to_string())?;

    let parent_oids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();
    let message = commit.message().unwrap_or("").to_string();

    let author = commit.author();
    let author_name = author.name().unwrap_or("").to_string();
    let author_email = author.email().unwrap_or("").to_string();
    let author_date = author.when().seconds();
    let author_offset = author.when().offset_minutes();

    let committer = commit.committer();
    let committer_name = committer.name().unwrap_or("").to_string();
    let committer_email = committer.email().unwrap_or("").to_string();
    let committer_date = committer.when().seconds();
    let committer_offset = committer.when().offset_minutes();

    let is_merge = commit.parent_count() > 1;

    Ok(CommitDetail {
        oid,
        message,
        author_name,
        author_email,
        author_date,
        author_offset,
        committer_name,
        committer_email,
        committer_date,
        committer_offset,
        parent_oids,
        is_merge,
    })
}

/// Core rewrite logic, separated from the Tauri command for testability.
/// `on_progress` is called with (current_index, total_count) during the walk.
pub fn rewrite_commit(
    repo: &Repository,
    target_oid: Oid,
    new_author_name: Option<&str>,
    new_author_email: Option<&str>,
    new_author_date: Option<i64>,
    new_author_offset: Option<i32>,
    new_committer_name: Option<&str>,
    new_committer_email: Option<&str>,
    new_committer_date: Option<i64>,
    new_committer_offset: Option<i32>,
    new_message: Option<&str>,
    on_progress: &dyn Fn(usize, usize),
) -> Result<RewriteResult, String> {
    let oid = target_oid.to_string();

    // Find the branch ref that HEAD points to
    let head = repo.head().map_err(|e| e.to_string())?;
    let branch_ref_name = head
        .name()
        .ok_or("Cannot rewrite history: HEAD is detached. Please check out a branch first.")?
        .to_string();
    let head_oid = head.target().ok_or("HEAD has no target")?;

    // Save a backup ref before rewriting
    let branch_shorthand = head
        .shorthand()
        .unwrap_or("unknown")
        .to_string();
    let backup_name = backup_ref_name(&branch_shorthand);
    repo.reference(
        &backup_name,
        head_oid,
        true,
        "git-history-editor: pre-rewrite backup",
    )
    .map_err(|e| format!("Failed to create backup ref: {}", e))?;

    // Collect all commits from HEAD back to root in reverse topological order (oldest first)
    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push(head_oid).map_err(|e| e.to_string())?;
    revwalk
        .set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)
        .map_err(|e| e.to_string())?;

    let all_oids: Vec<Oid> = revwalk.filter_map(|r| r.ok()).collect();

    // Check that the target commit is in this history
    if !all_oids.contains(&target_oid) {
        return Err("Target commit not found in current branch history".to_string());
    }

    // Walk through commits, rewriting from the target onward
    let mut oid_map: HashMap<Oid, Oid> = HashMap::new();
    let mut commits_rewritten: usize = 0;
    let total = all_oids.len();

    for (idx, current_oid) in all_oids.iter().enumerate() {
        if idx % 100 == 0 {
            on_progress(idx, total);
        }
        let commit = repo.find_commit(*current_oid).map_err(|e| e.to_string())?;

        // Check if any parent was rewritten or if this is the target
        let is_target = *current_oid == target_oid;
        let has_rewritten_parent = commit.parent_ids().any(|pid| oid_map.contains_key(&pid));

        if !is_target && !has_rewritten_parent {
            continue;
        }

        // Remap parents
        let new_parent_oids: Vec<Oid> = commit
            .parent_ids()
            .map(|pid| *oid_map.get(&pid).unwrap_or(&pid))
            .collect();
        let new_parents: Vec<git2::Commit> = new_parent_oids
            .iter()
            .map(|pid| repo.find_commit(*pid))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let parent_refs: Vec<&git2::Commit> = new_parents.iter().collect();

        // Determine author signature
        let orig_author = commit.author();
        let author_name_str;
        let author_email_str;
        let author = if is_target {
            author_name_str = new_author_name
                .unwrap_or(orig_author.name().unwrap_or(""))
                .to_string();
            author_email_str = new_author_email
                .unwrap_or(orig_author.email().unwrap_or(""))
                .to_string();
            let time = git2::Time::new(
                new_author_date.unwrap_or(orig_author.when().seconds()),
                new_author_offset.unwrap_or(orig_author.when().offset_minutes()),
            );
            Signature::new(&author_name_str, &author_email_str, &time)
                .map_err(|e| format!("Invalid author name or email: {}", e))?
        } else {
            author_name_str = orig_author.name().unwrap_or("").to_string();
            author_email_str = orig_author.email().unwrap_or("").to_string();
            Signature::new(
                &author_name_str,
                &author_email_str,
                &orig_author.when(),
            )
            .map_err(|e| e.to_string())?
        };

        // Determine committer signature
        let orig_committer = commit.committer();
        let committer_name_str;
        let committer_email_str;
        let committer = if is_target {
            committer_name_str = new_committer_name
                .unwrap_or(orig_committer.name().unwrap_or(""))
                .to_string();
            committer_email_str = new_committer_email
                .unwrap_or(orig_committer.email().unwrap_or(""))
                .to_string();
            let time = git2::Time::new(
                new_committer_date.unwrap_or(orig_committer.when().seconds()),
                new_committer_offset.unwrap_or(orig_committer.when().offset_minutes()),
            );
            Signature::new(&committer_name_str, &committer_email_str, &time)
                .map_err(|e| format!("Invalid committer name or email: {}", e))?
        } else {
            committer_name_str = orig_committer.name().unwrap_or("").to_string();
            committer_email_str = orig_committer.email().unwrap_or("").to_string();
            Signature::new(
                &committer_name_str,
                &committer_email_str,
                &orig_committer.when(),
            )
            .map_err(|e| e.to_string())?
        };

        // Determine message
        let message = if is_target {
            new_message
                .unwrap_or(commit.message().unwrap_or(""))
                .to_string()
        } else {
            commit.message().unwrap_or("").to_string()
        };

        let tree = commit.tree().map_err(|e| e.to_string())?;

        // Create the new commit
        let new_oid = repo
            .commit(None, &author, &committer, &message, &tree, &parent_refs)
            .map_err(|e| e.to_string())?;

        oid_map.insert(*current_oid, new_oid);
        commits_rewritten += 1;
    }

    // Update the branch ref to point to the new tip
    let new_tip = oid_map.get(&head_oid).ok_or("HEAD commit was not rewritten")?;
    repo.reference(
        &branch_ref_name,
        *new_tip,
        true,
        &format!("git-history-editor: rewrote commit {}", &oid[..8]),
    )
    .map_err(|e| format!("Failed to update branch ref (do you have write permissions?): {}", e))?;

    let new_target_oid = oid_map
        .get(&target_oid)
        .ok_or("Target commit was not rewritten")?;

    Ok(RewriteResult {
        old_oid: oid,
        new_oid: new_target_oid.to_string(),
        commits_rewritten,
    })
}

#[tauri::command]
pub fn update_commit(
    app: AppHandle,
    path: String,
    oid: String,
    new_author_name: Option<String>,
    new_author_email: Option<String>,
    new_author_date: Option<i64>,
    new_author_offset: Option<i32>,
    new_committer_name: Option<String>,
    new_committer_email: Option<String>,
    new_committer_date: Option<i64>,
    new_committer_offset: Option<i32>,
    new_message: Option<String>,
) -> Result<RewriteResult, String> {
    let repo = open_repo(&path)?;
    let target_oid = Oid::from_str(&oid).map_err(|e| e.to_string())?;

    rewrite_commit(
        &repo,
        target_oid,
        new_author_name.as_deref(),
        new_author_email.as_deref(),
        new_author_date,
        new_author_offset,
        new_committer_name.as_deref(),
        new_committer_email.as_deref(),
        new_committer_date,
        new_committer_offset,
        new_message.as_deref(),
        &|current, total| {
            let _ = app.emit("rewrite-progress", RewriteProgress { current, total });
        },
    )
}

#[derive(Serialize, Clone)]
pub struct BackupInfo {
    pub exists: bool,
    pub backup_oid: Option<String>,
    pub branch: String,
}

#[tauri::command]
pub fn check_backup(path: String) -> Result<BackupInfo, String> {
    let repo = open_repo(&path)?;

    let head = repo.head().ok();
    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "unknown".to_string());

    let ref_name = backup_ref_name(&branch);
    let result = match repo.find_reference(&ref_name) {
        Ok(reference) => {
            let oid = reference.target().map(|o| o.to_string());
            BackupInfo {
                exists: true,
                backup_oid: oid,
                branch,
            }
        }
        Err(_) => BackupInfo {
            exists: false,
            backup_oid: None,
            branch,
        },
    };
    Ok(result)
}

#[tauri::command]
pub fn restore_backup(path: String) -> Result<String, String> {
    let repo = open_repo(&path)?;

    let head = repo.head().map_err(|e| e.to_string())?;
    let branch_ref_name = head
        .name()
        .ok_or("Cannot restore: HEAD is detached.")?
        .to_string();
    let branch_shorthand = head
        .shorthand()
        .unwrap_or("unknown")
        .to_string();

    let ref_name = backup_ref_name(&branch_shorthand);
    let backup_ref = repo
        .find_reference(&ref_name)
        .map_err(|_| "No backup found for this branch.")?;
    let backup_oid = backup_ref
        .target()
        .ok_or("Backup ref has no target.")?;

    // Reset the branch to the backup OID
    repo.reference(
        &branch_ref_name,
        backup_oid,
        true,
        "git-history-editor: restored from pre-rewrite backup",
    )
    .map_err(|e| format!("Failed to restore: {}", e))?;

    // Delete the backup ref
    let mut backup_ref = repo
        .find_reference(&ref_name)
        .map_err(|e| e.to_string())?;
    backup_ref.delete().map_err(|e| format!("Restored successfully but failed to clean up backup ref: {}", e))?;

    Ok(backup_oid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Create a temp repo with the given number of commits on "main".
    fn create_test_repo(num_commits: usize) -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Configure so commits work
        let sig = Signature::now("Test User", "test@example.com").unwrap();

        {
            let mut parent_oid: Option<Oid> = None;

            for i in 0..num_commits {
                let mut index = repo.index().unwrap();
                let file_path = format!("file_{}.txt", i);
                let full_path = dir.path().join(&file_path);
                std::fs::write(&full_path, format!("content {}", i)).unwrap();
                index.add_path(Path::new(&file_path)).unwrap();
                index.write().unwrap();
                let tree_oid = index.write_tree().unwrap();
                let tree = repo.find_tree(tree_oid).unwrap();

                let parent_commit = parent_oid.map(|oid| repo.find_commit(oid).unwrap());
                let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
                let oid = repo
                    .commit(Some("HEAD"), &sig, &sig, &format!("Commit {}", i), &tree, &parents)
                    .unwrap();

                parent_oid = Some(oid);
            }
        }

        // Make sure HEAD points to refs/heads/main
        if num_commits > 0 {
            let head = repo.head().unwrap();
            if head.shorthand() != Some("main") {
                let mut branch = repo.find_branch(
                    head.shorthand().unwrap(),
                    git2::BranchType::Local,
                ).unwrap();
                branch.rename("main", true).unwrap();
            }
        }

        (dir, repo)
    }

    #[test]
    fn test_open_repository_with_commits() {
        let (dir, _repo) = create_test_repo(3);
        let result = open_repository(dir.path().to_str().unwrap().to_string());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.commit_count, 3);
        assert_eq!(info.branch, "main");
    }

    #[test]
    fn test_open_repository_empty() {
        let dir = TempDir::new().unwrap();
        Repository::init(dir.path()).unwrap();
        let result = open_repository(dir.path().to_str().unwrap().to_string());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.commit_count, 0);
    }

    #[test]
    fn test_open_repository_not_a_repo() {
        let dir = TempDir::new().unwrap();
        let result = open_repository(dir.path().to_str().unwrap().to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("not a Git repository") || err.contains("Failed to open repository"),
            "Unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_get_commits_pagination() {
        let (dir, _repo) = create_test_repo(5);
        let path = dir.path().to_str().unwrap().to_string();

        let first_page = get_commits(path.clone(), 0, 3).unwrap();
        assert_eq!(first_page.len(), 3);

        let second_page = get_commits(path.clone(), 3, 3).unwrap();
        assert_eq!(second_page.len(), 2);

        // No overlap
        assert_ne!(first_page[2].oid, second_page[0].oid);
    }

    #[test]
    fn test_get_commits_empty_repo() {
        let dir = TempDir::new().unwrap();
        Repository::init(dir.path()).unwrap();
        let result = get_commits(dir.path().to_str().unwrap().to_string(), 0, 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_get_commit_detail() {
        let (dir, _repo) = create_test_repo(1);
        let path = dir.path().to_str().unwrap().to_string();

        let commits = get_commits(path.clone(), 0, 10).unwrap();
        let detail = get_commit_detail(path, commits[0].oid.clone()).unwrap();

        assert_eq!(detail.message, "Commit 0");
        assert_eq!(detail.author_name, "Test User");
        assert_eq!(detail.author_email, "test@example.com");
        assert!(!detail.is_merge);
        assert!(detail.parent_oids.is_empty());
    }

    #[test]
    fn test_rewrite_commit_changes_message() {
        let (dir, repo) = create_test_repo(3);
        let path = dir.path().to_str().unwrap().to_string();

        let commits = get_commits(path.clone(), 0, 10).unwrap();
        // Get the oldest commit (last in the list since sorted newest-first)
        let oldest = &commits[2];

        let target_oid = Oid::from_str(&oldest.oid).unwrap();
        let result = rewrite_commit(
            &repo, target_oid,
            None, None, None, None,
            None, None, None, None,
            Some("New message"),
            &|_, _| {},
        ).unwrap();

        assert_eq!(result.commits_rewritten, 3); // Rewrites oldest + 2 descendants

        // Verify the new commit has the changed message
        let detail = get_commit_detail(path, result.new_oid.clone()).unwrap();
        assert_eq!(detail.message, "New message");
    }

    #[test]
    fn test_rewrite_commit_changes_author() {
        let (dir, repo) = create_test_repo(2);
        let path = dir.path().to_str().unwrap().to_string();

        let commits = get_commits(path.clone(), 0, 10).unwrap();
        let latest = &commits[0];

        let target_oid = Oid::from_str(&latest.oid).unwrap();
        let result = rewrite_commit(
            &repo, target_oid,
            Some("New Author"), Some("new@example.com"), None, None,
            None, None, None, None,
            None,
            &|_, _| {},
        ).unwrap();

        assert_eq!(result.commits_rewritten, 1); // Only HEAD commit, no descendants

        let detail = get_commit_detail(path, result.new_oid).unwrap();
        assert_eq!(detail.author_name, "New Author");
        assert_eq!(detail.author_email, "new@example.com");
    }

    #[test]
    fn test_rewrite_preserves_descendants() {
        let (dir, repo) = create_test_repo(3);
        let path = dir.path().to_str().unwrap().to_string();

        let commits_before = get_commits(path.clone(), 0, 10).unwrap();
        let oldest = &commits_before[2];

        let target_oid = Oid::from_str(&oldest.oid).unwrap();
        rewrite_commit(
            &repo, target_oid,
            None, None, None, None,
            None, None, None, None,
            Some("Changed root"),
            &|_, _| {},
        ).unwrap();

        // Reload commits after rewrite
        let commits_after = get_commits(path, 0, 10).unwrap();
        assert_eq!(commits_after.len(), 3);

        // All OIDs should be different (rewritten)
        for (before, after) in commits_before.iter().zip(commits_after.iter()) {
            assert_ne!(before.oid, after.oid);
        }

        // But messages of non-target commits should be preserved
        assert_eq!(commits_after[0].short_message, "Commit 2");
        assert_eq!(commits_after[1].short_message, "Commit 1");
        assert_eq!(commits_after[2].short_message, "Changed root");
    }

    #[test]
    fn test_backup_and_restore() {
        let (dir, repo) = create_test_repo(2);
        let path = dir.path().to_str().unwrap().to_string();

        let commits_before = get_commits(path.clone(), 0, 10).unwrap();
        let original_head_oid = commits_before[0].oid.clone();

        // Rewrite and verify backup exists
        let target_oid = Oid::from_str(&original_head_oid).unwrap();
        rewrite_commit(
            &repo, target_oid,
            None, None, None, None,
            None, None, None, None,
            Some("Rewritten"),
            &|_, _| {},
        ).unwrap();

        let backup = check_backup(path.clone()).unwrap();
        assert!(backup.exists);
        assert_eq!(backup.backup_oid.as_deref(), Some(original_head_oid.as_str()));

        // Restore
        let restored_oid = restore_backup(path.clone()).unwrap();
        assert_eq!(restored_oid, original_head_oid);

        // Backup should be gone
        let backup_after = check_backup(path.clone()).unwrap();
        assert!(!backup_after.exists);

        // Commits should be back to original
        let commits_after = get_commits(path, 0, 10).unwrap();
        assert_eq!(commits_after[0].oid, original_head_oid);
    }

    #[test]
    fn test_no_backup_initially() {
        let (dir, _repo) = create_test_repo(1);
        let backup = check_backup(dir.path().to_str().unwrap().to_string()).unwrap();
        assert!(!backup.exists);
        assert!(backup.backup_oid.is_none());
    }
}
