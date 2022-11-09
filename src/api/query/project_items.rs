use crate::api::Api;
use crate::variables;

const PROJECT_ITEMS: &str = r#"
    query ProjectItems($org:String!, $project_number:Int!, $cursor:String) {
        organization(login:$org) {
            project: projectV2(number:$project_number) {
                items(first:100, after:$cursor) {
                    page_info: pageInfo {
                        end_cursor: endCursor
                        has_next_page: hasNextPage
                    }
                    nodes {
                        id
                        content {
                            ... on Node {
                                id
                            }
                        }
                        field_values: fieldValues(last:100) {
                            nodes {
                                ... on ProjectV2ItemFieldSingleSelectValue {
                                    field {
                                        ... on ProjectV2SingleSelectField {
                                            name
                                        }
                                    }
                                    option_id: optionId
                                }
                            }
                        }
                    }
                }
            }
        }
    }
"#;

#[derive(Debug)]
pub struct ProjectItem {
    /// ID of the item itself:
    pub item_id: String,
    /// ID of the content (eg issue, PR) of this item:
    pub content_id: String,
    /// So we know which column the item is in:
    pub status_field_value_id: Option<String>,
}

pub async fn run(api: &Api, org: &str, project_number: usize) -> Result<Vec<ProjectItem>, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        organization: QueryProject
    }
    #[derive(serde::Deserialize)]
    struct QueryProject {
        project: QueryItems
    }
    #[derive(serde::Deserialize)]
    struct QueryItems {
        items: QueryNodes
    }
    #[derive(serde::Deserialize)]
    struct QueryNodes {
        nodes: Vec<QueryItem>,
        page_info: QueryPageInfo
    }
    #[derive(serde::Deserialize)]
    struct QueryPageInfo {
        end_cursor: Option<String>,
        has_next_page: bool
    }
    #[derive(serde::Deserialize)]
    struct QueryItem {
        id: String,
        content: QueryItemContent,
        field_values: QueryItemFieldValues
    }
    #[derive(serde::Deserialize)]
    struct QueryItemContent {
        id: String
    }
    #[derive(serde::Deserialize)]
    struct QueryItemFieldValues {
        nodes: Vec<QueryItemFieldValue>
    }
    #[derive(serde::Deserialize, Debug)]
    #[serde(untagged)]
    enum QueryItemFieldValue {
        SingleSelectField {
            option_id: String,
            field: QueryItemFieldDetails
        },
        Unknown {}
    }
    #[derive(serde::Deserialize, Debug)]
    struct QueryItemFieldDetails {
        name: String
    }

    let mut items = Vec::new();
    let mut cursor = None;
    loop {
        let res: QueryResult = api.query(PROJECT_ITEMS, variables!(
            "org": org,
            "project_number": project_number,
            "cursor": cursor
        )).await?;

        for item in res.organization.project.items.nodes {
            items.push(ProjectItem {
                content_id: item.content.id,
                item_id: item.id,
                status_field_value_id: item.field_values.nodes.into_iter().find_map(|n| {
                    // Only return the field value ID if we find the status field in our fields list:
                    match n {
                        QueryItemFieldValue::SingleSelectField { option_id, field } if field.name == "Status" => {
                            Some(option_id)
                        }
                        _ => None
                    }
                })
            })
        }

        cursor = res.organization.project.items.page_info.end_cursor;
        if !res.organization.project.items.page_info.has_next_page || cursor.is_none() {
            break
        }
    }


    Ok(items)
}