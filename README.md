# Jira MCP Server

A Model Context Protocol (MCP) server implementation for Jira Cloud REST API, built with Rust. This server enables AI assistants to interact with Jira projects, issues, sprints, and more through a standardized interface.

## Features

- 🚀 **Complete Jira API Coverage**: Access to Issues, Agile boards, Projects, Users, and fields
- 🔒 **Secure Authentication**: Basic Auth with environment variables
- ⚡ **High Performance**: Built with Rust for speed and reliability
- 🎯 **Type-Safe**: Strongly typed parameters and responses using JSON Schema
- 📦 **Easy Integration**: Works with Claude Desktop and other MCP clients like Zed or VsCode

## Installation

### Prerequisites

- Rust 1.90 or higher
- Jira Cloud account with API access

### Build from Source

```bash
git clone <repository-url>
cd mcp-jira
cargo build --release
```

The compiled binary will be available at `target/release/mcp-jira-rs`.

## Configuration

### Environment Variables

This MCP server requires three environment variables to connect to your Jira instance:

```bash
JIRA_WORKSPACE="your-workspace"     # Your Jira workspace subdomain (e.g., "mycompany" for mycompany.atlassian.net)
JIRA_USERNAME="your-email@example.com"  # Your Jira account email
JIRA_PASSWORD="your-api-token"      # Your Jira API token (not your account password)
```

### Getting Your Jira API Token

1. Go to [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens)
2. Click "Create API token"
3. Give it a label (e.g., "MCP Server")
4. Copy the generated token and use it as `JIRA_PASSWORD`

### Claude Desktop Configuration

Add to your Claude Desktop config file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "jira": {
      "command": "/path/to/mcp-jira-rs",
      "env": {
        "JIRA_WORKSPACE": "your-workspace",
        "JIRA_USERNAME": "your-email@example.com",
        "JIRA_PASSWORD": "your-api-token"
      }
    }
  }
}
```

## Available Tools

This MCP server provides 45 tools organized into the following categories:

### Issue Management

| Tool Name                    | Description                                | Jira API Reference                                                                                                                                   |
| ---------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-issue_create`          | Create a new issue or sub-task             | [Create Issue](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-post)                              |
| `jira-issue_get`             | Get issue details by ID or key             | [Get Issue](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-issueidorkey-get)                     |
| `jira-issue_edit`            | Edit an existing issue                     | [Edit Issue](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-issueidorkey-put)                    |
| `jira-issue_delete`          | Delete an issue (optionally with subtasks) | [Delete Issue](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-issueidorkey-delete)               |
| `jira-issue_assign`          | Assign an issue to a user                  | [Assign Issue](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-issueidorkey-assignee-put)         |
| `jira-issue_get_transitions` | Get available transitions for an issue     | [Get Transitions](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-issueidorkey-transitions-get)   |
| `jira-issue_transition`      | Perform a transition on an issue           | [Transition Issue](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issues/#api-rest-api-2-issue-issueidorkey-transitions-post) |

### Issue Comments

| Tool Name                   | Description                   | Jira API Reference                                                                                                                                          |
| --------------------------- | ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-issue_comment_get`    | Get all comments for an issue | [Get Comments](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-comments/#api-rest-api-2-issue-issueidorkey-comment-get)         |
| `jira-issue_comment_add`    | Add a comment to an issue     | [Add Comment](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-comments/#api-rest-api-2-issue-issueidorkey-comment-post)         |
| `jira-issue_comment_edit`   | Update an existing comment    | [Update Comment](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-comments/#api-rest-api-2-issue-issueidorkey-comment-id-put)    |
| `jira-issue_comment_delete` | Delete a comment              | [Delete Comment](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-comments/#api-rest-api-2-issue-issueidorkey-comment-id-delete) |

### Issue Social Features

| Tool Name                          | Description                              | Jira API Reference                                                                                                                                        |
| ---------------------------------- | ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-issue_social_add_watcher`    | Add a user as a watcher to an issue      | [Add Watcher](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-watchers/#api-rest-api-2-issue-issueidorkey-watchers-post)      |
| `jira-issue_social_delete_watcher` | Remove a user as a watcher from an issue | [Delete Watcher](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-watchers/#api-rest-api-2-issue-issueidorkey-watchers-delete) |
| `jira-issue_social_vote`           | Add the current user's vote to an issue  | [Add Vote](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-votes/#api-rest-api-2-issue-issueidorkey-votes-post)               |

### Issue Worklogs

| Tool Name                 | Description                | Jira API Reference                                                                                                                                       |
| ------------------------- | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-issue_worklog_get`  | Get worklogs for an issue  | [Get Worklogs](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-worklogs/#api-rest-api-2-issue-issueidorkey-worklog-get)      |
| `jira-issue_worklog_add`  | Add a worklog to an issue  | [Add Worklog](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-worklogs/#api-rest-api-2-issue-issueidorkey-worklog-post)      |
| `jira-issue_worklog_edit` | Update an existing worklog | [Update Worklog](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-worklogs/#api-rest-api-2-issue-issueidorkey-worklog-id-put) |

### Issue Links

| Tool Name                | Description                      | Jira API Reference                                                                                                                             |
| ------------------------ | -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-issue_link_create` | Create a link between two issues | [Create Issue Link](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-links/#api-rest-api-2-issuelink-post)          |
| `jira-issue_link_delete` | Delete an issue link             | [Delete Issue Link](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-links/#api-rest-api-2-issuelink-linkid-delete) |

### Issue Metadata

| Tool Name                             | Description                              | Jira API Reference                                                                                                                                                                      |
| ------------------------------------- | ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-issue_field_get`                | Get all issue fields (system and custom) | [Get Fields](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-fields/#api-rest-api-2-field-get)                                                              |
| `jira-issue_custom_field_get_options` | Get custom field options for a context   | [Get Custom Field Options](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-custom-field-options/#api-rest-api-2-field-fieldid-context-contextid-option-get) |
| `jira-issue_metadata_get_labels`      | Get all available labels                 | [Get Labels](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-labels/#api-rest-api-2-label-get)                                                                    |
| `jira-issue_metadata_get_priorities`  | Get all issue priorities                 | [Get Priorities](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-priorities/#api-rest-api-2-priority-search-get)                                            |
| `jira-issue_metadata_get_resolutions` | Get all issue resolutions                | [Get Resolutions](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-resolutions/#api-rest-api-2-resolution-search-get)                                        |

### Search & JQL

| Tool Name                 | Description                                       | Jira API Reference                                                                                                              |
| ------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `jira-search_execute_jql` | Search for issues using JQL (Jira Query Language) | [Search Issues](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-issue-search/#api-rest-api-2-search-post) |
| `jira-jql_parse`          | Parse and validate JQL queries                    | [Parse JQL](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-jql/#api-rest-api-2-jql-parse-post)           |

### Projects

| Tool Name                       | Description                          | Jira API Reference                                                                                                                                                      |
| ------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-project_get_all`          | Get all projects visible to the user | [Get All Projects](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-projects/#api-rest-api-2-project-search-get)                                   |
| `jira-project_get_versions`     | Get all versions for a project       | [Get Project Versions](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-versions/#api-rest-api-2-project-projectidorkey-version-get)       |
| `jira-project_create_version`   | Create a new version in a project    | [Create Version](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-versions/#api-rest-api-2-version-post)                                   |
| `jira-project_get_components`   | Get all components for a project     | [Get Project Components](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-components/#api-rest-api-2-project-projectidorkey-component-get) |
| `jira-project_create_component` | Create a new component in a project  | [Create Component](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-components/#api-rest-api-2-component-post)                             |
| `jira-project_get_roles`        | Get all project roles                | [Get Project Roles](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-project-roles/#api-rest-api-2-role-get)                                       |

### Users

| Tool Name              | Description                     | Jira API Reference                                                                                                              |
| ---------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `jira-user_search`     | Search for users                | [Find Users](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-user-search/#api-rest-api-2-user-search-get) |
| `jira-user_get_myself` | Get details of the current user | [Get Current User](https://developer.atlassian.com/cloud/jira/platform/rest/v2/api-group-myself/#api-rest-api-2-myself-get)     |

### Agile - Boards

| Tool Name                      | Description                           | Jira API Reference                                                                                                                           |
| ------------------------------ | ------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-agile_get_boards`        | Get all boards visible to the user    | [Get All Boards](https://developer.atlassian.com/cloud/jira/software/rest/api-group-board/#api-rest-agile-1-0-board-get)                     |
| `jira-agile_get_board`         | Get a board by ID                     | [Get Board](https://developer.atlassian.com/cloud/jira/software/rest/api-group-board/#api-rest-agile-1-0-board-boardid-get)                  |
| `jira-agile_get_board_issues`  | Get all issues from a board           | [Get Issues for Board](https://developer.atlassian.com/cloud/jira/software/rest/api-group-board/#api-rest-agile-1-0-board-boardid-issue-get) |
| `jira-agile_get_board_backlog` | Get all issues from a board's backlog | [Get Board Backlog](https://developer.atlassian.com/cloud/jira/software/rest/api-group-board/#api-rest-agile-1-0-board-boardid-backlog-get)  |
| `jira-agile_get_board_sprints` | Get all sprints for a board           | [Get All Sprints](https://developer.atlassian.com/cloud/jira/software/rest/api-group-board/#api-rest-agile-1-0-board-boardid-sprint-get)     |

### Agile - Sprints

| Tool Name                           | Description                                  | Jira API Reference                                                                                                                                |
| ----------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `jira-agile_create_sprint`          | Create a new sprint                          | [Create Sprint](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-post)                        |
| `jira-agile_get_sprint`             | Get sprint details by ID                     | [Get Sprint](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-get)                   |
| `jira-agile_update_sprint`          | Update a sprint (including starting/closing) | [Update Sprint](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-post)               |
| `jira-agile_delete_sprint`          | Delete a sprint                              | [Delete Sprint](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-delete)             |
| `jira-agile_move_issues_to_sprint`  | Move issues to a sprint                      | [Move Issues to Sprint](https://developer.atlassian.com/cloud/jira/software/rest/api-group-sprint/#api-rest-agile-1-0-sprint-sprintid-issue-post) |
| `jira-agile_move_issues_to_backlog` | Move issues to the backlog                   | [Move Issues to Backlog](https://developer.atlassian.com/cloud/jira/software/rest/api-group-backlog/#api-rest-agile-1-0-backlog-issue-post)       |

## Usage Examples

### Creating an Issue

```
Create a bug in project KEY with title "Login page not working" and description "Users cannot log in"
```

### Searching Issues

```
Find all open issues assigned to me in the last sprint
```

### Managing Sprints

```
Create a new sprint named "Sprint 23" starting today for 2 weeks
Move issues KEY-123 and KEY-124 to the active sprint
Close the current sprint and move incomplete items to backlog
```

### Bulk Operations

```
Assign all unassigned tasks in project XYZ to john@example.com
Add "needs-review" label to all issues in the current sprint
Delete all issues from the backlog (with confirmation)
```

## Architecture

### Tool Naming Convention

All tools follow the pattern: `jira-<family>_<action>_<resource>`

- **Prefix**: Always `jira-`
- **Family**: The domain (e.g., `issue`, `agile`, `project`, `user`)
- **Separator**: Underscore `_` (never dots or other characters)
- **Action**: The verb (e.g., `get`, `create`, `update`, `delete`)
- **Resource**: Optional additional specifier (e.g., `comment`, `watcher`)

### Type System

All parameters are strongly typed using JSON Schema validation. The server uses Rust's type system to ensure:

- Required fields are always present
- Optional fields are properly handled
- Enums are constrained to valid values
- Date/time formats follow ISO 8601

## Development

### Project Structure

```
mcp-jira/
├── src/
│   ├── main.rs           # Entry point
│   ├── jira.rs           # Core implementation
│   └── families/         # Data models (DTOs)
│       ├── mod.rs        # Module registry
│       ├── issue.rs      # Issue-related types
│       ├── agile.rs      # Agile-related types
│       ├── project.rs    # Project-related types
│       └── ...
├── Cargo.toml
└── README.md
```

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

## Troubleshooting

### Authentication Errors

- Verify your API token is correct (not your account password)
- Check that your email matches your Jira account
- Ensure your workspace name is just the subdomain (not the full URL)

### Permission Errors

Some operations require specific permissions:

- **Create/Edit Issues**: "Create Issues" permission
- **Delete Issues**: "Delete Issues" permission
- **Manage Sprints**: "Manage Sprints" permission (Jira Software)
- **Administer Projects**: "Administer Projects" permission

### Rate Limiting

Jira Cloud has rate limits:

- 150 requests per minute for authenticated users
- The server automatically handles 429 responses with exponential backoff

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Follow the tool naming conventions
4. Add appropriate type definitions in `src/families/`
5. Update this README with new tools
6. Submit a pull request

## Resources

- [Jira Cloud REST API Documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Jira Software REST API Documentation](https://developer.atlassian.com/cloud/jira/software/rest/)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [MCP Rust SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk)

## Support

For issues and questions:

- Open an issue on GitHub
- Check the [Jira API documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v2/)
- Review the [MCP documentation](https://modelcontextprotocol.io/)
