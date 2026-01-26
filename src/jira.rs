use crate::families;
use reqwest::header::CONTENT_TYPE;
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper},
    model::{ServerCapabilities, ServerInfo},
    tool_handler, tool_router,
};
use serde::Serialize;

#[derive(Debug)]
pub struct Jira {
    pub tool_router: ToolRouter<Jira>,
    client: reqwest::Client,
    workspace: String,
    username: String,
    password: String,
}

#[derive(Clone, Copy, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

#[tool_router]
impl Jira {
    pub fn new(workspace: &str, username: &str, password: &str) -> Self {
        let mut tool_router = Self::tool_router();

        for (_, route) in tool_router.map.iter_mut() {
            let map = std::sync::Arc::make_mut(&mut route.attr.input_schema);
            map.remove("$schema");
        }

        Self {
            tool_router,
            client: reqwest::Client::new(),
            workspace: workspace.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    fn base_url(&self) -> String {
        format!("https://{}.atlassian.net", self.workspace)
    }

    // =========================================================================
    // ISSUE OPERATIONS
    // =========================================================================

    #[rmcp::tool(
        name = "issue_mutate",
        description = r#"Unified tool for issue mutations (create, update, delete, assign, transition).

OPERATIONS:

1. CREATE - Creates a new issue
   {
     "operation": "create",
     "data": {
       "fields": {
         "project": {"key": "PROJ"},
         "summary": "Issue title",
         "issuetype": {"name": "Task"},
         "description": {...ADF format...}
       },
       "update": {...} (optional)
     }
   }

2. UPDATE - Updates an existing issue
   {
     "operation": "update",
     "issueIdOrKey": "PROJ-123",
     "data": {
       "fields": {"summary": "New title"},
       "update": {...} (optional),
       "notifyUsers": true (optional)
     }
   }

3. DELETE - Deletes an issue
   {
     "operation": "delete",
     "issueIdOrKey": "PROJ-123",
     "data": {"deleteSubtasks": "true"} (optional)
   }

4. ASSIGN - Assigns an issue to a user
   {
     "operation": "assign",
     "issueIdOrKey": "PROJ-123",
     "data": {"accountId": "5b10a..."}
   }

5. TRANSITION - Transitions issue to new status
   {
     "operation": "transition",
     "issueIdOrKey": "PROJ-123",
     "data": {
       "transition": {"id": "21"},
       "fields": {...} (optional),
       "update": {...} (optional)
     }
   }

BULK MODE (create only):
   {
     "operation": "create",
     "bulk": true,
     "data": [
       {"fields": {...}},
       {"fields": {...}}
     ]
   }

Rich text fields like 'description' must use ADF format. Use text_to_adf tool to convert plain text."#
    )]
    async fn issue_mutate(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueMutateParams>,
    ) -> String {
        let operation = params.operation.clone();
        let result = match operation {
            families::issue::IssueOperation::Create => {
                if params.bulk.unwrap_or(false) {
                    self.handle_bulk_create(params.data).await
                } else {
                    self.handle_create(params.data).await
                }
            }
            families::issue::IssueOperation::Update => {
                let key = match params.issue_id_or_key {
                    Some(k) => k.clone(),
                    None => return r#"{"error": "issueIdOrKey required for update"}"#.to_string(),
                };
                self.handle_update(key, params.data).await
            }
            families::issue::IssueOperation::Delete => {
                let key = match params.issue_id_or_key {
                    Some(k) => k.clone(),
                    None => return r#"{"error": "issueIdOrKey required for delete"}"#.to_string(),
                };
                self.handle_delete(key, params.data).await
            }
            families::issue::IssueOperation::Assign => {
                let key = match params.issue_id_or_key {
                    Some(k) => k.clone(),
                    None => return r#"{"error": "issueIdOrKey required for assign"}"#.to_string(),
                };
                self.handle_assign(key, params.data).await
            }
            families::issue::IssueOperation::Transition => {
                let key = match params.issue_id_or_key {
                    Some(k) => k.clone(),
                    None => return r#"{"error": "issueIdOrKey required for transition"}"#.to_string(),
                };
                self.handle_transition(key, params.data).await
            }
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }


    #[rmcp::tool(
        name = "issue_query",
        description = r#"Query issues - get single issue by ID/key or search with JQL.

SINGLE ISSUE:
   {
     "issueIdOrKey": "PROJ-123",
     "fields": ["summary", "status", "assignee"],
     "expand": "renderedFields,transitions",
     "includeTransitions": true
   }

SEARCH WITH JQL:
   {
     "jql": "project = PROJ AND status = Open",
     "fields": ["summary", "status"],
     "maxResults": 50,
     "startAt": 0
   }

Returns issue details or search results with issues array."#
    )]
    async fn issue_query(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueQueryParams>,
    ) -> String {
        if let Some(key) = params.issue_id_or_key.clone() {
            let params_clone = params;
            match self.get_issue_internal(key, params_clone).await {
                Ok(res) => res,
                Err(e) => e.to_string(),
            }
        } else if let Some(jql) = params.jql.clone() {
            let params_clone = params;
            match self.search_issues_internal(jql, params_clone).await {
                Ok(res) => res,
                Err(e) => e.to_string(),
            }
        } else {
            r#"{"error": "Either issueIdOrKey or jql is required"}"#.to_string()
        }
    }

    #[rmcp::tool(
        name = "issue_get_metadata",
        description = r#"Get metadata for creating issues in a project.

Returns available issue types and their required/optional fields.

USAGE:
   {
     "projectKey": "PROJ",
     "issueTypeName": "Story" (optional - omit for all types)
   }

Use this before creating issues to discover required fields and their formats."#
    )]
    async fn issue_get_metadata(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueMetadataParams>,
    ) -> String {
        let url = "/rest/api/3/issue/createmeta";
        let mut query = vec![("projectKeys", params.project_key)];

        if let Some(expand) = params.expand {
            query.push(("expand", expand));
        }

        if let Some(issue_type) = params.issue_type_name {
            query.push(("issuetypeNames", issue_type));
        }

        match self.send_request::<families::metadata::CreateMeta, ()>(&url, Method::Get, Some(&query), None).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // ISSUE CONTENT (COMMENTS & WORKLOGS)
    // =========================================================================

    #[rmcp::tool(
        name = "issue_content_manage",
        description = r#"Manage issue comments and worklogs.

COMMENT OPERATIONS:

Add:
   {
     "contentType": "comment",
     "operation": "add",
     "issueIdOrKey": "PROJ-123",
     "data": {
       "body": {...ADF format...},
       "visibility": {"type": "role", "value": "Developers"} (optional)
     }
   }

Update:
   {
     "contentType": "comment",
     "operation": "update",
     "issueIdOrKey": "PROJ-123",
     "contentId": "10001",
     "data": {"body": {...ADF...}}
   }

Delete:
   {
     "contentType": "comment",
     "operation": "delete",
     "issueIdOrKey": "PROJ-123",
     "contentId": "10001"
   }

Get:
   {
     "contentType": "comment",
     "operation": "get",
     "issueIdOrKey": "PROJ-123"
   }

WORKLOG OPERATIONS:

Add:
   {
     "contentType": "worklog",
     "operation": "add",
     "issueIdOrKey": "PROJ-123",
     "data": {
       "timeSpent": "3h 30m",
       "started": "2024-01-15T10:00:00.000+0000",
       "comment": {...ADF...} (optional)
     }
   }

Update/Delete/Get: Similar structure to comments.

Use text_to_adf tool to convert plain text to ADF for body/comment fields."#
    )]
    async fn issue_content_manage(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_content::IssueContentParams>,
    ) -> String {
        let result = match params.content_type.as_str() {
            "comment" => self.handle_comment_operation(params).await,
            "worklog" => self.handle_worklog_operation(params).await,
            ct => Err(format!("Unknown contentType '{}'. Valid: comment, worklog", ct).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_interact",
        description = r#"Social interactions with issues (watch, vote).

WATCH:
   {
     "action": "watch",
     "issueIdOrKey": "PROJ-123",
     "accountId": "5b10a..."
   }

UNWATCH:
   {
     "action": "unwatch",
     "issueIdOrKey": "PROJ-123",
     "accountId": "5b10a..."
   }

VOTE:
   {
     "action": "vote",
     "issueIdOrKey": "PROJ-123"
   }"#
    )]
    async fn issue_interact(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_content::IssueSocialParams>,
    ) -> String {
        let result = match params.action.as_str() {
            "watch" => {
                let account_id = match params.account_id {
                    Some(id) => id,
                    None => return r#"{"error": "accountId required for watch"}"#.to_string(),
                };
                self.add_watcher_internal(&params.issue_id_or_key, &account_id).await
            }
            "unwatch" => {
                let account_id = params.account_id.unwrap_or_default();
                self.remove_watcher_internal(&params.issue_id_or_key, &account_id).await
            }
            "vote" => self.add_vote_internal(&params.issue_id_or_key).await,
            action => Err(format!("Unknown action '{}'. Valid: watch, unwatch, vote", action).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_relations_manage",
        description = r#"Manage issue attachments and links.

ATTACHMENT OPERATIONS:

Get:
   {
     "relationType": "attachment",
     "operation": "get",
     "issueIdOrKey": "PROJ-123"
   }

Delete:
   {
     "relationType": "attachment",
     "operation": "delete",
     "issueIdOrKey": "PROJ-123",
     "relationId": "10001"
   }

LINK OPERATIONS:

Create:
   {
     "relationType": "link",
     "operation": "create",
     "issueIdOrKey": "PROJ-123",
     "data": {
       "type": {"name": "Blocks"},
       "inwardIssue": {"key": "PROJ-123"},
       "outwardIssue": {"key": "PROJ-456"}
     }
   }

Delete:
   {
     "relationType": "link",
     "operation": "delete",
     "issueIdOrKey": "PROJ-123",
     "relationId": "10001"
   }

Note: Attachment upload not supported via MCP (requires multipart/form-data)."#
    )]
    async fn issue_relations_manage(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_relations::IssueRelationsParams>,
    ) -> String {
        let result = match params.relation_type.as_str() {
            "attachment" => self.handle_attachment_operation(params).await,
            "link" => self.handle_link_operation(params).await,
            rt => Err(format!("Unknown relationType '{}'. Valid: attachment, link", rt).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // AGILE OPERATIONS
    // =========================================================================

    #[rmcp::tool(
        name = "agile_query",
        description = r#"Query agile resources (boards, sprints, issues, backlog).

BOARDS:
   {
     "resource": "board",
     "filters": {
       "type": "scrum",
       "name": "Team Board",
       "projectKeyOrId": "PROJ"
     },
     "startAt": 0,
     "maxResults": 50
   }

SPRINTS:
   {
     "resource": "sprint",
     "boardId": 1,
     "filters": {"state": "active,future"}
   }

SINGLE SPRINT:
   {
     "resource": "sprint",
     "sprintId": 42
   }

ISSUES (on board):
   {
     "resource": "issues",
     "boardId": 1,
     "filters": {
       "jql": "assignee = currentUser()",
       "fields": ["summary", "status"]
     }
   }

BACKLOG:
   {
     "resource": "backlog",
     "boardId": 1,
     "filters": {"jql": "priority = High"}
   }"#
    )]
    async fn agile_query(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::AgileQueryParams>,
    ) -> String {
        let resource = params.resource.clone();
        let result = match resource {
            families::agile::AgileQueryResource::Board => {
                if let Some(board_id) = params.board_id {
                    self.get_board_internal(board_id).await
                } else {
                    self.get_boards_internal(params).await
                }
            }
            families::agile::AgileQueryResource::Sprint => {
                if let Some(sprint_id) = params.sprint_id {
                    self.get_sprint_internal(sprint_id).await
                } else if let Some(board_id) = params.board_id {
                    self.get_sprints_internal(board_id, params).await
                } else {
                    Err("boardId or sprintId required for sprint queries".into())
                }
            }
            families::agile::AgileQueryResource::Issues => {
                let board_id = match params.board_id {
                    Some(id) => id,
                    None => return r#"{"error": "boardId required for issues query"}"#.to_string(),
                };
                self.get_board_issues_internal(board_id, params).await
            }
            families::agile::AgileQueryResource::Backlog => {
                let board_id = match params.board_id {
                    Some(id) => id,
                    None => return r#"{"error": "boardId required for backlog query"}"#.to_string(),
                };
                self.get_board_backlog_internal(board_id, params).await
            }
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "agile_sprint_manage",
        description = r#"Manage sprints (create, update, delete, start, close).

CREATE:
   {
     "operation": "create",
     "boardId": 1,
     "data": {
       "name": "Sprint 42",
       "startDate": "2024-01-15T09:00:00.000Z",
       "endDate": "2024-01-29T17:00:00.000Z",
       "goal": "Complete user authentication"
     }
   }

UPDATE:
   {
     "operation": "update",
     "sprintId": 42,
     "data": {
       "name": "Sprint 42 - Extended",
       "endDate": "2024-02-05T17:00:00.000Z"
     }
   }

START (same as update with state):
   {
     "operation": "start",
     "sprintId": 42,
     "data": {
       "state": "active",
       "startDate": "2024-01-15T09:00:00.000Z"
     }
   }

CLOSE:
   {
     "operation": "close",
     "sprintId": 42,
     "data": {"state": "closed"}
   }

DELETE:
   {
     "operation": "delete",
     "sprintId": 42
   }"#
    )]
    async fn agile_sprint_manage(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::AgileSprintManageParams>,
    ) -> String {
        let result = match params.operation.as_str() {
            "create" => {
                let board_id = match params.board_id {
                    Some(id) => id,
                    None => return r#"{"error": "boardId required for create"}"#.to_string(),
                };
                let data = match params.data {
                    Some(d) => d,
                    None => return r#"{"error": "data required for create"}"#.to_string(),
                };
                self.create_sprint_internal(board_id, data).await
            }
            "update" | "start" | "close" => {
                let sprint_id = match params.sprint_id {
                    Some(id) => id,
                    None => return r#"{"error": "sprintId required for update/start/close"}"#.to_string(),
                };
                let data = match params.data {
                    Some(d) => d,
                    None => return r#"{"error": "data required for update/start/close"}"#.to_string(),
                };
                self.update_sprint_internal(sprint_id, data).await
            }
            "delete" => {
                let sprint_id = match params.sprint_id {
                    Some(id) => id,
                    None => return r#"{"error": "sprintId required for delete"}"#.to_string(),
                };
                self.delete_sprint_internal(sprint_id).await
            }
            op => Err(format!("Unknown operation '{}'. Valid: create, update, start, close, delete", op).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "agile_move_issues",
        description = r#"Move issues to sprint or backlog.

TO SPRINT:
   {
     "destination": "sprint",
     "destinationId": 42,
     "issues": ["PROJ-123", "PROJ-456"]
   }

TO BACKLOG:
   {
     "destination": "backlog",
     "issues": ["PROJ-123", "PROJ-456"]
   }"#
    )]
    async fn agile_move_issues(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::AgileMoveIssuesParams>,
    ) -> String {
        let result = match params.destination.as_str() {
            "sprint" => {
                let sprint_id = match params.destination_id {
                    Some(id) => id,
                    None => return r#"{"error": "destinationId required for sprint"}"#.to_string(),
                };
                self.move_to_sprint_internal(sprint_id, params.issues).await
            }
            "backlog" => self.move_to_backlog_internal(params.issues).await,
            d => Err(format!("Unknown destination '{}'. Valid: sprint, backlog", d).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "agile_sprint_analyze",
        description = r#"Analyze sprint metrics and health.

USAGE:
   {
     "sprintId": 42,
     "metrics": ["velocity", "unestimated", "blocked", "completion"]
   }

Or for active sprint:
   {
     "boardId": 1,
     "metrics": ["velocity", "capacity"]
   }

METRICS:
- velocity: Story points completed
- unestimated: Issues without story points
- blocked: Blocked issues
- capacity: Total planned vs completed
- completion: Completion percentage

Returns sprint analysis with requested metrics."#
    )]
    async fn agile_sprint_analyze(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::AgileSprintAnalyzeParams>,
    ) -> String {
        match self.analyze_sprint_internal(params).await {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // METADATA OPERATIONS
    // =========================================================================

    #[rmcp::tool(
        name = "metadata_get_catalog",
        description = r#"Get metadata catalogs (labels, priorities, resolutions, statuses, issue types).

LABELS:
   {
     "catalogType": "labels",
     "startAt": 0,
     "maxResults": 50
   }

PRIORITIES:
   {
     "catalogType": "priorities"
   }

RESOLUTIONS:
   {
     "catalogType": "resolutions"
   }

ISSUE TYPES (for project):
   {
     "catalogType": "issueTypes",
     "projectKey": "PROJ"
   }

Returns list of available values for the specified catalog type."#
    )]
    async fn metadata_get_catalog(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::metadata::MetadataCatalogParams>,
    ) -> String {
        let result = match params.catalog_type.as_str() {
            "labels" => self.get_labels_internal(params).await,
            "priorities" => self.get_priorities_internal().await,
            "resolutions" => self.get_resolutions_internal().await,
            "issueTypes" => {
                if let Some(project_key) = params.project_key {
                    self.get_issue_types_for_project_internal(project_key).await
                } else {
                    self.get_all_issue_types_internal().await
                }
            }
            "statuses" => self.get_statuses_internal().await,
            ct => Err(format!("Unknown catalogType '{}'. Valid: labels, priorities, resolutions, statuses, issueTypes", ct).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "field_discover",
        description = r#"Discover fields and their configurations.

GLOBAL FIELDS:
   {
     "scope": "global",
     "fieldType": "custom",
     "searchTerm": "story"
   }

PROJECT FIELDS (with metadata):
   {
     "scope": "project",
     "scopeId": "PROJ",
     "includeOptions": true
   }

ISSUE TYPE FIELDS:
   {
     "scope": "issueType",
     "scopeId": "Story"
   }

Use this to discover field IDs (like customfield_10001 for Story Points) before updating issues."#
    )]
    async fn field_discover(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::metadata::FieldDiscoverParams>,
    ) -> String {
        let result = match params.scope.as_str() {
            "global" => self.get_fields_internal(params).await,
            "project" | "issueType" => {
                let scope_id = match params.scope_id {
                    Some(ref id) => id.clone(),
                    None => return serde_json::json!({"error": "scopeId required for project/issueType scope"}).to_string(),
                };
                let _scope = params.scope.clone();
                self.get_fields_for_scope_internal(params.scope.clone(), scope_id, params).await
            }
            s => Err(format!("Unknown scope '{}'. Valid: global, project, issueType", s).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // PROJECT OPERATIONS
    // =========================================================================

    #[rmcp::tool(
        name = "project_query",
        description = r#"Query project resources.

PROJECTS:
   {
     "resource": "project",
     "startAt": 0,
     "maxResults": 50
   }

VERSIONS:
   {
     "resource": "versions",
     "projectKey": "PROJ"
   }

COMPONENTS:
   {
     "resource": "components",
     "projectKey": "PROJ"
   }

ROLES:
   {
     "resource": "roles",
     "projectKey": "PROJ"
   }

ISSUE TYPES:
   {
     "resource": "issueTypes",
     "projectKey": "PROJ"
   }"#
    )]
    async fn project_query(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::project::ProjectQueryParams>,
    ) -> String {
        let result = match params.resource.as_str() {
            "project" => self.get_projects_internal(params).await,
            "versions" | "components" | "roles" | "issueTypes" => {
                let project_key = match params.project_key {
                    Some(ref k) => k.clone(),
                    None => return serde_json::json!({"error": "projectKey required"}).to_string(),
                };
                let resource = params.resource.clone();
                self.get_project_resource_internal(resource, project_key, params).await
            }
            r => Err(format!("Unknown resource '{}'. Valid: project, versions, components, roles, issueTypes", r).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "project_manage",
        description = r#"Manage project versions and components.

CREATE VERSION:
   {
     "resource": "version",
     "operation": "create",
     "projectKey": "PROJ",
     "data": {
       "name": "v1.0.0",
       "description": "First release",
       "releaseDate": "2024-12-31"
     }
   }

CREATE COMPONENT:
   {
     "resource": "component",
     "operation": "create",
     "projectKey": "PROJ",
     "data": {
       "name": "Backend",
       "description": "Backend services",
       "leadAccountId": "5b10a..."
     }
   }

UPDATE/DELETE: Similar structure with resourceId for existing items."#
    )]
    async fn project_manage(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::project::ProjectManageParams>,
    ) -> String {
        let result = match params.operation.as_str() {
            "create" => {
                let data = match params.data {
                    Some(d) => d,
                    None => return r#"{"error": "data required for create"}"#.to_string(),
                };
                match params.resource.as_str() {
                    "version" => self.create_version_internal(params.project_key, data).await,
                    "component" => self.create_component_internal(params.project_key, data).await,
                    r => Err(format!("Unknown resource '{}'. Valid: version, component", r).into()),
                }
            }
            op => Err(format!("Operation '{}' not yet implemented", op).into()),
        };

        match result {
            Ok(res) => res,
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // SEARCH & JQL
    // =========================================================================

    #[rmcp::tool(
        name = "search_execute_jql",
        description = r#"Execute JQL search query.

USAGE:
   {
     "jql": "project = PROJ AND status = Open",
     "fields": ["summary", "status", "assignee"],
     "maxResults": 50,
     "startAt": 0
   }

Returns paginated search results with issues array."#
    )]
    async fn search_execute_jql(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::jql::SearchParams>,
    ) -> String {
        let url = "/rest/api/3/search";
        match self.send_request::<families::issue::SearchResults, _>(&url, Method::Post, None, Some(&params)).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jql_parse",
        description = r#"Parse and validate JQL queries.

USAGE:
   {
     "queries": ["project = PROJ AND status = Open", "assignee = currentUser()"]
   }

Returns validation results for each query."#
    )]
    async fn jql_parse(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::jql::ParseJqlQueryParams>,
    ) -> String {
        let url = "/rest/api/3/jql/parse";
        let query = vec![("validation", "strict".to_string())];
        match self.send_request::<families::jql::ParsedJqlQueries, _>(&url, Method::Post, Some(&query), Some(&params)).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // USER OPERATIONS
    // =========================================================================

    #[rmcp::tool(
        name = "user_search",
        description = r#"Search for users.

USAGE:
   {
     "query": "john",
     "maxResults": 50
   }

Returns list of users matching the search query."#
    )]
    async fn user_search(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::user::SearchUsersParams>,
    ) -> String {
        let url = "/rest/api/3/user/search";
        let mut query = vec![("query", params.query)];
        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }
        if let Some(active) = params.include_active {
            query.push(("includeActive", active.to_string()));
        }
        if let Some(inactive) = params.include_inactive {
            query.push(("includeInactive", inactive.to_string()));
        }

        match self.send_request::<Vec<families::user::User>, ()>(&url, Method::Get, Some(&query), None).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "user_get_myself",
        description = r#"Get current user details.

USAGE:
   {
     "expand": "groups,applicationRoles"
   }

Returns current authenticated user information."#
    )]
    async fn user_get_myself(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::user::GetMyselfParams>,
    ) -> String {
        let url = "/rest/api/3/myself";
        let mut query = Vec::new();
        if let Some(expand) = params.expand {
            query.push(("expand", expand));
        }

        match self.send_request::<families::user::User, ()>(&url, Method::Get, Some(&query), None).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // HELPER TOOLS
    // =========================================================================

    #[rmcp::tool(
        name = "text_to_adf",
        description = r#"Convert plain text to Atlassian Document Format (ADF).

USAGE:
   {
     "text": "This is plain text",
     "style": "paragraph"
   }

STYLES:
- "paragraph" (default)
- "heading1"
- "heading2"
- "heading3"
- "codeblock"

Returns ADF JSON structure."#
    )]
    async fn text_to_adf(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::helpers::TextToAdfParams>,
    ) -> String {
        let adf = families::helpers::text_to_adf(&params.text, params.style);
        let response = families::helpers::AdfDocument { adf };
        serde_json::to_string(&response).unwrap_or_default()
    }

    // =========================================================================
    // ISSUE INTERNAL HANDLERS
    // =========================================================================

    async fn handle_create(
        &self,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let payload: families::issue::CreateIssueData = serde_json::from_value(data.0)?;
        let url = "/rest/api/3/issue";
        let response = self.send_request::<families::issue::CreatedIssue, _>(url, Method::Post, None, Some(&payload)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn handle_bulk_create(
        &self,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let items: Vec<families::issue::CreateIssueData> = serde_json::from_value(data.0)?;

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct BulkPayload {
            issue_updates: Vec<families::issue::CreateIssueData>,
        }

        let payload = BulkPayload { issue_updates: items };
        let url = "/rest/api/3/issue/bulk";
        let response = self.send_request::<families::issue::BulkCreatedIssues, _>(url, Method::Post, None, Some(&payload)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn handle_update(
        &self,
        key: String,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let payload: families::issue::UpdateIssueData = serde_json::from_value(data.0)?;
        let url = format!("/rest/api/3/issue/{}", key);
        let mut query = Vec::new();

        if let Some(notify) = payload.notify_users {
            query.push(("notifyUsers", notify.to_string()));
        }
        if let Some(override_editable) = payload.override_editable_flag {
            query.push(("overrideEditableFlag", override_editable.to_string()));
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            fields: Option<&'a families::JsonValue>,
            #[serde(skip_serializing_if = "Option::is_none")]
            update: Option<&'a families::JsonValue>,
        }

        let body = Body {
            fields: payload.fields.as_ref(),
            update: payload.update.as_ref(),
        };

        self.send_request::<serde_json::Value, _>(&url, Method::Put, Some(&query), Some(&body)).await?;
        Ok(format!(r#"{{"success": true, "message": "Issue {} updated successfully"}}"#, key))
    }

    async fn handle_delete(
        &self,
        key: String,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let payload: families::issue::DeleteIssueData = serde_json::from_value(data.0)?;
        let url = format!("/rest/api/3/issue/{}", key);
        let mut query = Vec::new();

        if let Some(delete_subtasks) = payload.delete_subtasks {
            query.push(("deleteSubtasks", delete_subtasks));
        }

        self.send_request::<serde_json::Value, ()>(&url, Method::Delete, Some(&query), None).await?;
        Ok(format!(r#"{{"success": true, "message": "Issue {} deleted successfully"}}"#, key))
    }

    async fn handle_assign(
        &self,
        key: String,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let payload: families::issue::AssignIssueData = serde_json::from_value(data.0)?;
        let url = format!("/rest/api/3/issue/{}/assignee", key);
        self.send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&payload)).await?;
        Ok(format!(r#"{{"success": true, "message": "Issue {} assigned successfully"}}"#, key))
    }

    async fn handle_transition(
        &self,
        key: String,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let payload: families::issue::TransitionIssueData = serde_json::from_value(data.0)?;
        let url = format!("/rest/api/3/issue/{}/transitions", key);
        self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&payload)).await?;
        Ok(format!(r#"{{"success": true, "message": "Issue {} transitioned successfully"}}"#, key))
    }

    async fn get_issue_internal(
        &self,
        key: String,
        params: families::issue::IssueQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/api/3/issue/{}", key);
        let mut query = Vec::new();

        if let Some(fields) = params.fields {
            query.push(("fields", fields.join(",")));
        }
        if let Some(expand) = params.expand {
            query.push(("expand", expand));
        }
        if let Some(properties) = params.properties {
            query.push(("properties", properties.join(",")));
        }

        let mut issue = self.send_request::<families::issue::Issue, ()>(&url, Method::Get, Some(&query), None).await?;

        if params.include_transitions.unwrap_or(false) {
            let trans_url = format!("/rest/api/3/issue/{}/transitions", key);
            if let Ok(trans_resp) = self.send_request::<families::issue::TransitionsResponse, ()>(&trans_url, Method::Get, None, None).await {
                issue.fields.insert(
                    "transitions".to_string(),
                    serde_json::to_value(trans_resp.transitions)?,
                );
            }
        }

        Ok(serde_json::to_string(&issue)?)
    }

    async fn search_issues_internal(
        &self,
        jql: String,
        params: families::issue::IssueQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/search";

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SearchBody {
            jql: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            fields: Option<Vec<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            expand: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_results: Option<i32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start_at: Option<i64>,
        }

        let body = SearchBody {
            jql,
            fields: params.fields,
            expand: params.expand,
            max_results: params.max_results,
            start_at: params.start_at,
        };

        let response = self.send_request::<families::issue::SearchResults, _>(url, Method::Post, None, Some(&body)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    // =========================================================================
    // ISSUE CONTENT INTERNAL HANDLERS
    // =========================================================================

    async fn handle_comment_operation(
        &self,
        params: families::issue_content::IssueContentParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match params.operation.as_str() {
            "add" => {
                let data = params.data.ok_or("data required for add")?;
                let payload: families::issue_content::CommentData = serde_json::from_value(data.0)?;
                let url = format!("/rest/api/3/issue/{}/comment", params.issue_id_or_key);
                let response = self.send_request::<families::issue_content::Comment, _>(&url, Method::Post, None, Some(&payload)).await?;
                Ok(serde_json::to_string(&response)?)
            }
            "update" => {
                let content_id = params.content_id.ok_or("contentId required for update")?;
                let data = params.data.ok_or("data required for update")?;
                let payload: families::issue_content::CommentData = serde_json::from_value(data.0)?;
                let url = format!("/rest/api/3/issue/{}/comment/{}", params.issue_id_or_key, content_id);
                let response = self.send_request::<families::issue_content::Comment, _>(&url, Method::Put, None, Some(&payload)).await?;
                Ok(serde_json::to_string(&response)?)
            }
            "delete" => {
                let content_id = params.content_id.ok_or("contentId required for delete")?;
                let url = format!("/rest/api/3/issue/{}/comment/{}", params.issue_id_or_key, content_id);
                self.send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None).await?;
                Ok(r#"{"success": true, "message": "Comment deleted successfully"}"#.to_string())
            }
            "get" => {
                let url = format!("/rest/api/3/issue/{}/comment", params.issue_id_or_key);
                let response = self.send_request::<families::issue_content::CommentsPage, ()>(&url, Method::Get, None, None).await?;
                Ok(serde_json::to_string(&response)?)
            }
            op => Err(format!("Unknown operation '{}'. Valid: add, update, delete, get", op).into()),
        }
    }

    async fn handle_worklog_operation(
        &self,
        params: families::issue_content::IssueContentParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match params.operation.as_str() {
            "add" => {
                let data = params.data.ok_or("data required for add")?;
                let payload: families::issue_content::WorklogData = serde_json::from_value(data.0)?;
                let url = format!("/rest/api/3/issue/{}/worklog", params.issue_id_or_key);
                let response = self.send_request::<families::issue_content::Worklog, _>(&url, Method::Post, None, Some(&payload)).await?;
                Ok(serde_json::to_string(&response)?)
            }
            "update" => {
                let content_id = params.content_id.ok_or("contentId required for update")?;
                let data = params.data.ok_or("data required for update")?;
                let payload: families::issue_content::WorklogData = serde_json::from_value(data.0)?;
                let url = format!("/rest/api/3/issue/{}/worklog/{}", params.issue_id_or_key, content_id);
                let response = self.send_request::<families::issue_content::Worklog, _>(&url, Method::Put, None, Some(&payload)).await?;
                Ok(serde_json::to_string(&response)?)
            }
            "delete" => {
                let content_id = params.content_id.ok_or("contentId required for delete")?;
                let url = format!("/rest/api/3/issue/{}/worklog/{}", params.issue_id_or_key, content_id);
                self.send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None).await?;
                Ok(r#"{"success": true, "message": "Worklog deleted successfully"}"#.to_string())
            }
            "get" => {
                let url = format!("/rest/api/3/issue/{}/worklog", params.issue_id_or_key);
                let response = self.send_request::<families::issue_content::WorklogsPage, ()>(&url, Method::Get, None, None).await?;
                Ok(serde_json::to_string(&response)?)
            }
            op => Err(format!("Unknown operation '{}'. Valid: add, update, delete, get", op).into()),
        }
    }

    async fn add_watcher_internal(
        &self,
        issue_key: &str,
        account_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/api/3/issue/{}/watchers", issue_key);
        self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&account_id)).await?;
        Ok(r#"{"success": true, "message": "Watcher added successfully"}"#.to_string())
    }

    async fn remove_watcher_internal(
        &self,
        issue_key: &str,
        account_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/api/3/issue/{}/watchers", issue_key);
        let query = vec![("accountId", account_id.to_string())];
        self.send_request::<serde_json::Value, ()>(&url, Method::Delete, Some(&query), None).await?;
        Ok(r#"{"success": true, "message": "Watcher removed successfully"}"#.to_string())
    }

    async fn add_vote_internal(
        &self,
        issue_key: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/api/3/issue/{}/votes", issue_key);
        self.send_request::<serde_json::Value, ()>(&url, Method::Post, None, None).await?;
        Ok(r#"{"success": true, "message": "Vote added successfully"}"#.to_string())
    }

    // =========================================================================
    // ISSUE RELATIONS INTERNAL HANDLERS
    // =========================================================================

    async fn handle_attachment_operation(
        &self,
        params: families::issue_relations::IssueRelationsParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match params.operation.as_str() {
            "get" => {
                let url = format!("/rest/api/3/issue/{}/attachments", params.issue_id_or_key);
                let response = self.send_request::<Vec<families::issue_relations::Attachment>, ()>(&url, Method::Get, None, None).await?;
                Ok(serde_json::to_string(&response)?)
            }
            "delete" => {
                let relation_id = params.relation_id.ok_or("relationId required for delete")?;
                let url = format!("/rest/api/3/attachment/{}", relation_id);
                self.send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None).await?;
                Ok(r#"{"success": true, "message": "Attachment deleted successfully"}"#.to_string())
            }
            op => Err(format!("Unknown operation '{}'. Valid: get, delete. Note: upload not supported via MCP", op).into()),
        }
    }

    async fn handle_link_operation(
        &self,
        params: families::issue_relations::IssueRelationsParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match params.operation.as_str() {
            "create" => {
                let data = params.data.ok_or("data required for create")?;
                let payload: families::issue_relations::CreateIssueLinkData = serde_json::from_value(data.0)?;
                let url = "/rest/api/3/issueLink";
                self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&payload)).await?;
                Ok(r#"{"success": true, "message": "Issue link created successfully"}"#.to_string())
            }
            "delete" => {
                let relation_id = params.relation_id.ok_or("relationId required for delete")?;
                let url = format!("/rest/api/3/issueLink/{}", relation_id);
                self.send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None).await?;
                Ok(r#"{"success": true, "message": "Issue link deleted successfully"}"#.to_string())
            }
            op => Err(format!("Unknown operation '{}'. Valid: create, delete", op).into()),
        }
    }

    // =========================================================================
    // AGILE INTERNAL HANDLERS
    // =========================================================================

    async fn get_board_internal(
        &self,
        board_id: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/board/{}", board_id);
        let response = self.send_request::<families::agile::Board, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_boards_internal(
        &self,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/agile/1.0/board";
        let mut query = Vec::new();

        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }

        if let Some(filters) = params.filters {
            let board_filters: families::agile::BoardFilters = serde_json::from_value(filters.0)?;
            if let Some(type_) = board_filters.r#type {
                query.push(("type", type_));
            }
            if let Some(name) = board_filters.name {
                query.push(("name", name));
            }
            if let Some(project) = board_filters.project_key_or_id {
                query.push(("projectKeyOrId", project));
            }
        }

        let response = self.send_request::<families::agile::PageBeanBoard, ()>(&url, Method::Get, Some(&query), None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_sprint_internal(
        &self,
        sprint_id: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/sprint/{}", sprint_id);
        let response = self.send_request::<families::agile::Sprint, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_sprints_internal(
        &self,
        board_id: i64,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/board/{}/sprint", board_id);
        let mut query = Vec::new();

        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }

        if let Some(filters) = params.filters {
            let sprint_filters: families::agile::SprintFilters = serde_json::from_value(filters.0)?;
            if let Some(state) = sprint_filters.state {
                query.push(("state", state));
            }
        }

        let response = self.send_request::<families::agile::PageBeanSprint, ()>(&url, Method::Get, Some(&query), None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn create_sprint_internal(
        &self,
        board_id: i64,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut sprint_data: families::agile::SprintData = serde_json::from_value(data.0)?;
        sprint_data.origin_board_id = Some(board_id);

        let url = "/rest/agile/1.0/sprint";
        let response = self.send_request::<families::agile::Sprint, _>(&url, Method::Post, None, Some(&sprint_data)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn update_sprint_internal(
        &self,
        sprint_id: i64,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let sprint_data: families::agile::SprintData = serde_json::from_value(data.0)?;
        let url = format!("/rest/agile/1.0/sprint/{}", sprint_id);
        let response = self.send_request::<families::agile::Sprint, _>(&url, Method::Put, None, Some(&sprint_data)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn delete_sprint_internal(
        &self,
        sprint_id: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/sprint/{}", sprint_id);
        self.send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None).await?;
        Ok(r#"{"success": true, "message": "Sprint deleted successfully"}"#.to_string())
    }

    async fn get_board_issues_internal(
        &self,
        board_id: i64,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/board/{}/issue", board_id);
        let mut query = Vec::new();

        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }

        if let Some(filters) = params.filters {
            let issue_filters: families::agile::IssuesFilters = serde_json::from_value(filters.0)?;
            if let Some(jql) = issue_filters.jql {
                query.push(("jql", jql));
            }
            if let Some(fields) = issue_filters.fields {
                query.push(("fields", fields.join(",")));
            }
        }

        let response = self.send_request::<families::agile::PageBeanIssue, ()>(&url, Method::Get, Some(&query), None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_board_backlog_internal(
        &self,
        board_id: i64,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/board/{}/backlog", board_id);
        let mut query = Vec::new();

        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }

        if let Some(filters) = params.filters {
            let issue_filters: families::agile::IssuesFilters = serde_json::from_value(filters.0)?;
            if let Some(jql) = issue_filters.jql {
                query.push(("jql", jql));
            }
        }

        let response = self.send_request::<families::agile::PageBeanIssue, ()>(&url, Method::Get, Some(&query), None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn move_to_sprint_internal(
        &self,
        sprint_id: i64,
        issues: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/agile/1.0/sprint/{}/issue", sprint_id);

        #[derive(Serialize)]
        struct Body {
            issues: Vec<String>,
        }

        let body = Body { issues };
        self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body)).await?;
        Ok(r#"{"success": true, "message": "Issues moved to sprint successfully"}"#.to_string())
    }

    async fn move_to_backlog_internal(
        &self,
        issues: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/agile/1.0/backlog/issue";

        #[derive(Serialize)]
        struct Body {
            issues: Vec<String>,
        }

        let body = Body { issues };
        self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body)).await?;
        Ok(r#"{"success": true, "message": "Issues moved to backlog successfully"}"#.to_string())
    }

    async fn analyze_sprint_internal(
        &self,
        params: families::agile::AgileSprintAnalyzeParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let sprint_id = if let Some(id) = params.sprint_id {
            id
        } else if let Some(board_id) = params.board_id {
            let sprints_url = format!("/rest/agile/1.0/board/{}/sprint", board_id);
            let query = vec![("state", "active".to_string())];
            let sprints = self.send_request::<families::agile::PageBeanSprint, ()>(&sprints_url, Method::Get, Some(&query), None).await?;
            sprints.values.first().ok_or("No active sprint found")?.id
        } else {
            return Err("Either sprintId or boardId is required".into());
        };

        let sprint = self.get_sprint_internal(sprint_id).await?;
        let sprint_obj: families::agile::Sprint = serde_json::from_str(&sprint)?;

        let metrics = params.metrics.unwrap_or_else(|| vec!["velocity".to_string(), "unestimated".to_string()]);
        
        // Dynamic Story Points Field Discovery
        let story_points_field_id = if metrics.contains(&"velocity".to_string()) 
            || metrics.contains(&"completion".to_string()) 
            || metrics.contains(&"unestimated".to_string()) {
            self.find_story_points_field_id().await?
        } else {
            "customfield_10016".to_string()
        };

        let issues_url = format!("/rest/agile/1.0/sprint/{}/issue", sprint_id);
        
        // Ensure we request the story points field explicitly
        let fields_param = format!("summary,status,{}", story_points_field_id);
        let query = vec![("fields", fields_param)];
        
        let issues_response = self.send_request::<families::agile::PageBeanIssue, ()>(&issues_url, Method::Get, Some(&query), None).await?;

        let mut analysis = families::agile::SprintAnalysis {
            sprint_id,
            sprint_name: sprint_obj.name,
            sprint_state: sprint_obj.state,
            velocity: None,
            unestimated_issues: None,
            blocked_issues: None,
            total_points: None,
            completed_points: None,
            completion_percentage: None,
        };

        if metrics.contains(&"velocity".to_string()) || metrics.contains(&"completion".to_string()) {
            let mut total_points = 0.0;
            let mut completed_points = 0.0;

            for issue in &issues_response.issues {
                if let Some(story_points) = issue.fields.get(&story_points_field_id) {
                    if let Some(points) = story_points.as_f64() {
                        total_points += points;
                        if let Some(status) = issue.fields.get("status") {
                            if let Some(status_obj) = status.as_object() {
                                if let Some(name) = status_obj.get("name").and_then(|n| n.as_str()) {
                                    if name == "Done" || name == "Closed" {
                                        completed_points += points;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if metrics.contains(&"velocity".to_string()) {
                analysis.velocity = Some(completed_points);
            }
            if metrics.contains(&"completion".to_string()) {
                analysis.total_points = Some(total_points);
                analysis.completed_points = Some(completed_points);
                if total_points > 0.0 {
                    analysis.completion_percentage = Some((completed_points / total_points) * 100.0);
                }
            }
        }

        if metrics.contains(&"unestimated".to_string()) {
            let unestimated: Vec<String> = issues_response.issues.iter()
                .filter(|issue| issue.fields.get(&story_points_field_id).and_then(|v| v.as_f64()).is_none())
                .map(|issue| issue.key.clone())
                .collect();
            if !unestimated.is_empty() {
                analysis.unestimated_issues = Some(unestimated);
            }
        }

        if metrics.contains(&"blocked".to_string()) {
            let blocked: Vec<String> = issues_response.issues.iter()
                .filter(|issue| {
                    issue.fields.get("status")
                        .and_then(|s| s.as_object())
                        .and_then(|o| o.get("name"))
                        .and_then(|n| n.as_str())
                        .map(|name| name.to_lowercase().contains("blocked"))
                        .unwrap_or(false)
                })
                .map(|issue| issue.key.clone())
                .collect();
            if !blocked.is_empty() {
                analysis.blocked_issues = Some(blocked);
            }
        }

        Ok(serde_json::to_string(&analysis)?)
    }

    // =========================================================================
    // METADATA INTERNAL HANDLERS
    // =========================================================================

    async fn get_labels_internal(
        &self,
        params: families::metadata::MetadataCatalogParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/label";
        let mut query = Vec::new();
        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }

        let response = self.send_request::<families::metadata::LabelsPage, ()>(&url, Method::Get, Some(&query), None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_priorities_internal(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/priority";
        let response = self.send_request::<Vec<families::metadata::Priority>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_resolutions_internal(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/resolution";
        let response = self.send_request::<Vec<families::metadata::Resolution>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_issue_types_for_project_internal(
        &self,
        project_key: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("/rest/api/3/project/{}", project_key);
        let response = self.send_request::<serde_json::Value, ()>(&url, Method::Get, None, None).await?;

        if let Some(issue_types) = response.get("issueTypes") {
            Ok(serde_json::to_string(issue_types)?)
        } else {
            Ok("[]".to_string())
        }
    }

    async fn get_all_issue_types_internal(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/issuetype";
        let response = self.send_request::<Vec<families::metadata::IssueType>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_statuses_internal(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/status";
        let response = self.send_request::<Vec<families::metadata::Status>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn fetch_fields_internal(&self) -> Result<Vec<families::metadata::Field>, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/field";
        self.send_request::<Vec<families::metadata::Field>, ()>(&url, Method::Get, None, None).await
    }

    async fn find_story_points_field_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let fields = self.fetch_fields_internal().await?;
        let candidates = ["Story Points", "Story point estimate"];

        for field in fields {
            if candidates.iter().any(|&c| field.name.eq_ignore_ascii_case(c)) {
                return Ok(field.id);
            }
        }

        Ok("customfield_10016".to_string())
    }

    async fn get_fields_internal(
        &self,
        params: families::metadata::FieldDiscoverParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut all_fields = self.fetch_fields_internal().await?;

        if let Some(field_type) = params.field_type {
            all_fields.retain(|f| {
                let is_custom = f.custom.unwrap_or(false);
                match field_type.as_str() {
                    "custom" => is_custom,
                    "system" => !is_custom,
                    _ => true,
                }
            });
        }

        if let Some(search_term) = params.search_term {
            let search_lower = search_term.to_lowercase();
            all_fields.retain(|f| {
                f.name.to_lowercase().contains(&search_lower)
                    || f.id.to_lowercase().contains(&search_lower)
            });
        }

        Ok(serde_json::to_string(&all_fields)?)
    }

    async fn get_fields_for_scope_internal(
        &self,
        scope: String,
        scope_id: String,
        _params: families::metadata::FieldDiscoverParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = match scope.as_str() {
            "project" => format!("/rest/api/3/issue/createmeta?projectKeys={}&expand=projects.issuetypes.fields", scope_id),
            _ => return self.get_fields_internal(_params).await,
        };

        let response = self.send_request::<serde_json::Value, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    // =========================================================================
    // PROJECT INTERNAL HANDLERS
    // =========================================================================

    async fn get_projects_internal(
        &self,
        params: families::project::ProjectQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "/rest/api/3/project/search";
        let mut query = Vec::new();

        if let Some(start_at) = params.start_at {
            query.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query.push(("maxResults", max_results.to_string()));
        }

        let response = self.send_request::<families::project::ProjectsPage, ()>(&url, Method::Get, Some(&query), None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_project_resource_internal(
        &self,
        resource: String,
        project_key: String,
        _params: families::project::ProjectQueryParams,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = match resource.as_str() {
            "versions" => format!("/rest/api/3/project/{}/versions", project_key),
            "components" => format!("/rest/api/3/project/{}/components", project_key),
            "roles" => format!("/rest/api/3/project/{}/role", project_key),
            "issueTypes" => format!("/rest/api/3/project/{}", project_key),
            _ => return Err(format!("Unknown resource '{}'", resource).into()),
        };

        let response = self.send_request::<serde_json::Value, ()>(&url, Method::Get, None, None).await?;

        if resource == "issueTypes" {
            if let Some(types) = response.get("issueTypes") {
                return Ok(serde_json::to_string(types)?);
            }
        }

        Ok(serde_json::to_string(&response)?)
    }

    async fn create_version_internal(
        &self,
        project_key: String,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut version_data: families::project::VersionData = serde_json::from_value(data.0)?;
        version_data.project = Some(project_key);

        let url = "/rest/api/3/version";
        let response = self.send_request::<families::project::Version, _>(&url, Method::Post, None, Some(&version_data)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn create_component_internal(
        &self,
        project_key: String,
        data: families::JsonValue,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut component_data: families::project::ComponentData = serde_json::from_value(data.0)?;
        component_data.project = Some(project_key);

        let url = "/rest/api/3/component";
        let response = self.send_request::<families::project::Component, _>(&url, Method::Post, None, Some(&component_data)).await?;
        Ok(serde_json::to_string(&response)?)
    }
    async fn send_request<T, B>(
        &self,
        url: &str,
        method: Method,
        query_params: Option<&Vec<(&str, String)>>,
        body: Option<&B>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let full_url = format!("{}{}", self.base_url(), url);
        let mut req_builder = match method {
            Method::Get => self.client.get(&full_url),
            Method::Post => self.client.post(&full_url),
            Method::Put => self.client.put(&full_url),
            Method::Delete => self.client.delete(&full_url),
        };

        req_builder = req_builder
            .basic_auth(&self.username, Some(&self.password))
            .header(CONTENT_TYPE, "application/json");

        if let Some(params) = query_params {
            req_builder = req_builder.query(params);
        }

        if let Some(b) = body {
            req_builder = req_builder.json(b);
        }

        let resp = req_builder.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            return Err(format!("HTTP {}: {}", status, text).into());
        }

        let res_text = resp.text().await?;
        if res_text.is_empty() || res_text == "null" {
            return serde_json::from_str("null").map_err(|e| e.into());
        }

        serde_json::from_str::<T>(&res_text).map_err(|e| e.into())
    }
}

#[tool_handler]
impl ServerHandler for Jira {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
