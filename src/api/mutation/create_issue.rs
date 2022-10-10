use crate::api::Api;
use crate::variables;

const MUTATION: &str = r#"
    mutation CreateIssue($repo_id:ID!, $title:String!, $body:String!) {
        createIssue(input:{ repositoryId:$repo_id, title:$title, body:$body}) {
            issue {
                id
            }
        }
    }
"#;

pub async fn run(api: &Api, repo_id: &str, title: &str, body: &str) -> Result<String, anyhow::Error> {
    #[derive(serde::Deserialize)]
    struct QueryResult {
        #[serde(rename = "createIssue")]
        create_issue: QueryCreateIssue
    }
    #[derive(serde::Deserialize)]
    struct QueryCreateIssue {
        issue: QueryCreateIssueId
    }
    #[derive(serde::Deserialize)]
    struct QueryCreateIssueId {
        id: String
    }

    let res: QueryResult = api.query(MUTATION, variables!{
        "repo_id": repo_id,
        "title": title,
        "body": body
    }).await?;

    Ok(res.create_issue.issue.id)
}