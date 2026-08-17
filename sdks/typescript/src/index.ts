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

export { EngramClient } from "./client";
export { EngramError } from "./errors";
export { CouncilSkill, type CouncilClient } from "./council";

export {
  AdminResource,
  AuthResource,
  BaseResource,
  ContextResource,
  GraphResource,
  MemoriesResource,
  SearchResource,
  type McpCaller,
} from "./resources";

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
  MemoryReplayAtTimeOptions,
  PromptTemplateOptions,
  ProactiveScanOptions,
  QueryTripletsOptions,
  ScopeListOptions,
  SearchOptions,
  SentimentTimelineOptions,
  SuggestAcquisitionOptions,
  TemporalContradictionsOptions,
  TemporalCreateOptions,
  TemporalInvalidateOptions,
  TemporalSnapshotOptions,
  UpdateOptions,
  UtilityScoreOptions,
} from "./types";
