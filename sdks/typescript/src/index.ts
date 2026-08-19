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
  MultimodalResource,
  SearchResource,
  type McpCaller,
} from "./resources/index.js";

export type {
  AddKnowledgeOptions,
  AgentStartOptions,
  AutoLinkOptions,
  BlockCreateOptions,
  BlockEditOptions,
  BlockGetOptions,
  BlockListOptions,
  BuildContextOptions,
  CacheClearOptions,
  CaptureScreenshotOptions,
  CheckAccessOptions,
  ClusterConceptsOptions,
  ClusterOptions,
  CoactivationReportOptions,
  ConceptCluster,
  CompressForContextOptions,
  ConsolidateOptions,
  CouncilSkillAskOptions,
  CouncilSkillOptions,
  CreateDailyOptions,
  CreateIdentityOptions,
  CreateOptions,
  DescribeImageOptions,
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
  IngestMediaOptions,
  LifecycleUpdateOptions,
  ListFactsOptions,
  ListMediaOptions,
  ListOptions,
  MediaAsset,
  MemoryCouncilOptions,
  MemoryDigestOptions,
  MemoryReplayAtTimeOptions,
  ProcessVideoOptions,
  PredictLinksOptions,
  PredictLinksResult,
  PredictedLink,
  ProgressEvent,
  ProgressEventData,
  PromptTemplateOptions,
  ProactiveScanOptions,
  QueryTripletsOptions,
  RealtimeEvent,
  RealtimeEventType,
  ScopeListOptions,
  SearchByImageOptions,
  SearchOptions,
  SentimentTimelineOptions,
  StreamEventsOptions,
  SuggestAcquisitionOptions,
  SyncMediaOptions,
  TemporalContradictionsOptions,
  TemporalCreateOptions,
  TemporalInvalidateOptions,
  TemporalSnapshotOptions,
  UpdateOptions,
  UtilityScoreOptions,
} from "./types.js";
