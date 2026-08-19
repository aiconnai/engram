/**
 * Engram Cloud TypeScript SDK
 *
 * Persistent memory infrastructure for AI agents and multi-agent systems.
 *
 * Usage:
 *   import { EngramClient } from "engram-client";
 *
 *   const client = new EngramClient({
 *     baseUrl: "https://your-engram-cloud.fly.dev",
 *     apiKey: "ek_...",
 *     tenant: "my-tenant",
 *   });
 *
 *   // Modular resource style:
 *   const memory = await client.memories.create("User prefers dark mode");
 *   const results = await client.search.search("user preferences");
 *
 *   // Direct backward-compatible style:
 *   const directMemory = await client.create("User prefers dark mode");
 *   const directResults = await client.search("user preferences");
 */

export { EngramClient } from "./client.js";
export { EngramError } from "./errors.js";
export { CouncilSkill, type CouncilClient } from "./council.js";

export {
  AdminResource,
  AuthResource,
  BaseResource,
  ContextResource,
  DreamResource,
  EventsResource,
  GraphResource,
  McpResourcesResource,
  MemoriesResource,
  SearchResource,
  type McpCaller,
} from "./resources/index.js";

export type {
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
  ProgressEventData,
  PromptTemplateOptions,
  ProactiveScanOptions,
  QueryTripletsOptions,
  RealtimeEvent,
  RealtimeEventType,
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
