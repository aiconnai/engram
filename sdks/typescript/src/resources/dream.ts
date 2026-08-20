import { BaseResource } from "./base.js";
import type {
  DreamApplyOptions,
  DreamCandidatesListOptions,
  DreamCreateOptions,
  DreamEvalOptions,
  DreamListOptions,
  DreamReviewOptions,
  DreamRunNowOptions,
} from "../types.js";

/**
 * Dream Phase and candidate review pipeline resources.
 */
export class DreamResource extends BaseResource {
  /**
   * Create and optionally run a reviewable dream snapshot job.
   */
  async create(options?: DreamCreateOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.run !== undefined) params.run = options.run;
    if (options?.jobId !== undefined) params.job_id = options.jobId;
    if (options?.instructions !== undefined) params.instructions = options.instructions;
    if (options?.maxMemories !== undefined) params.max_memories = options.maxMemories;
    if (options?.maxCandidates !== undefined) params.max_candidates = options.maxCandidates;
    if (options?.summaryMinMemories !== undefined) params.summary_min_memories = options.summaryMinMemories;
    return this.caller.mcpCall("dream_create", params);
  }

  /**
   * Inspect a dream job by ID.
   */
  async get(id: string): Promise<unknown> {
    return this.caller.mcpCall("dream_get", { id });
  }

  /**
   * List dream jobs by workspace and status.
   */
  async list(options?: DreamListOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.status !== undefined) params.status = options.status;
    if (options?.limit !== undefined) params.limit = options.limit;
    return this.caller.mcpCall("dream_list", params);
  }

  /**
   * Cancel a pending or running dream job.
   */
  async cancel(id: string): Promise<unknown> {
    return this.caller.mcpCall("dream_cancel", { id });
  }

  /**
   * Archive a terminal dream job.
   */
  async archive(id: string): Promise<unknown> {
    return this.caller.mcpCall("dream_archive", { id });
  }

  /**
   * List dream review candidates with optional filtering.
   */
  async candidatesList(options?: DreamCandidatesListOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.jobId !== undefined) params.job_id = options.jobId;
    if (options?.reviewState !== undefined) params.review_state = options.reviewState;
    if (options?.kind !== undefined) params.kind = options.kind;
    if (options?.proposedAction !== undefined) params.proposed_action = options.proposedAction;
    if (options?.limit !== undefined) params.limit = options.limit;
    return this.caller.mcpCall("dream_candidates_list", params);
  }

  /**
   * Inspect one candidate and its evidence sources.
   */
  async candidateGet(id: string): Promise<unknown> {
    return this.caller.mcpCall("dream_candidate_get", { id });
  }

  /**
   * Review a candidate (accept, edit, reject, archive).
   */
  async candidateReview(
    id: string,
    reviewState: "accepted" | "edited" | "rejected" | "archived",
    options?: DreamReviewOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      id,
      review_state: reviewState,
    };
    if (options?.editedContent !== undefined) params.edited_content = options.editedContent;
    if (options?.notes !== undefined) params.notes = options.notes;
    return this.caller.mcpCall("dream_candidate_review", params);
  }

  /**
   * Apply an accepted or edited candidate to canonical memory.
   * Requires confirm: true.
   */
  async candidateApply(id: string, options?: DreamApplyOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      id,
      confirm: options?.confirm ?? true,
    };
    if (options?.reviewerNotes !== undefined) params.reviewer_notes = options.reviewerNotes;
    return this.caller.mcpCall("dream_candidate_apply", params);
  }

  /**
   * Run deterministic local dream snapshot evaluation fixtures.
   */
  async evalRun(options?: DreamEvalOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.lane !== undefined) params.lane = options.lane;
    return this.caller.mcpCall("dream_eval_run", params);
  }

  /**
   * Trigger an immediate background consolidation pass across all workspaces.
   */
  async runNow(options?: DreamRunNowOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.semanticDedupThreshold !== undefined)
      params.semantic_dedup_threshold = options.semanticDedupThreshold;
    if (options?.dryRun !== undefined) params.dry_run = options.dryRun;
    return this.caller.mcpCall("dream_run_now", params);
  }

  /**
   * Retrieve dream consolidation status and token reduction metrics.
   */
  async status(options?: { workspace?: string }): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("dream_consolidation_status", params);
  }

  /**
   * Retrieve distilled procedural insights, rules, and thematic digests.
   */
  async insights(options?: { workspace?: string }): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("dream_insights", params);
  }
}

export interface DreamCallableResource extends DreamResource {
  (options?: DreamCreateOptions): Promise<unknown>;
}

export function createDreamCallable(
  resource: DreamResource
): DreamCallableResource {
  const callable = ((options?: DreamCreateOptions) =>
    resource.create(options)) as DreamCallableResource;

  return new Proxy(callable, {
    get(target, prop, receiver) {
      if (prop in target) {
        return Reflect.get(target, prop, receiver);
      }
      const val = Reflect.get(resource, prop);
      if (typeof val === "function") {
        return val.bind(resource);
      }
      return val;
    },
  });
}
