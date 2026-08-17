import { BaseResource } from "./base.js";
import type {
  AddKnowledgeOptions,
  CoactivationReportOptions,
  GraphMutateOptions,
  GraphQueryOptions,
  QueryTripletsOptions,
  TemporalContradictionsOptions,
  TemporalCreateOptions,
  TemporalInvalidateOptions,
  TemporalSnapshotOptions,
} from "../types.js";

export class GraphResource extends BaseResource {
  /**
   * Find memories related to a given memory.
   */
  async related(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_related", { id: memoryId });
  }

  /**
   * Create a typed edge between two memories.
   */
  async link(
    fromId: number,
    toId: number,
    edgeType: string = "related_to"
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_link", {
      from_id: fromId,
      to_id: toId,
      edge_type: edgeType,
    });
  }

  /**
   * Query the knowledge graph: relations, paths, multi-hop traversal, entity search, or export.
   */
  async query(options?: GraphQueryOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.action !== undefined) params.action = options.action;
    if (options?.id !== undefined) params.id = options.id;
    if (options?.fromId !== undefined) params.from_id = options.fromId;
    if (options?.toId !== undefined) params.to_id = options.toId;
    if (options?.depth !== undefined) params.depth = options.depth;
    if (options?.maxDepth !== undefined) params.max_depth = options.maxDepth;
    if (options?.edgeType !== undefined) params.edge_type = options.edgeType;
    if (options?.edgeTypes !== undefined) params.edge_types = options.edgeTypes;
    if (options?.direction !== undefined) params.direction = options.direction;
    if (options?.includeEntities !== undefined)
      params.include_entities = options.includeEntities;
    if (options?.query !== undefined) params.query = options.query;
    if (options?.format !== undefined) params.format = options.format;
    return this.caller.mcpCall("graph_query", params);
  }

  /**
   * Mutate the knowledge graph: link memories, remove cross-references, or extract entities.
   */
  async mutate(options?: GraphMutateOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.action !== undefined) params.action = options.action;
    if (options?.fromId !== undefined) params.from_id = options.fromId;
    if (options?.toId !== undefined) params.to_id = options.toId;
    if (options?.id !== undefined) params.id = options.id;
    if (options?.edgeType !== undefined) params.edge_type = options.edgeType;
    if (options?.strength !== undefined) params.strength = options.strength;
    if (options?.sourceContext !== undefined)
      params.source_context = options.sourceContext;
    if (options?.pinned !== undefined) params.pinned = options.pinned;
    return this.caller.mcpCall("graph_mutate", params);
  }

  /**
   * Detect potential conflicts between memories in a workspace.
   */
  async detectConflicts(workspace?: string): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (workspace !== undefined) params.workspace = workspace;
    return this.caller.mcpCall("memory_detect_conflicts", params);
  }

  /**
   * Resolve a previously detected conflict.
   */
  async resolveConflict(
    conflictId: string,
    resolution: string
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_resolve_conflict", {
      conflict_id: conflictId,
      resolution,
    });
  }

  /**
   * Generate a coactivation report for memories.
   */
  async coactivationReport(
    options?: CoactivationReportOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      limit: options?.limit ?? 50,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_coactivation_report", params);
  }

  /**
   * Query knowledge graph triplets (subject, predicate, object).
   */
  async queryTriplets(options?: QueryTripletsOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.subject !== undefined) params.subject = options.subject;
    if (options?.predicate !== undefined) params.predicate = options.predicate;
    if (options?.object !== undefined) params.object = options.object;
    return this.caller.mcpCall("memory_query_triplets", params);
  }

  /**
   * Add a knowledge triplet to the graph.
   */
  async addKnowledge(
    subject: string,
    predicate: string,
    object: string,
    options?: AddKnowledgeOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_add_knowledge", {
      subject,
      predicate,
      object,
      confidence: options?.confidence ?? 1.0,
    });
  }

  /**
   * Create a temporal relationship edge between entities.
   */
  async temporalCreate(
    fromEntity: string,
    toEntity: string,
    relation: string,
    options?: TemporalCreateOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      from_entity: fromEntity,
      to_entity: toEntity,
      relation,
      confidence: options?.confidence ?? 1.0,
    };
    if (options?.validFrom !== undefined) params.valid_from = options.validFrom;
    return this.caller.mcpCall("memory_temporal_create", params);
  }

  /**
   * Invalidate a temporal edge.
   */
  async temporalInvalidate(
    edgeId: string,
    options?: TemporalInvalidateOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { edge_id: edgeId };
    if (options?.reason !== undefined) params.reason = options.reason;
    return this.caller.mcpCall("memory_temporal_invalidate", params);
  }

  /**
   * Retrieve a temporal snapshot of the graph at a given point in time.
   */
  async temporalSnapshot(
    options?: TemporalSnapshotOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.timestamp !== undefined) params.timestamp = options.timestamp;
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_temporal_snapshot", params);
  }

  /**
   * Detect temporal contradictions in the graph.
   */
  async temporalContradictions(
    options?: TemporalContradictionsOptions
  ): Promise<{ contradictions: Array<{ memoryId: number; conflictingId: number; reason: string }> }> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_temporal_contradictions", params) as Promise<{
      contradictions: Array<{ memoryId: number; conflictingId: number; reason: string }>;
    }>;
  }

  /**
   * Evolve temporal state for an entity.
   */
  async temporalEvolve(entity: string): Promise<void> {
    await this.caller.mcpCall("memory_temporal_evolve", { entity });
  }
}
