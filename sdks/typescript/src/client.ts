import { EngramError } from "./errors.js";
import {
  AdminResource,
  AuthResource,
  ContextResource,
  DreamResource,
  EventsResource,
  GraphResource,
  McpResourcesResource,
  MemoriesResource,
  SearchResource,
  createDreamCallable,
  createSearchCallable,
  type DreamCallableResource,
  type McpCaller,
  type SearchCallableResource,
} from "./resources/index.js";
import type {
  AddKnowledgeOptions,
  AgentStartOptions,
  BlockCreateOptions,
  BlockEditOptions,
  BlockGetOptions,
  BlockListOptions,
  BuildContextOptions,
  CacheClearOptions,
  CheckAccessOptions,
  CoactivationReportOptions,
  CompressForContextOptions,
  ConsolidateOptions,
  CouncilSkillAskOptions,
  CouncilSkillOptions,
  CreateDailyOptions,
  CreateIdentityOptions,
  CreateOptions,
  DreamApplyOptions,
  DreamCandidatesListOptions,
  DreamCreateOptions,
  DreamEvalOptions,
  DreamListOptions,
  DreamReviewOptions,
  DreamRunNowOptions,
  EmbeddingMigrateOptions,
  EngramConfig,
  FactGraphOptions,
  FeedbackStatsOptions,
  FederationAddPeerOptions,
  FederationSearchOptions,
  GardenOptions,
  GardenPreviewOptions,
  GrantAccessOptions,
  GraphMutateOptions,
  GraphQueryOptions,
  LifecycleUpdateOptions,
  ListFactsOptions,
  ListOptions,
  MemoryCouncilOptions,
  MemoryDigestOptions,
  MemoryReplayAtTimeOptions,
  ProgressEvent,
  PromptTemplateOptions,
  ProactiveScanOptions,
  QueryTripletsOptions,
  RealtimeEvent,
  ScopeListOptions,
  SearchOptions,
  SentimentTimelineOptions,
  StreamEventsOptions,
  SuggestAcquisitionOptions,
  TemporalContradictionsOptions,
  TemporalCreateOptions,
  TemporalInvalidateOptions,
  TemporalSnapshotOptions,
  UpdateOptions,
  UtilityScoreOptions,
} from "./types.js";

export class EngramClient implements McpCaller {
  private readonly baseUrl: string;
  private readonly headers: Record<string, string>;
  private readonly timeout: number;
  private requestId: number;

  public readonly memories: MemoriesResource;
  public readonly search: SearchCallableResource;
  public readonly graph: GraphResource;
  public readonly context: ContextResource;
  public readonly dream: DreamCallableResource;
  public readonly auth: AuthResource;
  public readonly admin: AdminResource;
  public readonly events: EventsResource;
  public readonly resources: McpResourcesResource;

  constructor(config: EngramConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.timeout = config.timeout ?? 30000;
    this.headers = {
      Authorization: `Bearer ${config.apiKey}`,
      "X-Tenant-Slug": config.tenant,
      "Content-Type": "application/json",
    };
    this.requestId = 1;

    this.memories = new MemoriesResource(this);
    this.search = createSearchCallable(new SearchResource(this));
    this.graph = new GraphResource(this);
    this.context = new ContextResource(this);
    this.dream = createDreamCallable(new DreamResource(this));
    this.auth = new AuthResource(this);
    this.admin = new AdminResource(this);
    this.events = new EventsResource(this);
    this.resources = new McpResourcesResource(this);
  }

  async mcpCall(
    method: string,
    params: Record<string, unknown> = {}
  ): Promise<unknown> {
    const id = this.requestId;
    this.requestId += 1;
    const payload = {
      jsonrpc: "2.0",
      id,
      method: "tools/call",
      params: { name: method, arguments: params },
    };

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeout);

    try {
      const resp = await fetch(`${this.baseUrl}/v1/mcp`, {
        method: "POST",
        headers: this.headers,
        body: JSON.stringify(payload),
        signal: controller.signal,
      });

      if (!resp.ok) {
        throw new EngramError(`HTTP ${resp.status}: ${resp.statusText}`);
      }

      const result = (await resp.json()) as {
        error?: { message?: string };
        result?: unknown;
      };

      if (result.error) {
        throw new EngramError(result.error.message ?? "Unknown error");
      }

      return result.result;
    } finally {
      clearTimeout(timer);
    }
  }

  // ==========================================
  // Direct & Backward-Compatible Memory Methods
  // ==========================================

  create(content: string, options?: CreateOptions): Promise<unknown> {
    return this.memories.create(content, options);
  }

  memoryCreate(content: string, options?: CreateOptions): Promise<unknown> {
    return this.memories.create(content, options);
  }

  get(memoryId: number): Promise<unknown> {
    return this.memories.get(memoryId);
  }

  memoryGet(memoryId: number): Promise<unknown> {
    return this.memories.get(memoryId);
  }

  update(memoryId: number, options: UpdateOptions): Promise<unknown> {
    return this.memories.update(memoryId, options);
  }

  memoryUpdate(memoryId: number, options: UpdateOptions): Promise<unknown> {
    return this.memories.update(memoryId, options);
  }

  delete(memoryId: number): Promise<unknown> {
    return this.memories.delete(memoryId);
  }

  memoryDelete(memoryId: number): Promise<unknown> {
    return this.memories.delete(memoryId);
  }

  list(options?: ListOptions): Promise<unknown> {
    return this.memories.list(options);
  }

  memoryList(options?: ListOptions): Promise<unknown> {
    return this.memories.list(options);
  }

  createDaily(content: string, options?: CreateDailyOptions): Promise<unknown> {
    return this.memories.createDaily(content, options);
  }

  memoryCreateDaily(
    content: string,
    options?: CreateDailyOptions
  ): Promise<unknown> {
    return this.memories.createDaily(content, options);
  }

  compress(memoryId: number): Promise<unknown> {
    return this.memories.compress(memoryId);
  }

  memoryCompress(memoryId: number): Promise<unknown> {
    return this.memories.compress(memoryId);
  }

  decompress(memoryId: number): Promise<unknown> {
    return this.memories.decompress(memoryId);
  }

  memoryDecompress(memoryId: number): Promise<unknown> {
    return this.memories.decompress(memoryId);
  }

  compressForContext(
    memoryIds: number[],
    tokenBudget: number
  ): Promise<unknown> {
    return this.memories.compressForContext(memoryIds, tokenBudget);
  }

  memoryCompressForContext(
    memoryIds: number[],
    tokenBudget: number
  ): Promise<unknown> {
    return this.memories.compressForContext(memoryIds, tokenBudget);
  }

  consolidate(workspace: string, options?: ConsolidateOptions): Promise<unknown> {
    return this.memories.consolidate(workspace, options);
  }

  memoryConsolidate(
    workspace: string,
    options?: ConsolidateOptions
  ): Promise<unknown> {
    return this.memories.consolidate(workspace, options);
  }

  synthesis(memoryIds: number[]): Promise<unknown> {
    return this.memories.synthesis(memoryIds);
  }

  memorySynthesis(memoryIds: number[]): Promise<unknown> {
    return this.memories.synthesis(memoryIds);
  }

  detectUpdates(memoryId: number): Promise<unknown> {
    return this.memories.detectUpdates(memoryId);
  }

  memoryDetectUpdates(memoryId: number): Promise<unknown> {
    return this.memories.detectUpdates(memoryId);
  }

  utilityScore(
    memoryId: number,
    options?: UtilityScoreOptions
  ): Promise<unknown> {
    return this.memories.utilityScore(memoryId, options);
  }

  memoryUtilityScore(
    memoryId: number,
    options?: UtilityScoreOptions
  ): Promise<unknown> {
    return this.memories.utilityScore(memoryId, options);
  }

  sentimentAnalyze(memoryId: number): Promise<unknown> {
    return this.memories.sentimentAnalyze(memoryId);
  }

  memorySentimentAnalyze(memoryId: number): Promise<unknown> {
    return this.memories.sentimentAnalyze(memoryId);
  }

  sentimentTimeline(options?: SentimentTimelineOptions): Promise<unknown> {
    return this.memories.sentimentTimeline(options);
  }

  memorySentimentTimeline(
    options?: SentimentTimelineOptions
  ): Promise<unknown> {
    return this.memories.sentimentTimeline(options);
  }

  reflect(memoryId: number): Promise<unknown> {
    return this.memories.reflect(memoryId);
  }

  memoryReflect(memoryId: number): Promise<unknown> {
    return this.memories.reflect(memoryId);
  }

  memoryReplayAtTime(
    memoryId: number,
    timestamp: string,
    options?: MemoryReplayAtTimeOptions
  ): Promise<unknown> {
    return this.memories.replayAtTime(memoryId, timestamp, options);
  }

  replayAtTime(
    memoryId: number,
    timestamp: string,
    options?: MemoryReplayAtTimeOptions
  ): Promise<unknown> {
    return this.memories.replayAtTime(memoryId, timestamp, options);
  }

  // ==========================================
  // Direct & Backward-Compatible Search Methods
  // ==========================================

  memorySearch(query: string, options?: SearchOptions): Promise<unknown> {
    return this.search(query, options);
  }

  memoryCouncil(
    prompt: string,
    options?: MemoryCouncilOptions
  ): Promise<unknown> {
    return this.search.council(prompt, options);
  }

  council(
    prompt: string,
    options?: MemoryCouncilOptions
  ): Promise<unknown> {
    return this.search.council(prompt, options);
  }

  explainSearch(results: unknown[]): Promise<unknown> {
    return this.search.explain(results);
  }

  feedback(
    query: string,
    memoryId: number,
    signal: string
  ): Promise<unknown> {
    return this.search.feedback(query, memoryId, signal);
  }

  feedbackStats(options?: FeedbackStatsOptions): Promise<unknown> {
    return this.search.feedbackStats(options);
  }

  // ==========================================
  // Direct & Backward-Compatible Graph Methods
  // ==========================================

  related(memoryId: number): Promise<unknown> {
    return this.graph.related(memoryId);
  }

  memoryRelated(memoryId: number): Promise<unknown> {
    return this.graph.related(memoryId);
  }

  link(
    fromId: number,
    toId: number,
    edgeType: string = "related_to"
  ): Promise<unknown> {
    return this.graph.link(fromId, toId, edgeType);
  }

  memoryLink(
    fromId: number,
    toId: number,
    edgeType: string = "related_to"
  ): Promise<unknown> {
    return this.graph.link(fromId, toId, edgeType);
  }

  detectConflicts(workspace?: string): Promise<unknown> {
    return this.graph.detectConflicts(workspace);
  }

  resolveConflict(conflictId: string, resolution: string): Promise<unknown> {
    return this.graph.resolveConflict(conflictId, resolution);
  }

  coactivationReport(options?: CoactivationReportOptions): Promise<unknown> {
    return this.graph.coactivationReport(options);
  }

  queryTriplets(options?: QueryTripletsOptions): Promise<unknown> {
    return this.graph.queryTriplets(options);
  }

  addKnowledge(
    subject: string,
    predicate: string,
    object: string,
    options?: AddKnowledgeOptions
  ): Promise<unknown> {
    return this.graph.addKnowledge(subject, predicate, object, options);
  }

  temporalCreate(
    fromEntity: string,
    toEntity: string,
    relation: string,
    options?: TemporalCreateOptions
  ): Promise<unknown> {
    return this.graph.temporalCreate(fromEntity, toEntity, relation, options);
  }

  temporalInvalidate(
    edgeId: string,
    options?: TemporalInvalidateOptions
  ): Promise<unknown> {
    return this.graph.temporalInvalidate(edgeId, options);
  }

  temporalSnapshot(options?: TemporalSnapshotOptions): Promise<unknown> {
    return this.graph.temporalSnapshot(options);
  }

  temporalContradictions(
    options?: TemporalContradictionsOptions
  ): Promise<{ contradictions: Array<{ memoryId: number; conflictingId: number; reason: string }> }> {
    return this.graph.temporalContradictions(options);
  }

  temporalEvolve(entity: string): Promise<void> {
    return this.graph.temporalEvolve(entity);
  }

  graphQuery(options?: GraphQueryOptions): Promise<unknown> {
    return this.graph.query(options);
  }

  graphMutate(options?: GraphMutateOptions): Promise<unknown> {
    return this.graph.mutate(options);
  }

  // ==========================================
  // Direct & Backward-Compatible Context Methods
  // ==========================================

  extractFacts(memoryId: number): Promise<unknown> {
    return this.context.extractFacts(memoryId);
  }

  listFacts(options?: ListFactsOptions): Promise<unknown> {
    return this.context.listFacts(options);
  }

  factGraph(options?: FactGraphOptions): Promise<unknown> {
    return this.context.factGraph(options);
  }

  buildContext(
    query: string,
    options?: BuildContextOptions
  ): Promise<unknown> {
    return this.context.build(query, options);
  }

  promptTemplate(
    templateName: string,
    options?: PromptTemplateOptions
  ): Promise<unknown> {
    return this.context.promptTemplate(templateName, options);
  }

  tokenEstimate(content: string): Promise<unknown> {
    return this.context.tokenEstimate(content);
  }

  blockGet(
    blockType: string,
    label: string,
    options?: BlockGetOptions
  ): Promise<unknown> {
    return this.context.blockGet(blockType, label, options);
  }

  blockEdit(
    blockType: string,
    label: string,
    content: string,
    options?: BlockEditOptions
  ): Promise<unknown> {
    return this.context.blockEdit(blockType, label, content, options);
  }

  blockList(options?: BlockListOptions): Promise<unknown> {
    return this.context.blockList(options);
  }

  blockCreate(
    blockType: string,
    label: string,
    content: string,
    options?: BlockCreateOptions
  ): Promise<unknown> {
    return this.context.blockCreate(blockType, label, content, options);
  }

  // ==========================================
  // Direct & Backward-Compatible Auth Methods
  // ==========================================

  createIdentity(
    canonicalId: string,
    displayName: string,
    options?: CreateIdentityOptions
  ): Promise<unknown> {
    return this.auth.createIdentity(canonicalId, displayName, options);
  }

  resolveIdentity(alias: string): Promise<unknown> {
    return this.auth.resolveIdentity(alias);
  }

  scopeSet(memoryId: number, scopePath: string): Promise<void> {
    return this.auth.scopeSet(memoryId, scopePath);
  }

  scopeGet(memoryId: number): Promise<unknown> {
    return this.auth.scopeGet(memoryId);
  }

  scopeList(
    scopePath: string,
    options?: ScopeListOptions
  ): Promise<unknown> {
    return this.auth.scopeList(scopePath, options);
  }

  scopeInherit(scopePath: string, parentPath: string): Promise<unknown> {
    return this.auth.scopeInherit(scopePath, parentPath);
  }

  scopeIsolate(scopePath: string): Promise<unknown> {
    return this.auth.scopeIsolate(scopePath);
  }

  grantAccess(
    agentId: string,
    scopePath: string,
    options?: GrantAccessOptions
  ): Promise<unknown> {
    return this.auth.grantAccess(agentId, scopePath, options);
  }

  revokeAccess(agentId: string, scopePath: string): Promise<unknown> {
    return this.auth.revokeAccess(agentId, scopePath);
  }

  listGrants(agentId: string): Promise<unknown> {
    return this.auth.listGrants(agentId);
  }

  checkAccess(
    agentId: string,
    scopePath: string,
    options?: CheckAccessOptions
  ): Promise<unknown> {
    return this.auth.checkAccess(agentId, scopePath, options);
  }

  // ==========================================
  // Direct & Backward-Compatible Admin Methods
  // ==========================================

  stats(): Promise<unknown> {
    return this.admin.stats();
  }

  agentStart(options?: AgentStartOptions): Promise<unknown> {
    return this.admin.agentStart(options);
  }

  agentStop(): Promise<unknown> {
    return this.admin.agentStop();
  }

  agentStatus(): Promise<unknown> {
    return this.admin.agentStatus();
  }

  agentMetrics(): Promise<unknown> {
    return this.admin.agentMetrics();
  }

  agentConfigure(config: Record<string, unknown>): Promise<unknown> {
    return this.admin.agentConfigure(config);
  }

  garden(options?: GardenOptions): Promise<unknown> {
    return this.admin.garden(options);
  }

  gardenPreview(options?: GardenPreviewOptions): Promise<unknown> {
    return this.admin.gardenPreview(options);
  }

  gardenUndo(operationId: string): Promise<unknown> {
    return this.admin.gardenUndo(operationId);
  }

  suggestAcquisition(
    options?: SuggestAcquisitionOptions
  ): Promise<unknown> {
    return this.admin.suggestAcquisition(options);
  }

  proactiveScan(options?: ProactiveScanOptions): Promise<unknown> {
    return this.admin.proactiveScan(options);
  }

  cacheStats(): Promise<unknown> {
    return this.admin.cacheStats();
  }

  cacheClear(options?: CacheClearOptions): Promise<unknown> {
    return this.admin.cacheClear(options);
  }

  embeddingProviders(): Promise<unknown> {
    return this.admin.embeddingProviders();
  }

  embeddingMigrate(options?: EmbeddingMigrateOptions): Promise<unknown> {
    return this.admin.embeddingMigrate(options);
  }

  federationAddPeer(
    url: string,
    apiKey: string,
    options?: FederationAddPeerOptions
  ): Promise<unknown> {
    return this.admin.federationAddPeer(url, apiKey, options);
  }

  federationRemovePeer(peerId: string): Promise<unknown> {
    return this.admin.federationRemovePeer(peerId);
  }

  federationListPeers(): Promise<unknown> {
    return this.admin.federationListPeers();
  }

  federationSearch(
    query: string,
    options?: FederationSearchOptions
  ): Promise<unknown> {
    return this.admin.federationSearch(query, options);
  }

  federationShare(memoryId: number, peerId: string): Promise<unknown> {
    return this.admin.federationShare(memoryId, peerId);
  }

  federationSyncStatus(): Promise<unknown> {
    return this.admin.federationSyncStatus();
  }

  lifecycleUpdate(
    id: number,
    options?: LifecycleUpdateOptions
  ): Promise<unknown> {
    return this.admin.lifecycleUpdate(id, options);
  }

  digest(topic: string, options?: MemoryDigestOptions): Promise<unknown> {
    return this.search.digest(topic, options);
  }

  memoryDigest(topic: string, options?: MemoryDigestOptions): Promise<unknown> {
    return this.search.digest(topic, options);
  }

  dreamRunNow(options?: DreamRunNowOptions): Promise<unknown> {
    return this.dream.runNow(options);
  }

  /**
   * Stream real-time and SSE progress events.
   */
  async *streamEvents(
    options?: StreamEventsOptions
  ): AsyncIterable<RealtimeEvent> {
    yield* this.events.stream(this.baseUrl, this.headers, options);
  }

  /**
   * Watch progress events for a specific progress token.
   */
  async watchProgress(
    token: string | number,
    onProgress: (event: ProgressEvent) => void,
    signal?: AbortSignal
  ): Promise<() => void> {
    return this.events.watchProgress(
      this.baseUrl,
      this.headers,
      token,
      onProgress,
      signal
    );
  }

  /**
   * List all resource templates exposed by the MCP server.
   */
  resourceList(): Promise<unknown> {
    return this.resources.list();
  }

  /**
   * Read an MCP resource by URI.
   */
  resourceRead(uri: string): Promise<unknown> {
    return this.resources.read(uri);
  }

  /**
   * Subscribe to live updates for an MCP resource URI.
   */
  resourceSubscribe(uri: string): Promise<unknown> {
    return this.resources.subscribe(uri);
  }

  /**
   * Unsubscribe from updates for an MCP resource URI.
   */
  resourceUnsubscribe(uri: string): Promise<unknown> {
    return this.resources.unsubscribe(uri);
  }
}
