use crate::api::{ Api, query::{ self, project_details::ToolsProject } };
use crate::utils;
use tracing::{ info_span };

pub struct SyncDraftPrOpts<'a> {
    pub api: &'a Api,
    pub project_details: &'a ToolsProject,
    pub field_status_value_name: &'a str,
    pub team_group_name: &'a str,
    pub team_members: &'a [String],
    pub team_repos: &'a [String],
    pub org: &'a str
}

pub async fn sync_draft_prs(opts: SyncDraftPrOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncDraftPrOpts {
        api,
        project_details,
        field_status_value_name,
        team_group_name,
        team_members,
        team_repos,
        org
    } = opts;

    let span = info_span!("sync_draft_prs");
    let _ = span.enter();

    // Get all PRs in draft status:
    let issue_ids_in_draft: Vec<String> = query::team_prs::run(api, org, team_group_name, team_members, team_repos)
        .await?
        .into_iter()
        .filter(|issue| issue.draft)
        .map(|issue| issue.id)
        .collect();

    // Sync to the project board:
    utils::sync_issues_to_project(utils::SyncIssuesToProjectOpts {
        api,
        project_details,
        field_status_value_name,
        org,
        issue_ids: &issue_ids_in_draft
    }).await?;

    Ok(())
}
