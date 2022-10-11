use crate::api::{ Api, query::{ self, project_details::ToolsProject }, mutation };

pub struct SyncPrsNeedingReviewOpts<'a> {
    pub api: &'a Api,
    pub project_details: &'a ToolsProject,
    pub field_status_value_name: &'a str,
    pub team_group_name: &'a str,
    pub org: &'a str
}

pub async fn sync_prs_needing_review(opts: SyncPrsNeedingReviewOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncPrsNeedingReviewOpts {
        api,
        project_details,
        field_status_value_name,
        team_group_name,
        org
    } = opts;

    // the id of the status column we want to get items for:
    let status_field_value_id = project_details.status.options
        .iter()
        .find(|o| o.name.trim().to_ascii_lowercase().starts_with(&field_status_value_name.to_ascii_lowercase()))
        .map(|o| &*o.id)
        .ok_or(anyhow::anyhow!("Could not find the '{field_status_value_name}' status in the local project board"))?;

    // Get all PRs needing review from the board:
    let issues_needing_review = query::issues_needing_review::run(api, team_group_name).await?;
    // Get all of the items currently on the board in the column we care about:
    let items: Vec<_> = query::project_items::run(api, org, project_details.number)
        .await?
        .into_iter()
        .filter(|item| item.status_field_value_id.as_deref() == Some(status_field_value_id))
        .collect();

    // Do a naive diff to work out which issues to add and which items to remove:
    let issue_ids_to_add: Vec<_> = issues_needing_review
        .iter()
        .filter(|issue_id| !items.iter().any(|item| &item.content_id == *issue_id))
        .collect();
    let item_ids_to_remove: Vec<_> = items
        .iter()
        .filter(|item| !issues_needing_review.iter().any(|issue| issue == &item.content_id))
        .map(|item| &*item.item_id)
        .collect();

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

    for item_id in item_ids_to_remove {
        mutation::remove_item_from_project::run(
            api,
            &project_details.id,
            item_id
        ).await?;
    }

    Ok(())
}