//! Diesel model structs for the `workflows`, `workflow_nodes`, `workflow_edges`,
//! `triggers`, `trigger_conditions`, `trigger_actions`, `lambdas`, `lambda_env_vars`,
//! `webhooks`, and `webhook_subscriptions` tables.
//!
//! Each table has three structs:
//! - Read model (Queryable/Selectable) for SELECT queries
//! - Insert model (Insertable) for INSERT queries
//! - Update model (AsChangeset) for partial UPDATE queries

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::models::enums::WorkflowNodeType;
use crate::iiz::schema::iiz::{
    lambda_env_vars, lambdas, trigger_actions, trigger_conditions, triggers, webhook_subscriptions,
    webhooks, workflow_edges, workflow_nodes, workflows,
};

// ---------------------------------------------------------------------------
// workflows
// ---------------------------------------------------------------------------

/// Read model for the `iiz.workflows` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = workflows)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Workflow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub canvas_json: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new workflow.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = workflows)]
pub struct NewWorkflow {
    pub account_id: Uuid,
    pub name: String,
    pub canvas_json: Option<serde_json::Value>,
    pub status: String,
}

/// Update model for partial workflow updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = workflows)]
pub struct UpdateWorkflow {
    pub name: Option<String>,
    pub canvas_json: Option<Option<serde_json::Value>>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// workflow_nodes
// ---------------------------------------------------------------------------

/// Read model for the `iiz.workflow_nodes` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = workflow_nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkflowNode {
    pub id: Uuid,
    pub account_id: Uuid,
    pub workflow_id: Uuid,
    pub node_type: WorkflowNodeType,
    pub event_type: Option<String>,
    pub action_type: Option<String>,
    pub condition_type: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub label: Option<String>,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new workflow node.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = workflow_nodes)]
pub struct NewWorkflowNode {
    pub account_id: Uuid,
    pub workflow_id: Uuid,
    pub node_type: WorkflowNodeType,
    pub event_type: Option<String>,
    pub action_type: Option<String>,
    pub condition_type: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub label: Option<String>,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
}

/// Update model for partial workflow node updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = workflow_nodes)]
pub struct UpdateWorkflowNode {
    pub node_type: Option<WorkflowNodeType>,
    pub event_type: Option<Option<String>>,
    pub action_type: Option<Option<String>>,
    pub condition_type: Option<Option<String>>,
    pub config_json: Option<Option<serde_json::Value>>,
    pub label: Option<Option<String>>,
    pub position_x: Option<Option<f32>>,
    pub position_y: Option<Option<f32>>,
}

// ---------------------------------------------------------------------------
// workflow_edges
// ---------------------------------------------------------------------------

/// Read model for the `iiz.workflow_edges` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = workflow_edges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkflowEdge {
    pub id: Uuid,
    pub account_id: Uuid,
    pub workflow_id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub label: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new workflow edge.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = workflow_edges)]
pub struct NewWorkflowEdge {
    pub account_id: Uuid,
    pub workflow_id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub label: Option<String>,
    pub sort_order: i32,
}

/// Update model for partial workflow edge updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = workflow_edges)]
pub struct UpdateWorkflowEdge {
    pub from_node_id: Option<Uuid>,
    pub to_node_id: Option<Uuid>,
    pub label: Option<Option<String>>,
    pub sort_order: Option<i32>,
}

// ---------------------------------------------------------------------------
// triggers
// ---------------------------------------------------------------------------

/// Read model for the `iiz.triggers` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = triggers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Trigger {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub trigger_event: String,
    pub run_on: Option<String>,
    pub runs_7d: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new trigger.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `runs_7d` is a system-maintained counter.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = triggers)]
pub struct NewTrigger {
    pub account_id: Uuid,
    pub name: String,
    pub trigger_event: String,
    pub run_on: Option<String>,
    pub status: String,
}

/// Update model for partial trigger updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `runs_7d` is a system-maintained counter and is excluded.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = triggers)]
pub struct UpdateTrigger {
    pub name: Option<String>,
    pub trigger_event: Option<String>,
    pub run_on: Option<Option<String>>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// trigger_conditions
// ---------------------------------------------------------------------------

/// Read model for the `iiz.trigger_conditions` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = trigger_conditions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TriggerCondition {
    pub id: Uuid,
    pub account_id: Uuid,
    pub trigger_id: Uuid,
    pub sort_order: i32,
    pub field: String,
    pub operator: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new trigger condition.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = trigger_conditions)]
pub struct NewTriggerCondition {
    pub account_id: Uuid,
    pub trigger_id: Uuid,
    pub sort_order: i32,
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Update model for partial trigger condition updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = trigger_conditions)]
pub struct UpdateTriggerCondition {
    pub sort_order: Option<i32>,
    pub field: Option<String>,
    pub operator: Option<String>,
    pub value: Option<String>,
}

// ---------------------------------------------------------------------------
// trigger_actions
// ---------------------------------------------------------------------------

/// Read model for the `iiz.trigger_actions` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = trigger_actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TriggerAction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub trigger_id: Uuid,
    pub sort_order: i32,
    pub action_type: String,
    pub action_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new trigger action.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = trigger_actions)]
pub struct NewTriggerAction {
    pub account_id: Uuid,
    pub trigger_id: Uuid,
    pub sort_order: i32,
    pub action_type: String,
    pub action_config: Option<serde_json::Value>,
}

/// Update model for partial trigger action updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = trigger_actions)]
pub struct UpdateTriggerAction {
    pub sort_order: Option<i32>,
    pub action_type: Option<String>,
    pub action_config: Option<Option<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// lambdas
// ---------------------------------------------------------------------------

/// Read model for the `iiz.lambdas` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = lambdas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Lambda {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub runtime: String,
    pub code: String,
    pub handler: String,
    pub timeout_ms: i32,
    pub memory_mb: i32,
    pub last_invoked_at: Option<DateTime<Utc>>,
    pub invocation_count: i32,
    pub error_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new lambda.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `last_invoked_at`, `invocation_count`, and `error_count` are system-maintained.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = lambdas)]
pub struct NewLambda {
    pub account_id: Uuid,
    pub name: String,
    pub runtime: String,
    pub code: String,
    pub handler: String,
    pub timeout_ms: i32,
    pub memory_mb: i32,
}

/// Update model for partial lambda updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `last_invoked_at`, `invocation_count`, and `error_count` are system-maintained.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = lambdas)]
pub struct UpdateLambda {
    pub name: Option<String>,
    pub runtime: Option<String>,
    pub code: Option<String>,
    pub handler: Option<String>,
    pub timeout_ms: Option<i32>,
    pub memory_mb: Option<i32>,
}

// ---------------------------------------------------------------------------
// lambda_env_vars
// ---------------------------------------------------------------------------

/// Read model for the `iiz.lambda_env_vars` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = lambda_env_vars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LambdaEnvVar {
    pub id: Uuid,
    pub account_id: Uuid,
    pub lambda_id: Uuid,
    pub key: String,
    pub value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new lambda environment variable.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = lambda_env_vars)]
pub struct NewLambdaEnvVar {
    pub account_id: Uuid,
    pub lambda_id: Uuid,
    pub key: String,
    pub value: Option<String>,
}

/// Update model for partial lambda environment variable updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = lambda_env_vars)]
pub struct UpdateLambdaEnvVar {
    pub key: Option<String>,
    pub value: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// webhooks
// ---------------------------------------------------------------------------

/// Read model for the `iiz.webhooks` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = webhooks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Webhook {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub trigger_event: Option<String>,
    pub callback_url: String,
    pub method: String,
    pub body_type: String,
    pub headers: Option<serde_json::Value>,
    pub secret: Option<String>,
    pub retry_count: i32,
    pub retry_delay_secs: i32,
    pub status: String,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new webhook.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
/// `last_triggered_at` is system-maintained.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = webhooks)]
pub struct NewWebhook {
    pub account_id: Uuid,
    pub name: String,
    pub trigger_event: Option<String>,
    pub callback_url: String,
    pub method: String,
    pub body_type: String,
    pub headers: Option<serde_json::Value>,
    pub secret: Option<String>,
    pub retry_count: i32,
    pub retry_delay_secs: i32,
    pub status: String,
}

/// Update model for partial webhook updates.
/// Only non-`None` fields are included in the UPDATE statement.
/// `last_triggered_at` is system-maintained.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = webhooks)]
pub struct UpdateWebhook {
    pub name: Option<String>,
    pub trigger_event: Option<Option<String>>,
    pub callback_url: Option<String>,
    pub method: Option<String>,
    pub body_type: Option<String>,
    pub headers: Option<Option<serde_json::Value>>,
    pub secret: Option<Option<String>>,
    pub retry_count: Option<i32>,
    pub retry_delay_secs: Option<i32>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// webhook_subscriptions
// ---------------------------------------------------------------------------

/// Read model for the `iiz.webhook_subscriptions` table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = webhook_subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WebhookSubscription {
    pub id: Uuid,
    pub account_id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert model for creating a new webhook subscription.
/// `id`, `created_at`, `updated_at`, and `deleted_at` are set by database defaults.
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = webhook_subscriptions)]
pub struct NewWebhookSubscription {
    pub account_id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
}

/// Update model for partial webhook subscription updates.
/// Only non-`None` fields are included in the UPDATE statement.
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = webhook_subscriptions)]
pub struct UpdateWebhookSubscription {
    pub event_type: Option<String>,
}
