use crate::api::Api;
use crate::variables;

const MUTATION: &str = r#"
    mutation RemoveItemFromProject($project_id:ID!, $item_id:ID!) {
        deleteProjectV2Item(input:{projectId:$project_id, itemId:$item_id}) {
            deletedItemId
        }
    }
"#;

/// Returns an "item ID" which represents the project card.
pub async fn run(api: &Api, project_id: &str, item_id: &str) -> Result<(), anyhow::Error> {
    #[derive(serde::Deserialize)]
    struct QueryResult {}

    let _res: QueryResult = api.query(MUTATION, variables!{
        "project_id": project_id,
        "item_id": item_id
    }).await?;

    Ok(())
}