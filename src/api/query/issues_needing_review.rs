use crate::api::Api;
use crate::variables;

const NEEDS_REVIEW: &str = r#"
    query IssuesNeedingTeamReview($mentions_query:String!, $assigned_query:String!) {
        mentions: search(last:100, query:$mentions_query, type:ISSUE) {
            nodes {
                ... on PullRequest {
                    id
                }
            }
        }
        assigned: search(last:100, query:$assigned_query, type:ISSUE) {
            nodes {
                ... on PullRequest {
                    id
                }
            }
        }
    }
"#;

pub async fn run(api: &Api, org: &str, team_group_name: &str) -> Result<Vec<String>, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        mentions: QuerySearch,
        assigned: QuerySearch
    }
    #[derive(serde::Deserialize)]
    struct QuerySearch {
        nodes: Vec<QueryIssue>
    }
    #[derive(serde::Deserialize)]
    struct QueryIssue {
        id: String
    }

    // Find all PRs where the tools team is an assigned reviewer. These will disappear once anybody has reviewed them but might at least
    // help to catch some PRs we've been asked to review (perhaps on external repos).
    let assigned_query = format!("is:pr is:open draft:false sort:updated-desc org:{org} -team:{team_group_name} team-review-requested:{team_group_name}");
    // Find all PRs where our team group is in the body (why? because if you request a review from a team, the team disappears as soon as one
    // person has reviewed the PR, and that's no good becasue we want the PR to show up until merged)
    let mentions_query = format!("is:pr is:open draft:false sort:updated-desc org:{org} in:body '{team_group_name}'");

    let res: QueryResult = api.query(NEEDS_REVIEW, variables!(
        "assigned_query": assigned_query,
        "mentions_query": mentions_query
    )).await?;

    Ok(res.mentions.nodes.into_iter().chain(res.assigned.nodes).map(|n| n.id).collect())
}