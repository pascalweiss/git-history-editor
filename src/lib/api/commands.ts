import { invoke } from "@tauri-apps/api/core";

export interface RepoInfo {
  path: string;
  branch: string;
  commit_count: number;
}

export interface CommitSummary {
  oid: string;
  short_message: string;
  author_name: string;
  author_email: string;
  author_date: number;
}

export interface CommitDetail {
  oid: string;
  message: string;
  author_name: string;
  author_email: string;
  author_date: number;
  author_offset: number;
  committer_name: string;
  committer_email: string;
  committer_date: number;
  committer_offset: number;
  parent_oids: string[];
  is_merge: boolean;
}

export interface RewriteResult {
  old_oid: string;
  new_oid: string;
  commits_rewritten: number;
}

export async function openRepository(path: string): Promise<RepoInfo> {
  return invoke("open_repository", { path });
}

export async function getCommits(
  path: string,
  offset: number,
  limit: number
): Promise<CommitSummary[]> {
  return invoke("get_commits", { path, offset, limit });
}

export async function getCommitDetail(
  path: string,
  oid: string
): Promise<CommitDetail> {
  return invoke("get_commit_detail", { path, oid });
}

export interface UpdateCommitParams {
  path: string;
  oid: string;
  newAuthorName?: string;
  newAuthorEmail?: string;
  newAuthorDate?: number;
  newAuthorOffset?: number;
  newCommitterName?: string;
  newCommitterEmail?: string;
  newCommitterDate?: number;
  newCommitterOffset?: number;
  newMessage?: string;
}

export async function updateCommit(
  params: UpdateCommitParams
): Promise<RewriteResult> {
  return invoke("update_commit", {
    path: params.path,
    oid: params.oid,
    newAuthorName: params.newAuthorName ?? null,
    newAuthorEmail: params.newAuthorEmail ?? null,
    newAuthorDate: params.newAuthorDate ?? null,
    newAuthorOffset: params.newAuthorOffset ?? null,
    newCommitterName: params.newCommitterName ?? null,
    newCommitterEmail: params.newCommitterEmail ?? null,
    newCommitterDate: params.newCommitterDate ?? null,
    newCommitterOffset: params.newCommitterOffset ?? null,
    newMessage: params.newMessage ?? null,
  });
}
