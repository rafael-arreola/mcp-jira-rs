# Jira MCP Server

A Model Context Protocol (MCP) server implementation for Jira Cloud REST API, built with Rust. This server enables AI assistants to interact with Jira projects, issues, sprints, and more through a standardized interface.

## Features

- 🚀 **Complete Jira API Coverage**: Access to Issues, Agile boards, Projects, Users, and fields
- 🔒 **Secure Authentication**: Basic Auth with environment variables
- ⚡ **High Performance**: Built with Rust for speed and reliability
- 🎯 **Type-Safe**: Strongly typed parameters and responses using JSON Schema
- 📦 **Easy Integration**: Works with Claude Desktop and other MCP clients

## Installation

You can use this package directly with `npx` without installation, or install it globally.

### Prerequisites

- Jira Cloud account with API access
- Node.js installed (to use `npx`)

### Using with Claude Desktop (Recommended)

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
        "JIRA_PASSWORD": "your-api-token"
      }
    }
  }
}
```

### Global Installation

To install the CLI tool globally on your system:

```bash
npm install -g @rafael-arreola/jira-rs
```

Then you can run it directly:

```bash
jira-rs --help
```

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

## Available Tools

This MCP server provides 45+ tools organized into the following categories:

- **Issue Management**: Create, get, edit, delete, assign, and transition issues.
- **Issue Comments**: Add, edit, and delete comments.
- **Issue Social**: Manage watchers and votes.
- **Issue Worklogs**: Track time on issues.
- **Issue Links**: Link issues together.
- **Issue Metadata**: Access fields, labels, priorities, and resolutions.
- **Search & JQL**: Advanced searching using JQL.
- **Projects**: Manage projects, versions, components, and roles.
- **Users**: Search for users and get profile details.
- **Agile**: Manage boards, sprints, and backlogs.

For a detailed list of all tools and their usage, please refer to the [main repository documentation](https://github.com/rafael-arreola/mcp-jira-rs).

## Architecture

This NPM package is a lightweight wrapper that automatically downloads and executes the high-performance Rust binary tailored for your operating system and architecture.

## Support

For issues and questions:

- Open an issue on [GitHub](https://github.com/rafael-arreola/mcp-jira-rs)
- Check the [Jira API documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v2/)
- Review the [MCP documentation](https://modelcontextprotocol.io/)
