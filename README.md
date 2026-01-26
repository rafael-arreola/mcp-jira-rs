# Jira MCP Server

A Model Context Protocol (MCP) server implementation for Jira Cloud REST API, built with Rust. This server enables AI assistants to interact with Jira projects, issues, sprints, and more through a standardized interface.

## Features

- 🚀 **Complete Jira API Coverage**: Access to Issues, Agile boards, Projects, Users, and fields
- 🔒 **Secure Authentication**: Basic Auth with environment variables
- ⚡ **High Performance**: Built with Rust for speed and reliability
- 🎯 **Type-Safe**: Strongly typed parameters and responses using JSON Schema
- 🧠 **Smart Context**: Dynamically discovers field IDs (like "Story Points") automatically
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

The compiled binary will be available at `target/release/jira-rs`.

## Configuration

### Environment Variables

This MCP server requires three environment variables to connect to your Jira instance:

```bash
JIRA_WORKSPACE="your-workspace"     # Your Jira workspace subdomain (e.g., "mycompany" for mycompany.atlassian.net)
JIRA_USERNAME="your-email@example.com"  # Your Jira account email
JIRA_TOKEN="your-api-token"      # Your Jira API token (not your account password)
```

### Getting Your Jira API Token

1. Go to [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens)
2. Click "Create API token"
3. Give it a label (e.g., "MCP Server")
4. Copy the generated token and use it as `JIRA_TOKEN`

### Claude Desktop Configuration

Add to your Claude Desktop config file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "jira": {
      "command": "npx",
      "args": ["-y", "@rafael-arreola/jira-rs"],
      "env": {
        "JIRA_WORKSPACE": "your-workspace",
        "JIRA_USERNAME": "your-email@example.com",
        "JIRA_TOKEN": "your-api-token"
      }
    }
  }
}
```

## Available Tools

This MCP server provides **19 consolidated tools** (reduced from 50) for maximum compatibility with MCP clients:

### 🎯 Issue Management (3 tools)

| Tool Name            | Description                                                                                        |
| -------------------- | -------------------------------------------------------------------------------------------------- |
| `issue_mutate`       | **Unified CRUD**: Create, update, delete, assign, transition issues. Supports bulk create.         |
| `issue_query`        | **Query & Search**: Get single issue by ID/key or search with JQL. Includes transitions.           |
| `issue_get_metadata` | **Field Discovery**: Get available fields and requirements for creating issues (Typed CreateMeta). |

### 📝 Content & Social (3 tools)

| Tool Name                | Description                                                                   |
| ------------------------ | ----------------------------------------------------------------------------- |
| `issue_content_manage`   | **Comments & Worklogs**: Add, update, delete, get comments and time tracking. |
| `issue_interact`         | **Social Actions**: Watch/unwatch issues, add votes.                          |
| `issue_relations_manage` | **Links & Attachments**: Create/delete issue links, get/delete attachments.   |

### 🏃 Agile Operations (4 tools)

| Tool Name              | Description                                                                    |
| ---------------------- | ------------------------------------------------------------------------------ |
| `agile_query`          | **Boards & Sprints**: Query boards, sprints, issues, and backlog with filters. |
| `agile_sprint_manage`  | **Sprint Lifecycle**: Create, update, delete, start, close sprints.            |
| `agile_move_issues`    | **Issue Movement**: Move issues to sprint or backlog (up to 50 per operation). |
| `agile_sprint_analyze` | **✨ NEW**: Analyze sprint health. **Smartly detects "Story Points"** field.   |

### 🏷️ Metadata & Discovery (2 tools)

| Tool Name              | Description                                                                            |
| ---------------------- | -------------------------------------------------------------------------------------- |
| `metadata_get_catalog` | **Global Catalogs**: Get labels, priorities, resolutions, **statuses**, issue types.   |
| `field_discover`       | **Field Search**: Discover fields globally, by project, or by issue type with options. |

### 📂 Project Management (2 tools)

| Tool Name        | Description                                                                      |
| ---------------- | -------------------------------------------------------------------------------- |
| `project_query`  | **Project Resources**: Query projects, versions, components, roles, issue types. |
| `project_manage` | **Resource Creation**: Create versions and components in projects.               |

### 🔍 Search & Users (4 tools)

| Tool Name            | Description                                                              |
| -------------------- | ------------------------------------------------------------------------ |
| `search_execute_jql` | **JQL Search**: Execute JQL queries with field selection and pagination. |
| `jql_parse`          | **JQL Validation**: Parse and validate JQL queries before execution.     |
| `user_search`        | **Find Users**: Search for users by query string with pagination.        |
| `user_get_myself`    | **Current User**: Get authenticated user details with groups and roles.  |

### 🛠️ Helpers (1 tool)

| Tool Name     | Description                                                                              |
| ------------- | ---------------------------------------------------------------------------------------- |
| `text_to_adf` | **✨ NEW**: Convert plain text to Atlassian Document Format (paragraph, headings, code). |

## Usage Examples

### Creating an Issue

```json
{
  "operation": "create",
  "data": {
    "fields": {
      "project": { "key": "PROJ" },
      "summary": "Login page not working",
      "issuetype": { "name": "Bug" },
      "description": { ... }
    }
  }
}
```

### Analyzing a Sprint

```json
{
  "sprintId": 42,
  "metrics": ["velocity", "completion", "unestimated"]
}
```

### Managing Sprints

```json
{
  "operation": "start",
  "sprintId": 42,
  "data": {
    "startDate": "2024-01-25T10:00:00.000Z",
    "state": "active"
  }
}
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
