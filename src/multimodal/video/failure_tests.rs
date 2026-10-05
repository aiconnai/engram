//! Failure, cancellation and ownership tests for the video pipeline (C6).
//!
//! `ffprobe`/`ffmpeg` are replaced by shell scripts injected through
//! `VideoProcessor`'s binary overrides, and the vision provider is an offline
//! stub, so nothing here touches the network or a real encoder. Each failure
//! case asserts the three post-conditions of the brief: a bounded, typed
//! error; no surviving child or grandchild process; no leftover
//! `engram_frames_*` directory.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::*;
use crate::multimodal::process::test_support::{pid_alive, read_pid, write_script};
use crate::multimodal::vision::{ImageDescription, VisionOptions};

const PROBE_JSON: &str = r#"{"streams":[{"codec_type":"video","width":64,"height":48,"codec_name":"h264","duration":"10.0"}],"format":{"duration":"10.0"}}"#;

struct Rig {
    dir: tempfile::TempDir,
    video: PathBuf,
    frames_base: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let video = dir.path().join("clip.mp4");
        std::fs::write(&video, b"not really a video").unwrap();
        let frames_base = dir.path().join("frames-base");
        std::fs::create_dir(&frames_base).unwrap();
        Self {
            dir,
            video,
            frames_base,
        }
    }

    fn script(&self, name: &str, body: &str) -> String {
        write_script(self.dir.path(), name, body)
            .to_string_lossy()
            .into_owned()
    }

    fn ffprobe_ok(&self) -> String {
        self.script("ffprobe", &format!("echo '{PROBE_JSON}'"))
    }

    fn ffmpeg_two_frames(&self) -> String {
        self.script(
            "ffmpeg",
            "for last; do :; done\nd=$(dirname \"$last\")\nprintf x > \"$d/frame_001.png\"\nprintf y > \"$d/frame_002.png\"",
        )
    }

    fn processor(&self, ffprobe: String, ffmpeg: String) -> VideoProcessor {
        VideoProcessor {
            ffprobe_bin: ffprobe,
            ffmpeg_bin: ffmpeg,
            probe_timeout: Duration::from_secs(10),
            extract_timeout: Duration::from_secs(10),
            vision_timeout: Duration::from_secs(10),
            frames_base: Some(self.frames_base.clone()),
        }
    }

    fn leftover_frame_dirs(&self) -> Vec<PathBuf> {
        std::fs::read_dir(&self.frames_base)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("engram_frames_"))
            })
            .collect()
    }
}

struct StubVision {
    /// 1-based frame whose call fails; `None` = never.
    fail_on: Option<usize>,
    hang: bool,
    calls: std::sync::atomic::AtomicUsize,
}

impl StubVision {
    fn ok() -> Self {
        Self {
            fail_on: None,
            hang: false,
            calls: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl VisionProvider for StubVision {
    async fn describe_image(
        &self,
        _input: VisionInput,
        _opts: VisionOptions,
    ) -> Result<ImageDescription> {
        let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        if self.hang {
            std::future::pending::<()>().await;
        }
        if self.fail_on == Some(n) {
            return Err(EngramError::Internal("stub vision outage".to_string()));
        }
        Ok(ImageDescription {
            text: format!("frame {n}"),
            model: "stub".to_string(),
            provider: "stub".to_string(),
        })
    }

    fn provider_name(&self) -> &str {
        "stub"
    }
}

fn assert_gone(pid_file: &Path, what: &str) {
    let pid = read_pid(pid_file);
    assert!(!pid_alive(pid), "{what} (pid {pid}) is still running");
}

#[test]
fn hanging_ffmpeg_times_out_is_reaped_and_leaves_no_directory() {
    let rig = Rig::new();
    let child_pid = rig.dir.path().join("ffmpeg.pid");
    let grand_pid = rig.dir.path().join("sleep.pid");
    let ffmpeg = rig.script(
        "ffmpeg",
        &format!(
            "echo $$ > {}\nsleep 300 &\necho $! > {}\nwait",
            child_pid.display(),
            grand_pid.display()
        ),
    );
    let mut p = rig.processor(rig.ffprobe_ok(), ffmpeg);
    p.extract_timeout = Duration::from_millis(500);

    let started = Instant::now();
    let err = p.extract_keyframes_owned(&rig.video, 3).unwrap_err();

    assert!(err.to_string().contains("timed out"), "{err}");
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_gone(&child_pid, "ffmpeg");
    assert_gone(&grand_pid, "ffmpeg's child");
    assert!(rig.leftover_frame_dirs().is_empty());
}

#[test]
fn hanging_ffprobe_times_out_and_is_reaped() {
    let rig = Rig::new();
    let pid_file = rig.dir.path().join("ffprobe.pid");
    let ffprobe = rig.script(
        "ffprobe",
        &format!("echo $$ > {}\nsleep 300", pid_file.display()),
    );
    let mut p = rig.processor(ffprobe, rig.ffmpeg_two_frames());
    p.probe_timeout = Duration::from_millis(500);

    let err = p.extract_metadata(&rig.video).unwrap_err();

    assert!(err.to_string().contains("timed out"), "{err}");
    assert_gone(&pid_file, "ffprobe");
}

#[test]
fn failing_ffmpeg_reports_stderr_and_removes_directory() {
    let rig = Rig::new();
    let ffmpeg = rig.script("ffmpeg", "echo 'codec exploded' >&2\nexit 3");
    let p = rig.processor(rig.ffprobe_ok(), ffmpeg);

    let err = p.extract_keyframes(&rig.video, 3).unwrap_err();

    assert!(err.to_string().contains("codec exploded"), "{err}");
    assert!(rig.leftover_frame_dirs().is_empty());
}

#[test]
fn ffmpeg_spawn_error_removes_directory() {
    let rig = Rig::new();
    let p = rig.processor(rig.ffprobe_ok(), "/nonexistent/engram-ffmpeg".to_string());

    let err = p.extract_keyframes(&rig.video, 3).unwrap_err();

    assert!(matches!(err, EngramError::Config(_)), "{err}");
    assert!(
        rig.leftover_frame_dirs().is_empty(),
        "spawn failure leaked {:?}",
        rig.leftover_frame_dirs()
    );
}

#[test]
fn ffmpeg_success_without_frames_is_an_error_not_an_empty_success() {
    let rig = Rig::new();
    let ffmpeg = rig.script("ffmpeg", "exit 0");
    let p = rig.processor(rig.ffprobe_ok(), ffmpeg);

    let err = p.extract_keyframes(&rig.video, 3).unwrap_err();

    assert!(err.to_string().contains("no frames"), "{err}");
    assert!(rig.leftover_frame_dirs().is_empty());
}

#[tokio::test]
async fn vision_failure_after_extraction_removes_frames() {
    let rig = Rig::new();
    let p = rig.processor(rig.ffprobe_ok(), rig.ffmpeg_two_frames());
    let vision = StubVision {
        fail_on: Some(2),
        ..StubVision::ok()
    };

    let err = p
        .create_video_memory(&rig.video, &vision)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("frame 2 of 2"), "{err}");
    assert!(err.to_string().contains("stub vision outage"), "{err}");
    assert!(rig.leftover_frame_dirs().is_empty());
}

#[tokio::test]
async fn slow_vision_provider_times_out_and_removes_frames() {
    let rig = Rig::new();
    let mut p = rig.processor(rig.ffprobe_ok(), rig.ffmpeg_two_frames());
    p.vision_timeout = Duration::from_millis(300);
    let vision = StubVision {
        hang: true,
        ..StubVision::ok()
    };

    let started = Instant::now();
    let err = p
        .create_video_memory(&rig.video, &vision)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("timed out"), "{err}");
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(rig.leftover_frame_dirs().is_empty());
}

#[tokio::test]
async fn cancelling_the_pipeline_removes_frames() {
    let rig = Rig::new();
    let mut p = rig.processor(rig.ffprobe_ok(), rig.ffmpeg_two_frames());
    p.vision_timeout = Duration::from_secs(60); // longer than the caller's patience
    let vision = StubVision {
        hang: true,
        ..StubVision::ok()
    };

    let outcome = tokio::time::timeout(
        Duration::from_millis(300),
        p.create_video_memory(&rig.video, &vision),
    )
    .await;

    assert!(outcome.is_err(), "caller timeout must cancel the future");
    assert!(
        rig.leftover_frame_dirs().is_empty(),
        "cancelled pipeline leaked {:?}",
        rig.leftover_frame_dirs()
    );
}

#[tokio::test]
async fn success_transfers_ownership_and_cleans_after_last_consumer() {
    let rig = Rig::new();
    let p = rig.processor(rig.ffprobe_ok(), rig.ffmpeg_two_frames());

    let memory = p
        .create_video_memory(&rig.video, &StubVision::ok())
        .await
        .unwrap();
    assert_eq!(memory.keyframe_descriptions.len(), 2);
    assert_eq!(rig.leftover_frame_dirs().len(), 1);
    assert!(memory.frames_path.iter().all(|f| f.exists()));

    // A second consumer keeps the frames alive after the first is dropped.
    let second = memory.clone();
    drop(memory);
    assert!(
        second.frames_path.iter().all(|f| f.exists()),
        "frames deleted while a consumer still holds them"
    );

    drop(second);
    assert!(
        rig.leftover_frame_dirs().is_empty(),
        "last drop must clean up"
    );
}

#[test]
fn extract_keyframes_hands_the_directory_to_the_caller() {
    let rig = Rig::new();
    let p = rig.processor(rig.ffprobe_ok(), rig.ffmpeg_two_frames());

    let frames = p.extract_keyframes(&rig.video, 3).unwrap();

    assert_eq!(frames.len(), 2);
    assert!(
        frames.iter().all(|f| f.exists()),
        "caller-owned frames deleted"
    );
    assert_eq!(rig.leftover_frame_dirs().len(), 1);
}

#[test]
fn frames_directory_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let rig = Rig::new();
    let p = rig.processor(rig.ffprobe_ok(), rig.ffmpeg_two_frames());
    let (dir, _) = p.extract_keyframes_owned(&rig.video, 3).unwrap();
    let mode = std::fs::metadata(dir.path()).unwrap().permissions().mode();
    assert_eq!(
        mode & 0o077,
        0,
        "frames dir must not be group/world accessible"
    );
}

#[test]
fn hash_file_matches_a_known_digest_across_chunk_boundaries() {
    // 3 full 64 KiB read chunks plus a 17-byte tail: exercises the streaming
    // loop (multiple reads, short final read). Memory use is not asserted.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("chunks.bin");
    let data: Vec<u8> = (0..3 * 64 * 1024 + 17)
        .map(|i| ((i * 7 + 3) % 251) as u8)
        .collect();
    std::fs::write(&file, &data).unwrap();
    assert_eq!(
        hash_file(&file).unwrap(),
        "sha256:c25a5579ff52dc38ec8432a2b51ce89f026cb77ccd038ceb048bffb1b45e7011"
    );
}
