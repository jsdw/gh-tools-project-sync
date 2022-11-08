use crate::api::{ Api, query::{ self, project_details::ToolsProject }, mutation };
use tracing::info;

/// Options for `sync_issues_to_project`
pub struct SyncIssuesToProjectOpts<'a> {
    pub api: &'a Api,
    pub org: &'a str,
    pub field_status_value_name: &'a str,
    pub project_details: &'a ToolsProject,
    pub issue_ids: &'a [String],
}

/// Sync issue IDs given with items in the `field_status_value_name` field in the project.
pub async fn sync_issues_to_project(opts: SyncIssuesToProjectOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncIssuesToProjectOpts {
        api,
        org,
        field_status_value_name,
        project_details,
        issue_ids,
    } = opts;

    // the id of the status column we want to get items for:
    let status_field_value_id = project_details.status.options
        .iter()
        .find(|o| o.name.trim().to_ascii_lowercase().starts_with(&field_status_value_name.to_ascii_lowercase()))
        .map(|o| &*o.id)
        .ok_or(anyhow::anyhow!("Could not find the '{field_status_value_name}' status in the local project board"))?;

    // Items in that column:
    let items: Vec<_> = query::project_items::run(api, org, project_details.number)
        .await?
        .into_iter()
        .filter(|item| item.status_field_value_id.as_deref() == Some(status_field_value_id))
        .collect();

    // Do a naive diff to work out which issues to add and which items to remove:
    let issue_ids_to_add: Vec<_> = issue_ids
        .iter()
        .filter(|issue_id| !items.iter().any(|item| &item.content_id == *issue_id))
        .collect();
    let item_ids_to_remove: Vec<_> = items
        .iter()
        .filter(|item| !issue_ids.iter().any(|issue| issue == &item.content_id))
        .map(|item| &*item.item_id)
        .collect();

    if !issue_ids_to_add.is_empty() {
        info!("✅ creating {} items in `{}` on project board", issue_ids_to_add.len(), field_status_value_name);
        for issue_id in issue_ids_to_add {
            let item_id = mutation::add_item_to_project::run(
                api,
                issue_id,
                &project_details.id
            ).await?;
            mutation::update_item_field_in_project::run(
                api,
                &project_details.id,
                &item_id,
                &project_details.status.id,
                status_field_value_id
            ).await?;
        }
    }

    if !item_ids_to_remove.is_empty() {
        info!("❌ removing {} items in `{}` on project board", item_ids_to_remove.len(),field_status_value_name);
        for item_id in item_ids_to_remove {
            mutation::remove_item_from_project::run(
                api,
                &project_details.id,
                item_id
            ).await?;
        }
    }

    Ok(())
}