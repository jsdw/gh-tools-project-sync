use crate::api::Api;
use crate::variables;

const ISSUES_QUERY: &str = r#"
    query ClosedPrs($query:String!) {
        search(last:100, query:$query, type:ISSUE) {
            nodes {
                kind: __typename,
                ... on Issue {
                    id
                }
                ... on PullRequest {
                    id
                }
            }
        }
    }
"#;

pub async fn run(api: &Api, org: &str, user_names: &[String]) -> Result<Vec<Issue>, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        search: QuerySearch
    }
    #[derive(serde::Deserialize)]
    struct QuerySearch {
        nodes: Vec<QueryPullRequest>
    }
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum QueryPullRequest {
        Issue { kind: Kind, id: String },
        Unknown {}
    }

    // Build our search query. Want output a bit like:
    // "state:closed closed:>=2022-10-03 org:paritytech assignee:jsdw assignee:niklasad1"
    let mut user_names_query = String::new();
    for name in user_names {
        user_names_query.push_str(" assignee:");
        user_names_query.push_str(&name);
    }
    let a_month_ago = {
        let now = time::OffsetDateTime::now_utc();
        let format = time::format_description::well_known::Iso8601::DEFAULT;
        let a_month_ago: time::OffsetDateTime = now - (time::Duration::DAY * 28);
        a_month_ago.format(&format).expect("valid iso8601")
    };
    let query = format!("state:closed closed:>={a_month_ago} org:{org} {user_names_query}");

    let res: QueryResult = api.query(ISSUES_QUERY, variables!(
        "org": org,
        "query": query
    )).await?;

    let pr_ids = res.search.nodes
        .into_iter()
        .filter_map(|n| {
            match n {
                QueryPullRequest::Issue { kind, id } => Some(Issue { kind, id }),
                _ => None
            }
        })
        .collect();

    Ok(pr_ids)
}

#[derive(Copy, Clone, PartialEq, Eq, serde::Deserialize)]
pub enum Kind {
    Issue,
    PullRequest
}

pub struct Issue {
    pub kind: Kind,
    pub id: String
}