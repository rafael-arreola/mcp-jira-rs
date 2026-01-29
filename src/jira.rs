use crate::domains;
use reqwest::header::CONTENT_TYPE;
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper},
    model::{ServerCapabilities, ServerInfo},
    tool_handler, tool_router,
};
use std::collections::HashMap;

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

    /// =========================================================================
    /// HELPERS
    /// =========================================================================
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
        let full_url = if url.starts_with("http") {
            url.to_string()
        } else {
            format!("{}{}", self.base_url(), url)
        };

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

    async fn find_field_id(&self, name: &str) -> Option<String> {
        let url = "/rest/api/3/field";
        let fields: Vec<serde_json::Value> = self
            .send_request::<Vec<serde_json::Value>, ()>(url, Method::Get, None, None::<&()>)
            .await
            .ok()?;
        for field in fields {
            if let Some(field_name) = field.get("name").and_then(|n| n.as_str()) {
                if field_name.eq_ignore_ascii_case(name) {
                    return field
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string());
                }
            }
        }
        None
    }

    async fn get_editable_field_id(&self, issue_key: &str, possible_names: &[&str]) -> Option<String> {
        let url = format!("/rest/api/3/issue/{}/editmeta", issue_key);
        let meta: serde_json::Value = self
            .send_request::<serde_json::Value, ()>(&url, Method::Get, None, None::<&()>)
            .await
            .ok()?;

        let fields = meta.get("fields")?.as_object()?;

        for (id, field_info) in fields {
            if let Some(name) = field_info.get("name").and_then(|n| n.as_str()) {
                for target in possible_names {
                    if name.eq_ignore_ascii_case(target) {
                        return Some(id.clone());
                    }
                }
            }
        }
        None
    }

    async fn resolve_issue_type_id(
        &self,
        project_key: &str,
        issue_type: domains::enums::IssueType,
    ) -> Option<(String, bool)> {
        let url = format!(
            "/rest/api/3/issue/createmeta?projectKeys={}&expand=projects.issuetypes",
            project_key
        );
        let meta: serde_json::Value = self
            .send_request::<serde_json::Value, ()>(&url, Method::Get, None, None::<&()>)
            .await
            .ok()?;

        let projects = meta.get("projects")?.as_array()?;
        let project = projects
            .iter()
            .find(|p| p.get("key").and_then(|k| k.as_str()).unwrap_or("") == project_key)?;

        let types = project.get("issuetypes")?.as_array()?;
        let target = issue_type.to_string();

        for t in types {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let untranslated = t
                .get("untranslatedName")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if name.eq_ignore_ascii_case(&target) || untranslated.eq_ignore_ascii_case(&target) {
                let id = t.get("id").and_then(|v| v.as_str())?.to_string();
                let is_subtask = t.get("subtask").and_then(|v| v.as_bool()).unwrap_or(false);
                return Some((id, is_subtask));
            }
        }

        None
    }

    async fn find_transition_id(
        &self,
        issue_key: &str,
        target_status: domains::enums::Status,
    ) -> Option<String> {
        let url = format!("/rest/api/3/issue/{}/transitions", issue_key);
        let resp: domains::issue::TransitionResponse = self
            .send_request::<_, ()>(&url, Method::Get, None, None::<&()>)
            .await
            .ok()?;

        let target_name = target_status.to_string(); // e.g. "In Progress"

        for transition in &resp.transitions {
            if transition.name.eq_ignore_ascii_case(&target_name) {
                return Some(transition.id.clone());
            }
            if transition.to.name.eq_ignore_ascii_case(&target_name) {
                return Some(transition.id.clone());
            }
        }

        let target_category = match target_status {
            domains::enums::Status::ToDo => "new",
            domains::enums::Status::InProgress | domains::enums::Status::InReview => {
                "indeterminate"
            }
            domains::enums::Status::Done | domains::enums::Status::Cancelled => "done",
            domains::enums::Status::Blocked => "indeterminate",
        };

        for transition in &resp.transitions {
            if let Some(cat) = &transition.to.status_category {
                if cat.key.eq_ignore_ascii_case(target_category) {
                    return Some(transition.id.clone());
                }
            }
        }

        None
    }

    async fn resolve_assignee(&self, assignee: &str) -> Option<String> {
        if assignee.eq_ignore_ascii_case("me") {
            let resp: domains::user::User = self
                .send_request::<_, ()>("/rest/api/3/myself", Method::Get, None, None::<&()>)
                .await
                .ok()?;
            return Some(resp.account_id);
        }
        if assignee.eq_ignore_ascii_case("unassigned") {
            return Some("".to_string());
        }
        Some(assignee.to_string())
    }

    /// =========================================================================
    /// PHASE 1: Creation Domain
    /// =========================================================================

    #[rmcp::tool(
        name = "issue_create",
        description = "Creates an issue in Jira. Use this tool to create Stories, Bugs, Epics, Tasks, and Sub-tasks. It handles complex fields like ADF, priority IDs, and Epic linking automatically."
    )]
    async fn issue_create(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueCreateArgs>,
    ) -> String {
        let url = "/rest/api/3/issue";
        let mut fields = HashMap::new();

        fields.insert(
            "project".to_string(),
            serde_json::json!({ "key": params.project_key }),
        );

        let (issue_type_id, _is_subtask) = match self
            .resolve_issue_type_id(&params.project_key, params.issue_type)
            .await
        {
            Some(res) => res,
            None => {
                return format!(
                    r#"{{"error": "Could not find valid issue type ID for '{}' in project {}\n"}}"#, // Added newline for clarity
                    params.issue_type, params.project_key
                );
            }
        };
        fields.insert(
            "issuetype".to_string(),
            serde_json::json!({ "id": issue_type_id }),
        );

        fields.insert("summary".to_string(), serde_json::json!(params.summary));

        if let Some(desc) = params.description {
            fields.insert(
                "description".to_string(),
                domains::helpers::text_to_adf(&desc, domains::helpers::AdfStyle::Paragraph).0,
            );
        }

        if let Some(priority) = params.priority {
            fields.insert(
                "priority".to_string(),
                serde_json::json!({ "name": priority }),
            );
        }

        if let Some(parent) = params.parent_key {
            fields.insert("parent".to_string(), serde_json::json!({ "key": parent }));
        }

        if let Some(labels) = params.labels {
            fields.insert("labels".to_string(), serde_json::json!(labels));
        }

        if let Some(components) = params.components {
            let comps: Vec<_> = components
                .into_iter()
                .map(|c| serde_json::json!({ "name": c }))
                .collect();
            fields.insert("components".to_string(), serde_json::json!(comps));
        }

        if let Some(sp) = params.story_points {
            if let Some(sp_field) = self.find_field_id("Story Points").await {
                fields.insert(sp_field, serde_json::json!(sp));
            }
        }

        if let Some(sp_estimate) = params.story_point_estimate {
            if let Some(sp_field) = self.find_field_id("Story point estimate").await {
                fields.insert(sp_field, serde_json::json!(sp_estimate));
            }
        }

        let body = serde_json::json!({ "fields": fields });

        match self
            .send_request::<domains::issue::CreatedIssue, _>(url, Method::Post, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    /// =========================================================================
    /// PHASE 2: Management Domain
    /// =========================================================================

    #[rmcp::tool(
        name = "issue_update_status",
        description = "Moves an issue to a new workflow status (Transition)."
    )]
    async fn issue_update_status(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueUpdateStatusArgs>,
    ) -> String {
        let transition_id = match self
            .find_transition_id(&params.issue_key, params.status)
            .await
        {
            Some(id) => id,
            None => {
                return format!(
                    r#"{{"error": "Transition to '{}' not found for issue {}\n"}}"#, // Added newline for clarity
                    params.status, params.issue_key
                );
            }
        };

        let url = format!("/rest/api/3/issue/{}/transitions", params.issue_key);
        let body = serde_json::json!({ "transition": { "id": transition_id } });

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Issue {} moved to {}\n"}}"#, // Added newline for clarity
                params.issue_key, params.status
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_assign",
        description = "Assigns the issue to a specific user."
    )]
    async fn issue_assign(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueAssignArgs>,
    ) -> String {
        let account_id = match self.resolve_assignee(&params.assignee).await {
            Some(id) => id,
            None => {
                return format!(
                    r#"{{"error": "Could not resolve assignee '{}'\n"}}"#, // Added newline for clarity
                    params.assignee
                );
            }
        };

        let url = format!("/rest/api/3/issue/{}/assignee", params.issue_key);
        let body = serde_json::json!({ "accountId": if account_id.is_empty() { None } else { Some(account_id) } });

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body))
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Issue {} assigned to {}\n"}}"#, // Added newline for clarity
                params.issue_key, params.assignee
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_edit_details",
        description = "Modifies informational fields of an existing issue."
    )]
    async fn issue_edit_details(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueEditDetailsArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_key);
        let mut fields = HashMap::new();

        if let Some(summary) = params.summary {
            fields.insert("summary".to_string(), serde_json::json!(summary));
        }

        if let Some(desc) = params.description {
            fields.insert(
                "description".to_string(),
                domains::helpers::text_to_adf(&desc, domains::helpers::AdfStyle::Paragraph).0,
            );
        }

        if let Some(issue_type) = params.issue_type {
            // Extract project key from issue key (e.g., "PROJ-123" -> "PROJ")
            let project_key = params.issue_key.split('-').next().unwrap_or("");
            
            if let Some((id, _)) = self.resolve_issue_type_id(project_key, issue_type).await {
                fields.insert(
                    "issuetype".to_string(),
                    serde_json::json!({ "id": id }),
                );
            }
        }

        if let Some(priority) = params.priority {
            fields.insert(
                "priority".to_string(),
                serde_json::json!({ "name": priority }),
            );
        }

        if let Some(labels) = params.labels {
            fields.insert("labels".to_string(), serde_json::json!(labels));
        }

        if let Some(components) = params.components {
            let comps: Vec<_> = components
                .into_iter()
                .map(|c| serde_json::json!({ "name": c }))
                .collect();
            fields.insert("components".to_string(), serde_json::json!(comps));
        }

        let body = serde_json::json!({ "fields": fields });

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body))
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Issue {} updated successfully\n"}}"#, // Added newline for clarity
                params.issue_key
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_set_story_points",
        description = "Sets the story point estimation for an issue. Automatically detects the correct field (Story Points or Story point estimate)."
    )]
    async fn issue_set_story_points(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueSetStoryPointsArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_key);
        let mut fields = HashMap::new();

        // Check which field is actually editable for this issue
        let field_id = self
            .get_editable_field_id(
                &params.issue_key,
                &["Story Points", "Story point estimate"],
            )
            .await;

        match field_id {
            Some(id) => {
                fields.insert(id, serde_json::json!(params.story_points));
                let body = serde_json::json!({ "fields": fields });

                match self
                    .send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body))
                    .await
                {
                    Ok(_) => format!(
                        r#"{{"success": true, "message": "Story points set to {} for issue {}\n"}}"#,
                        params.story_points, params.issue_key
                    ),
                    Err(e) => e.to_string(),
                }
            }
            None => {
                // Fallback: Try global search if editmeta fails (though unlikely to work if editmeta didn't have it)
                let legacy_id = self.find_field_id("Story Points").await;
                let nextgen_id = self.find_field_id("Story point estimate").await;
                
                format!(
                    r#"{{"error": "Could not find an editable 'Story Points' field for issue {}. Detected fields in instance: Story Points (Classic) = {:?}, Story point estimate (Next-Gen) = {:?}. Please ensure the field is on the issue's EDIT screen.\n"}}"#,
                    params.issue_key, legacy_id, nextgen_id
                )
            }
        }
    }

    #[rmcp::tool(
        name = "issue_add_comment",
        description = "Adds a comment to an issue."
    )]
    async fn issue_add_comment(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueAddCommentArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/comment", params.issue_key);
        let body = serde_json::json!({ "body": domains::helpers::text_to_adf(&params.comment, domains::helpers::AdfStyle::Paragraph).0 });

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_link",
        description = "Creates a semantic link between two issues."
    )]
    async fn issue_link(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueLinkArgs>,
    ) -> String {
        let url = "/rest/api/3/issueLink";
        let body = serde_json::json!({
            "type": { "name": params.link_type },
            "inwardIssue": { "key": params.source_issue_key },
            "outwardIssue": { "key": params.target_issue_key }
        });

        match self
            .send_request::<serde_json::Value, _>(url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Linked {} to {} with type {}\n"}}"#, // Added newline for clarity
                params.source_issue_key, params.target_issue_key, params.link_type
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(name = "issue_log_work", description = "Logs time spent on an issue.")]
    async fn issue_log_work(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueLogWorkArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/worklog", params.issue_key);
        let mut body = HashMap::new();
        body.insert(
            "timeSpent".to_string(),
            serde_json::json!(params.time_spent),
        );

        if let Some(started) = params.started {
            body.insert("started".to_string(), serde_json::json!(started));
        }

        if let Some(comment) = params.comment {
            body.insert(
                "comment".to_string(),
                domains::helpers::text_to_adf(&comment, domains::helpers::AdfStyle::Paragraph).0,
            );
        }

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_delete",
        description = "Permanently deletes an issue from Jira."
    )]
    async fn issue_delete(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueDeleteArgs>,
    ) -> String {
        let mut url = format!("/rest/api/3/issue/{}", params.issue_key);
        if let Some(delete_subtasks) = params.delete_subtasks {
            url = format!("{}?deleteSubtasks={}", url, delete_subtasks);
        }

        match self
            .send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Issue {} deleted successfully\n"}}"#, // Added newline for clarity
                params.issue_key
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_archive",
        description = "Archives a list of issues. Archiving an issue removes it from the index and search results but preserves the data."
    )]
    async fn issue_archive(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueArchiveArgs>,
    ) -> String {
        let url = "/rest/api/3/issue/archive";
        let body = serde_json::json!({ "issueIdsOrKeys": params.issue_keys });

        match self
            .send_request::<serde_json::Value, _>(url, Method::Put, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_unarchive",
        description = "Unarchives (restores) a list of previously archived issues."
    )]
    async fn issue_unarchive(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueUnarchiveArgs>,
    ) -> String {
        let url = "/rest/api/3/issue/archive/restore";
        let body = serde_json::json!({ "issueIdsOrKeys": params.issue_keys });

        match self
            .send_request::<serde_json::Value, _>(url, Method::Put, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_delete_comment",
        description = "Deletes a specific comment."
    )]
    async fn issue_delete_comment(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueDeleteCommentArgs>,
    ) -> String {
        let url = format!(
            "/rest/api/3/issue/{}/comment/{}",
            params.issue_key, params.comment_id
        );

        match self
            .send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Comment {} deleted successfully\n"}}"#, // Added newline for clarity
                params.comment_id
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_delete_link",
        description = "Removes a link between two issues."
    )]
    async fn issue_delete_link(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueDeleteLinkArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issueLink/{}", params.link_id);

        match self
            .send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Link {} deleted successfully\n"}}"#, // Added newline for clarity
                params.link_id
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_set_parent",
        description = "Links an existing Story/Task to an Epic, or removes the parent link. Use this to organize existing issues under Epics in your project hierarchy."
    )]
    async fn issue_set_parent(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueSetParentArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_key);
        let mut fields = HashMap::new();

        if params.parent_key.is_empty() {
            fields.insert("parent".to_string(), serde_json::Value::Null);
        } else {
            fields.insert(
                "parent".to_string(),
                serde_json::json!({ "key": params.parent_key }),
            );
        }

        let body = serde_json::json!({ "fields": fields });

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body))
            .await
        {
            Ok(_) => {
                if params.parent_key.is_empty() {
                    format!(
                        r#"{{"success": true, "message": "Parent removed from issue {}\n"}}"#,
                        params.issue_key
                    )
                } else {
                    format!(
                        r#"{{"success": true, "message": "Issue {} linked to parent {}\n"}}"#,
                        params.issue_key, params.parent_key
                    )
                }
            }
            Err(e) => {
                // If modern field fails, try legacy "Epic Link" field
                let error_msg = e.to_string();
                if error_msg.contains("parent") || error_msg.contains("not found") {
                    // Attempt legacy Epic Link field
                    if let Some(epic_link_field) = self.find_field_id("Epic Link").await {
                        let mut legacy_fields = HashMap::new();
                        if params.parent_key.is_empty() {
                            legacy_fields.insert(epic_link_field, serde_json::Value::Null);
                        } else {
                            legacy_fields
                                .insert(epic_link_field, serde_json::json!(params.parent_key));
                        }
                        let legacy_body = serde_json::json!({ "fields": legacy_fields });

                        return match self
                            .send_request::<serde_json::Value, _>(
                                &url,
                                Method::Put,
                                None,
                                Some(&legacy_body),
                            )
                            .await
                        {
                            Ok(_) => {
                                if params.parent_key.is_empty() {
                                    format!(
                                        r#"{{"success": true, "message": "Epic link removed from issue {} (legacy field)\n"}}"#,
                                        params.issue_key
                                    )
                                } else {
                                    format!(
                                        r#"{{"success": true, "message": "Issue {} linked to Epic {} (legacy field)\n"}}"#,
                                        params.issue_key, params.parent_key
                                    )
                                }
                            }
                            Err(legacy_err) => format!(
                                r#"{{"error": "Failed with both parent and Epic Link fields. Modern error: {}. Legacy error: {}"}}"#,
                                error_msg, legacy_err
                            ),
                        };
                    }
                }
                error_msg
            }
        }
    }

    /// =========================================================================
    /// PHASE 3: Search & Discovery Domain
    /// =========================================================================

    #[rmcp::tool(
        name = "search_issues",
        description = "Searches for issues using natural language text or specific filters. Use 'filter' parameter to reduce context by 70-90%. Supports same filter syntax as issue_get."
    )]
    async fn search_issues(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::jql::SearchIssuesArgs>,
    ) -> String {
        let url = "/rest/api/3/search/jql";
        let mut jql_parts = Vec::new();
        let mut order_by_clause = None;

        if let Some(text) = params.text {
            jql_parts.push(format!("text ~ \"{}\"", text));
        }
        if let Some(status) = params.status {
            jql_parts.push(format!("status = \"{}\"", status));
        }
        if let Some(assignee) = params.assignee {
            let acc_id = self.resolve_assignee(&assignee).await.unwrap_or(assignee);
            if acc_id.is_empty() {
                jql_parts.push("assignee is EMPTY".to_string());
            } else {
                jql_parts.push(format!("assignee = \"{}\"", acc_id));
            }
        }
        if let Some(raw_jql) = params.jql {
            // Robust parsing of ORDER BY
            let lower_jql = raw_jql.to_lowercase();
            if let Some(idx) = lower_jql.rfind("order by") {
                let (predicate, order) = raw_jql.split_at(idx);
                if !predicate.trim().is_empty() {
                    jql_parts.push(format!("({})", predicate.trim()));
                }
                order_by_clause = Some(order.to_string());
            } else {
                jql_parts.push(format!("({})", raw_jql));
            }
        }

        let mut jql = jql_parts.join(" AND ");

        if let Some(order) = order_by_clause {
            jql.push_str(" ");
            jql.push_str(&order);
        }

        let mut body = HashMap::new();
        body.insert("jql".to_string(), serde_json::json!(jql));
        if let Some(limit) = params.limit {
            body.insert("maxResults".to_string(), serde_json::json!(limit));
        }
        body.insert("fieldsByKeys".to_string(), serde_json::json!(true));

        // Agregar filtrado de campos si se especificó
        if let Some(filter_str) = params.filter {
            let parsed = domains::helpers::parse_field_filter(&filter_str);
            let fields: Vec<&str> = parsed.split(',').collect();
            body.insert("fields".to_string(), serde_json::json!(fields));
        }

        match self
            .send_request::<serde_json::Value, _>(url, Method::Post, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "issue_get",
        description = "Retrieves issue details. Use 'filter' parameter to reduce context by 70-90%. Presets: 'minimal', 'basic', 'standard', 'detailed'. Custom: 'id key summary'. Use 'fields_list' to discover custom fields."
    )]
    async fn issue_get(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::IssueGetArgs>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_key);

        // Construir query params si hay filtro
        let mut query_params = Vec::new();
        if let Some(filter_str) = params.filter {
            let parsed = domains::helpers::parse_field_filter(&filter_str);
            query_params.push(("fields", parsed));
        }

        let query = if query_params.is_empty() {
            None
        } else {
            Some(&query_params)
        };

        match self
            .send_request::<domains::issue::Issue, ()>(&url, Method::Get, query, None::<&()>)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "fields_list",
        description = "Lists all available Jira fields for filtering. Returns field IDs, names, types, and whether they're custom fields. Use this once per session to discover which fields you can use in 'filter' parameters of other tools. System fields (summary, status) are standard across all Jira instances. Custom fields (Story Points, Sprint) are specific to this workspace."
    )]
    async fn fields_list(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::issue::FieldsListArgs>,
    ) -> String {
        let url = "/rest/api/3/field";

        match self
            .send_request::<Vec<serde_json::Value>, ()>(url, Method::Get, None, None::<&()>)
            .await
        {
            Ok(fields) => {
                // Simplificar respuesta para reducir contexto
                let simplified: Vec<_> = fields
                    .into_iter()
                    .filter_map(|field| {
                        let id = field.get("id")?.as_str()?;
                        let name = field.get("name")?.as_str()?;
                        let is_custom = field.get("custom")?.as_bool().unwrap_or(false);

                        // Filtrar por tipo si se especificó
                        if let Some(ref filter) = params.field_type {
                            match filter.to_lowercase().as_str() {
                                "system" if is_custom => return None,
                                "custom" if !is_custom => return None,
                                _ => {}
                            }
                        }

                        let field_type = field
                            .get("schema")
                            .and_then(|s| s.get("type"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown");

                        Some(serde_json::json!({
                            "id": id,
                            "name": name,
                            "type": field_type,
                            "custom": is_custom
                        }))
                    })
                    .collect();

                serde_json::to_string(&serde_json::json!({
                    "total": simplified.len(),
                    "fields": simplified,
                    "usage": "Use field 'id' values in filter parameters. Example: filter='id key summary customfield_10016'"
                })).unwrap_or_default()
            }
            Err(e) => format!(r#"{{"error": "Failed to fetch fields: {}"}}"#, e),
        }
    }

    #[rmcp::tool(
        name = "agile_rank_issues",
        description = "Reorders issues in the backlog or board."
    )]
    async fn agile_rank_issues(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::agile::AgileRankIssuesArgs>,
    ) -> String {
        let url = "/rest/agile/1.0/issue/rank";
        let mut body = HashMap::new();
        body.insert("issues".to_string(), serde_json::json!(params.issue_keys));

        if let Some(after) = params.after_issue_key {
            body.insert("rankAfterIssue".to_string(), serde_json::json!(after));
        }
        if let Some(before) = params.before_issue_key {
            body.insert("rankBeforeIssue".to_string(), serde_json::json!(before));
        }

        match self
            .send_request::<serde_json::Value, _>(url, Method::Put, None, Some(&body))
            .await
        {
            Ok(_) => {
                r#"{{"success": true, "message": "Issues reordered successfully"}}"#.to_string()
            }
            Err(e) => e.to_string(),
        }
    }

    /// =========================================================================
    /// PHASE 4: Agile Domain (Sprints)
    /// =========================================================================
    #[rmcp::tool(
        name = "board_get_sprints",
        description = "Lists sprints associated with a board or project."
    )]
    async fn board_get_sprints(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::sprint::BoardGetSprintsArgs>,
    ) -> String {
        // First, find the board ID
        let board_id = if let Some(name) = params.board_name {
            let url = "/rest/agile/1.0/board";
            let query = vec![("name", name)];
            let resp: serde_json::Value = self
                .send_request::<_, ()>(url, Method::Get, Some(&query), None::<&()>)
                .await
                .unwrap_or(serde_json::Value::Null);
            resp.get("values")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|b| b.get("id"))
                .and_then(|id| id.as_i64())
        } else if let Some(project) = params.project_key {
            let url = "/rest/agile/1.0/board";
            let query = vec![("projectKeyOrId", project)];
            let resp: serde_json::Value = self
                .send_request::<_, ()>(url, Method::Get, Some(&query), None::<&()>)
                .await
                .unwrap_or(serde_json::Value::Null);
            resp.get("values")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|b| b.get("id"))
                .and_then(|id| id.as_i64())
        } else {
            None
        };

        let board_id = match board_id {
            Some(id) => id,
            None => return r#"{{"error": "Board not found"}}"#.to_string(),
        };

        let url = format!("/rest/agile/1.0/board/{}/sprint", board_id);
        let mut query = Vec::new();
        if let Some(state) = params.state {
            query.push(("state", state.to_string()));
        }

        match self
            .send_request::<serde_json::Value, ()>(&url, Method::Get, Some(&query), None::<&()>)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "board_get_backlog",
        description = "Gets issues in the backlog."
    )]
    async fn board_get_backlog(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::agile::BoardGetBacklogArgs>,
    ) -> String {
        let board_id = if let Some(name) = params.board_name {
            let url = "/rest/agile/1.0/board";
            let query = vec![("name", name)];
            let resp: serde_json::Value = self
                .send_request::<_, ()>(url, Method::Get, Some(&query), None::<&()>)
                .await
                .unwrap_or(serde_json::Value::Null);
            resp.get("values")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|b| b.get("id"))
                .and_then(|id| id.as_i64())
        } else if let Some(project) = params.project_key {
            let url = "/rest/agile/1.0/board";
            let query = vec![("projectKeyOrId", project)];
            let resp: serde_json::Value = self
                .send_request::<_, ()>(url, Method::Get, Some(&query), None::<&()>)
                .await
                .unwrap_or(serde_json::Value::Null);
            resp.get("values")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|b| b.get("id"))
                .and_then(|id| id.as_i64())
        } else {
            None
        };

        let board_id = match board_id {
            Some(id) => id,
            None => return r#"{{"error": "Board not found"}}"#.to_string(),
        };

        let url = format!("/rest/agile/1.0/board/{}/backlog", board_id);

        match self
            .send_request::<serde_json::Value, ()>(&url, Method::Get, None, None::<&()>)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(name = "sprint_create", description = "Creates a new planned sprint.")]
    async fn sprint_create(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::sprint::SprintCreateArgs>,
    ) -> String {
        let url = "/rest/agile/1.0/sprint";
        let mut body = HashMap::new();
        body.insert(
            "originBoardId".to_string(),
            serde_json::json!(params.board_id),
        );
        body.insert("name".to_string(), serde_json::json!(params.name));

        if let Some(goal) = params.goal {
            body.insert("goal".to_string(), serde_json::json!(goal));
        }
        if let Some(start) = params.start_date {
            body.insert("startDate".to_string(), serde_json::json!(start));
        }
        if let Some(end) = params.end_date {
            body.insert("endDate".to_string(), serde_json::json!(end));
        }

        match self
            .send_request::<serde_json::Value, _>(url, Method::Post, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "sprint_update",
        description = "Updates a sprint's details or changes its state (Start/Close)."
    )]
    async fn sprint_update(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::sprint::SprintUpdateArgs>,
    ) -> String {
        // First, get current sprint data to merge with updates
        let get_url = format!("/rest/agile/1.0/sprint/{}", params.sprint_id);
        let current_sprint: serde_json::Value = match self
            .send_request::<serde_json::Value, ()>(&get_url, Method::Get, None, None::<&()>)
            .await
        {
            Ok(sprint) => sprint,
            Err(e) => return format!(r#"{{"error": "Failed to get sprint: {}"}}"#, e),
        };

        let mut body = HashMap::new();

        // Handle name with validation
        let final_name = if let Some(new_name) = params.name {
            if new_name.len() > 30 {
                return format!(
                    r#"{{"error": "Sprint name must be 30 characters or less (got {} characters)"}}"#,
                    new_name.len()
                );
            }
            new_name
        } else {
            // Use current name if not updating
            current_sprint["name"]
                .as_str()
                .unwrap_or("Sprint")
                .to_string()
        };
        body.insert("name".to_string(), serde_json::json!(final_name));

        // Handle state - use current if not updating
        let final_state = if let Some(new_state) = params.state {
            new_state.to_string()
        } else {
            current_sprint["state"]
                .as_str()
                .unwrap_or("future")
                .to_string()
        };
        body.insert("state".to_string(), serde_json::json!(final_state));

        // Optional fields
        if let Some(goal) = params.goal {
            body.insert("goal".to_string(), serde_json::json!(goal));
        } else if let Some(current_goal) = current_sprint["goal"].as_str() {
            body.insert("goal".to_string(), serde_json::json!(current_goal));
        }

        if let Some(start) = params.start_date {
            body.insert("startDate".to_string(), serde_json::json!(start));
        } else if let Some(current_start) = current_sprint["startDate"].as_str() {
            body.insert("startDate".to_string(), serde_json::json!(current_start));
        }

        if let Some(end) = params.end_date {
            body.insert("endDate".to_string(), serde_json::json!(end));
        } else if let Some(current_end) = current_sprint["endDate"].as_str() {
            body.insert("endDate".to_string(), serde_json::json!(current_end));
        }

        let url = format!("/rest/agile/1.0/sprint/{}", params.sprint_id);
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "sprint_add_issues",
        description = "Moves issues into a specific sprint."
    )]
    async fn sprint_add_issues(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::sprint::SprintAddIssuesArgs>,
    ) -> String {
        let url = format!("/rest/agile/1.0/sprint/{}/issue", params.sprint_id);
        let body = serde_json::json!({ "issues": params.issue_keys });

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Issues added to sprint {}\n"}}"#, // Added newline for clarity
                params.sprint_id
            ),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(name = "sprint_delete", description = "Deletes a planned sprint.")]
    async fn sprint_delete(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<domains::sprint::SprintDeleteArgs>,
    ) -> String {
        let url = format!("/rest/agile/1.0/sprint/{}", params.sprint_id);

        match self
            .send_request::<serde_json::Value, ()>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => format!(
                r#"{{"success": true, "message": "Sprint {} deleted successfully\n"}}"#, // Added newline for clarity
                params.sprint_id
            ),
            Err(e) => e.to_string(),
        }
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
