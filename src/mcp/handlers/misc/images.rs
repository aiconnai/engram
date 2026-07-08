//! Image handling: upload and migrate memory images.

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

pub fn memory_upload_image(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::{upload_image, ImageStorageConfig, LocalImageStorage};

    let memory_id = match params.get("memory_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "memory_id is required"}),
    };

    let file_path = match params.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "file_path is required"}),
    };

    let image_index = params
        .get("image_index")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let caption = params.get("caption").and_then(|v| v.as_str());

    let config = ImageStorageConfig::default();
    let image_storage = match LocalImageStorage::new(config.local_dir) {
        Ok(s) => s,
        Err(e) => return json!({"error": format!("Failed to initialize image storage: {}", e)}),
    };

    ctx.storage
        .with_connection(|conn| {
            let image_ref = upload_image(
                conn,
                &image_storage,
                memory_id,
                file_path,
                image_index,
                caption,
            )?;
            Ok(json!({
                "success": true,
                "image": image_ref
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_migrate_images(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::{migrate_images, ImageStorageConfig, LocalImageStorage};

    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let config = ImageStorageConfig::default();
    let image_storage = match LocalImageStorage::new(config.local_dir) {
        Ok(s) => s,
        Err(e) => return json!({"error": format!("Failed to initialize image storage: {}", e)}),
    };

    ctx.storage
        .with_connection(|conn| {
            let result = migrate_images(conn, &image_storage, dry_run)?;
            Ok(json!(result))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
