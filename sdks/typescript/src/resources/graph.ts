import { BaseResource } from "./base.js";
import type {
  AddKnowledgeOptions,
  AutoLinkOptions,
  ClusterConceptsOptions,
  ClusterOptions,
  CoactivationReportOptions,
  ConceptCluster,
  GraphMutateOptions,
  GraphQueryOptions,
  PredictLinksOptions,
  PredictLinksResult,
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
  async detectConflicts(_workspace?: string): Promise<unknown> {
    return this.caller.mcpCall("memory_detect_conflicts", {});
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
      strategy: resolution,
    });
  }

  /**
   * Generate a coactivation report for memories.
   */
  async coactivationReport(
    _options?: CoactivationReportOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_coactivation_report", {});
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
    _predicate?: string,
    _object?: string,
    _options?: AddKnowledgeOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { fact: subject };
    return this.caller.mcpCall("memory_ingest_fact", params);
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
    const fromId = Number.parseInt(fromEntity, 10) || 1;
    const toId = Number.parseInt(toEntity, 10) || 2;
    const params: Record<string, unknown> = {
      from_id: fromId,
      to_id: toId,
      relation,
      valid_from: options?.validFrom ?? "1970-01-01T00:00:00Z",
    };
    if (options?.confidence !== undefined) params.confidence = options.confidence;
    return this.caller.mcpCall("temporal_add_edge", params);
  }

  /**
   * Invalidate a temporal edge.
   */
  async temporalInvalidate(
    edgeId: string,
    options?: TemporalInvalidateOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("temporal_diff", {
      t1: String(edgeId),
      t2: (options as { invalidatedAt?: string } | undefined)?.invalidatedAt ?? "now",
    });
  }

  /**
   * Retrieve a temporal snapshot of the graph at a given point in time.
   */
  async temporalSnapshot(
    options?: TemporalSnapshotOptions
  ): Promise<unknown> {
    const timestamp = options?.timestamp ?? new Date().toISOString();
    const params: Record<string, unknown> = {
      timestamp,
      scope: (options as { scope?: string } | undefined)?.scope ?? options?.workspace ?? "global",
    };
    return this.caller.mcpCall("memory_temporal_snapshot", params);
  }

  /**
   * Detect temporal contradictions in the graph.
   */
  async temporalContradictions(
    _options?: TemporalContradictionsOptions
  ): Promise<{ contradictions: Array<{ memoryId: number; conflictingId: number; reason: string }> }> {
    return this.caller.mcpCall("temporal_contradictions", {}) as Promise<{
      contradictions: Array<{ memoryId: number; conflictingId: number; reason: string }>;
    }>;
  }

  /**
   * Evolve temporal state for an entity.
   */
  async temporalEvolve(entity: string): Promise<void> {
    const edgeId = Number.parseInt(entity, 10) || 1;
    await this.caller.mcpCall("temporal_add_edge", {
      from_id: edgeId,
      to_id: edgeId,
      relation: "evolves",
      valid_from: "1970-01-01T00:00:00Z",
    });
  }

  /**
   * Predict missing or implicit links in the knowledge graph.
   */
  async predictLinks(
    options?: PredictLinksOptions
  ): Promise<PredictLinksResult> {
    const params: Record<string, unknown> = {};
    if (options?.memoryId !== undefined) params.memory_id = options.memoryId;
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.minConfidence !== undefined)
      params.min_confidence = options.minConfidence;
    if (options?.topK !== undefined) params.top_k = options.topK;
    if (options?.algorithm !== undefined) params.algorithm = options.algorithm;
    if (options?.autoApply !== undefined) params.auto_apply = options.autoApply;
    return this.caller.mcpCall("memory_predict_links", params) as Promise<PredictLinksResult>;
  }

  /**
   * Cluster memories into semantic concept nodes.
   */
  async clusterConcepts(
    options?: ClusterConceptsOptions
  ): Promise<{ count: number; concepts: ConceptCluster[] }> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.minClusterSize !== undefined)
      params.min_cluster_size = options.minClusterSize;
    if (options?.maxClusters !== undefined)
      params.max_clusters = options.maxClusters;
    return this.caller.mcpCall("memory_cluster_concepts", params) as Promise<{
      count: number;
      concepts: ConceptCluster[];
    }>;
  }

  /**
   * Run semantic + temporal auto-linker.
   */
  async autoLink(options?: AutoLinkOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.similarityThreshold !== undefined)
      params.similarity_threshold = options.similarityThreshold;
    if (options?.timeWindowMinutes !== undefined)
      params.time_window_minutes = options.timeWindowMinutes;
    return this.caller.mcpCall("memory_auto_link", params);
  }

  /**
   * Run community detection clustering.
   */
  async cluster(options?: ClusterOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.minClusterSize !== undefined)
      params.min_cluster_size = options.minClusterSize;
    if (options?.resolution !== undefined)
      params.resolution = options.resolution;
    if (options?.linkTypes !== undefined) params.link_types = options.linkTypes;
    return this.caller.mcpCall("memory_cluster", params);
  }

  /**
   * Get cluster containing a specific memory.
   */
  async getCluster(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_get_cluster", { memory_id: memoryId });
  }

  /**
   * List all detected clusters.
   */
  async listClusters(algorithm?: string): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (algorithm !== undefined) params.algorithm = algorithm;
    return this.caller.mcpCall("memory_list_clusters", params);
  }
}

