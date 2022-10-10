mod api;
mod sync_milestones;

use api::Api;
use sync_milestones::{ sync_milestones, SyncMilestoneOpts };

const ORG: &str = "paritytech";
const REPO_NAMES: &[&str] = &[
    "subxt",
    "jsonrpsee",
    "soketto",
    "scale-decode",
    "scale-value",
    "scale-bits",
    "substrate-telemetry"
];
const PROJECT_REPO_NAME: &str = "tools-team-milestones";
const TOOLS_ROADMAP_PROJECT_NUMBER: usize = 22;
const PUBLIC_ROADMAP_PROJECT_NUMBER: usize = 27;
const ROADMAP_TEAM_NAME: &str = "Tools";
const MILESTONE_STATUS_NAME: &str = "milestone";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Init the logging.
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
        .init();

    let repo_names: Vec<String> = REPO_NAMES.into_iter().map(|r| r.to_string()).collect();

    // Get the access token:
    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(e) => anyhow::bail!("Could not obtain GITHUB_TOKEN env var: {e}"),
    };

    // Spin up an API client to talk to github.
    let api = Api::new(token);

    // Sync milestones to project boards.
    sync_milestones(SyncMilestoneOpts {
        api: &api,
        local_issue_repo_name: PROJECT_REPO_NAME,
        local_project_number: TOOLS_ROADMAP_PROJECT_NUMBER,
        local_project_milestone_status: MILESTONE_STATUS_NAME,
        roadmap_project_number: PUBLIC_ROADMAP_PROJECT_NUMBER,
        org: ORG,
        repos_to_sync: &repo_names,
        roadmap_team_name: ROADMAP_TEAM_NAME,
    }).await?;

    Ok(())
}