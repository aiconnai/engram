import { BaseResource } from "./base.js";

/**
 * MCP Resource discovery, reading, and dynamic subscription management.
 */
export class McpResourcesResource extends BaseResource {
  /**
   * List all available resource URI templates from the server.
   */
  async list(): Promise<unknown> {
    return this.caller.rpcCall
      ? this.caller.rpcCall("resources/list")
      : Promise.resolve({});
  }

  /**
   * Read a resource by its URI.
   *
   * @param uri The resource URI (e.g. `engram://stats`, `engram://memory/1`, `engram://workspace/dev/memories`).
   */
  async read(uri: string): Promise<unknown> {
    return this.caller.rpcCall
      ? this.caller.rpcCall("resources/read", { uri })
      : Promise.resolve({});
  }

  /**
   * Subscribe to live updates for a resource URI.
   *
   * @param uri The resource URI to subscribe to.
   */
  async subscribe(uri: string): Promise<unknown> {
    return this.caller.rpcCall
      ? this.caller.rpcCall("resources/subscribe", { uri })
      : Promise.resolve({});
  }

  /**
   * Unsubscribe from updates for a resource URI.
   *
   * @param uri The resource URI to unsubscribe from.
   */
  async unsubscribe(uri: string): Promise<unknown> {
    return this.caller.rpcCall
      ? this.caller.rpcCall("resources/unsubscribe", { uri })
      : Promise.resolve({});
  }
}
