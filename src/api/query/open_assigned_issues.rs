use crate::api::Api;
use crate::variables;

const ISSUES_QUERY: &str = r#"
    query OpenAssignedIssues($query:String!) {
        search(last:100, query:$query, type:ISSUE) {
            nodes {
                ... on Issue {
                    id,
                    repository{ name }
                }
            }
        }
    }
"#;

pub async fn run(api: &Api, org: &str, user_names: &[String], local_issue_repo_name: &str) -> Result<Vec<String>, anyhow::Error> {
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
    struct QueryRepository {
        name: String,
    }
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum QueryIssue {
        Issue { id: String, repository: QueryRepository },
        Unknown {}
    }

    // Build our search query. Want output a bit like:
    // "state:open org:paritytech assignee:jsdw assignee:niklasad1"
    let mut user_names_query = String::new();
    for name in user_names {
        user_names_query.push_str(" assignee:");
        user_names_query.push_str(&name);
    }
    let query = format!("state:open org:{org} {user_names_query}");

    let res: QueryResult = api.query(ISSUES_QUERY, variables!(
        "org": org,
        "query": query
    )).await?;

    let issue_ids = res.search.nodes
        .into_iter()
        .filter_map(|n| {
            match n {
                QueryIssue::Issue { id, repository } if repository.name != local_issue_repo_name => Some(id),
                _ => None
            }
        })
        .collect();

    Ok(issue_ids)
}