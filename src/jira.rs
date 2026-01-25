use crate::families;
use reqwest::header::CONTENT_TYPE;
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper},
    model::{ServerCapabilities, ServerInfo},
    tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

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
        
        // Sanitize schemas to remove "$schema" which confuses Gemini
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

    // =========================================================================
    // BEGIN: Issues
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_create",
        description = "Creates an issue or a sub-task. The 'fields' parameter must be a JSON object. IMPORTANT: Rich text fields like 'description' must be provided in Atlassian Document Format (ADF) JSON, not plain text."
    )]
    async fn create_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::CreateIssueParams>,
    ) -> String {
        let url = "/rest/api/3/issue";
        match self
            .send_request::<families::issue::CreatedIssue, _>(
                url,
                Method::Post,
                None,
                Some(&params),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_get",
        description = "Returns the details for an issue. The issue is identified by its ID or key. If the identifier doesn't match an issue, a case-insensitive search and check for moved issues is performed."
    )]
    async fn get_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::GetIssueParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(fields) = &params.fields {
            query_params.push(("fields", fields.join(",")));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }
        if let Some(properties) = &params.properties {
            query_params.push(("properties", properties.join(",")));
        }
        if let Some(update_history) = params.update_history {
            query_params.push(("updateHistory", update_history.to_string()));
        }

        match self
            .send_request::<families::issue::Issue, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_edit",
        description = "Edits an issue. IMPORTANT: Rich text fields like 'description' in 'fields' or 'update' must be provided in Atlassian Document Format (ADF) JSON, not plain text."
    )]
    async fn edit_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::EditIssueParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(notify) = params.notify_users {
            query_params.push(("notifyUsers", notify.to_string()));
        }
        if let Some(override_editable) = params.override_editable_flag {
            query_params.push(("overrideEditableFlag", override_editable.to_string()));
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
            fields: params.fields.as_ref(),
            update: params.update.as_ref(),
        };

        match self
            .send_request::<serde_json::Value, _>(
                &url,
                Method::Put,
                Some(&query_params),
                Some(&body),
            )
            .await
        {
            Ok(_) => "Issue updated successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_delete",
        description = "Deletes an issue. An issue cannot be deleted if it has one or more subtasks. To delete an issue with subtasks, set deleteSubtasks to true."
    )]
    async fn delete_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::DeleteIssueParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(delete_subtasks) = &params.delete_subtasks {
            query_params.push(("deleteSubtasks", delete_subtasks.clone()));
        }

        match self
            .send_request::<serde_json::Value, _>(
                &url,
                Method::Delete,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(_) => "Issue deleted successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_assign",
        description = "Assigns an issue to a user. Use this operation when the calling user does not have the 'Edit Issues' permission but has the 'Assign issue' permission for the project. Set accountId to -1 to assign to default, or null to unassign."
    )]
    async fn assign_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::AssignIssueParams>,
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

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Put, None, Some(&body))
            .await
        {
            Ok(_) => "Issue assigned successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_get_transitions",
        description = "Returns either all transitions or a transition that can be performed by the user on an issue, based on the issue's status. Transition issues permission is required."
    )]
    async fn get_transitions(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::GetTransitionsParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/transitions", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(transition_id) = &params.transition_id {
            query_params.push(("transitionId", transition_id.clone()));
        }
        if let Some(skip) = params.skip_remote_only_condition {
            query_params.push(("skipRemoteOnlyCondition", skip.to_string()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TransitionResponse {
            transitions: Vec<families::issue::IssueTransition>,
        }

        match self
            .send_request::<TransitionResponse, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_transition",
        description = "Performs an issue transition and, if the transition has a screen, updates the fields from the transition screen. To update fields, specify them in the fields or update parameters."
    )]
    async fn transition_issue(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::TransitionIssueParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/transitions", params.issue_id_or_key);
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            transition: &'a families::issue::IssueTransition,
            #[serde(skip_serializing_if = "Option::is_none")]
            fields: Option<&'a families::JsonValue>,
            #[serde(skip_serializing_if = "Option::is_none")]
            update: Option<&'a families::JsonValue>,
        }
        let body = Body {
            transition: &params.transition,
            fields: params.fields.as_ref(),
            update: params.update.as_ref(),
        };

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => "Issue transitioned successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Comments
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_comment_get",
        description = "Returns all comments for an issue. Browse projects permission is required for the project containing the comment."
    )]
    async fn get_comments(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_comment::GetCommentsParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/comment", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(order_by) = &params.order_by {
            query_params.push(("orderBy", order_by.clone()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::issue_comment::PageBeanComment, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_comment_add",
        description = "Adds a comment to an issue. Browse projects and Add comments project permissions are required."
    )]
    async fn add_comment(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_comment::AddCommentParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/comment", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        // Helper to convert plain string to ADF
        fn text_to_adf(text: &str) -> families::JsonValue {
            families::JsonValue(serde_json::json!({
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": text
                            }
                        ]
                    }
                ]
            }))
        }

        let final_body = if params.body.is_string() {
            text_to_adf(params.body.as_str().unwrap_or_default())
        } else {
            params.body.clone()
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            body: families::JsonValue,
            #[serde(skip_serializing_if = "Option::is_none")]
            visibility: Option<&'a families::issue_comment::Visibility>,
            #[serde(skip_serializing_if = "Option::is_none")]
            properties: Option<&'a Vec<families::issue_comment::EntityProperty>>,
        }
        let body = Body {
            body: final_body,
            visibility: params.visibility.as_ref(),
            properties: params.properties.as_ref(),
        };

        match self
            .send_request::<families::issue_comment::Comment, _>(
                &url,
                Method::Post,
                Some(&query_params),
                Some(&body),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_comment_edit",
        description = "Updates a comment. Edit all comments project permission is required to update any comment or Edit own comments to update comment created by the user."
    )]
    async fn update_comment(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_comment::UpdateCommentParams,
        >,
    ) -> String {
        let url = format!(
            "/rest/api/3/issue/{}/comment/{}",
            params.issue_id_or_key, params.comment_id
        );
        let mut query_params = Vec::new();
        if let Some(notify) = params.notify_users {
            query_params.push(("notifyUsers", notify.to_string()));
        }
        if let Some(override_editable) = params.override_editable_flag {
            query_params.push(("overrideEditableFlag", override_editable.to_string()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            body: &'a families::JsonValue,
            #[serde(skip_serializing_if = "Option::is_none")]
            visibility: Option<&'a families::issue_comment::Visibility>,
            #[serde(skip_serializing_if = "Option::is_none")]
            properties: Option<&'a Vec<families::issue_comment::EntityProperty>>,
        }
        let body = Body {
            body: &params.body,
            visibility: params.visibility.as_ref(),
            properties: params.properties.as_ref(),
        };

        match self
            .send_request::<families::issue_comment::Comment, _>(
                &url,
                Method::Put,
                Some(&query_params),
                Some(&body),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_comment_delete",
        description = "Deletes a comment. Delete all comments project permission is required to delete any comment or Delete own comments to delete comment created by the user."
    )]
    async fn delete_comment(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_comment::DeleteCommentParams,
        >,
    ) -> String {
        let url = format!(
            "/rest/api/3/issue/{}/comment/{}",
            params.issue_id_or_key, params.comment_id
        );
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => "Comment deleted successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Watchers & Votes
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_social_add_watcher",
        description = "Adds a user as a watcher of an issue by passing the account ID of the user. If no user is specified the calling user is added. Manage watcher list project permission is required."
    )]
    async fn add_watcher(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_social::AddWatcherParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/watchers", params.issue_id_or_key);
        let body = params.account_id;
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => "Watcher added successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_social_delete_watcher",
        description = "Deletes a user as a watcher of an issue. Manage watcher list project permission is required."
    )]
    async fn delete_watcher(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_social::DeleteWatcherParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/watchers", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(account_id) = &params.account_id {
            query_params.push(("accountId", account_id.clone()));
        }

        match self
            .send_request::<serde_json::Value, _>(
                &url,
                Method::Delete,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(_) => "Watcher removed successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_social_vote",
        description = "Adds the user's vote to an issue. This is the equivalent of the user clicking Vote on an issue in Jira."
    )]
    async fn add_vote(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_social::IssueSocialParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/votes", params.issue_id_or_key);
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, None::<&()>)
            .await
        {
            Ok(_) => "Vote added successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Worklogs
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_worklog_get",
        description = "Returns worklogs for an issue (ordered by created time), starting from the oldest worklog or from the worklog started on or after a date and time."
    )]
    async fn get_worklogs(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_worklog::GetWorklogsParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/worklog", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(started_after) = params.started_after {
            query_params.push(("startedAfter", started_after.to_string()));
        }
        if let Some(started_before) = params.started_before {
            query_params.push(("startedBefore", started_before.to_string()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::issue_worklog::PageBeanWorklog, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_worklog_add",
        description = "Adds a worklog to an issue. Browse projects and Work on issues project permissions are required."
    )]
    async fn add_worklog(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_worklog::AddWorklogParams>,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/worklog", params.issue_id_or_key);
        let mut query_params = Vec::new();
        if let Some(notify) = params.notify_users {
            query_params.push(("notifyUsers", notify.to_string()));
        }
        if let Some(adjust) = &params.adjust_estimate {
            query_params.push(("adjustEstimate", adjust.clone()));
        }
        if let Some(new_estimate) = &params.new_estimate {
            query_params.push(("newEstimate", new_estimate.clone()));
        }
        if let Some(reduce) = &params.reduce_by {
            query_params.push(("reduceBy", reduce.clone()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }
        if let Some(override_editable) = params.override_editable_flag {
            query_params.push(("overrideEditableFlag", override_editable.to_string()));
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            comment: Option<&'a families::JsonValue>,
            #[serde(skip_serializing_if = "Option::is_none")]
            started: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            time_spent: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            time_spent_seconds: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            visibility: Option<&'a families::issue_comment::Visibility>,
            #[serde(skip_serializing_if = "Option::is_none")]
            properties: Option<&'a Vec<families::issue_comment::EntityProperty>>,
        }
        let body = Body {
            comment: params.comment.as_ref(),
            started: params.started.as_ref(),
            time_spent: params.time_spent.as_ref(),
            time_spent_seconds: params.time_spent_seconds,
            visibility: params.visibility.as_ref(),
            properties: params.properties.as_ref(),
        };

        match self
            .send_request::<families::issue_worklog::Worklog, _>(
                &url,
                Method::Post,
                Some(&query_params),
                Some(&body),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_worklog_edit",
        description = "Updates a worklog. Edit all worklogs project permission is required to update any worklog or Edit own worklogs to update worklogs created by the user."
    )]
    async fn update_worklog(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_worklog::UpdateWorklogParams,
        >,
    ) -> String {
        let url = format!(
            "/rest/api/3/issue/{}/worklog/{}",
            params.issue_id_or_key, params.worklog_id
        );
        let mut query_params = Vec::new();
        if let Some(notify) = params.notify_users {
            query_params.push(("notifyUsers", notify.to_string()));
        }
        if let Some(adjust) = &params.adjust_estimate {
            query_params.push(("adjustEstimate", adjust.clone()));
        }
        if let Some(new_estimate) = &params.new_estimate {
            query_params.push(("newEstimate", new_estimate.clone()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }
        if let Some(override_editable) = params.override_editable_flag {
            query_params.push(("overrideEditableFlag", override_editable.to_string()));
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            comment: Option<&'a families::JsonValue>,
            #[serde(skip_serializing_if = "Option::is_none")]
            started: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            time_spent: Option<&'a String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            time_spent_seconds: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            visibility: Option<&'a families::issue_comment::Visibility>,
            #[serde(skip_serializing_if = "Option::is_none")]
            properties: Option<&'a Vec<families::issue_comment::EntityProperty>>,
        }
        let body = Body {
            comment: params.comment.as_ref(),
            started: params.started.as_ref(),
            time_spent: params.time_spent.as_ref(),
            time_spent_seconds: params.time_spent_seconds,
            visibility: params.visibility.as_ref(),
            properties: params.properties.as_ref(),
        };

        match self
            .send_request::<families::issue_worklog::Worklog, _>(
                &url,
                Method::Put,
                Some(&query_params),
                Some(&body),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Links & Metadata
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_link_create",
        description = "Creates a link between two issues. Use this operation to indicate a relationship between two issues."
    )]
    async fn create_issue_link(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_link::CreateIssueLinkParams,
        >,
    ) -> String {
        let url = "/rest/api/3/issueLink";
        match self
            .send_request::<serde_json::Value, _>(url, Method::Post, None, Some(&params))
            .await
        {
            Ok(_) => "Issue link created successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_link_delete",
        description = "Deletes an issue link. Browse project project permission for all projects containing the issues in the link is required."
    )]
    async fn delete_issue_link(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_link::DeleteIssueLinkParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/issueLink/{}", params.link_id);
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => "Issue link deleted successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Attachments
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_attachment_get",
        description = "Returns the attachments for an issue. Browse projects permission is required for the project containing the issue."
    )]
    async fn get_attachments(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_attachment::GetAttachmentsParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/issue/{}/attachments", params.issue_id_or_key);
        match self
            .send_request::<Vec<families::issue_attachment::Attachment>, _>(
                &url,
                Method::Get,
                None,
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_attachment_delete",
        description = "Deletes an attachment. Delete own attachments or Delete all attachments project permission is required."
    )]
    async fn delete_attachment(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_attachment::DeleteAttachmentParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/attachment/{}", params.attachment_id);
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => "Attachment deleted successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_metadata_get_labels",
        description = "Returns a paginated list of labels for the global label field."
    )]
    async fn get_labels(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue_metadata::GetLabelsParams>,
    ) -> String {
        let url = "/rest/api/3/label";
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }

        match self
            .send_request::<families::issue_metadata::PageBeanString, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_metadata_get_priorities",
        description = "Returns a paginated list of priorities. The list can contain all priorities or a subset determined by IDs or project IDs."
    )]
    async fn get_priorities(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_metadata::SearchPrioritiesParams,
        >,
    ) -> String {
        let url = "/rest/api/3/priority/search";
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(ids) = &params.id {
            for id in ids {
                query_params.push(("id", id.clone()));
            }
        }
        if let Some(p_ids) = &params.project_id {
            for p_id in p_ids {
                query_params.push(("projectId", p_id.clone()));
            }
        }
        if let Some(name) = &params.priority_name {
            query_params.push(("priorityName", name.clone()));
        }
        if let Some(only_default) = params.only_default {
            query_params.push(("onlyDefault", only_default.to_string()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::issue_metadata::PageBeanPriority, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_metadata_get_resolutions",
        description = "Returns a paginated list of resolutions. The list can contain all resolutions or a subset determined by IDs or whether they are defaults."
    )]
    async fn get_resolutions(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_metadata::SearchResolutionsParams,
        >,
    ) -> String {
        let url = "/rest/api/3/resolution/search";
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(ids) = &params.id {
            for id in ids {
                query_params.push(("id", id.clone()));
            }
        }
        if let Some(only_default) = params.only_default {
            query_params.push(("onlyDefault", only_default.to_string()));
        }

        match self
            .send_request::<families::issue_metadata::PageBeanResolution, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Fields
    // =========================================================================

    #[rmcp::tool(
        name = "jira-issue_field_get",
        description = "Returns system and custom issue fields. This operation only returns the fields that the user has permission to view."
    )]
    async fn get_fields(&self) -> String {
        let url = "/rest/api/3/field";
        match self
            .send_request::<Vec<families::issue_field::Field>, ()>(url, Method::Get, None, None)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-issue_custom_field_get_options",
        description = "Returns a paginated list of all custom field option for a context. Options are returned first then cascading options, in the order they display in Jira."
    )]
    async fn get_custom_field_options(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::issue_custom_field::GetCustomFieldOptionsParams,
        >,
    ) -> String {
        let url = format!(
            "/rest/api/3/field/{}/context/{}/option",
            params.field_id, params.context_id
        );
        let mut query_params = Vec::new();
        if let Some(option_id) = params.option_id {
            query_params.push(("optionId", option_id.to_string()));
        }
        if let Some(only_options) = params.only_options {
            query_params.push(("onlyOptions", only_options.to_string()));
        }
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }

        match self
            .send_request::<families::issue_custom_field::PageBeanCustomFieldOption, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Search (JQL)
    // =========================================================================

    #[rmcp::tool(
        name = "jira-search_execute_jql",
        description = "Searches for issues using JQL enhanced search. If you need read-after-write consistency, you can utilize the reconcileIssues parameter to ensure stronger consistency assurances."
    )]
    async fn search_issues(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::issue::SearchIssuesPostParams>,
    ) -> String {
        let url = "/rest/api/3/search/jql";
        match self
            .send_request::<families::issue::SearchResults, _>(
                url,
                Method::Post,
                None,
                Some(&params),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-jql_parse",
        description = "Parses and validates JQL queries. Validation is performed in context of the current user."
    )]
    async fn parse_jql(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::jql::ParseJqlQueryParams>,
    ) -> String {
        let url = "/rest/api/3/jql/parse";
        let mut query_params = Vec::new();
        query_params.push(("validation", "strict".to_string())); // Default to strict validation

        match self
            .send_request::<families::jql::ParsedJqlQueries, _>(
                url,
                Method::Post,
                Some(&query_params),
                Some(&params),
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Projects & Components
    // =========================================================================

    #[rmcp::tool(
        name = "jira-project_get_all",
        description = "Returns a paginated list of projects visible to the user. Projects are returned only where the user has Browse Projects or Administer projects permission."
    )]
    async fn get_projects(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::project::SearchProjectsParams>,
    ) -> String {
        let url = "/rest/api/3/project/search";
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(order_by) = &params.order_by {
            query_params.push(("orderBy", order_by.clone()));
        }
        if let Some(query) = &params.query {
            query_params.push(("query", query.clone()));
        }
        if let Some(type_key) = &params.type_key {
            query_params.push(("typeKey", type_key.clone()));
        }
        if let Some(action) = &params.action {
            query_params.push(("action", action.clone()));
        }
        if let Some(status) = &params.status {
            query_params.push(("status", status.clone()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }
        if let Some(category_id) = params.category_id {
            query_params.push(("categoryId", category_id.to_string()));
        }
        if let Some(ids) = &params.id {
            for id in ids {
                query_params.push(("id", id.to_string()));
            }
        }
        if let Some(keys) = &params.keys {
            for key in keys {
                query_params.push(("keys", key.clone()));
            }
        }

        match self
            .send_request::<families::project::PageBeanProject, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-project_get_versions",
        description = "Returns a paginated list of all versions in a project. Browse Projects project permission is required."
    )]
    async fn get_project_versions(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::project::GetProjectVersionsParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/project/{}/version", params.project_id_or_key);
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(order_by) = &params.order_by {
            query_params.push(("orderBy", order_by.clone()));
        }
        if let Some(query) = &params.query {
            query_params.push(("query", query.clone()));
        }
        if let Some(status) = &params.status {
            query_params.push(("status", status.clone()));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::project::PageBeanVersion, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-project_create_version",
        description = "Creates a project version. Administer Jira global permission or Administer Projects project permission is required."
    )]
    async fn create_version(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::project::CreateVersionParams>,
    ) -> String {
        let url = "/rest/api/3/version";
        match self
            .send_request::<families::project::Version, _>(url, Method::Post, None, Some(&params))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-project_get_components",
        description = "Returns all components in a project. Browse Projects project permission is required."
    )]
    async fn get_project_components(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::project::GetProjectComponentsParams,
        >,
    ) -> String {
        let url = format!("/rest/api/3/project/{}/component", params.project_id_or_key);
        // We revert to using PageBeanComponentWithIssueCount because the API returned a map, not a sequence.
        match self
            .send_request::<families::project::PageBeanComponentWithIssueCount, serde_json::Value>(
                &url,
                Method::Get,
                None,
                None,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-project_create_component",
        description = "Creates a component. Administer projects project permission or Administer Jira global permission is required."
    )]
    async fn create_component(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::project::CreateComponentParams>,
    ) -> String {
        let url = "/rest/api/3/component";
        match self
            .send_request::<families::project::Component, _>(url, Method::Post, None, Some(&params))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-project_get_roles",
        description = "Returns a list of roles. Permissions required: Browse users and groups global permission."
    )]
    async fn get_project_roles(&self) -> String {
        let url = "/rest/api/3/role";
        match self
            .send_request::<Vec<families::project::ProjectRole>, ()>(url, Method::Get, None, None)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Users
    // =========================================================================

    #[rmcp::tool(
        name = "jira-user_search",
        description = "Returns a list of users that match the search string. Browse users and groups global permission is required."
    )]
    async fn search_users(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::user::SearchUsersParams>,
    ) -> String {
        let url = "/rest/api/3/user/search";
        let mut query_params = Vec::new();
        query_params.push(("query", params.query));
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(active) = params.include_active {
            query_params.push(("includeActive", active.to_string()));
        }
        if let Some(inactive) = params.include_inactive {
            query_params.push(("includeInactive", inactive.to_string()));
        }

        match self
            .send_request::<Vec<families::user::User>, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-user_get_myself",
        description = "Returns details for the current user. Permission to access Jira is required."
    )]
    async fn get_myself(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::user::GetMyselfParams>,
    ) -> String {
        let url = "/rest/api/3/myself";
        let mut query_params = Vec::new();
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::user::User, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // BEGIN: Agile (Boards & Sprints)
    // =========================================================================

    #[rmcp::tool(
        name = "jira-agile_get_boards",
        description = "Returns all boards. This operation may be filtered on the board name, type and project key or ID."
    )]
    async fn get_boards(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::GetBoardsParams>,
    ) -> String {
        let url = "/rest/agile/1.0/board";
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(type_) = &params.r#type {
            query_params.push(("type", type_.clone()));
        }
        if let Some(name) = &params.name {
            query_params.push(("name", name.clone()));
        }
        if let Some(project) = &params.project_key_or_id {
            query_params.push(("projectKeyOrId", project.clone()));
        }

        match self
            .send_request::<families::agile::PageBeanBoard, _>(
                url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_get_board",
        description = "Returns the board for the given board ID. This board will only be returned if the user has permission to view it."
    )]
    async fn get_board(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::GetBoardParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/board/{}", params.board_id);
        match self
            .send_request::<families::agile::Board, _>(&url, Method::Get, None, None::<&()>)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_get_board_sprints",
        description = "Returns all sprints from a board, for a given board ID. This only includes sprints that the user has permission to view."
    )]
    async fn get_board_sprints(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::GetBoardSprintsParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/board/{}/sprint", params.board_id);
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(state) = &params.state {
            query_params.push(("state", state.clone()));
        }

        match self
            .send_request::<families::agile::PageBeanSprint, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_get_board_issues",
        description = "Returns all issues from a board, for a given board ID. This only includes issues that the user has permission to view."
    )]
    async fn get_board_issues(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::GetBoardIssuesParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/board/{}/issue", params.board_id);
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(jql) = &params.jql {
            query_params.push(("jql", jql.clone()));
        }
        if let Some(validate) = params.validate_query {
            query_params.push(("validateQuery", validate.to_string()));
        }
        if let Some(fields) = &params.fields {
            query_params.push(("fields", fields.join(",")));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::agile::PageBeanIssue, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_get_board_backlog",
        description = "Returns all issues from the backlog of a board, for a given board ID. This only includes issues that the user has permission to view."
    )]
    async fn get_board_backlog(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::GetBoardBacklogParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/board/{}/backlog", params.board_id);
        let mut query_params = Vec::new();
        if let Some(start_at) = params.start_at {
            query_params.push(("startAt", start_at.to_string()));
        }
        if let Some(max_results) = params.max_results {
            query_params.push(("maxResults", max_results.to_string()));
        }
        if let Some(jql) = &params.jql {
            query_params.push(("jql", jql.clone()));
        }
        if let Some(validate) = params.validate_query {
            query_params.push(("validateQuery", validate.to_string()));
        }
        if let Some(fields) = &params.fields {
            query_params.push(("fields", fields.join(",")));
        }
        if let Some(expand) = &params.expand {
            query_params.push(("expand", expand.clone()));
        }

        match self
            .send_request::<families::agile::PageBeanIssue, _>(
                &url,
                Method::Get,
                Some(&query_params),
                None::<&()>,
            )
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_create_sprint",
        description = "Creates a future sprint. Sprints that are in a closed state cannot be created."
    )]
    async fn create_sprint(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::CreateSprintParams>,
    ) -> String {
        let url = "/rest/agile/1.0/sprint";
        match self
            .send_request::<families::agile::Sprint, _>(url, Method::Post, None, Some(&params))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_get_sprint",
        description = "Returns the sprint for a given sprint ID. The sprint will only be returned if the user has permission to view it."
    )]
    async fn get_sprint(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::GetSprintParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/sprint/{}", params.sprint_id);
        match self
            .send_request::<families::agile::Sprint, _>(&url, Method::Get, None, None::<&()>)
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_update_sprint",
        description = "Performs a partial update of a sprint. A sprint can be started by updating the state to 'active', or closed by updating the state to 'closed'."
    )]
    async fn update_sprint(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::UpdateSprintParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/sprint/{}", params.sprint_id);

        let mut name = params.name.clone();
        let mut state = params.state.clone();
        let mut start_date = params.start_date.clone();
        let mut end_date = params.end_date.clone();
        let mut goal = params.goal.clone();

        // If any required field for validation (like name or state) is missing, fetch current sprint data
        if name.is_none() || state.is_none() || start_date.is_none() || end_date.is_none() {
            if let Ok(sprint) = self
                .send_request::<families::agile::Sprint, _>(&url, Method::Get, None, None::<&()>)
                .await
            {
                if name.is_none() {
                    name = Some(sprint.name);
                }
                if state.is_none() {
                    state = Some(sprint.state);
                }
                if start_date.is_none() {
                    start_date = sprint.start_date;
                }
                if end_date.is_none() {
                    end_date = sprint.end_date;
                }
                // Goal is optional in update, but good to preserve if we fetched it, though params.goal takes precedence if set.
                if goal.is_none() {
                    goal = sprint.goal;
                }
            }
        }

        // We need to exclude sprint_id from the body
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start_date: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            end_date: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            state: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            goal: Option<String>,
        }
        let body = Body {
            name,
            start_date,
            end_date,
            state,
            goal,
        };

        match self
            .send_request::<families::agile::Sprint, _>(&url, Method::Put, None, Some(&body))
            .await
        {
            Ok(res) => serde_json::to_string(&res).unwrap_or_default(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_delete_sprint",
        description = "Deletes a sprint. Once a sprint is deleted, all open issues in the sprint are moved to the backlog."
    )]
    async fn delete_sprint(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::DeleteSprintParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/sprint/{}", params.sprint_id);
        match self
            .send_request::<serde_json::Value, _>(&url, Method::Delete, None, None::<&()>)
            .await
        {
            Ok(_) => "Sprint deleted successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_move_issues_to_sprint",
        description = "Moves issues to a sprint, for a given sprint ID. Issues can only be moved to open or active sprints. The maximum number of issues that can be moved in one operation is 50."
    )]
    async fn move_issues_to_sprint(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<families::agile::MoveIssuesToSprintParams>,
    ) -> String {
        let url = format!("/rest/agile/1.0/sprint/{}/issue", params.sprint_id);
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            issues: &'a Vec<String>,
        }
        let body = Body {
            issues: &params.issues,
        };

        match self
            .send_request::<serde_json::Value, _>(&url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => "Issues moved to sprint successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    #[rmcp::tool(
        name = "jira-agile_move_issues_to_backlog",
        description = "Moves issues to the backlog. This operation is equivalent to removing issues from a sprint."
    )]
    async fn move_issues_to_backlog(
        &self,
        wrapper::Parameters(params): wrapper::Parameters<
            families::agile::MoveIssuesToBacklogParams,
        >,
    ) -> String {
        let url = "/rest/agile/1.0/backlog/issue";
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            issues: &'a Vec<String>,
        }
        let body = Body {
            issues: &params.issues,
        };

        match self
            .send_request::<serde_json::Value, _>(url, Method::Post, None, Some(&body))
            .await
        {
            Ok(_) => "Issues moved to backlog successfully".to_string(),
            Err(e) => e.to_string(),
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

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
        let full_url = format!("https://{}.atlassian.net{}", self.workspace, url);
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
            return Err(format!("Error {}: {}", status, text).into());
        }

        let res_text = resp.text().await?;
        if res_text.is_empty() || res_text == "null" {
            return serde_json::from_str("null").map_err(|e| e.into());
        }

        match serde_json::from_str::<T>(&res_text) {
            Ok(v) => Ok(v),
            Err(e) => {
                eprintln!("Failed to parse response from {}: {}", url, res_text);
                Err(e.into())
            }
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
