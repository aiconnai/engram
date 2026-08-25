import { BaseResource } from "./base.js";
import type {
  ModelPurpose,
  ModelRouteResolution,
  ModelRouteResolveOptions,
  ModelRoutesListOptions,
  ModelRoutesListResult,
} from "../types.js";

export class ModelRoutingResource extends BaseResource {
  /**
   * Deterministically resolve the active or preferred model route for a given AI capability / purpose (RFC 0011).
   */
  async resolve(
    purposeOrOptions: ModelPurpose | ModelRouteResolveOptions,
    preferredProvider?: string
  ): Promise<ModelRouteResolution> {
    const params: Record<string, unknown> = {};

    if (typeof purposeOrOptions === "string") {
      params.purpose = purposeOrOptions;
      if (preferredProvider) params.preferred_provider = preferredProvider;
    } else {
      params.purpose = purposeOrOptions.purpose;
      if (purposeOrOptions.preferredProvider) {
        params.preferred_provider = purposeOrOptions.preferredProvider;
      }
    }

    return this.caller.mcpCall(
      "model_route_resolve",
      params
    ) as Promise<ModelRouteResolution>;
  }

  /**
   * List all declared model routes and their capabilities, cost classes, and fallback policies (RFC 0011).
   */
  async list(options?: ModelRoutesListOptions): Promise<ModelRoutesListResult> {
    const params: Record<string, unknown> = {};
    if (options?.purpose) params.purpose = options.purpose;
    return this.caller.mcpCall(
      "model_routes_list",
      params
    ) as Promise<ModelRoutesListResult>;
  }
}
