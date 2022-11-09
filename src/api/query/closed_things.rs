use crate::api::Api;
use crate::variables;

const CLOSED_ISSUES_QUERY: &str = r#"
query ClosedIssues($query:String!, $cursor:String) {
    search(after:$cursor, last: 100, query:$query, type: ISSUE) {
        nodes {
            ... on Issue {
                id
            }
        }
        page: pageInfo {
            has_next_page: hasNextPage
            cursor: endCursor
        }
    }
}
"#;

const MERGED_PRS_QUERY: &str = r#"
    query MergedPrs($query:String!, $cursor:String) {
        search(after:$cursor, last: 100, query:$query, type: ISSUE) {
            nodes {
                ... on PullRequest {
                    id
                }
            }
            page: pageInfo {
                has_next_page: hasNextPage
                cursor: endCursor
            }
        }
    }
"#;

pub async fn run(api: &Api, org: &str, user_names: &[String]) -> Result<ClosedThings, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        search: QuerySearch
    }
    #[derive(serde::Deserialize)]
    struct QuerySearch {
        nodes: Vec<QueryItem>,
        page: PageInfo
    }
    #[derive(serde::Deserialize)]
    struct PageInfo {
        has_next_page: bool,
        cursor: Option<String>
    }
    #[derive(serde::Deserialize)]
    struct QueryItem {
        id: String
    }

    let mut assignees_query = String::new();
    for name in user_names {
        assignees_query.push_str(" assignee:");
        assignees_query.push_str(&name);
    }

    let mut authors_query = String::new();
    for name in user_names {
        authors_query.push_str(" author:");
        authors_query.push_str(&name);
    }

    // We'll get all things closed in the last 28 days
    let a_month_ago = {
        let now = time::OffsetDateTime::now_utc();
        let format = time::format_description::well_known::Iso8601::DEFAULT;
        let a_month_ago: time::OffsetDateTime = now - (time::Duration::DAY * 28);
        a_month_ago.format(&format).expect("valid iso8601")
    };

    let closed_issues_query = format!("type:issue state:closed closed:>={a_month_ago} org:{org} {assignees_query}");
    let merged_prs_query = format!("type:pr is:merged state:closed closed:>={a_month_ago} org:{org} {authors_query}");

    async fn do_search(api: &Api, ql: &str, query: &str) -> Result<Vec<String>, anyhow::Error> {
        let mut cursor = None;
        let mut ids = Vec::new();
        loop {
            let res: QueryResult = api.query(ql, variables!(
                "query": query,
                "cursor": cursor
            )).await?;

            let new_ids = res.search.nodes.into_iter().map(|n| n.id);
            ids.extend(new_ids);

            cursor = res.search.page.cursor;
            if !res.search.page.has_next_page || cursor.is_none() {
                break;
            }
        }
        Ok(ids)
    }

    let closed_issues = do_search(api, CLOSED_ISSUES_QUERY, &closed_issues_query).await?;
    let merged_prs = do_search(api, MERGED_PRS_QUERY, &merged_prs_query).await?;


    Ok(ClosedThings {
        closed_issues,
        merged_prs
    })
}

pub struct ClosedThings {
    pub closed_issues: Vec<String>,
    pub merged_prs: Vec<String>
}