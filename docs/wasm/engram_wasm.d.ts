/* tslint:disable */
/* eslint-disable */

/**
 * Score a document against query terms using BM25.
 *
 * # Arguments (JS)
 *
 * * `query_terms_json` — JSON array of query term strings, e.g. `["rust","fast"]`
 * * `doc_terms_json`   — JSON array of document tokens
 * * `doc_count`        — Total documents in corpus
 * * `avg_doc_len`      — Average document length in tokens
 * * `k1`               — BM25 k1 parameter (default 1.5)
 * * `b`                — BM25 b parameter  (default 0.75)
 *
 * # Returns
 *
 * BM25 relevance score >= 0.0.
 */
export function bm25_score(query_terms_json: string, doc_terms_json: string, doc_count: number, avg_doc_len: number, k1: number, b: number): number;

/**
 * Tokenize a text string into BM25-compatible lowercase tokens.
 *
 * Returns a JSON array of token strings.
 */
export function bm25_tokenize(text: string): string;

/**
 * Compute cosine similarity between two embedding vectors.
 *
 * Both vectors must be JSON arrays of numbers. Returns 0.0 on error.
 *
 * # Returns
 *
 * Cosine similarity in [-1.0, 1.0]. Returns 0.0 if either vector is all zeros.
 */
export function cosine_similarity(vec_a_json: string, vec_b_json: string): number;

/**
 * Extract entities from text.
 *
 * Returns a JSON array of entity objects:
 * ```json
 * [
 *   {
 *     "text": "@alice",
 *     "normalized": "alice",
 *     "entity_type": "mention",
 *     "confidence": 0.9,
 *     "position": 6,
 *     "count": 1
 *   }
 * ]
 * ```
 *
 * `entity_type` is one of: `"mention"`, `"email"`, `"url"`, `"name"`.
 */
export function extract_entities(text: string): string;

/**
 * Extract entities with a custom maximum count.
 */
export function extract_entities_limited(text: string, max_entities: number): string;

/**
 * BFS traversal from `start`, up to `max_depth` hops.
 *
 * # Arguments
 *
 * * `edges_json` — JSON array of `{"from": u64, "to": u64}` objects.
 * * `start`      — Start node ID.
 * * `max_depth`  — Maximum hops (0 = start node only).
 *
 * # Returns
 *
 * JSON array of `{"node": u64, "depth": usize}` objects in BFS order.
 */
export function graph_bfs(edges_json: string, start: bigint, max_depth: number): string;

/**
 * Find the shortest undirected path between `start` and `end`.
 *
 * # Arguments
 *
 * * `edges_json` — JSON array of `{"from": u64, "to": u64}` objects.
 * * `start`      — Source node ID.
 * * `end`        — Target node ID.
 *
 * # Returns
 *
 * JSON array of node IDs forming the path, or `null` if no path exists.
 */
export function graph_shortest_path(edges_json: string, start: bigint, end: bigint): string;

/**
 * Merge keyword and semantic ranked lists (standard hybrid-search pattern).
 *
 * # Arguments
 *
 * * `keyword_ids_json`  — JSON array of doc IDs in keyword rank order (best first).
 * * `semantic_ids_json` — JSON array of doc IDs in semantic rank order (best first).
 * * `keyword_weight`    — Weight for keyword list (default 1.0).
 * * `semantic_weight`   — Weight for semantic list (default 1.0).
 * * `k`                 — RRF constant (0 = default 60.0).
 *
 * # Returns
 *
 * JSON array of `{"doc_id": u64, "score": f64}` sorted by score descending.
 */
export function rrf_hybrid(keyword_ids_json: string, semantic_ids_json: string, keyword_weight: number, semantic_weight: number, k: number): string;

/**
 * Merge multiple ranked lists using Reciprocal Rank Fusion.
 *
 * # Arguments
 *
 * * `lists_json` — JSON array of ranked lists. Each list is
 *   `{"items": [{"doc_id": u64, "rank": usize}], "weight": f64}`.
 *   `weight` is optional and defaults to 1.0.
 * * `k`          — RRF constant. Pass 0 to use the default (60.0).
 *
 * # Returns
 *
 * JSON array of `{"doc_id": u64, "score": f64}` sorted by score descending.
 */
export function rrf_merge(lists_json: string, k: number): string;

/**
 * Compute a TF-IDF embedding vector for `text`.
 *
 * # Arguments
 *
 * * `text`       — Text to embed.
 * * `dimensions` — Output vector size. Pass 0 to use the default (384).
 *
 * # Returns
 *
 * JSON array of `f32` values, length = `dimensions`.
 */
export function tfidf_embed(text: string, dimensions: number): string;

/**
 * Return the engram-wasm version string.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly bm25_score: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly bm25_tokenize: (a: number, b: number, c: number) => void;
    readonly cosine_similarity: (a: number, b: number, c: number, d: number) => number;
    readonly extract_entities: (a: number, b: number, c: number) => void;
    readonly extract_entities_limited: (a: number, b: number, c: number, d: number) => void;
    readonly graph_bfs: (a: number, b: number, c: number, d: bigint, e: number) => void;
    readonly graph_shortest_path: (a: number, b: number, c: number, d: bigint, e: bigint) => void;
    readonly rrf_hybrid: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly rrf_merge: (a: number, b: number, c: number, d: number) => void;
    readonly tfidf_embed: (a: number, b: number, c: number, d: number) => void;
    readonly version: (a: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
