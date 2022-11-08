use crate::api::Api;
use crate::variables;
use std::collections::HashSet;

const NEEDS_REVIEW: &str = r#"
    query TeamPullRequests($mentions_query:String!, $assigned_query:String!, $team_query:String!) {
        mentions: search(last:100, query:$mentions_query, type:ISSUE) {
            nodes {
                ... on PullRequest {
                    id,
                    draft: isDraft
                }
            }
        }
        assigned: search(last:100, query:$assigned_query, type:ISSUE) {
            nodes {
                ... on PullRequest {
                    id,
                    draft: isDraft
                }
            }
        }
        team: search(last:100, query:$team_query, type:ISSUE) {
            nodes {
                ... on PullRequest {
                    id,
                    draft: isDraft
                }
            }
        }
    }
"#;

pub async fn run(api: &Api, org: &str, team_group_name: &str, team_members:&[String], team_repos:&[String]) -> Result<Vec<Issue>, anyhow::Error> {
    // The shape we want to deserialize to.
    #[derive(serde::Deserialize)]
    struct QueryResult {
        mentions: QuerySearch,
        assigned: QuerySearch,
        team: QuerySearch
    }
    #[derive(serde::Deserialize)]
    struct QuerySearch {
        nodes: Vec<Issue>
    }

    // Find all PRs where the tools team is an assigned reviewer. These will disappear once anybody has reviewed them but might at least
    // help to catch some PRs we've been asked to review (perhaps on external repos).
    let assigned_query = format!("is:pr is:open sort:updated-desc org:{org} team-review-requested:{team_group_name}");
    // Find all PRs where our team group is in the body (why? because if you request a review from a team, the team disappears as soon as one
    // person has reviewed the PR, and that's no good becasue we want the PR to show up until merged)
    let mentions_query = format!("is:pr is:open sort:updated-desc org:{org} in:body '{team_group_name}'");
    // Find all PRs that are authored by team members in team controlled repos.
    let mut team_repos_query = String::new();
    for repo in team_repos {
        team_repos_query.push_str(" repo:");
        team_repos_query.push_str(org);
        team_repos_query.push_str("/");
        team_repos_query.push_str(repo);
    }
    let mut team_members_query = String::new();
    for name in team_members {
        team_members_query.push_str(" author:");
        team_members_query.push_str(&name);
    }
    let team_query = format!("is:pr is:open {team_repos_query} {team_members_query}");

    let res: QueryResult = api.query(NEEDS_REVIEW, variables!(
        "assigned_query": assigned_query,
        "mentions_query": mentions_query,
        "team_query": team_query
    )).await?;

    // Remove any dupes:
    let set: HashSet<Issue> = res.team.nodes.into_iter()
        .chain(res.assigned.nodes)
        .chain(res.mentions.nodes)
        .collect();

    Ok(set.into_iter().collect())
}

#[derive(Hash, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Issue {
    pub id: String,
    pub draft: bool
}