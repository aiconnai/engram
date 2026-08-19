import { BaseResource } from "./base.js";
import type {
  ProgressEvent,
  RealtimeEvent,
  RealtimeEventType,
  StreamEventsOptions,
} from "../types.js";

export class EventsResource extends BaseResource {
  /**
   * Parse a raw Server-Sent Events line/chunk into a structured RealtimeEvent.
   */
  parseEvent(eventChunk: string): RealtimeEvent | null {
    const lines = eventChunk.split("\n");
    let eventType: RealtimeEventType = "memory_created";
    let dataStr = "";
    let id: number | undefined;

    for (const line of lines) {
      if (line.startsWith("event:")) {
        eventType = line.slice(6).trim() as RealtimeEventType;
      } else if (line.startsWith("id:")) {
        const parsedId = Number.parseInt(line.slice(3).trim(), 10);
        if (!Number.isNaN(parsedId)) {
          id = parsedId;
        }
      } else if (line.startsWith("data:")) {
        dataStr = line.slice(5).trim();
      }
    }

    if (!dataStr) {
      return null;
    }

    try {
      const parsed = JSON.parse(dataStr) as Record<string, unknown>;
      return {
        seqId: id ?? (parsed.seq_id as number | undefined),
        type: eventType,
        timestamp: (parsed.timestamp as string) ?? new Date().toISOString(),
        memoryId: parsed.memory_id as number | undefined,
        preview: parsed.preview as string | undefined,
        changes: parsed.changes as string[] | undefined,
        data: (parsed.data as Record<string, unknown>) ?? parsed,
      };
    } catch {
      return null;
    }
  }

  /**
   * Consume Server-Sent Events from `GET /v1/events` as an AsyncIterable.
   */
  async *stream(
    baseUrl: string,
    headers: Record<string, string>,
    options?: StreamEventsOptions
  ): AsyncIterable<RealtimeEvent> {
    const params = new URLSearchParams();
    if (options?.eventTypes) {
      const types = Array.isArray(options.eventTypes)
        ? options.eventTypes.join(",")
        : options.eventTypes;
      params.set("event_types", types);
    }
    if (options?.workspace) {
      params.set("workspace", options.workspace);
    }

    const query = params.toString();
    const url = `${baseUrl}/v1/events${query ? `?${query}` : ""}`;

    const reqHeaders: Record<string, string> = {
      ...headers,
      Accept: "text/event-stream",
    };
    if (options?.lastEventId !== undefined) {
      reqHeaders["Last-Event-Id"] = String(options.lastEventId);
    }

    const response = await fetch(url, {
      method: "GET",
      headers: reqHeaders,
      signal: options?.signal,
    });

    if (!response.ok) {
      throw new Error(`SSE stream failed with status ${response.status}`);
    }

    if (!response.body) {
      return;
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const parts = buffer.split("\n\n");
        buffer = parts.pop() ?? "";

        for (const part of parts) {
          const trimmed = part.trim();
          if (!trimmed || trimmed.startsWith(":")) {
            continue; // Skip comments/keepalives
          }
          const event = this.parseEvent(trimmed);
          if (event) {
            yield event;
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }

  /**
   * Watch for progress notifications matching a specific progressToken.
   */
  async watchProgress(
    baseUrl: string,
    headers: Record<string, string>,
    token: string | number,
    onProgress: (event: ProgressEvent) => void,
    signal?: AbortSignal
  ): Promise<() => void> {
    const controller = new AbortController();
    const combinedSignal = signal ?? controller.signal;

    (async () => {
      try {
        const eventStream = this.stream(baseUrl, headers, {
          eventTypes: ["progress"],
          signal: combinedSignal,
        });

        for await (const event of eventStream) {
          if (event.type === "progress" && event.data) {
            const dataToken =
              event.data.progress_token ?? event.data.progressToken;
            if (String(dataToken) === String(token)) {
              onProgress({
                seqId: event.seqId,
                type: "progress",
                timestamp: event.timestamp,
                preview: event.preview,
                data: {
                  progressToken: dataToken as string | number,
                  progress: (event.data.progress as number) ?? 0,
                  total: event.data.total as number | undefined,
                  message: (event.data.message as string) ?? event.preview,
                  workspace: event.data.workspace as string | undefined,
                },
              });
            }
          }
        }
      } catch {
        // Stream aborted or closed
      }
    })();

    return () => controller.abort();
  }
}
