use crate::api::{ Api, query::{ self, project_details::ToolsProject } };
use crate::utils;
use tracing::{ info_span };

pub struct SyncAssignedIssuesOpts<'a> {
   pub api: &'a Api,
   pub local_issue_repo_name: &'a str,
   pub project_details: &'a ToolsProject,
   pub field_status_value_name: &'a str,
   pub team_members: &'a [String],
   pub org: &'a str
}

pub async fn sync_assigned_issues(opts: SyncAssignedIssuesOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncAssignedIssuesOpts {
        api,
        local_issue_repo_name,
        project_details,
        field_status_value_name,
        team_members,
        org
    } = opts;

    let span = info_span!("sync_assigned_issues");
    let _ = span.enter();

    // Get all open assigned issues we want on the board:
    let assigned_issue_ids = query::open_assigned_issues::run(api, org, team_members, local_issue_repo_name).await?;

    // Sync to the project board:
    utils::sync_issues_to_project(utils::SyncIssuesToProjectOpts {
        api,
        project_details,
        field_status_value_name,
        org,
        issue_ids: &assigned_issue_ids
    }).await?;

    Ok(())
}