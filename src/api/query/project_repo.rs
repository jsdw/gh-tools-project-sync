use crate::api::Api;
use crate::variables;
use std::collections::HashMap;

const PROJECT_ISSUES_QUERY: &str = r#"
    query ProjectRepo($org: String!, $repo: String!) {
        repository(owner: $org, name: $repo) {
            id
            issues(first:100, orderBy:{field:UPDATED_AT, direction:DESC}) {
                nodes {
                    id
                    title
                    state
                    body
                    projectItems(last:100) {
                        nodes {
                            id
                            project {
                                number
                            }
                            status: fieldValueByName(name:"Status") {
                                ... on ProjectV2ItemFieldSingleSelectValue {
                                    optionId
                                }
                            }
                            deadline: fieldValueByName(name:"Deadline") {
                                ... on ProjectV2ItemFieldSingleSelectValue {
                                    optionId
                                }
                            }
                            team: fieldValueByName(name:"Team") {
                                ... on ProjectV2ItemFieldSingleSelectValue {
                                    optionId
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
pub struct ProjectRepo {
    pub id: String,
    pub issues: Vec<ProjectIssue>
}

#[derive(Debug)]
pub struct ProjectIssue {
    pub id: String,
    pub title: String,
    pub state: crate::api::common::State,
    pub description: String,
    /// None if not in this project.
    pub tools_project: Option<ToolsProject>,
    /// None if not in this project.
    pub roadmap_project: Option<RoadmapProject>
}

#[derive(Debug)]
pub struct ToolsProject {
    pub item_id: String,
    pub status_id: Option<String>
}

#[derive(Debug)]
pub struct RoadmapProject {
    pub item_id: String,
    pub status_id: Option<String>,
    pub deadline_id: Option<String>,
    pub team_id: Option<String>,
}

pub async fn run(api: &Api, org: &str, repo_name: &str, tools_project: usize, roadmap_project: usize) -> Result<ProjectRepo, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        repository: QueryRepository
    }
    #[derive(serde::Deserialize)]
    struct QueryRepository {
        id: String,
        issues: QueryIssues
    }
    #[derive(serde::Deserialize)]
    struct QueryIssues {
        nodes: Vec<QueryIssue>
    }
    #[derive(serde::Deserialize)]
    struct QueryIssue {
        id: String,
        title: String,
        state: crate::api::common::State,
        #[serde(rename = "body")]
        body_text: String,
        #[serde(rename = "projectItems")]
        project_items: QueryProjectItems
    }
    #[derive(serde::Deserialize)]
    struct QueryProjectItems {
        nodes: Vec<QueryProjectItem>
    }
    #[derive(serde::Deserialize)]
    struct QueryProjectItem {
        id: String,
        project: QueryProjectNumber,
        // Projects that aren't the ones we're looking for
        // might have all sorts of random stuff, so be flexible
        // here.
        #[serde(flatten)]
        rest: HashMap<String, serde_json::Value>,
    }
    #[derive(serde::Deserialize)]
    struct QueryProjectNumber {
        number: usize
    }

    let res: QueryResult = api.query(PROJECT_ISSUES_QUERY, variables!(
        "org": org,
        "repo": repo_name
    )).await?;

    let issues = res.repository.issues.nodes.into_iter().map(|issue| {
        let tools_project = issue.project_items.nodes.iter().find(|item| {
            item.project.number == tools_project
        });
        let roadmap_project = issue.project_items.nodes.iter().find(|item| {
            item.project.number == roadmap_project
        });

        let get_field_id = |map: &HashMap<String,serde_json::Value>, field: &str| {
            map.get(field)
                .and_then(|s| s.as_object())
                .and_then(|s| s.get("optionId"))
                .and_then(|n| n.as_str())
                .map(|n| n.to_owned())
        };

        ProjectIssue {
            description: issue.body_text,
            id: issue.id,
            state: issue.state,
            title: issue.title,
            tools_project: tools_project.map(|p| {
                ToolsProject {
                    item_id: p.id.clone(),
                    status_id: get_field_id(&p.rest, "status")
                }
            }),
            roadmap_project: roadmap_project.map(|p| {
                RoadmapProject {
                    item_id: p.id.clone(),
                    status_id: get_field_id(&p.rest, "status"),
                    deadline_id: get_field_id(&p.rest, "deadline"),
                    team_id: get_field_id(&p.rest, "team")
                }
            }),
        }
    });

    Ok(ProjectRepo {
        id: res.repository.id,
        issues: issues.collect()
    })
}
