/**
 * Engram Official GitHub Pages Interactive Engine
 * Integrates pure-Rust WebAssembly (`engram-wasm`) directly in the browser
 * for client-side BM25 scoring, TF-IDF vector embeddings, entity extraction,
 * graph traversal, and RRF rank fusion.
 */

import initWasm, * as wasm from './wasm/engram_wasm.js';

let wasmLoaded = false;

document.addEventListener('DOMContentLoaded', () => {
  initTerminalSimulation();
  initComparisonDemo();
  initPlayground();
  initSdkTabs();
  initCopyButtons();
  initWasmEngine();
});

/* =========================================================================
   0. WASM Engine Initialization
   ========================================================================= */
async function initWasmEngine() {
  const badge = document.getElementById('wasm-badge');
  try {
    await initWasm();
    wasmLoaded = true;
    if (badge) {
      badge.textContent = `⚡ Rust WASM v${wasm.version()} Active`;
      badge.style.color = '#10b981';
      badge.style.borderColor = 'rgba(16, 185, 129, 0.4)';
    }
  } catch (err) {
    console.warn('WASM module initialization fallback:', err);
    if (badge) {
      badge.textContent = '⚡ Engine: JS Simulation';
      badge.style.color = '#f59e0b';
    }
  }
}

/* =========================================================================
   1. The Forgetting — Interactive Split Comparison Chat Demo
   ========================================================================= */
const chatScript = {
  userPrompt: "What was our decision regarding database connection pooling and async workers in the backend?",
  forgetAgent: "I don't have access to your previous sessions. Could you please remind me which database and async runtime you chose?",
  rememberAgent: "Retrieved from workspace `backend` (decision mem_98f4a1, salience 0.95):\n\nOn July 14th, you decided to size the PostgreSQL connection pool to 25 connections (5s timeout) and use Tokio async/await for all I/O workers, reserving std::thread for CPU compute."
};

function initComparisonDemo() {
  const replayBtn = document.getElementById('replay-demo');
  const chatForget = document.querySelector('.chat[data-pane="forget"]');
  const chatRemember = document.querySelector('.chat[data-pane="remember"]');

  if (!chatForget || !chatRemember) return;

  function runDemo() {
    chatForget.replaceChildren();
    chatRemember.replaceChildren();

    // Step 1: User message appears on both sides
    const userMsg1 = document.createElement('div');
    userMsg1.className = 'chat-bubble user';
    userMsg1.textContent = chatScript.userPrompt;

    const userMsg2 = document.createElement('div');
    userMsg2.className = 'chat-bubble user';
    userMsg2.textContent = chatScript.userPrompt;

    chatForget.appendChild(userMsg1);
    chatRemember.appendChild(userMsg2);

    // Step 2: Agent responses
    setTimeout(() => {
      const agentForget = document.createElement('div');
      agentForget.className = 'chat-bubble agent failure';
      const fTitle = document.createElement('strong');
      fTitle.textContent = 'Claude (without memory):';
      agentForget.appendChild(fTitle);
      agentForget.appendChild(document.createElement('br'));
      agentForget.appendChild(document.createTextNode(chatScript.forgetAgent));
      chatForget.appendChild(agentForget);

      const agentRemember = document.createElement('div');
      agentRemember.className = 'chat-bubble agent success';
      const rTitle = document.createElement('strong');
      rTitle.textContent = 'Claude (with Engram MCP):';
      agentRemember.appendChild(rTitle);
      agentRemember.appendChild(document.createElement('br'));
      
      const lines = chatScript.rememberAgent.split('\n');
      lines.forEach((line, idx) => {
        if (idx > 0) agentRemember.appendChild(document.createElement('br'));
        agentRemember.appendChild(document.createTextNode(line));
      });
      chatRemember.appendChild(agentRemember);
    }, 600);
  }

  runDemo();

  if (replayBtn) {
    replayBtn.addEventListener('click', () => {
      runDemo();
    });
  }
}

/* =========================================================================
   2. Terminal Simulation
   ========================================================================= */
const terminalScenarios = {
  mcp: [
    { type: 'line', prompt: '$', cmd: 'engram-server --transport stdio' },
    { type: 'banner', text: '  Engram v0.24.0 (local-first persistent memory server)' },
    { type: 'dim', text: '  Database: ~/.local/share/engram/memories.db [SQLite 3.45 + WAL]' },
    { type: 'dim', text: '  Search: Hybrid BM25 (FTS5) + Cosine Vectors (MiniLM ONNX + HNSW) + Fuzzy' },
    { type: 'dim', text: '  MCP Protocol: JSON-RPC 2.0 active over stdio [243 tools registered]' },
    { type: 'break' },
    { type: 'success', tag: '[MCP INITIALIZED]', text: ' Handshake from host: Claude Code v1.0.18' },
    { type: 'dim', text: '  Capabilities: tools, resources, prompts, logging, completions' },
    { type: 'success', tag: '[READY]', text: ' Loaded 1,482 project memories across 4 workspaces in 1.4ms' }
  ],

  search: [
    { type: 'line', prompt: '$', cmd: 'engram-cli search "asynch awiat rust" --workspace backend --explain' },
    { type: 'dim', text: '  Executing 3-way hybrid search with Reciprocal Rank Fusion (RRF)...' },
    { type: 'break' },
    { type: 'highlight', tag: '[MATCH 1]', score: 'Score: 0.982', detail: ' (BM25: 0.94, Vector: 0.99, Fuzzy: 0.92)' },
    { type: 'dim', text: '  ID: mem_98f4a1 | Workspace: backend | Type: decision | Salience: 0.95' },
    { type: 'cmd', text: '  "Use Tokio async/await for all I/O-bound workers; reserve std::thread for CPU compute."' },
    { type: 'dim', text: '  Entities: [Tokio, Rust, Async/Await] · Provenance: session_2026_07_14.jsonl' },
    { type: 'break' },
    { type: 'highlight', tag: '[MATCH 2]', score: 'Score: 0.874', detail: ' (BM25: 0.82, Vector: 0.91, Fuzzy: 0.79)' },
    { type: 'dim', text: '  ID: mem_42c8d9 | Workspace: backend | Type: pattern | Salience: 0.81' },
    { type: 'cmd', text: '  "Async channel buffer depth must be bounded at 1024 messages to prevent memory ballooning."' }
  ],

  graph: [
    { type: 'line', prompt: '$', cmd: 'engram-cli graph traverse --entity "AuthService" --depth 2 --output json' },
    { type: 'dim', text: '  Traversing knowledge graph from canonical identity \'AuthService\'...' },
    { type: 'break' },
    { type: 'purple', tag: '[ENTITY]', name: 'AuthService', detail: ' (Aliases: [auth-api, authentication-worker, idp-client])' },
    { type: 'graph-edge', label: '(depends_on)', target: 'VaultSecrets', detail: ' [mem_12a0f7: "API keys rotated daily via Vault"]' },
    { type: 'graph-edge', label: '(stores_in)', target: 'PostgreSQL', detail: ' [mem_33b8c2: "User credentials stored with Argon2id hash"]' },
    { type: 'graph-edge', label: '(accessed_by)', target: 'GatewayWorker', detail: ' [mem_77d1e4: "JWT bearer validation executed at edge proxy"]' },
    { type: 'break' },
    { type: 'dim', text: '  Resolved 4 nodes, 3 relations in 0.82ms. Shortest path to DB: 1 hop.' }
  ]
};

function renderTerminal(scenarioKey) {
  const terminalBody = document.getElementById('hero-terminal');
  if (!terminalBody || !terminalScenarios[scenarioKey]) return;

  terminalBody.replaceChildren();

  const items = terminalScenarios[scenarioKey];
  items.forEach(item => {
    if (item.type === 'line') {
      const row = document.createElement('div');
      const pSpan = document.createElement('span');
      pSpan.className = 't-prompt';
      pSpan.textContent = item.prompt + ' ';
      const cSpan = document.createElement('span');
      cSpan.className = 't-cmd';
      cSpan.textContent = item.cmd;
      row.appendChild(pSpan);
      row.appendChild(cSpan);
      terminalBody.appendChild(row);
    } else if (item.type === 'banner') {
      const row = document.createElement('div');
      row.className = 't-banner';
      row.textContent = item.text;
      terminalBody.appendChild(row);
    } else if (item.type === 'dim') {
      const row = document.createElement('div');
      row.className = 't-dim';
      row.textContent = item.text;
      terminalBody.appendChild(row);
    } else if (item.type === 'break') {
      terminalBody.appendChild(document.createElement('br'));
    } else if (item.type === 'success') {
      const row = document.createElement('div');
      const tag = document.createElement('span');
      tag.className = 't-success';
      tag.textContent = item.tag;
      row.appendChild(tag);
      row.appendChild(document.createTextNode(item.text));
      terminalBody.appendChild(row);
    } else if (item.type === 'highlight') {
      const row = document.createElement('div');
      const tag = document.createElement('span');
      tag.className = 't-highlight';
      tag.textContent = item.tag + ' ';
      const sc = document.createElement('span');
      sc.className = 't-cyan';
      sc.textContent = item.score;
      row.appendChild(tag);
      row.appendChild(sc);
      row.appendChild(document.createTextNode(item.detail));
      terminalBody.appendChild(row);
    } else if (item.type === 'cmd') {
      const row = document.createElement('div');
      const sc = document.createElement('span');
      sc.className = 't-cmd';
      sc.textContent = item.text;
      row.appendChild(sc);
      terminalBody.appendChild(row);
    } else if (item.type === 'purple') {
      const row = document.createElement('div');
      const tag = document.createElement('span');
      tag.className = 't-purple';
      tag.textContent = item.tag + ' ';
      const name = document.createElement('span');
      name.className = 't-cyan';
      name.textContent = item.name;
      row.appendChild(tag);
      row.appendChild(name);
      row.appendChild(document.createTextNode(item.detail));
      terminalBody.appendChild(row);
    } else if (item.type === 'graph-edge') {
      const row = document.createElement('div');
      row.appendChild(document.createTextNode('  ├── '));
      const edge = document.createElement('span');
      edge.className = 't-success';
      edge.textContent = item.label;
      const target = document.createElement('span');
      target.className = 't-cyan';
      target.textContent = ' ──> ' + item.target;
      row.appendChild(edge);
      row.appendChild(target);
      row.appendChild(document.createTextNode(item.detail));
      terminalBody.appendChild(row);
    }
  });
}

function initTerminalSimulation() {
  const tabBtns = document.querySelectorAll('.terminal-tab-btn');
  if (!tabBtns.length) return;

  renderTerminal('mcp');

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      tabBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const target = btn.getAttribute('data-tab');
      renderTerminal(target);
    });
  });
}

/* =========================================================================
   3. Interactive Memory Playground (Client-Side Rust WASM Integration)
   ========================================================================= */

// Sample in-memory corpus for live in-browser WASM search
const inMemoryCorpus = [
  {
    id: 1,
    content: "PostgreSQL connection pooling configured with max 25 connections and 5s timeout. Use Tokio async/await for I/O workers.",
    type: "decision",
    workspace: "backend",
    entities: ["PostgreSQL", "Tokio", "ConnectionPool", "Rust"]
  },
  {
    id: 2,
    content: "API authentication uses RS256 JWT tokens with public verification keys cached from HashiCorp Vault.",
    type: "architecture",
    workspace: "security",
    entities: ["Vault", "JWT", "RS256", "AuthService"]
  },
  {
    id: 3,
    content: "Docker container setup: Redis caching layer with 512MB RAM and eviction policy allkeys-lru.",
    type: "infra",
    workspace: "prod",
    entities: ["Docker", "Redis", "Cache"]
  },
  {
    id: 4,
    content: "Database migration scripts must run in atomic transactions with zero-downtime backwards compatibility.",
    type: "pattern",
    workspace: "backend",
    entities: ["PostgreSQL", "Migration", "Database"]
  }
];

function initPlayground() {
  const pgTabs = document.querySelectorAll('.pg-tab-btn');
  const pgOutput = document.getElementById('pg-json-output');
  const actionBtn = document.getElementById('pg-action-btn');

  if (!pgOutput || !actionBtn) return;

  let activeMode = 'wasm-search';

  pgTabs.forEach(btn => {
    btn.addEventListener('click', () => {
      pgTabs.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      activeMode = btn.getAttribute('data-mode');
      updatePlaygroundUI(activeMode);
    });
  });

  actionBtn.addEventListener('click', () => {
    executePlaygroundAction(activeMode);
  });
}

function updatePlaygroundUI(mode) {
  const inputContainer = document.getElementById('pg-inputs-dynamic');
  const actionBtn = document.getElementById('pg-action-btn');
  if (!inputContainer || !actionBtn) return;

  inputContainer.replaceChildren();

  function createInputGroup(labelText, inputElement) {
    const group = document.createElement('div');
    group.className = 'pg-input-group';
    const label = document.createElement('label');
    label.textContent = labelText;
    group.appendChild(label);
    group.appendChild(inputElement);
    return group;
  }

  if (mode === 'wasm-search') {
    actionBtn.textContent = 'Run WASM Hybrid Search';

    const qInput = document.createElement('input');
    qInput.type = 'text';
    qInput.id = 'pg-search-query';
    qInput.className = 'pg-input';
    qInput.value = 'postgres connection pooling tokio';
    inputContainer.appendChild(createInputGroup('Search Query (Live WASM BM25 + Vector + RRF)', qInput));

    const wsInput = document.createElement('input');
    wsInput.type = 'text';
    wsInput.id = 'pg-search-workspace';
    wsInput.className = 'pg-input';
    wsInput.value = 'backend';
    inputContainer.appendChild(createInputGroup('Workspace Filter', wsInput));

  } else if (mode === 'wasm-ner') {
    actionBtn.textContent = 'Run WASM Entity Extraction';

    const textInput = document.createElement('input');
    textInput.type = 'text';
    textInput.id = 'pg-ner-text';
    textInput.className = 'pg-input';
    textInput.value = 'Deploy @alex commit to https://api.engram.dev for PostgreSQL on 2026-08-19';
    inputContainer.appendChild(createInputGroup('Input Text for Entity Extraction', textInput));

  } else if (mode === 'wasm-graph') {
    actionBtn.textContent = 'Run WASM Graph Path Finding';

    const startSelect = document.createElement('select');
    startSelect.id = 'pg-graph-start';
    startSelect.className = 'pg-select';
    [
      { id: '1', name: 'Node 1 (AuthService)' },
      { id: '2', name: 'Node 2 (Vault)' },
      { id: '3', name: 'Node 3 (PostgreSQL)' },
      { id: '4', name: 'Node 4 (GatewayWorker)' }
    ].forEach(n => {
      const o = document.createElement('option');
      o.value = n.id;
      o.textContent = n.name;
      startSelect.appendChild(o);
    });
    inputContainer.appendChild(createInputGroup('Start Node', startSelect));

    const endSelect = document.createElement('select');
    endSelect.id = 'pg-graph-end';
    endSelect.className = 'pg-select';
    [
      { id: '3', name: 'Node 3 (PostgreSQL)' },
      { id: '2', name: 'Node 2 (Vault)' },
      { id: '4', name: 'Node 4 (GatewayWorker)' },
      { id: '1', name: 'Node 1 (AuthService)' }
    ].forEach(n => {
      const o = document.createElement('option');
      o.value = n.id;
      o.textContent = n.name;
      endSelect.appendChild(o);
    });
    inputContainer.appendChild(createInputGroup('Target Node', endSelect));

  } else if (mode === 'create') {
    actionBtn.textContent = 'Execute memory_create';

    const contentInput = document.createElement('input');
    contentInput.type = 'text';
    contentInput.id = 'pg-create-content';
    contentInput.className = 'pg-input';
    contentInput.value = 'Use AES-256-GCM encryption for cloud backup snapshots';
    inputContainer.appendChild(createInputGroup('Memory Content', contentInput));

    const typeSelect = document.createElement('select');
    typeSelect.id = 'pg-create-type';
    typeSelect.className = 'pg-select';
    [
      { val: 'decision', text: 'decision (Permanent)' },
      { val: 'architecture', text: 'architecture' },
      { val: 'daily', text: 'daily (24h Auto-Expire)' }
    ].forEach(opt => {
      const o = document.createElement('option');
      o.value = opt.val;
      o.textContent = opt.text;
      typeSelect.appendChild(o);
    });
    inputContainer.appendChild(createInputGroup('Memory Type', typeSelect));

    const wsInput = document.createElement('input');
    wsInput.type = 'text';
    wsInput.id = 'pg-create-workspace';
    wsInput.className = 'pg-input';
    wsInput.value = 'default';
    inputContainer.appendChild(createInputGroup('Workspace', wsInput));

  } else if (mode === 'contradiction') {
    actionBtn.textContent = 'Execute memory_temporal_contradictions';

    const wsInput = document.createElement('input');
    wsInput.type = 'text';
    wsInput.id = 'pg-contra-workspace';
    wsInput.className = 'pg-input';
    wsInput.value = 'prod';
    inputContainer.appendChild(createInputGroup('Target Workspace', wsInput));

    const scopeSelect = document.createElement('select');
    scopeSelect.className = 'pg-select';
    [
      { val: 'all', text: 'Full Workspace Conflict Analysis' },
      { val: 'recent', text: 'Past 7 Days' }
    ].forEach(opt => {
      const o = document.createElement('option');
      o.value = opt.val;
      o.textContent = opt.text;
      scopeSelect.appendChild(o);
    });
    inputContainer.appendChild(createInputGroup('Scan Scope', scopeSelect));
  }
}

function executePlaygroundAction(mode) {
  const output = document.getElementById('pg-json-output');
  const latencyBadge = document.getElementById('pg-latency-badge');
  if (!output) return;

  const startTime = performance.now();

  if (mode === 'wasm-search') {
    const query = document.getElementById('pg-search-query')?.value || 'postgres';
    const workspace = document.getElementById('pg-search-workspace')?.value || 'backend';

    if (wasmLoaded) {
      // 1. WASM BM25 tokenization
      const queryTokens = JSON.parse(wasm.bm25_tokenize(query));
      const queryVector = wasm.tfidf_embed(query, 384);
      const queryEntities = JSON.parse(wasm.extract_entities(query));

      const scoredDocs = inMemoryCorpus.map(doc => {
        const docTokens = JSON.parse(wasm.bm25_tokenize(doc.content));
        const bm25 = wasm.bm25_score(
          JSON.stringify(queryTokens),
          JSON.stringify(docTokens),
          inMemoryCorpus.length,
          15.0,
          1.5,
          0.75
        );
        const docVector = wasm.tfidf_embed(doc.content, 384);
        const cosine = wasm.cosine_similarity(queryVector, docVector);
        return {
          id: doc.id,
          content: doc.content,
          workspace: doc.workspace,
          bm25_score: parseFloat(bm25.toFixed(4)),
          vector_similarity: parseFloat(cosine.toFixed(4)),
          entities: doc.entities
        };
      });

      // Rank by BM25 and Vector
      const kwRanked = [...scoredDocs].sort((a, b) => b.bm25_score - a.bm25_score).map(d => d.id);
      const vecRanked = [...scoredDocs].sort((a, b) => b.vector_similarity - a.vector_similarity).map(d => d.id);

      // WASM RRF Fusion
      const rrfResults = JSON.parse(wasm.rrf_hybrid(
        JSON.stringify(kwRanked),
        JSON.stringify(vecRanked),
        1.0,
        1.0,
        60.0
      ));

      const rrfMap = new Map(rrfResults.map(r => [r.doc_id, r.score]));

      const finalHits = scoredDocs
        .map(d => ({
          ...d,
          rrf_fused_score: parseFloat((rrfMap.get(d.id) || 0).toFixed(4))
        }))
        .sort((a, b) => b.rrf_fused_score - a.rrf_fused_score);

      const elapsed = (performance.now() - startTime).toFixed(3);
      if (latencyBadge) latencyBadge.textContent = `WASM Latency: ${elapsed}ms`;

      const res = {
        engine: `engram-wasm v${wasm.version()} (Client-side Pure Rust)`,
        execution_environment: "WebAssembly in Browser",
        latency_ms: parseFloat(elapsed),
        query_analysis: {
          raw_query: query,
          bm25_tokens: queryTokens,
          entities_detected: queryEntities
        },
        fusion_algorithm: "Reciprocal Rank Fusion (RRF k=60)",
        matches_found: finalHits.length,
        results: finalHits
      };
      output.textContent = JSON.stringify(res, null, 2);
    } else {
      // Fallback
      const res = {
        status: "success",
        query,
        workspace,
        algorithm: "Reciprocal Rank Fusion (RRF)",
        results: inMemoryCorpus
      };
      output.textContent = JSON.stringify(res, null, 2);
    }

  } else if (mode === 'wasm-ner') {
    const text = document.getElementById('pg-ner-text')?.value || '';
    if (wasmLoaded) {
      const entities = JSON.parse(wasm.extract_entities(text));
      const elapsed = (performance.now() - startTime).toFixed(3);
      if (latencyBadge) latencyBadge.textContent = `WASM Latency: ${elapsed}ms`;

      const res = {
        engine: `engram-wasm v${wasm.version()} (Named Entity Recognition)`,
        input_text: text,
        latency_ms: parseFloat(elapsed),
        extracted_entities_count: entities.length,
        entities: entities
      };
      output.textContent = JSON.stringify(res, null, 2);
    }

  } else if (mode === 'wasm-graph') {
    const startNode = BigInt(document.getElementById('pg-graph-start')?.value || '1');
    const endNode = BigInt(document.getElementById('pg-graph-end')?.value || '3');

    const edges = [
      { from: 1, to: 2 }, // AuthService -> Vault
      { from: 1, to: 4 }, // AuthService -> GatewayWorker
      { from: 4, to: 3 }, // GatewayWorker -> PostgreSQL
      { from: 2, to: 3 }  // Vault -> PostgreSQL
    ];

    if (wasmLoaded) {
      const bfsTree = JSON.parse(wasm.graph_bfs(JSON.stringify(edges), startNode, 3));
      const shortestPath = JSON.parse(wasm.graph_shortest_path(JSON.stringify(edges), startNode, endNode));
      const elapsed = (performance.now() - startTime).toFixed(3);
      if (latencyBadge) latencyBadge.textContent = `WASM Latency: ${elapsed}ms`;

      const nodeLabels = {
        1: 'AuthService',
        2: 'Vault',
        3: 'PostgreSQL',
        4: 'GatewayWorker'
      };

      const res = {
        engine: `engram-wasm v${wasm.version()} (Graph Engine)`,
        start_node: { id: Number(startNode), label: nodeLabels[Number(startNode)] },
        target_node: { id: Number(endNode), label: nodeLabels[Number(endNode)] },
        latency_ms: parseFloat(elapsed),
        shortest_path_hops: shortestPath ? shortestPath.map(id => nodeLabels[id] || id) : null,
        bfs_traversal: bfsTree.map(item => ({
          node_id: item.node,
          label: nodeLabels[item.node] || item.node,
          depth: item.depth
        }))
      };
      output.textContent = JSON.stringify(res, null, 2);
    }

  } else if (mode === 'create') {
    const content = document.getElementById('pg-create-content')?.value || 'Sample memory';
    const type = document.getElementById('pg-create-type')?.value || 'decision';
    const ws = document.getElementById('pg-create-workspace')?.value || 'default';
    const newId = 'mem_' + Math.random().toString(36).substr(2, 6);

    let extractedEnts = ['AES-256-GCM', 'Snapshot', 'Encryption'];
    if (wasmLoaded) {
      const ents = JSON.parse(wasm.extract_entities(content));
      if (ents.length > 0) {
        extractedEnts = ents.map(e => e.normalized);
      }
    }

    const elapsed = (performance.now() - startTime).toFixed(3);
    if (latencyBadge) latencyBadge.textContent = `Latency: ${elapsed}ms`;

    const res = {
      jsonrpc: '2.0',
      result: {
        id: newId,
        content: content,
        memory_type: type,
        workspace: ws,
        salience_score: type === 'daily' ? 0.5 : 0.95,
        created_at: new Date().toISOString(),
        indexing: {
          bm25_fts5: 'indexed',
          vector_embedding: '384-dim (MiniLM ONNX + HNSW)',
          entities_extracted: extractedEnts
        }
      }
    };
    output.textContent = JSON.stringify(res, null, 2);

  } else if (mode === 'contradiction') {
    const elapsed = (performance.now() - startTime).toFixed(3);
    if (latencyBadge) latencyBadge.textContent = `Latency: ${elapsed}ms`;

    const res = {
      jsonrpc: '2.0',
      result: {
        workspace: 'prod',
        conflicts_detected: 0,
        status: 'CONSISTENT',
        checked_pairs: 48,
        temporal_anomalies: [],
        quality_score: 0.98,
        summary: 'All decisions and state assertions adhere to strict temporal order without conflicting directives.'
      }
    };
    output.textContent = JSON.stringify(res, null, 2);
  }
}

/* =========================================================================
   4. SDK Showcase Tabs
   ========================================================================= */
const sdkSnippets = {
  rust: `// Cargo.toml: engram-core = "0.24.0"
use engram_core::{EngramEngine, MemoryBuilder, SearchQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = EngramEngine::open_local("~/.local/share/engram/memories.db")?;

    // Create a permanent memory
    let memory_id = engine.store_memory(
        MemoryBuilder::new("PostgreSQL connection pool sized to 25 connections")
            .workspace("backend")
            .memory_type("decision")
            .salience_boost(0.95)
            .build()?
    ).await?;

    // 3-way hybrid search (BM25 + Vectors + Fuzzy)
    let results = engine.search(SearchQuery::new("postgres conn pool").workspace("backend")).await?;
    for hit in results {
        println!("Found [score {:.2}]: {}", hit.score, hit.content);
    }
    Ok(())
}`,

  python: `# pip install engram-client
import asyncio
from engram_client import EngramClient

async def main():
    async with EngramClient(base_url="http://localhost:8080") as client:
        # Create a structured project memory
        memory = await client.create(
            content="API keys are rotated daily via HashiCorp Vault",
            workspace="security",
            memory_type="decision"
        )
        print(f"Stored memory ID: {memory.id}")

        # Execute hybrid search with typo tolerance
        results = await client.search(
            query="hashicorp vault rotat",
            workspace="security"
        )
        for item in results.memories:
            print(f"[{item.score:.2f}] {item.content}")

if __name__ == "__main__":
    asyncio.run(main())`,

  typescript: `// npm install @aiconnai/engram-client
import { EngramClient } from '@aiconnai/engram-client';

async function main() {
  const client = new EngramClient({ baseUrl: 'http://localhost:8080' });

  // Store persistent context
  const memory = await client.create({
    content: 'User prefers dark mode and JetBrains Mono code font',
    workspace: 'user-preferences',
    memoryType: 'permanent'
  });

  // Hybrid search with entity resolution
  const searchResults = await client.search({
    query: 'user font preference',
    workspace: 'user-preferences'
  });

  console.log('Top match:', searchResults.matches[0]?.content);
}

main().catch(console.error);`,

  mcp: `// ~/.claude/mcp.json or .cursor/mcp.json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ENGRAM_DB_PATH": "~/.local/share/engram/memories.db",
        "ENGRAM_TOOL_TIER": "standard",
        "ENGRAM_EMBEDDING_MODEL": "local"
      }
    }
  }
}`,

  cli: `# Install Engram CLI via Homebrew or Cargo
brew install aiconnai/engram/engram

# Start persistent MCP stdio server
engram-server --transport stdio

# Direct CLI memory query & graph traversal
engram-cli search "database schema migration" --workspace backend
engram-cli graph traverse --entity "AuthService" --depth 2
engram-cli project-context scan .`
};

function initSdkTabs() {
  const tabBtns = document.querySelectorAll('.code-tab-btn');
  const codeDisplay = document.getElementById('sdk-code-display');
  if (!codeDisplay || !tabBtns.length) return;

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      tabBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const target = btn.getAttribute('data-lang');
      if (sdkSnippets[target]) {
        codeDisplay.textContent = sdkSnippets[target];
      }
    });
  });
}

/* =========================================================================
   5. Copy Buttons
   ========================================================================= */
function initCopyButtons() {
  const copyBtns = document.querySelectorAll('.copy-btn');
  copyBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const textToCopy = btn.getAttribute('data-clipboard-text') ||
                         btn.previousElementSibling?.textContent ||
                         '';
      if (textToCopy) {
        navigator.clipboard.writeText(textToCopy.trim()).then(() => {
          const originalHTML = btn.innerHTML;
          btn.replaceChildren();
          
          const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
          svg.setAttribute('width', '16');
          svg.setAttribute('height', '16');
          svg.setAttribute('viewBox', '0 0 24 24');
          svg.setAttribute('fill', 'none');
          svg.setAttribute('stroke', '#10b981');
          svg.setAttribute('stroke-width', '2');

          const polyline = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
          polyline.setAttribute('points', '20 6 9 17 4 12');
          svg.appendChild(polyline);
          btn.appendChild(svg);

          setTimeout(() => {
            btn.innerHTML = originalHTML;
          }, 2000);
        });
      }
    });
  });
}
