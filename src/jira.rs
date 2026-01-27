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

        // Remove $schema to ensure compatibility with Gemini
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
        name = "issue_create",
        description = "Creates an issue or a sub-task from a JSON representation. The content of the issue is defined using the 'fields' parameter. This tool uses strict typing to guide you on standard fields (summary, description, priority, etc.), while custom fields can be added dynamically. Use 'issue_get_required_fields' first to understand what is needed."
    )]
    async fn create_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueCreateParams>,
    ) -> String {
        let url = "/rest/api/3/issue";

        // Convert strict struct to generic JSON to inject project/issuetype
        // This follows the 'Local Body' pattern implicitly by manipulating the JSON before sending
        let mut fields_json = serde_json::to_value(params.fields).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        
        if let Some(obj) = fields_json.as_object_mut() {
            obj.insert("project".to_string(), serde_json::json!({ "key": params.project_key }));
            obj.insert("issuetype".to_string(), serde_json::json!({ "name": params.issue_type }));
        }

        let payload = families::issue::CreateIssueData {
            fields: families::JsonValue(fields_json),
            update: params.update,
        };

        match self.send_request::<families::issue::CreatedIssue, _>(url, Method::Post, None, Some(&payload)).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_update",
        description = "Edits an issue. Issue properties may be updated as part of the edit. Note that issue transition is not supported here; use 'issue_transition' for that."
    )]
    async fn update_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueUpdateParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_id_or_key);
        
        let mut query = Vec::new();
        if let Some(notify) = params.notify_users {
            query.push(("notifyUsers", notify.to_string()));
        }

        let fields_val = params.fields.map(|f| families::JsonValue(serde_json::to_value(f).unwrap_or(serde_json::Value::Null)));

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            fields: Option<families::JsonValue>,
            #[serde(skip_serializing_if = "Option::is_none")]
            update: Option<&'a families::JsonValue>,
        }

        let body = Body {
            fields: fields_val,
            update: params.update.as_ref(),
        };

        match self.send_request::<serde_json::Value, _>(&url, Method::Put, Some(&query), Some(&body)).await {
            Ok(_) => format!(r#"{{"success": true, "message": "Issue {} updated successfully"}}"#, params.issue_id_or_key),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_assign",
        description = "Assigns an issue to a user. Use this operation when the calling user does not have the 'Edit Issues' permission but has the 'Assign issue' permission."
    )]
    async fn assign_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueAssignParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/assignee", params.issue_id_or_key);

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            account_id: &'a str,
        }
        let body = Body {
            account_id: &params.account_id,
        };

        match self.send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body)).await {
            Ok(_) => format!(r#"{{"success": true, "message": "Issue {} assigned successfully"}}"#, params.issue_id_or_key),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_transition",
        description = "Performs an issue transition and, if the transition has a screen, updates the fields from the transition screen."
    )]
    async fn transition_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueTransitionParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/transitions", params.issue_id_or_key);
        
        let fields_val = params.fields.map(|f| families::JsonValue(serde_json::to_value(f).unwrap_or(serde_json::Value::Null)));

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            transition: families::issue::TransitionRef,
            #[serde(skip_serializing_if = "Option::is_none")]
            fields: Option<families::JsonValue>,
        }

        let body = Body {
            transition: families::issue::TransitionRef { id: params.transition_id },
            fields: fields_val,
        };

        match self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body)).await {
            Ok(_) => format!(r#"{{"success": true, "message": "Issue {} transitioned successfully"}}"#, params.issue_id_or_key),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_delete",
        description = "Deletes an issue. An issue cannot be deleted if it has one or more subtasks. To delete an issue with subtasks, set deleteSubtasks to true."
    )]
    async fn delete_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueDeleteParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_id_or_key);
        let mut query = Vec::new();

        if let Some(delete_subtasks) = params.delete_subtasks {
            query.push(("deleteSubtasks", delete_subtasks.to_string()));
        }

        match self.send_request::<serde_json::Value, ()>(&url, Method::Delete, Some(&query), None).await {
            Ok(_) => format!(r#"{{"success": true, "message": "Issue {} deleted successfully"}}"#, params.issue_id_or_key),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_query",
        description = r#"Query issues - get single issue by ID/key or search with JQL. Returns issue details or search results."#
    )]
    async fn query_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueQueryParams>,
    ) -> String {
        if let Some(key) = params.issue_id_or_key.clone() {
            // Get Single Issue Logic
            let url = format!("/rest/api/3/issue/{}", key);
            let mut query = Vec::new();

            if let Some(fields) = &params.fields {
                query.push(("fields", fields.join(",")));
            }
            if let Some(expand) = &params.expand {
                query.push(("expand", expand.clone()));
            }

            match self.send_request::<families::issue::Issue, ()>(&url, Method::Get, Some(&query), None).await {
                Ok(mut issue) => {
                     // Internal check for transitions if requested
                     if params.include_transitions.unwrap_or(false) {
                        let trans_url = format!("/rest/api/3/issue/{}/transitions", key);
                        if let Ok(trans_resp) = self.send_request::<families::issue::TransitionsResponse, ()>(&trans_url, Method::Get, None, None).await {
                            if let Ok(val) = serde_json::to_value(trans_resp.transitions) {
                                issue.fields.insert("transitions".to_string(), val);
                            }
                        }
                    }
                    serde_json::to_string(&issue).unwrap_or_default()
                },
                Err(e) => e.to_string(),
            }
        } else if let Some(jql) = params.jql.clone() {
            // JQL Search Logic
            let url = "/rest/api/3/search/jql";
            
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body {
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

            let body = Body {
                jql,
                fields: params.fields,
                expand: params.expand,
                max_results: params.max_results,
                start_at: params.start_at,
            };

            match self.send_request::<families::issue::SearchResults, _>(url, Method::Post, None, Some(&body)).await {
                Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
                Err(e) => e.to_string(),
            }
        } else {
            r#"{"error": "Either issueIdOrKey or jql is required"}"#.to_string()
        }
    }

    #[rmcp::tool(
        name = "issue_get_required_fields",
        description = "Returns ONLY the required fields for creating an issue of a specific type in a project. Highly recommended to use this before issue_create to avoid validation errors."
    )]
    async fn get_required_fields(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::IssueRequiredFieldsParams>,
    ) -> String {
        let url = "/rest/api/3/issue/createmeta";
        let query = vec![
            ("projectKeys", params.project_key.clone()),
            ("issuetypeNames", params.issue_type_name.clone()),
            ("expand", "projects.issuetypes.fields".to_string()),
        ];

        match self.send_request::<serde_json::Value, ()>(&url, Method::Get, Some(&query), None).await {
            Ok(res) => {
                let mut required_fields = Vec::new();
                if let Some(projects) = res.get("projects").and_then(|p| p.as_array()) {
                    for project in projects {
                        if let Some(issue_types) = project.get("issuetypes").and_then(|it| it.as_array()) {
                            for it in issue_types {
                                if let Some(fields) = it.get("fields").and_then(|f| f.as_object()) {
                                    for (field_id, field_info) in fields {
                                        let is_required = field_info.get("required").and_then(|r| r.as_bool()).unwrap_or(false);
                                        if is_required {
                                            required_fields.push(families::issue::RequiredFieldInfo {
                                                id: field_id.clone(),
                                                name: field_info.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown").to_string(),
                                                required: true,
                                                field_type: field_info.get("schema").and_then(|s| s.get("type")).and_then(|t| t.as_str()).unwrap_or("any").to_string(),
                                                allowed_values: field_info.get("allowedValues").and_then(|v| serde_json::from_value(v.clone()).ok()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                serde_json::to_string(&required_fields).unwrap_or_default()
            },
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "field_search_by_name",
        description = "Searches for a field by its visible name and returns its ID (e.g., searches 'Story Points' and returns 'customfield_10016')."
    )]
    async fn search_field_by_name(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::FieldSearchByNameParams>,
    ) -> String {
        // Internal helper to reuse logic
        match self.fetch_fields_internal().await {
            Ok(fields) => {
                let query = params.query.to_lowercase();
                let results: Vec<_> = fields.into_iter()
                    .filter(|f| f.name.to_lowercase().contains(&query))
                    .collect();
                serde_json::to_string(&results).unwrap_or_default()
            }
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_get_metadata",
        description = r#"Get metadata for creating issues in a project. Returns available issue types and their required/optional fields."#
    )]
    async fn get_issue_metadata(
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
        description = "Manage issue comments and worklogs."
    )]
    async fn manage_content(
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
        description = "Social interactions with issues (watch, vote)."
    )]
    async fn interact_issue(
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
        description = "Manage issue attachments and links."
    )]
    async fn manage_relations(
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
        description = "Query agile resources (boards, sprints, issues, backlog)."
    )]
    async fn query_agile(
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
        description = "Manage sprints (create, update, delete, start, close)."
    )]
    async fn manage_sprint(
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
        description = "Move issues to sprint or backlog."
    )]
    async fn move_issues(
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
        description = "Analyze sprint metrics and health."
    )]
    async fn analyze_sprint(
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
        description = "Get metadata catalogs (labels, priorities, resolutions, statuses, issue types)."
    )]
    async fn get_catalog(
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
        description = "Discover fields and their configurations."
    )]
    async fn discover_fields(
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
        description = "Query project resources."
    )]
    async fn query_project(
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
        description = "Manage project versions and components."
    )]
    async fn manage_project(
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
        description = "Execute JQL search query."
    )]
    async fn execute_jql(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::jql::SearchParams>,
    ) -> String {
        let url = "/rest/api/3/search/jql";
        match self.send_request::<families::issue::SearchResults, _>(&url, Method::Post, None, Some(&params)).await {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jql_parse",
        description = "Parse and validate JQL queries."
    )]
    async fn parse_jql(
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
        description = "Search for users."
    )]
    async fn search_user(
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
        description = "Get current user details."
    )]
    async fn get_myself(
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
        description = "Convert plain text to Atlassian Document Format (ADF)."
    )]
    async fn convert_to_adf(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::helpers::TextToAdfParams>,
    ) -> String {
        let adf = families::helpers::text_to_adf(&params.text, params.style);
        let response = families::helpers::AdfDocument { adf };
        serde_json::to_string(&response).unwrap_or_default()
    }

    // =========================================================================
    // HELPER INTERNAL METHODS
    // =========================================================================

    async fn fetch_fields_internal(&self) -> Result<Vec<families::metadata::Field>, Box<dyn std::error::Error + Send + Sync>> {
        let url = "/rest/api/3/field";
        self.send_request::<Vec<families::metadata::Field>, ()>(&url, Method::Get, None, None).await
    }

    async fn handle_comment_operation(
        &self,
        params: families::issue_content::IssueContentParams,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/api/3/issue/{}/watchers", issue_key);
        self.send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&account_id)).await?;
        Ok(r#"{"success": true, "message": "Watcher added successfully"}"#.to_string())
    }

    async fn remove_watcher_internal(
        &self,
        issue_key: &str,
        account_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/api/3/issue/{}/watchers", issue_key);
        let query = vec![("accountId", account_id.to_string())];
        self.send_request::<serde_json::Value, ()>(&url, Method::Delete, Some(&query), None).await?;
        Ok(r#"{"success": true, "message": "Watcher removed successfully"}"#.to_string())
    }

    async fn add_vote_internal(
        &self,
        issue_key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/api/3/issue/{}/votes", issue_key);
        self.send_request::<serde_json::Value, ()>(&url, Method::Post, None, None).await?;
        Ok(r#"{"success": true, "message": "Vote added successfully"}"#.to_string())
    }

    async fn handle_attachment_operation(
        &self,
        params: families::issue_relations::IssueRelationsParams,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn get_board_internal(
        &self,
        board_id: i64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/agile/1.0/board/{}", board_id);
        let response = self.send_request::<families::agile::Board, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_boards_internal(
        &self,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/agile/1.0/sprint/{}", sprint_id);
        let response = self.send_request::<families::agile::Sprint, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_sprints_internal(
        &self,
        board_id: i64,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let sprint_data: families::agile::SprintData = serde_json::from_value(data.0)?;
        let url = format!("/rest/agile/1.0/sprint/{}", sprint_id);
        let response = self.send_request::<families::agile::Sprint, _>(&url, Method::Put, None, Some(&sprint_data)).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn delete_sprint_internal(
        &self,
        sprint_id: i64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/agile/1.0/sprint/{}", sprint_id);
        self.send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None).await?;
        Ok(r#"{"success": true, "message": "Sprint deleted successfully"}"#.to_string())
    }

    async fn get_board_issues_internal(
        &self,
        board_id: i64,
        params: families::agile::AgileQueryParams,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn get_priorities_internal(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = "/rest/api/3/priority";
        let response = self.send_request::<Vec<families::metadata::Priority>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_resolutions_internal(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = "/rest/api/3/resolution";
        let response = self.send_request::<Vec<families::metadata::Resolution>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_issue_types_for_project_internal(
        &self,
        project_key: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("/rest/api/3/project/{}", project_key);
        let response = self.send_request::<serde_json::Value, ()>(&url, Method::Get, None, None).await?;

        if let Some(issue_types) = response.get("issueTypes") {
            Ok(serde_json::to_string(issue_types)?)
        } else {
            Ok("[]".to_string())
        }
    }

    async fn get_all_issue_types_internal(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = "/rest/api/3/issuetype";
        let response = self.send_request::<Vec<families::metadata::IssueType>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn get_statuses_internal(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = "/rest/api/3/status";
        let response = self.send_request::<Vec<families::metadata::Status>, ()>(&url, Method::Get, None, None).await?;
        Ok(serde_json::to_string(&response)?)
    }

    async fn find_story_points_field_id(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
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