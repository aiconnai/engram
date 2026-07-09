use super::*;

#[test]
fn test_create_image_memory_with_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "A screenshot of the dashboard".to_string(),
                    memory_type: MemoryType::Image,
                    media_url: Some("local:///tmp/dashboard.png".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create image memory");
    assert_eq!(memory.memory_type, MemoryType::Image);
    assert_eq!(
        memory.media_url.as_deref(),
        Some("local:///tmp/dashboard.png")
    );
}

#[test]
fn test_create_audio_memory_with_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Meeting recording transcript".to_string(),
                    memory_type: MemoryType::Audio,
                    media_url: Some("local:///tmp/meeting.mp3".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create audio memory");
    assert_eq!(memory.memory_type, MemoryType::Audio);
    assert_eq!(
        memory.media_url.as_deref(),
        Some("local:///tmp/meeting.mp3")
    );
}

#[test]
fn test_create_video_memory_with_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Keynote presentation video".to_string(),
                    memory_type: MemoryType::Video,
                    media_url: Some("https://cdn.example.com/keynote.mp4".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create video memory");
    assert_eq!(memory.memory_type, MemoryType::Video);
    assert_eq!(
        memory.media_url.as_deref(),
        Some("https://cdn.example.com/keynote.mp4")
    );
}

#[test]
fn test_get_memory_returns_media_url() {
    let storage = open_test_storage();
    let created = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Image with URL".to_string(),
                    memory_type: MemoryType::Image,
                    media_url: Some("local:///tmp/image.png".to_string()),
                    ..Default::default()
                },
            )
        })
        .expect("create");
    let fetched = storage
        .with_connection(|conn| get_memory(conn, created.id))
        .expect("get");
    assert_eq!(fetched.media_url, created.media_url);
}

#[test]
fn test_create_image_memory_without_media_url() {
    let storage = open_test_storage();
    let memory = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Image described in text only".to_string(),
                    memory_type: MemoryType::Image,
                    media_url: None,
                    ..Default::default()
                },
            )
        })
        .expect("create image memory without media_url");
    assert_eq!(memory.memory_type, MemoryType::Image);
    assert!(memory.media_url.is_none());
}

#[test]
fn test_update_memory_sets_media_url() {
    let storage = open_test_storage();
    let created = storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Image memory".to_string(),
                    memory_type: MemoryType::Image,
                    ..Default::default()
                },
            )
        })
        .expect("create");
    let updated = storage
        .with_transaction(|conn| {
            update_memory(
                conn,
                created.id,
                &UpdateMemoryInput {
                    media_url: Some(Some("local:///tmp/updated.png".to_string())),
                    content: None,
                    memory_type: None,
                    tags: None,
                    metadata: None,
                    importance: None,
                    scope: None,
                    ttl_seconds: None,
                    event_time: None,
                    trigger_pattern: None,
                },
            )
        })
        .expect("update");
    assert_eq!(
        updated.media_url.as_deref(),
        Some("local:///tmp/updated.png")
    );
}

#[test]
fn test_memory_type_is_multimodal() {
    assert!(MemoryType::Image.is_multimodal());
    assert!(MemoryType::Audio.is_multimodal());
    assert!(MemoryType::Video.is_multimodal());
    assert!(!MemoryType::Note.is_multimodal());
    assert!(!MemoryType::Episodic.is_multimodal());
}
