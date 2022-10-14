# Github Milestone => Project Sync

**This code is somewhat hardcoded to a specific use case, so feature requests and such for other use cases will be closed.**

When this tool is running, the following rules apply:

- Any milestones that you create in the watched repositories will be synced to our local project board.
  - You can manually add issues to the roadmap and project board too directly and they will be left alone.
- If the milestone title begins with `[public]`, the milestone will also be synced to the public roadmap.
  - If you want the milestone to be public, remember to give it a **Due Date** as well (ideally one that is
    currently visible on the public roadmap).
- If you're working on an issue, assign it to yourself and it'll show up on the local project board.
- If you have an open PR you'd like reviewed:
  - Open PRs in team projects created by team members will show up on the board automatically.
  - Any open PRs in the organisation mentioning the github team group in their description will show up.
  - Any open PRs where the team group is an assigned reviewer will show up (but note; github removes the assigned review once anybody reviews the PR, which would lead to it being removed from the project board again).

## Dev notes

This tool requires a **github access token** to be provided via an env var that has permission to create, edit and delete issues and project items.

The tool is stateless, and on each run will ensure that the above are kept in sync. It tries to limit the number of API calls made on each run to only those that are absolutely necessary.

The idea is that this can run at some time interval (eg every 15 minutes) as a cron job in order to keep things synced to project boards.

The tool uses the github GraphQL API. It's _very_ highly recommended that in order to develop and debug, you install something like `GraphiQL` (with URL `https://api.github.com/graphql` and `Authorization: bearer $TOKEN` header), which makes it possible to explore the Github GraphQL API and create/debug calls.

### Cross compiling from a Mac

Just because I'm on a mac and want to compile this for linux boxes:

```
brew install FiloSottile/musl-cross/musl-cross
rustup target add x86_64-unknown-linux-musl
TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl
```