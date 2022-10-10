use crate::api::{ Api, common::State };
use crate::variables;

const MUTATION: &str = r#"
    mutation UpdateIssue($issue_id:ID!, $title:String, $body:String, $state:IssueState) {
        updateIssue(input:{ id:$issue_id, title:$title, body:$body, state:$state }) {
            issue {
                id
            }
        }
    }
"#;

pub async fn run(api: &Api, issue_id: &str, title: Option<&str>, body: Option<&str>, state: Option<State>) -> Result<(), anyhow::Error> {
    if title.is_none() && body.is_none() && state.is_none() {
        // Nothing to do if nothing given to update.
        return Ok(())
    }

    let mut variables = variables!{
        "issue_id": issue_id
    };

    // If a field is present but set to null, it'll be unset on the
    // issue. We Just want to ignore anything not provided.
    if let Some(title) = title {
        variables.push("title", title);
    }
    if let Some(body) = body {
        variables.push("body", body);
    }
    if let Some(state) = state {
        variables.push("state", state);
    }

    #[derive(serde::Deserialize)]
    struct QueryResult {}

    let _res: QueryResult = api.query(MUTATION, variables).await?;

    Ok(())
}