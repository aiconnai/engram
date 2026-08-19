import { BaseResource } from "./base.js";
import type {
  CaptureScreenshotOptions,
  DescribeImageOptions,
  IngestMediaOptions,
  ListMediaOptions,
  ProcessVideoOptions,
  SearchByImageOptions,
  SyncMediaOptions,
} from "../types.js";

/**
 * Multimodal vision, audio, video, screenshot, and media asset operations (RFC 0009).
 */
export class MultimodalResource extends BaseResource {
  /**
   * Describe an image file using the configured vision provider.
   */
  async describeImage(
    imagePath: string,
    options?: DescribeImageOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_describe_image", {
      image_path: imagePath,
      ...(options?.prompt && { prompt: options.prompt }),
      ...(options?.maxTokens && { max_tokens: options.maxTokens }),
    });
  }

  /**
   * Transcribe an audio file using the configured audio transcriber.
   */
  async transcribeAudio(audioPath: string): Promise<unknown> {
    return this.caller.mcpCall("memory_transcribe_audio", {
      audio_path: audioPath,
    });
  }

  /**
   * Capture a desktop or window screenshot.
   */
  async captureScreenshot(
    options?: CaptureScreenshotOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_capture_screenshot", {
      ...(options?.displayIndex !== undefined && {
        display_index: options.displayIndex,
      }),
      ...(options?.delaySeconds !== undefined && {
        delay_seconds: options.delaySeconds,
      }),
    });
  }

  /**
   * Process and extract key frames from a video file.
   */
  async processVideo(
    videoPath: string,
    options?: ProcessVideoOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_process_video", {
      video_path: videoPath,
      ...(options?.extractFrames !== undefined && {
        extract_frames: options.extractFrames,
      }),
      ...(options?.maxFrames !== undefined && {
        max_frames: options.maxFrames,
      }),
    });
  }

  /**
   * List indexed media assets.
   */
  async listMedia(options?: ListMediaOptions): Promise<unknown> {
    return this.caller.mcpCall("memory_list_media", {
      ...(options?.mediaType && { media_type: options.mediaType }),
      ...(options?.limit !== undefined && { limit: options.limit }),
    });
  }

  /**
   * Search memories by image semantic similarity.
   */
  async searchByImage(
    imagePath: string,
    options?: SearchByImageOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_search_by_image", {
      image_path: imagePath,
      ...(options?.limit !== undefined && { limit: options.limit }),
      ...(options?.minScore !== undefined && { min_score: options.minScore }),
      ...(options?.workspace && { workspace: options.workspace }),
      ...(options?.strategy && { strategy: options.strategy }),
    });
  }

  /**
   * Ingest and index a media asset (image, diagram, audio, video) into a durable memory.
   */
  async ingestMedia(options: IngestMediaOptions): Promise<unknown> {
    return this.caller.mcpCall("memory_ingest_media", {
      media_path: options.mediaPath,
      ...(options.mediaType && { media_type: options.mediaType }),
      ...(options.content && { content: options.content }),
      ...(options.workspace && { workspace: options.workspace }),
      ...(options.tags && { tags: options.tags }),
      ...(options.importance !== undefined && {
        importance: options.importance,
      }),
    });
  }

  /**
   * Sync local media assets to S3/R2 cloud storage.
   */
  async syncMedia(options?: SyncMediaOptions): Promise<unknown> {
    return this.caller.mcpCall("memory_sync_media", {
      ...(options?.dryRun !== undefined && { dry_run: options.dryRun }),
    });
  }
}
