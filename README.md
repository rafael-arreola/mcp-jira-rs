# Jira MCP Server

A Model Context Protocol (MCP) server for Jira Cloud, built with Rust. This server enables AI assistants to manage issues, sprints, and boards directly through a standardized interface.

## Features

- 🚀 **Full Jira API Support**: Management of Issues, Sprints, Boards, and Backlog.
- 🧠 **Smart Context**: Automatic conversion to Atlassian Document Format (ADF).
- ⚡ **High Performance**: Extremely fast native Rust implementation.
- 🔍 **Field Filtering**: Reduces token usage (70-90%) via smart filters (`minimal`, `basic`, `standard`, `detailed`).
- 🎯 **Automatic Detection**: Identifies custom fields like "Story Points" without manual configuration.

## Installation

### Prerequisites

- Rust 1.80 or higher.
- A Jira Cloud account and an API Token.

### Build

```bash
git clone https://github.com/rafael-arreola/mcp-jira-rs
cd mcp-jira-rs
cargo build --release
```

The binary will be available at `target/release/jira-rs`.

## Configuration

### Environment Variables

The server requires the following environment variables:

```bash
JIRA_WORKSPACE="your-subdomain"      # e.g., "mycompany" for mycompany.atlassian.net
JIRA_USERNAME="your@email.com"        # Your Atlassian account email
JIRA_TOKEN="your-api-token"           # Generated at id.atlassian.com
```

### Claude Desktop Configuration

Add this to your `claude_desktop_config.json` file:

```json
{
  "mcpServers": {
    "jira": {
      "command": "npx",
      "args": ["-y", "@rafael-arreola/jira-rs@latest"],
      "env": {
        "JIRA_WORKSPACE": "your-subdomain",
        "JIRA_USERNAME": "your@email.com",
        "JIRA_TOKEN": "your-api-token"
      }
    }
  }
}
```

## Available Tools (21)

### 🎫 Issue Management

| Tool                  | Description                                                           |
| --------------------- | --------------------------------------------------------------------- |
| `issue_create`        | Creates Stories, Bugs, Epics, Tasks, and Sub-tasks.                   |
| `issue_get`           | Retrieves issue details with smart **Field Filtering**.               |
| `search_issues`       | Searches issues using JQL or plain text with result limits.           |
| `issue_edit_details`  | Updates summary, description, priority, labels, and issue type.       |
| `issue_set_story_points`| Sets the story point estimation for an issue.                       |
| `issue_update_status` | Transitions issues through the workflow (e.g., "To Do" to "Done").    |
| `issue_assign`        | Assigns issues to users (supports "me", "unassigned", or Account ID). |
| `issue_delete`        | Permanently deletes an issue.                                         |

### 💬 Content and Links

| Tool                   | Description                                                       |
| ---------------------- | ----------------------------------------------------------------- |
| `issue_add_comment`    | Adds comments (supports plain text with ADF conversion).          |
| `issue_delete_comment` | Deletes specific comments by ID.                                  |
| `issue_link`           | Creates semantic links between issues (Blocks, Relates to, etc.). |
| `issue_delete_link`    | Removes existing links between issues.                            |
| `issue_set_parent`     | Links an existing Story/Task to an Epic or removes the link.      |
| `issue_log_work`       | Logs time worked on a task.                                       |

### 🏃 Agile Operations

| Tool                | Description                                                    |
| ------------------- | -------------------------------------------------------------- |
| `board_get_sprints` | Lists sprints for a board or project (active, future, closed). |
| `board_get_backlog` | Retrieves all issues in a board's backlog.                     |
| `sprint_create`     | Creates a new planned sprint.                                  |
| `sprint_update`     | Starts, closes, or updates sprint metadata.                    |
| `sprint_add_issues` | Moves issues to a specific sprint.                             |
| `sprint_delete`     | Deletes a planned sprint.                                      |
| `agile_rank_issues` | Reorders issues (Rank) in the backlog or board.                |

### 🔍 Discovery

| Tool          | Description                                                  |
| ------------- | ------------------------------------------------------------ |
| `fields_list` | Discovers available fields and their IDs for use in filters. |

## Usage Examples

### Retrieve an issue while saving tokens

To avoid saturating the AI context, use the `filter` parameter:

```json
{
  "issueKey": "PROJ-123",
  "filter": "basic"
}
```

_Available presets: `minimal`, `basic`, `standard`, `detailed`._

### Create an issue with story points

Use `storyPoints` for Classic (Company-managed) projects and `storyPointEstimate` for Next-Gen (Team-managed) projects.

```json
{
  "projectKey": "PROJ",
  "summary": "Implement OAuth2 authentication",
  "issueType": "Story",
  "storyPoints": 5,
  "storyPointEstimate": 3
}
```

### Set Story Points

Use `issue_set_story_points` to update the estimation of an existing issue. The tool automatically detects if the project uses "Story Points" or "Story point estimate".

```json
{
  "issueKey": "PROJ-123",
  "storyPoints": 8
}
```

### Change Issue Type

Use `issue_edit_details` to convert an issue to a different type (e.g., from Task to Bug):

```json
{
  "issueKey": "PROJ-123",
  "issueType": "Bug"
}
```

### Link an existing Story to an Epic

Use `issue_set_parent` to organize Stories under Epics:

```json
{
  "issueKey": "PROJ-123",
  "parentKey": "PROJ-100"
}
```

To remove the link, pass an empty `parentKey`:

```json
{
  "issueKey": "PROJ-123",
  "parentKey": ""
}
```

## Troubleshooting

- **Error 401/403**: Verify that `JIRA_TOKEN` is an API Token and not your personal password.
- **Fields not found**: If a custom field does not update, use `fields_list` to find its actual ID (e.g., `customfield_10016`).

## Contributing

If you wish to add a tool, please add the corresponding DTO in `src/domains/` and the implementation in `src/jira.rs` using the `#[rmcp::tool]` macro.
