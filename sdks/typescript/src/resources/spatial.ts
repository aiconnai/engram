import { BaseResource } from "./base.js";
import type {
  PalaceNavigateOptions,
  PalaceVisualizeOptions,
  PalaceVisualizeResult,
  RoomSearchOptions,
} from "../types.js";

export class SpatialResource extends BaseResource {
  /**
   * Navigate the Memory Palace: discover active wings, rooms, and drawer counts.
   */
  async palaceNavigate(options?: PalaceNavigateOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      workspace: options?.workspace ?? "default",
    };
    if (options?.wing) params.wing = options.wing;
    return this.caller.mcpCall("palace_navigate", params);
  }

  /**
   * Generate or export a visual representation of the memory palace in HTML, ASCII, SVG, Mermaid, or JSON format.
   */
  async visualize(options?: PalaceVisualizeOptions): Promise<PalaceVisualizeResult> {
    const params: Record<string, unknown> = {
      workspace: options?.workspace ?? "default",
      format: options?.format ?? "html",
    };
    if (options?.wing) params.wing = options.wing;
    if (options?.outputPath) params.output_path = options.outputPath;
    return this.caller.mcpCall("palace_visualize", params) as Promise<PalaceVisualizeResult>;
  }

  /**
   * Search memories scoped within a specific spatial room and wing using hybrid retrieval.
   */
  async roomSearch(options: RoomSearchOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      wing: options.wing,
      query: options.query,
      limit: options.limit ?? 10,
    };
    if (options.room) params.room = options.room;
    if (options.workspace) params.workspace = options.workspace;
    return this.caller.mcpCall("room_search", params);
  }

  /**
   * Open a specific memory drawer by ID to read its full verbatim content and metadata.
   */
  async drawerOpen(id: number): Promise<unknown> {
    return this.caller.mcpCall("drawer_open", { id });
  }
}
