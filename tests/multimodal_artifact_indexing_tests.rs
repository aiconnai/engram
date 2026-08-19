//! Multi-Modal Artifact Indexing & Perceptual Hashing Tests.

#[cfg(feature = "multimodal")]
mod multimodal_tests {
    use engram::multimodal::hashing::{
        compute_content_hash, compute_perceptual_hash, format_phash, hamming_distance,
        is_visual_duplicate, parse_phash,
    };
    use engram::storage::Storage;
    use tempfile::tempdir;

    #[test]
    fn test_perceptual_hashing_and_distance() {
        let image_sample_1: Vec<u8> = (0..500).map(|i| ((i * 3) % 255) as u8).collect();
        let mut image_sample_2 = image_sample_1.clone();
        // Slightly modified pixel byte
        image_sample_2[100] = image_sample_2[100].wrapping_add(1);

        let h1 = compute_perceptual_hash(&image_sample_1);
        let h2 = compute_perceptual_hash(&image_sample_2);

        let hex1 = format_phash(h1);
        let hex2 = format_phash(h2);

        assert_eq!(hex1.len(), 16);
        assert_eq!(hex2.len(), 16);
        assert_eq!(parse_phash(&hex1), Some(h1));
        assert_eq!(parse_phash(&hex2), Some(h2));

        let dist = hamming_distance(h1, h2);
        assert!(
            dist <= 1,
            "Distance should be very small for nearly identical images"
        );
        assert!(is_visual_duplicate(h1, h2, 5));
    }

    #[test]
    fn test_content_hash_sha256() {
        let data = b"multimodal-diagram-asset";
        let hash = compute_content_hash(data);
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, compute_content_hash(data));
    }

    #[test]
    fn test_media_asset_storage_insertion() {
        let storage = Storage::open_in_memory().unwrap();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("mock_diagram.png");
        std::fs::write(&file_path, b"mock png payload bytes").unwrap();

        storage
            .with_transaction(|conn| {
                let input = engram::types::CreateMemoryInput {
                    content: "Architecture Diagram: Microservices Flow".to_string(),
                    memory_type: engram::types::MemoryType::Image,
                    workspace: Some("infra".to_string()),
                    media_url: Some(format!("local://{}", file_path.display())),
                    ..Default::default()
                };
                let mem = engram::storage::queries::create_memory(conn, &input)?;

                conn.execute(
                    "INSERT INTO media_assets (
                        memory_id, media_type, file_hash, file_path, file_size, mime_type
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        mem.id,
                        "image",
                        "test_hash_123",
                        file_path.to_str().unwrap(),
                        22i64,
                        "image/png",
                    ],
                )?;

                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM media_assets WHERE memory_id = ?1",
                    [mem.id],
                    |row| row.get(0),
                )?;
                assert_eq!(count, 1);
                Ok(())
            })
            .unwrap();
    }
}
