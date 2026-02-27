//! CRUD handlers for `iiz.knowledge_banks` and its `knowledge_bank_documents` sub-resource.
//!
//! The top-level knowledge banks CRUD uses the `crud_handlers!` macro.
//! The document sub-resource requires manual handlers because:
//! - Documents are scoped under a parent bank via `/knowledge-banks/{bank_id}/documents`
//! - Path extractors need a parent_id + document_id tuple for single-resource routes
//!
//! Note: `knowledge_bank_embeddings` is intentionally NOT exposed -- embeddings are
//! created by background processing, not through the CRUD API.
//!
//! All handlers use RLS-scoped connections for account_id filtering.

// --- Standard CRUD for knowledge_banks via macro ---

use crate::iiz::models::ai_tools::{KnowledgeBank, NewKnowledgeBank, UpdateKnowledgeBank};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::knowledge_banks,
    entity: KnowledgeBank,
    new_entity: NewKnowledgeBank,
    update_entity: UpdateKnowledgeBank,
);

// --- Manual handlers for the documents sub-resource ---

use crate::iiz::models::ai_tools::{
    KnowledgeBankDocument, NewKnowledgeBankDocument, UpdateKnowledgeBankDocument,
};

/// List documents belonging to a specific knowledge bank.
///
/// GET `/ai-tools/knowledge-banks/{bank_id}/documents?page=1&per_page=25`
pub async fn list_documents(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<ListResponse<KnowledgeBankDocument>>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;
    let (offset, limit) = params.normalize();

    use crate::iiz::schema::iiz::knowledge_bank_documents::dsl::*;

    let total: i64 = knowledge_bank_documents
        .filter(bank_id.eq(parent_id))
        .count()
        .get_result(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<KnowledgeBankDocument> = knowledge_bank_documents
        .filter(bank_id.eq(parent_id))
        .order(created_at.desc())
        .offset(offset)
        .limit(limit)
        .load(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    let meta = PaginationMeta::new(params.page.max(1), limit, total);
    Ok(axum::Json(ListResponse {
        pagination: meta,
        items,
    }))
}

/// Get a single document by ID within a knowledge bank.
///
/// GET `/ai-tools/knowledge-banks/{bank_id}/documents/{id}`
pub async fn get_document(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::Json<KnowledgeBankDocument>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::knowledge_bank_documents::dsl::*;

    let item: KnowledgeBankDocument = knowledge_bank_documents
        .filter(bank_id.eq(parent_id))
        .filter(id.eq(child_id))
        .first(&mut *conn)
        .await
        .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Create a new document in a knowledge bank.
///
/// POST `/ai-tools/knowledge-banks/{bank_id}/documents`
///
/// The `bank_id` from the URL path is injected into the payload to ensure
/// consistency -- the caller does not need to include it in the JSON body.
pub async fn create_document(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path(parent_id): Path<Uuid>,
    axum::Json(mut payload): axum::Json<NewKnowledgeBankDocument>,
) -> Result<(axum::http::StatusCode, axum::Json<KnowledgeBankDocument>), ApiError> {
    // Override bank_id from URL path for consistency
    payload.bank_id = parent_id;

    let mut conn = get_tenant_conn(&state, &auth).await?;

    let item: KnowledgeBankDocument =
        diesel::insert_into(crate::iiz::schema::iiz::knowledge_bank_documents::table)
            .values(&payload)
            .get_result(&mut *conn)
            .await
            .map_err(ApiError::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(item)))
}

/// Update a document within a knowledge bank.
///
/// PUT `/ai-tools/knowledge-banks/{bank_id}/documents/{id}`
pub async fn update_document(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
    axum::Json(payload): axum::Json<UpdateKnowledgeBankDocument>,
) -> Result<axum::Json<KnowledgeBankDocument>, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::knowledge_bank_documents::dsl::*;

    let item: KnowledgeBankDocument = diesel::update(
        knowledge_bank_documents
            .filter(bank_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(&payload)
    .get_result(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::Json(item))
}

/// Soft-delete a document from a knowledge bank.
///
/// DELETE `/ai-tools/knowledge-banks/{bank_id}/documents/{id}`
pub async fn delete_document(
    axum::extract::State(state): axum::extract::State<IizState>,
    auth: AuthContext,
    Path((parent_id, child_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    let mut conn = get_tenant_conn(&state, &auth).await?;

    use crate::iiz::schema::iiz::knowledge_bank_documents::dsl::*;

    diesel::update(
        knowledge_bank_documents
            .filter(bank_id.eq(parent_id))
            .filter(id.eq(child_id)),
    )
    .set(deleted_at.eq(Some(chrono::Utc::now())))
    .execute(&mut *conn)
    .await
    .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
