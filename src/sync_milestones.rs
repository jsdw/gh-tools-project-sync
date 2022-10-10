use crate::api::{ Api, query::{self, project_details::{RoadmapProject, ToolsProject}, milestones::Milestone}, mutation, common::State };
use tracing::{ info_span, warn, info };

#[derive(Debug, Copy, Clone)]
pub struct SyncMilestoneOpts<'a> {
    pub api: &'a Api,
    /// The org in which the repos we're talking about live.
    pub org: &'a str,
    /// Number for the "local"/team project board we want to add milestones to.
    pub local_project_number: usize,
    /// Issues synced to the local project will be given whichever status has a name starting with this.
    pub local_project_milestone_status: &'a str,
    /// Name of the repo that we'll create the issues in which are kept in sync with
    /// our milestones and are shown in the project boards.
    pub local_issue_repo_name: &'a str,
    /// Number of the public parity roadmap project.
    pub roadmap_project_number: usize,
    /// Name of your team as it appears on the parity roadmap project.
    pub roadmap_team_name: &'a str,
    /// A list of repos to find and sync milestones in.
    pub repos_to_sync: &'a [String]
}

/// Sync milestones across our `repos_to_sync` to the project boards.
pub async fn sync_milestones(opts: SyncMilestoneOpts<'_>) -> Result<(), anyhow::Error> {
    let SyncMilestoneOpts {
        api,
        org,
        local_project_number,
        local_project_milestone_status,
        local_issue_repo_name,
        roadmap_project_number,
        roadmap_team_name,
        repos_to_sync
    } = opts;

    // Details for the tools and roadmap projects that we'll find useful:
    let project_details = query::project_details::run(&api, org, local_project_number, roadmap_project_number).await?;
    // Details for the repo that will hold the issues that are kept in sync with milestones:
    let project_repo = query::project_repo::run(&api, org, local_issue_repo_name, local_project_number, roadmap_project_number).await?;
    // All of the milestones found in target repositories:
    let milestones_by_repo = query::milestones::run(&api, org, &repos_to_sync).await?;

    // Look at each milestone (the last 100 most recently updated for every project, open or closed)
    // and make sure that the project boards and such are all in sync with them.
    for (repo, milestones) in &milestones_by_repo {
        for milestone in milestones {
            let span = info_span!("sync_milestone", milestone.number, milestone.title);
            let _ = span.enter();

            let milestone_number = milestone.number;
            let milestone_title = &milestone.title;
            let milestone_body = milestone.description.trim_end_matches('\n');
            let milestone_url = format!("https://github.com/{org}/{repo}/milestone/{milestone_number}");

            // A milestone should be on the public roadmap only if its title starts with "[public]":
            let is_milestone_public = milestone_title.to_ascii_lowercase().starts_with("[public]");
            let milestone_title = match is_milestone_public {
                true => milestone_title["[public]".len()..].trim_start_matches(' ').to_string(),
                false => milestone_title.to_string()
            };

            // The details we want the corresponding issue to have:
            let expected_title = format!("[{repo}] {milestone_title}");
            let expected_body = format!("{milestone_body}\n\n---\n\nHere is the corresponding GitHub milestone:\n\n{milestone_url}\n");
            let expected_state = milestone.state;

            // We match milestones to issues by looking for issues that link to the milestone.
            // Why? Because we generate the links ourselves and the user can't change them by
            // editing the milestone (unlike the title or body).
            let issue = project_repo
                .issues
                .iter()
                .find(|issue| issue.description.contains(&milestone_url));

            match issue {
                // # There is an issue which lines up with the milestone already; make sure it's in sync!
                Some(issue) => {
                    // Make sure that the issue text/description/state is in sync with the milestone:
                    let update_title = (issue.title != expected_title).then_some(&*expected_title);
                    let update_body = (issue.description != expected_body).then_some(&*expected_body);
                    let update_state = (issue.state != expected_state).then_some(expected_state);

                    if update_title.is_some() || update_body.is_some() || update_state.is_some() {
                        info!("☑️  updating issue");
                        mutation::update_issue::run(&api, &issue.id, update_title, update_body, update_state).await?;
                    }

                    match &issue.tools_project {
                        // ## there's already a tools project item; keep it in sync.
                        Some(tools_project) => {
                            let expected_status_id = get_tools_project_status_id(&project_details.tools, local_project_milestone_status)?;
                            let do_update_status = tools_project.status_id.as_deref() != Some(expected_status_id);
                            if do_update_status {
                                info!("☑️  updating local project status");
                                mutation::update_issue_field_in_project::run(
                                    &api,
                                    &project_details.tools.id,
                                    &tools_project.item_id,
                                    &project_details.tools.status.id,
                                    expected_status_id
                                ).await?;
                            }
                        },
                        // ## No tools project item; make one.
                        None => {
                            info!("✅ creating issue");
                            add_tools_project_item(
                                &api,
                                &issue.id,
                                &project_details.tools,
                                local_project_milestone_status
                            ).await?;
                        }
                    }

                    match &issue.roadmap_project {
                        // ## there's already a roadmap project item; keep it in sync.
                        Some(roadmap_project) => {
                            if !is_milestone_public {
                                // ah but we don't want it to be public now, so remove it from the roadmap.
                                info!("❌ removing from public roadmap");
                                mutation::remove_issue_from_project::run(&api, &project_details.roadmap.id, &roadmap_project.item_id).await?;
                            } else {
                                // sync status
                                let expected_status_id = get_roadmap_project_status_id(&project_details.roadmap, expected_state)?;
                                let do_update_status = roadmap_project.status_id.as_deref() != Some(expected_status_id);
                                if do_update_status {
                                    info!("☑️  updating public roadmap item status");
                                    mutation::update_issue_field_in_project::run(
                                        &api,
                                        &project_details.roadmap.id,
                                        &roadmap_project.item_id,
                                        &project_details.roadmap.status.id,
                                        expected_status_id
                                    ).await?;
                                }

                                // sync team
                                let expected_team_id = get_roadmap_project_team_id(&project_details.roadmap, roadmap_team_name)?;
                                let do_update_team = roadmap_project.team_id.as_deref() != Some(expected_team_id);
                                if do_update_team {
                                    info!("☑️  updating public roadmap item team");
                                    mutation::update_issue_field_in_project::run(
                                        &api,
                                        &project_details.roadmap.id,
                                        &roadmap_project.item_id,
                                        &project_details.roadmap.team.id,
                                        expected_team_id
                                    ).await?;
                                }

                                // sync deadline
                                let expected_deadline = milestone
                                    .due_on
                                    .as_ref()
                                    .and_then(|due| try_get_matching_roadmap_deadline(&project_details.roadmap, &due.time));
                                let do_update_deadline = roadmap_project.deadline_id.as_deref() != expected_deadline;
                                if do_update_deadline {
                                    match expected_deadline {
                                        Some(deadline) => {
                                            info!("☑️  updating public roadmap item deadline");
                                            mutation::update_issue_field_in_project::run(
                                                &api,
                                                &project_details.roadmap.id,
                                                &roadmap_project.item_id,
                                                &project_details.roadmap.deadline.id,
                                                deadline
                                            ).await?;
                                        },
                                        None => {
                                            warn!("🛑 milestone due date not found on roadmap");
                                            mutation::clear_issue_field_in_project::run(
                                                &api,
                                                &project_details.roadmap.id,
                                                &roadmap_project.item_id,
                                                &project_details.roadmap.deadline.id,
                                            ).await?;
                                        }
                                    }
                                }
                            }
                        },
                        // ## No roadmap project item? make one if needed.
                        None => {
                            if is_milestone_public {
                                info!("✅ adding to public roadmap");
                                add_roadmap_project_item(
                                    &api,
                                    &issue.id,
                                    &milestone,
                                    &project_details.roadmap,
                                    roadmap_team_name
                                ).await?;
                            }
                        }
                    }
                },
                // # There is not a corresponding issue. Create new issue and assign it to projects as needed.
                None => {
                    // If the milestone is closed, and we can't find an issue for it, just ignore it.
                    // the issue might not have been in the top 100 returned or something. We don't
                    // really care at this point if it's closed anyway.
                    if milestone.state == State::CLOSED {
                        continue
                    }

                    // Create an issue:
                    info!("✅ creating issue");
                    let issue_id = mutation::create_issue::run(
                        &api,
                        &project_repo.id,
                        &expected_title,
                        &expected_body
                    ).await?;

                    // Add the issue to our tools project
                    info!("✅ creating local project item");
                    add_tools_project_item(
                        &api,
                        &issue_id,
                        &project_details.tools,
                        local_project_milestone_status
                    ).await?;

                    // If the milestone is tagged [public], add it to the roadmap too.
                    if is_milestone_public {
                        info!("✅ creating roadmap project item");
                        add_roadmap_project_item(
                            &api,
                            &issue_id,
                            &milestone,
                            &project_details.roadmap,
                            roadmap_team_name
                        ).await?;
                    }
                }
            }

        }
    }

    Ok(())
}

async fn add_tools_project_item(api: &Api, issue_id: &str, tools_project: &ToolsProject, milestone_status_name: &str) -> Result<(), anyhow::Error> {
    let tools_item_id = mutation::assign_issue_to_project::run(&api, &issue_id, &tools_project.id).await?;
    mutation::update_issue_field_in_project::run(
        &api,
        &tools_project.id,
        &tools_item_id,
        &tools_project.status.id,
        get_tools_project_status_id(&tools_project, milestone_status_name)?
    ).await?;
    Ok(())
}

async fn add_roadmap_project_item(api: &Api, issue_id: &str, milestone: &Milestone, roadmap_project: &RoadmapProject, roadmap_team_name: &str) -> Result<(), anyhow::Error> {
    let roadmap_item_id = mutation::assign_issue_to_project::run(&api, &issue_id, &roadmap_project.id).await?;

    // Status (Open or Closed as per the milestone)
    mutation::update_issue_field_in_project::run(
        &api,
        &roadmap_project.id,
        &roadmap_item_id,
        &roadmap_project.status.id,
        get_roadmap_project_status_id(&roadmap_project, milestone.state)?
    ).await?;

    // Team (Tools, or as configured above)
    mutation::update_issue_field_in_project::run(
        &api,
        &roadmap_project.id,
        &roadmap_item_id,
        &roadmap_project.team.id,
        get_roadmap_project_team_id(&roadmap_project, roadmap_team_name)?
    ).await?;

    // Column for due date (match it up to the milestone due date, remove if no due date or no matching column).
    let due_field_id = milestone
        .due_on
        .as_ref()
        .and_then(|due| try_get_matching_roadmap_deadline(&roadmap_project, &due.time));
    match due_field_id {
        Some(due_field_id) => {
            mutation::update_issue_field_in_project::run(
                &api,
                &roadmap_project.id,
                &roadmap_item_id,
                &roadmap_project.deadline.id,
                due_field_id
            ).await?;
        },
        None => {
            warn!("🛑 milestone due date not found on roadmap");
            mutation::clear_issue_field_in_project::run(
                &api,
                &roadmap_project.id,
                &roadmap_item_id,
                &roadmap_project.deadline.id,
            ).await?;
        }
    }

    Ok(())
}

fn get_tools_project_status_id<'a>(details: &'a query::project_details::ToolsProject, milestone_status_name: &str) -> Result<&'a str, anyhow::Error> {
    details.status.options
        .iter()
        .find(|o| o.name.trim().to_ascii_lowercase().starts_with(milestone_status_name))
        .map(|o| &*o.id)
        .ok_or(anyhow::anyhow!("Could not find the '{milestone_status_name}' status in the local project board"))
}

fn get_roadmap_project_status_id(details: &query::project_details::RoadmapProject, state: State) -> Result<&str, anyhow::Error> {
    let state_str = match state {
        State::CLOSED => "closed",
        State::OPEN => "open"
    };

    details.status.options
        .iter()
        .find(|o| o.name.trim().to_ascii_lowercase().starts_with(state_str))
        .map(|o| &*o.id)
        .ok_or(anyhow::anyhow!("We expect to find a state like 'open' or 'closed' on the local project board"))
}

fn get_roadmap_project_team_id<'a>(details: &'a query::project_details::RoadmapProject, team: &str) -> Result<&'a str, anyhow::Error> {
    details.team.options
        .iter()
        .find(|o| o.name.trim().to_ascii_lowercase().starts_with(&team.to_ascii_lowercase()))
        .map(|o| &*o.id)
        .ok_or(anyhow::anyhow!("we expect the Team field to with a value like '{team}' on the roadmap project board"))
}

fn try_get_matching_roadmap_deadline<'a>(details: &'a query::project_details::RoadmapProject, date: &time::OffsetDateTime) -> Option<&'a str> {
    let format = time::format_description::parse("[month repr:short] [year]")
        .expect("should be valid date format");
    let date_str = date.format(&format)
        .expect("date should format properly");

    details.deadline.options
        .iter()
        .find(|o| o.name.trim().to_ascii_lowercase().starts_with(&date_str.to_ascii_lowercase()))
        .map(|o| &*o.id)
}
