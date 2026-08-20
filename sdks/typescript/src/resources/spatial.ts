import { BaseResource } from "./base.js";
import type {
  PalaceNavigateOptions,
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
