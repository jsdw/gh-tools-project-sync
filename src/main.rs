mod api;
mod utils;
mod sync_milestones;
mod sync_assigned_issues;
mod sync_closed_things;
mod sync_draft_prs;
mod sync_prs_needing_review;

use api::Api;
use sync_milestones::{ sync_milestones, SyncMilestoneOpts };
use sync_assigned_issues::{ sync_assigned_issues, SyncAssignedIssuesOpts };
use sync_closed_things::{ sync_closed_things, SyncClosedThingOpts };
use sync_draft_prs::{ sync_draft_prs, SyncDraftPrOpts };
use sync_prs_needing_review::{ sync_prs_needing_review, SyncPrsNeedingReviewOpts };

// The organisation to search for projects, repos and issues in.
const ORG: &str = "paritytech";

// The rpositories within the above organisation that we will
// sync milestones from.
const REPO_NAMES: &[&str] = &[
    "subxt",
    "jsonrpsee",
    "soketto",
    "scale-decode",
    "scale-encode",
    "scale-value",
    "scale-bits",
    "substrate-telemetry",
    "desub",
];

// Team members that we'll search for assigned issues for to
// sync those to our local project board.
const TEAM_MEMBERS: &[&str] = &[
    "jsdw",
    "niklasad1",
    "lexnv",
    "tadeohepperle",
];

// The repository within the organisation above to use to create
// issues in whose sole purpose is to be kept in sync with milestones
// and be something that can be added to project boards.
const PROJECT_REPO_NAME: &str = "subxt-team-milestones";

// The number of the "local" project. This project is expected to have
// a "Status" field with statuses beginning with the following text.
const LOCAL_PROJECT_NUMBER: usize = 22;

// Statuses to look for in the local project to sync lists of milestones,
// issues assigned to team members, and PRs needing review from the team.
const MILESTONE_STATUS_NAME: &str = "milestone";
const ASSIGNED_ISSUE_STATUS_NAME: &str = "in progress";
const DRAFT_PR_STATUS_NAME: &str = "draft prs";
const NEEDS_REVIEW_STATUS_NAME: &str = "needs review";
const FINISHED_PR_STATUS_NAME: &str = "closed prs";
const FINISHED_ISSUE_STATUS_NAME: &str = "closed issues";

// Any PRs assigned this group to review them will show up in the NEEDS_REVIEW
// status on the local project board.
const TOOLS_TEAM_GROUP: &str = "paritytech/subxt-team";

// The public roadmap project number. We implicitly expect this to have three
// fields:
// - Status (a single select with values like "open" and "closed")
// - Deadline (a single select field with dates in the format "Aug 2022" (3 letter month then 4 digit year))
// - Team (a single select field with team names)
const PUBLIC_ROADMAP_PROJECT_NUMBER: usize = 27;

// The team name to set on public roadmap issues in the "team" single select field.
const ROADMAP_TEAM_NAME: &str = "Subxt";

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
        LOCAL_PROJECT_NUMBER,
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
        local_issue_repo_name: PROJECT_REPO_NAME,
        project_details: &project_details.tools,
        field_status_value_name: ASSIGNED_ISSUE_STATUS_NAME,
        team_members: &team_members,
        org: ORG,
    }).await?;

    // Sync draft PRs:
    sync_draft_prs(SyncDraftPrOpts {
        api: &api,
        project_details: &project_details.tools,
        field_status_value_name: DRAFT_PR_STATUS_NAME,
        team_group_name: TOOLS_TEAM_GROUP,
        team_members: &team_members,
        team_repos: &repo_names,
        org: ORG,
    }).await?;

    // Sync PRs needing review:
    sync_prs_needing_review(SyncPrsNeedingReviewOpts {
        api: &api,
        project_details: &project_details.tools,
        field_status_value_name: NEEDS_REVIEW_STATUS_NAME,
        team_group_name: TOOLS_TEAM_GROUP,
        team_members: &team_members,
        team_repos: &repo_names,
        org: ORG,
    }).await?;

    // Sync closed issues and PRs:
    sync_closed_things(SyncClosedThingOpts {
        api: &api,
        project_details: &project_details.tools,
        closed_pr_status_name: FINISHED_PR_STATUS_NAME,
        closed_issue_status_name: FINISHED_ISSUE_STATUS_NAME,
        team_members: &team_members,
        org: ORG,
    }).await?;

    Ok(())
}
