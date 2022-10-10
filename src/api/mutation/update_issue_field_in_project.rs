use crate::api::Api;
use crate::variables;

const MUTATION: &str = r#"
    mutation UpdateIssueFieldInProject($item_id:ID!, $project_id:ID!, $field_id:ID!, $field_value:String!) {
        updateProjectV2ItemFieldValue(input:{
            itemId:$item_id,
            projectId:$project_id,
            fieldId:$field_id,
            # It just happens that we only care about updating single select options:
            value: { singleSelectOptionId:$field_value }
        }) {
            clientMutationId
        }
    }
"#;

/// Returns an "item ID" which represents the project card.
pub async fn run(api: &Api, project_id: &str, item_id: &str, field_id: &str, field_value: &str) -> Result<(), anyhow::Error> {
    #[derive(serde::Deserialize)]
    struct QueryResult {}

    let _res: QueryResult = api.query(MUTATION, variables!{
        "project_id": project_id,
        "item_id": item_id,
        "field_id": field_id,
        "field_value": field_value
    }).await?;

    Ok(())
}