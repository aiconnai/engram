use std::collections::HashMap;

use super::core::TursoBackend;
use crate::error::Result;
use crate::storage::backend::{
    BatchCreateResult, BatchDeleteResult, HealthStatus, StorageBackend, StorageStats,
};
use crate::types::{
    CreateMemoryInput, CrossReference, EdgeType, ListOptions, Memory, MemoryId, SearchOptions,
    SearchResult, UpdateMemoryInput,
};

use super::impls_crud;
use super::impls_maintenance;
use super::impls_query;
use super::impls_relations;

#[allow(deprecated)]
impl StorageBackend for TursoBackend {
    fn create_memory(&self, input: CreateMemoryInput) -> Result<Memory> {
        impls_crud::create_memory(self, input)
    }

    fn create_memories_batch(&self, inputs: Vec<CreateMemoryInput>) -> Result<BatchCreateResult> {
        impls_crud::create_memories_batch(self, inputs)
    }

    fn get_memory(&self, id: MemoryId) -> Result<Option<Memory>> {
        impls_crud::get_memory(self, id)
    }

    fn delete_memories_batch(&self, ids: Vec<MemoryId>) -> Result<BatchDeleteResult> {
        impls_crud::delete_memories_batch(self, ids)
    }

    fn update_memory(&self, id: MemoryId, input: UpdateMemoryInput) -> Result<Memory> {
        impls_crud::update_memory(self, id, input)
    }

    fn delete_memory(&self, id: MemoryId) -> Result<()> {
        impls_crud::delete_memory(self, id)
    }

    fn list_memories(&self, options: ListOptions) -> Result<Vec<Memory>> {
        impls_query::list_memories(self, options)
    }

    fn count_memories(&self, options: ListOptions) -> Result<i64> {
        impls_query::count_memories(self, options)
    }

    fn search_memories(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>> {
        impls_query::search_memories(self, query, options)
    }

    fn create_crossref(
        &self,
        from_id: MemoryId,
        to_id: MemoryId,
        edge_type: EdgeType,
        score: f32,
    ) -> Result<CrossReference> {
        impls_relations::create_crossref(self, from_id, to_id, edge_type, score)
    }

    fn get_crossrefs(&self, memory_id: MemoryId) -> Result<Vec<CrossReference>> {
        impls_relations::get_crossrefs(self, memory_id)
    }

    fn delete_crossref(&self, from_id: MemoryId, to_id: MemoryId) -> Result<()> {
        impls_relations::delete_crossref(self, from_id, to_id)
    }

    fn list_tags(&self) -> Result<Vec<(String, i64)>> {
        impls_relations::list_tags(self)
    }

    fn get_memories_by_tag(&self, tag: &str, limit: Option<usize>) -> Result<Vec<Memory>> {
        impls_relations::get_memories_by_tag(self, tag, limit)
    }

    fn list_workspaces(&self) -> Result<Vec<(String, i64)>> {
        impls_relations::list_workspaces(self)
    }

    fn get_workspace_stats(&self, workspace: &str) -> Result<HashMap<String, i64>> {
        impls_relations::get_workspace_stats(self, workspace)
    }

    fn move_to_workspace(&self, ids: Vec<MemoryId>, workspace: &str) -> Result<usize> {
        impls_relations::move_to_workspace(self, ids, workspace)
    }

    fn get_stats(&self) -> Result<StorageStats> {
        impls_maintenance::get_stats(self)
    }

    fn health_check(&self) -> Result<HealthStatus> {
        impls_maintenance::health_check(self)
    }

    fn optimize(&self) -> Result<()> {
        impls_maintenance::optimize(self)
    }

    fn backend_name(&self) -> &'static str {
        "turso"
    }

    fn schema_version(&self) -> Result<i32> {
        impls_maintenance::schema_version(self)
    }
}

// TransactionalBackend and CloudSyncBackend impls for TursoBackend live in
// impls_maintenance.rs (moved during the ENG storage file-size split).

// Turso tests moved to tests/turso_backend_tests.rs (integration test)
// to avoid libsql/rusqlite SQLite initialization conflict under --all-features.
// Run with: cargo test --test turso_backend_tests --features turso
