use crate::api::Api;
use crate::variables;

const PROJECTS_QUERY: &str = r#"
    query ProjectsQuery($org: String!, $tools_project:Int!, $roadmap_project:Int!) {
        organization(login: $org) {
            tools: projectV2(number: $tools_project) {
                id
                number,
                status: field(name:"Status") {
                    ... on ProjectV2SingleSelectField {
                        id
                        name
                        options {
                            id
                            name
                        }
                    }
                }
            }
            roadmap: projectV2(number: $roadmap_project) {
                id
                number,
                status: field(name:"Status") {
                    ... on ProjectV2SingleSelectField {
                        id
                        name
                        options {
                            id
                            name
                        }
                    }
                }
                team: field(name:"Team") {
                    ... on ProjectV2SingleSelectField {
                        id
                        name
                        options {
                            id
                            name
                        }
                    }
                }
                deadline: field(name:"Deadline") {
                    ... on ProjectV2SingleSelectField {
                        id
                        name
                        options {
                            id
                            name
                        }
                    }
                }
            }
        }
    }
"#;

#[derive(Debug, serde::Deserialize)]
pub struct Projects {
    pub tools: ToolsProject,
    pub roadmap: RoadmapProject
}

#[derive(Debug, serde::Deserialize)]
pub struct ToolsProject {
    pub id: String,
    pub number: usize,
    pub status: Field
}

#[derive(Debug, serde::Deserialize)]
pub struct RoadmapProject {
    pub id: String,
    pub number: usize,
    pub status: Field,
    pub team: Field,
    pub deadline: Field,
}

#[derive(Debug, serde::Deserialize)]
pub struct Field {
    pub id: String,
    pub options: Vec<FieldOption>
}

#[derive(Debug, serde::Deserialize)]
pub struct FieldOption {
    pub id: String,
    pub name: String,
}

pub async fn run(api: &Api, org: &str, tools_project: usize, roadmap_project: usize) -> Result<Projects, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        organization: Projects
    }

    let res: QueryResult = api.query(PROJECTS_QUERY, variables!(
        "org": org,
        "tools_project": tools_project,
        "roadmap_project": roadmap_project
    )).await?;

    Ok(res.organization)
}
