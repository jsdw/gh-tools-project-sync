use crate::api::Api;
use crate::variables;

const MUTATION: &str = r#"
    mutation AssignIssueToProject($project_id:ID!, $issue_id:ID!) {
        res: addProjectV2ItemById(input: {projectId:$project_id, contentId:$issue_id}) {
            item {
                id
            }
        }
    }
"#;

/// Returns an "item ID" which represents the project card.
pub async fn run(api: &Api, issue_id: &str, project_id:&str) -> Result<String, anyhow::Error> {
    #[derive(serde::Deserialize)]
    struct QueryResult {
        res: QueryAddIssue
    }
    #[derive(serde::Deserialize)]
    struct QueryAddIssue {
        item: QueryAddItemId
    }
    #[derive(serde::Deserialize)]
    struct QueryAddItemId {
        id: String
    }

    let res: QueryResult = api.query(MUTATION, variables!{
        "project_id": project_id,
        "issue_id": issue_id
    }).await?;

    Ok(res.res.item.id)
}