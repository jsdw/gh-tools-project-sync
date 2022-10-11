mod api;
mod sync_milestones;
mod sync_assigned_issues;
mod sync_prs_needing_review;

use api::Api;
use sync_milestones::{ sync_milestones, SyncMilestoneOpts };
use sync_assigned_issues::{ sync_assigned_issues, SyncAssignedIssuesOpts };
use sync_prs_needing_review::{ sync_prs_needing_review, SyncPrsNeedingReviewOpts };

const ORG: &str = "paritytech";
const REPO_NAMES: &[&str] = &[
    "subxt",
    "jsonrpsee",
    "soketto",
    "scale-decode",
    "scale-value",
    "scale-bits",
    "substrate-telemetry",
];
const TEAM_MEMBERS: &[&str] = &[
    "jsdw",
    "niklasad1",
    "Xanewok",
    "lexnv",
];
const PROJECT_REPO_NAME: &str = "tools-team-milestones";
const TOOLS_ROADMAP_PROJECT_NUMBER: usize = 22;
const PUBLIC_ROADMAP_PROJECT_NUMBER: usize = 27;
const ROADMAP_TEAM_NAME: &str = "Tools";
const MILESTONE_STATUS_NAME: &str = "milestone";
const ASSIGNED_ISSUE_STATUS_NAME: &str = "assigned";
const NEEDS_REVIEW_STATUS_NAME: &str = "needs review";
const TOOLS_TEAM_GROUP: &str = "paritytech/tools-team";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Init the logging.
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
        .init();

    let repo_names: Vec<String> = REPO_NAMES.into_iter().map(|r| r.to_string()).collect();
    let team_members: Vec<String> = TEAM_MEMBERS.into_iter().map(|r| r.to_string()).collect();

    // Get the access token:
    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(e) => anyhow::bail!("Could not obtain GITHUB_TOKEN env var: {e}"),
    };

    // Spin up an API client to talk to github.
    let api = Api::new(token);

    // Project details used by a few places:
    let project_details = api::query::project_details::run(
        &api,
        ORG,
        TOOLS_ROADMAP_PROJECT_NUMBER,
        PUBLIC_ROADMAP_PROJECT_NUMBER
    ).await?;

    // Sync milestones to project boards.
    sync_milestones(SyncMilestoneOpts {
        api: &api,
        project_details: &project_details,
        local_issue_repo_name: PROJECT_REPO_NAME,
        local_project_milestone_status: MILESTONE_STATUS_NAME,
        org: ORG,
        repos_to_sync: &repo_names,
        roadmap_team_name: ROADMAP_TEAM_NAME,
    }).await?;

    // Sync assigned issues:
    sync_assigned_issues(SyncAssignedIssuesOpts {
        api: &api,
        project_details: &project_details.tools,
        field_status_value_name: ASSIGNED_ISSUE_STATUS_NAME,
        team_members: &team_members,
        org: ORG,
    }).await?;

    // Sync PRs needing review
    sync_prs_needing_review(SyncPrsNeedingReviewOpts {
        api: &api,
        project_details: &project_details.tools,
        field_status_value_name: NEEDS_REVIEW_STATUS_NAME,
        team_group_name: TOOLS_TEAM_GROUP,
        org: ORG,
    }).await?;

    Ok(())
}

// - Get all items in team project board
// - get project details so we know what statuses they all are in.
//
// For issues assigned to team members:
// - search to find the issue IDs
// - find the items in the "Assigned" column; remove any with issue IDs that don't match. add any not found.
//
// For issues needing review:
// - search to find issues with tools-team as a reviewer
// - find the items in the "Needs Review" column; remove any with issue IDs that don't match and add any not found.