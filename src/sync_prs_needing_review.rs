use crate::api::{ Api, query::{ self, project_details::ToolsProject } };
use crate::utils;
use tracing::{ info_span };

pub struct SyncPrsNeedingReviewOpts<'a> {
    pub api: &'a Api,
    pub project_details: &'a ToolsProject,
    pub field_status_value_name: &'a str,
    pub team_group_name: &'a str,
    pub team_members: &'a [String],
    pub team_repos: &'a [String],
    pub org: &'a str
}

pub async fn sync_prs_needing_review(opts: SyncPrsNeedingReviewOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncPrsNeedingReviewOpts {
        api,
        project_details,
        field_status_value_name,
        team_group_name,
        team_members,
        team_repos,
        org
    } = opts;

    let span = info_span!("sync_prs_needing_review");
    let _ = span.enter();

    // Get all PRs needing review from the board:
    let issue_ids_needing_review: Vec<String> = query::team_prs::run(api, org, team_group_name, team_members, team_repos)
        .await?
        .into_iter()
        .filter(|issue| !issue.draft)
        .map(|issue| issue.id)
        .collect();

    // Sync to the project board:
    utils::sync_issues_to_project(utils::SyncIssuesToProjectOpts {
        api,
        project_details,
        field_status_value_name,
        org,
        issue_ids: &issue_ids_needing_review
    }).await?;

    Ok(())
}
