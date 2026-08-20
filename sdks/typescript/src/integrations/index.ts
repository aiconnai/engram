/**
 * Agent Framework Integrations for Engram TypeScript SDK.
 */

export * as langchain from "./langchain.js";
export * as llamaindex from "./llamaindex.js";
export * as aiSdk from "./ai_sdk.js";
export * as ai_sdk from "./ai_sdk.js";

// LangChain exports
export {
  EngramChatMessageHistory as LangChainChatMessageHistory,
  EngramVectorStore as LangChainVectorStore,
  type LangChainMessage,
  type LangChainDocument,
  type EngramChatMessageHistoryOptions as LangChainChatMessageHistoryOptions,
  type EngramVectorStoreOptions as LangChainVectorStoreOptions,
} from "./langchain.js";

// LlamaIndex exports
export {
  EngramVectorStore as LlamaIndexVectorStore,
  EngramChatStore as LlamaIndexChatStore,
  EngramDocumentStore as LlamaIndexDocumentStore,
  type LlamaIndexNode,
  type LlamaIndexQuery,
  type LlamaIndexQueryResult,
  type LlamaIndexChatMessage,
  type EngramLlamaIndexOptions,
} from "./llamaindex.js";

// Vercel AI SDK exports
export {
  createEngramMemoryTool,
  engramChatHistory,
  EngramChatHistoryAdapter,
  type CoreChatMessage,
  type EngramMemoryToolOptions,
  type EngramChatHistoryOptions,
  type EngramMemoryTool,
} from "./ai_sdk.js";
