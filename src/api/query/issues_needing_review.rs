use crate::api::Api;
use crate::variables;

const NEEDS_REVIEW: &str = r#"
    query IssuesNeedingTeamReview($query:String!) {
        search(last:100, query:$query, type:ISSUE) {
            nodes {
                ... on PullRequest {
                    id
                }
            }
        }
    }
"#;

pub async fn run(api: &Api, team_group_name: &str) -> Result<Vec<String>, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        search: QuerySearch
    }
    #[derive(serde::Deserialize)]
    struct QuerySearch {
        nodes: Vec<QueryIssue>
    }
    #[derive(serde::Deserialize)]
    struct QueryIssue {
        id: String
    }

    // Build our search query. Want output a bit like:
    // "is:pr is:open team-review-requested:paritytech/tools-team"
    let query = format!("is:pr is:open team-review-requested:{team_group_name}");

    let res: QueryResult = api.query(NEEDS_REVIEW, variables!(
        "query": query
    )).await?;

    Ok(res.search.nodes.into_iter().map(|n| n.id).collect())
}