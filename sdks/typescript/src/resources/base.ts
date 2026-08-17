export interface McpCaller {
  mcpCall(
    method: string,
    params?: Record<string, unknown>
  ): Promise<unknown>;
}

export abstract class BaseResource {
  constructor(protected readonly caller: McpCaller) {}
}
