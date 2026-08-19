export interface EngramConfig {
  baseUrl: string;
  apiKey: string;
  tenant: string;
  timeout?: number;
}

export interface CreateOptions {
  memoryType?: string;
  tags?: string[];
  workspace?: string;
  metadata?: Record<string, unknown>;
  importance?: number;
  mediaUrl?: string;
}

export interface CreateDailyOptions {
  tags?: string[];
  workspace?: string;
  ttlSeconds?: number;
  metadata?: Record<string, unknown>;
}

export interface CreateIdentityOptions {
  aliases?: string[];
  metadata?: Record<string, unknown>;
}

export interface ListOptions {
  limit?: number;
  offset?: number;
  workspace?: string;
  workspaces?: string[];
  memoryType?: string;
  tags?: string[];
  tier?: string;
  sortBy?: string;
  sortOrder?: string;
  filter?: Record<string, unknown>;
}

export interface SearchOptions {
  limit?: number;
  workspace?: string;
  workspaces?: string[];
  tags?: string[];
  memoryType?: string;
  tier?: string;
  includeArchived?: boolean;
  filter?: Record<string, unknown>;
  global?: boolean;
}

export interface MemoryCouncilOptions {
  conversationId?: string;
  councilUrl?: string;
  timeoutSeconds?: number;
  includeRawStages?: boolean;
  persist?: boolean;
  workspace?: string;
  memoryTags?: string[];
}

export interface MemoryReplayAtTimeOptions {
  eventType?: string;
  includeEvents?: boolean;
  includeFailed?: boolean;
  includeDryRuns?: boolean;
  eventLimit?: number;
}

export interface CouncilSkillOptions {
  defaultWorkspace?: string;
  defaultTimeoutSeconds?: number;
  defaultIncludeRawStages?: boolean;
}

export interface CouncilSkillAskOptions {
  persist?: boolean;
  workspace?: string;
  timeoutSeconds?: number;
  includeRawStages?: boolean;
  conversationId?: string;
  councilUrl?: string;
  memoryTags?: string[];
}

export interface UpdateOptions {
  content?: string;
  tags?: string[];
  metadata?: Record<string, unknown>;
  importance?: number;
  mediaUrl?: string | null;
}

// -- Compression --

export interface CompressForContextOptions {
  memoryIds: number[];
  tokenBudget: number;
}

export interface ConsolidateOptions {
  threshold?: number;
}

// -- Agentic Evolution --

export interface UtilityScoreOptions {
  signal?: string;
}

export interface SentimentTimelineOptions {
  workspace?: string;
  limit?: number;
}

// -- Advanced Graph --

export interface CoactivationReportOptions {
  workspace?: string;
  limit?: number;
}

export interface QueryTripletsOptions {
  subject?: string;
  predicate?: string;
  object?: string;
}

export interface AddKnowledgeOptions {
  confidence?: number;
}

// -- Autonomous Agent --

export interface AgentStartOptions {
  workspace?: string;
}

export interface GardenOptions {
  workspace?: string;
  dryRun?: boolean;
}

export interface GardenPreviewOptions {
  workspace?: string;
}

export interface SuggestAcquisitionOptions {
  workspace?: string;
}

export interface ProactiveScanOptions {
  workspace?: string;
}

// -- Retrieval Excellence --

export interface CacheClearOptions {
  workspace?: string;
}

export interface EmbeddingMigrateOptions {
  fromProvider?: string;
  toProvider?: string;
}

export interface FeedbackStatsOptions {
  workspace?: string;
}

// -- Context Engineering --

export interface ListFactsOptions {
  memoryId?: number;
  workspace?: string;
  limit?: number;
}

export interface FactGraphOptions {
  workspace?: string;
}

export interface BuildContextOptions {
  strategy?: string;
  tokenBudget?: number;
  workspace?: string;
}

export interface PromptTemplateOptions {
  memories?: unknown[];
}

export interface BlockGetOptions {
  workspace?: string;
}

export interface BlockEditOptions {
  workspace?: string;
  reason?: string;
}

export interface BlockListOptions {
  blockType?: string;
  workspace?: string;
}

export interface BlockCreateOptions {
  workspace?: string;
  maxTokens?: number;
}

// -- Consolidated Facades (Phase 3c) --

export interface LifecycleUpdateOptions {
  action?:
    | "promote"
    | "promote_permanent"
    | "decay"
    | "expire"
    | "score"
    | "explain"
    | "transition"
    | "restore";
  canonicalTier?: boolean;
  ttlSeconds?: number;
  state?: "active" | "stale" | "archived" | "purged";
  reason?: string;
  persist?: boolean;
  workspace?: string;
  dryRun?: boolean;
}

export interface GraphQueryOptions {
  action?:
    | "relations"
    | "traverse"
    | "path"
    | "entities"
    | "search_entities"
    | "stats"
    | "export";
  id?: number;
  fromId?: number;
  toId?: number;
  depth?: number;
  maxDepth?: number;
  edgeType?: string;
  edgeTypes?: string[];
  direction?: "both" | "outgoing" | "incoming";
  includeEntities?: boolean;
  query?: string;
  format?: "html" | "json";
}

export interface GraphMutateOptions {
  action?: "link" | "unlink" | "extract_entities";
  fromId?: number;
  toId?: number;
  id?: number;
  edgeType?: string;
  strength?: number;
  sourceContext?: string;
  pinned?: boolean;
}

// -- Temporal Graph --

export interface TemporalCreateOptions {
  validFrom?: string;
  confidence?: number;
}

export interface TemporalInvalidateOptions {
  reason?: string;
}

export interface TemporalSnapshotOptions {
  timestamp?: string;
  workspace?: string;
}

export interface TemporalContradictionsOptions {
  workspace?: string;
}

export interface ScopeListOptions {
  recursive?: boolean;
}

// -- Scope Grants --

export interface GrantAccessOptions {
  permissions?: string;
  grantedBy?: string;
}

export interface CheckAccessOptions {
  permission?: string;
}

// -- Federation --

export interface FederationAddPeerOptions {
  name?: string;
}

export interface FederationSearchOptions {
  limit?: number;
}

// -- Retrieval Digest --

export interface MemoryDigestOptions {
  workspace?: string;
  mode?: "brief" | "standard" | "deep";
  limit?: number;
  relatedDepth?: number;
  totalBudget?: number;
  includeTypes?: string[];
  timeframe?: "1h" | "24h" | "7d" | "30d" | "all";
  includeGraph?: boolean;
  includeOperationalContext?: boolean;
  includeNextActions?: boolean;
  currentGitBranch?: string;
  currentCommitHash?: string;
}

// -- Dream Phase & Candidates --

export interface DreamCreateOptions {
  workspace?: string;
  run?: boolean;
  jobId?: string;
  instructions?: string;
  maxMemories?: number;
  maxCandidates?: number;
  summaryMinMemories?: number;
}

export interface DreamListOptions {
  workspace?: string;
  status?: "pending" | "running" | "completed" | "failed" | "canceled" | "archived";
  limit?: number;
}

export interface DreamCandidatesListOptions {
  workspace?: string;
  jobId?: string;
  reviewState?: "pending" | "accepted" | "edited" | "rejected" | "applied" | "archived";
  kind?: string;
  proposedAction?: string;
  limit?: number;
}

export interface DreamReviewOptions {
  editedContent?: string;
  notes?: string;
}

export interface DreamApplyOptions {
  confirm?: boolean;
  reviewerNotes?: string;
}

export interface DreamEvalOptions {
  workspace?: string;
  lane?: string;
}

export interface DreamRunNowOptions {
  workspace?: string;
}

// -- Realtime & SSE Events --

export type RealtimeEventType =
  | "memory_created"
  | "memory_updated"
  | "memory_deleted"
  | "crossref_created"
  | "crossref_deleted"
  | "sync_started"
  | "sync_completed"
  | "sync_failed"
  | "progress";

export interface RealtimeEvent {
  seqId?: number;
  type: RealtimeEventType;
  timestamp: string;
  memoryId?: number;
  preview?: string;
  changes?: string[];
  data?: Record<string, unknown>;
}

export interface ProgressEventData {
  progressToken: string | number;
  progress: number;
  total?: number;
  message?: string;
  workspace?: string;
}

export interface ProgressEvent {
  seqId?: number;
  type: "progress";
  timestamp: string;
  preview?: string;
  data: ProgressEventData;
}

export interface StreamEventsOptions {
  eventTypes?: RealtimeEventType[] | string;
  workspace?: string;
  lastEventId?: number | string;
  signal?: AbortSignal;
}
