use crate::api::{ Api, query::{ self, project_details::ToolsProject } };
use crate::utils;
use tracing::{ info_span };

pub struct SyncClosedThingOpts<'a> {
    pub api: &'a Api,
    pub project_details: &'a ToolsProject,
    pub closed_pr_status_name: &'a str,
    pub closed_issue_status_name: &'a str,
    pub team_members: &'a [String],
    pub org: &'a str
}

pub async fn sync_closed_things(opts: SyncClosedThingOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncClosedThingOpts {
        api,
        project_details,
        closed_pr_status_name,
        closed_issue_status_name,
        team_members,
        org
    } = opts;

    let span = info_span!("sync_assigned_issues");
    let _ = span.enter();

    // Get all open assigned issues we want on the board:
    let closed_things = query::closed_things::run(api, org, team_members).await?;

    // Sync closed issues to the project board:
    utils::sync_issues_to_project(utils::SyncIssuesToProjectOpts {
        api,
        project_details,
        field_status_value_name: closed_issue_status_name,
        org,
        issue_ids: &closed_things.closed_issues
    }).await?;

    // Sync closed PRs to the project board:
    utils::sync_issues_to_project(utils::SyncIssuesToProjectOpts {
        api,
        project_details,
        field_status_value_name: closed_pr_status_name,
        org,
        issue_ids: &closed_things.merged_prs
    }).await?;

    Ok(())
}
