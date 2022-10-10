use crate::api::Api;
use crate::variables;
use std::collections::HashMap;

const MILESTONES_QUERY: &str = r#"
    query MilestonesQuery($org: String!, $repo: String!) {
        repository(owner: $org, name: $repo) {
            milestones(first:100, orderBy:{ field:UPDATED_AT, direction:DESC}) {
                nodes {
                    number
                    title
                    description
                    dueOn
                    state
                }
            }
        }
    }
"#;

#[derive(Debug, serde::Deserialize)]
pub struct Milestone {
    pub number: usize,
    pub title: String,
    pub state: crate::api::common::State,
    #[serde(rename = "dueOn")]
    pub due_on: Option<DueDate>,
    pub description: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct DueDate {
    #[serde(with = "time::serde::iso8601")]
    pub time: time::OffsetDateTime
}

pub async fn run(api: &Api, org: &str, repo_names: &[String]) -> Result<HashMap<String, Vec<Milestone>>, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        repository: QueryRepository
    }
    #[derive(serde::Deserialize)]
    struct QueryRepository {
        milestones: QueryMilestones
    }
    #[derive(serde::Deserialize)]
    struct QueryMilestones {
        nodes: Vec<Milestone>
    }

    let mut milestones_by_repo = HashMap::new();
    for repo in repo_names {
        let res: QueryResult = api.query(MILESTONES_QUERY, variables!(
            "org": org,
            "repo": repo
        )).await?;

        let milestones = res.repository.milestones.nodes;
        milestones_by_repo.insert(repo.to_string(), milestones);
    }

    Ok(milestones_by_repo)
}