use git2::{Oid, Repository, Signature, Sort};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
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
pub struct RewriteResult {
    pub old_oid: String,
    pub new_oid: String,
    pub commits_rewritten: usize,
}

fn open_repo(path: &str) -> Result<Repository, String> {
    Repository::open(path).map_err(|e| format!("Failed to open repository: {}", e))
}

#[tauri::command]
pub fn open_repository(path: String) -> Result<RepoInfo, String> {
    let repo = open_repo(&path)?;

    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD (detached)".to_string());

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;
    let commit_count = revwalk.count();

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
    revwalk.push_head().map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn update_commit(
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

    // Find the branch ref that HEAD points to
    let head = repo.head().map_err(|e| e.to_string())?;
    let branch_ref_name = head
        .name()
        .ok_or("HEAD is not a symbolic reference")?
        .to_string();
    let head_oid = head.target().ok_or("HEAD has no target")?;

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

    for current_oid in &all_oids {
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
                .as_deref()
                .unwrap_or(orig_author.name().unwrap_or(""))
                .to_string();
            author_email_str = new_author_email
                .as_deref()
                .unwrap_or(orig_author.email().unwrap_or(""))
                .to_string();
            let time = git2::Time::new(
                new_author_date.unwrap_or(orig_author.when().seconds()),
                new_author_offset.unwrap_or(orig_author.when().offset_minutes()),
            );
            Signature::new(&author_name_str, &author_email_str, &time).map_err(|e| e.to_string())?
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
                .as_deref()
                .unwrap_or(orig_committer.name().unwrap_or(""))
                .to_string();
            committer_email_str = new_committer_email
                .as_deref()
                .unwrap_or(orig_committer.email().unwrap_or(""))
                .to_string();
            let time = git2::Time::new(
                new_committer_date.unwrap_or(orig_committer.when().seconds()),
                new_committer_offset.unwrap_or(orig_committer.when().offset_minutes()),
            );
            Signature::new(&committer_name_str, &committer_email_str, &time).map_err(|e| e.to_string())?
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
                .as_deref()
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
    .map_err(|e| e.to_string())?;

    let new_target_oid = oid_map
        .get(&target_oid)
        .ok_or("Target commit was not rewritten")?;

    Ok(RewriteResult {
        old_oid: oid,
        new_oid: new_target_oid.to_string(),
        commits_rewritten,
    })
}
