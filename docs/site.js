/**
 * Engram Official GitHub Pages Interactive Engine
 * Handles terminal simulation, comparison chat playback, interactive memory playground, SDK tabs, and code copying.
 */

document.addEventListener('DOMContentLoaded', () => {
  initTerminalSimulation();
  initComparisonDemo();
  initPlayground();
  initSdkTabs();
  initCopyButtons();
});

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
    { type: 'edge', rel: '(depends_on)', target: 'VaultSecrets', desc: ' [mem_12a0f7: "API keys rotated daily via Vault"]' },
    { type: 'edge', rel: '(stores_in)', target: 'PostgreSQL', desc: ' [mem_33b8c2: "User credentials stored with Argon2id"]' },
    { type: 'edge', rel: '(accessed_by)', target: 'GatewayWorker', desc: ' [mem_77d1e4: "JWT bearer validation executed at edge"]' },
    { type: 'break' },
    { type: 'dim', text: '  Resolved 4 nodes, 3 relations in 0.82ms. Shortest path to DB: 1 hop.' }
  ]
};

function renderTerminal(scenarioKey) {
  const terminalBody = document.getElementById('hero-terminal');
  if (!terminalBody) return;
  const items = terminalScenarios[scenarioKey] || [];
  terminalBody.replaceChildren();

  items.forEach(item => {
    if (item.type === 'break') {
      terminalBody.appendChild(document.createElement('br'));
      return;
    }
    const div = document.createElement('div');
    if (item.type === 'line') {
      const p = document.createElement('span');
      p.className = 't-prompt';
      p.textContent = item.prompt + ' ';
      const c = document.createElement('span');
      c.className = 't-cmd';
      c.textContent = item.cmd;
      div.appendChild(p);
      div.appendChild(c);
    } else if (item.type === 'banner') {
      div.className = 't-banner';
      div.textContent = item.text;
    } else if (item.type === 'dim') {
      div.className = 't-dim';
      div.textContent = item.text;
    } else if (item.type === 'cmd') {
      div.className = 't-cmd';
      div.textContent = item.text;
    } else if (item.type === 'success') {
      const tag = document.createElement('span');
      tag.className = 't-success';
      tag.textContent = item.tag;
      div.appendChild(tag);
      div.appendChild(document.createTextNode(item.text));
    } else if (item.type === 'highlight') {
      div.className = 't-highlight';
      div.appendChild(document.createTextNode(item.tag + ' '));
      const s = document.createElement('span');
      s.className = 't-cyan';
      s.textContent = item.score;
      div.appendChild(s);
      div.appendChild(document.createTextNode(item.detail));
    } else if (item.type === 'purple') {
      const tag = document.createElement('span');
      tag.className = 't-purple';
      tag.textContent = item.tag + ' ';
      const n = document.createElement('span');
      n.className = 't-cyan';
      n.textContent = item.name;
      div.appendChild(tag);
      div.appendChild(n);
      div.appendChild(document.createTextNode(item.detail));
    } else if (item.type === 'edge') {
      div.appendChild(document.createTextNode('  ├── '));
      const rel = document.createElement('span');
      rel.className = 't-success';
      rel.textContent = item.rel;
      div.appendChild(rel);
      div.appendChild(document.createTextNode(' ──> '));
      const target = document.createElement('span');
      target.className = 't-cyan';
      target.textContent = item.target;
      div.appendChild(target);
      div.appendChild(document.createTextNode(item.desc));
    }
    terminalBody.appendChild(div);
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
   3. Interactive Memory Playground (In-Browser Simulation)
   ========================================================================= */
function initPlayground() {
  const pgTabs = document.querySelectorAll('.pg-tab-btn');
  const pgOutput = document.getElementById('pg-json-output');
  const actionBtn = document.getElementById('pg-action-btn');

  if (!pgOutput || !actionBtn) return;

  let activeMode = 'create';

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

  if (mode === 'create') {
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

  } else if (mode === 'search') {
    actionBtn.textContent = 'Execute memory_search';

    const qInput = document.createElement('input');
    qInput.type = 'text';
    qInput.id = 'pg-search-query';
    qInput.className = 'pg-input';
    qInput.value = 'postgre connection pooling';
    inputContainer.appendChild(createInputGroup('Search Query (supports typos & semantic concepts)', qInput));

    const wsInput = document.createElement('input');
    wsInput.type = 'text';
    wsInput.id = 'pg-search-workspace';
    wsInput.className = 'pg-input';
    wsInput.value = 'prod';
    inputContainer.appendChild(createInputGroup('Workspace Filter', wsInput));

  } else if (mode === 'traverse') {
    actionBtn.textContent = 'Execute memory_traverse';

    const entSelect = document.createElement('select');
    entSelect.id = 'pg-traverse-entity';
    entSelect.className = 'pg-select';
    ['AuthService', 'PostgreSQL', 'Vault'].forEach(ent => {
      const o = document.createElement('option');
      o.value = ent;
      o.textContent = ent;
      entSelect.appendChild(o);
    });
    inputContainer.appendChild(createInputGroup('Target Entity / Identity', entSelect));

    const depthInput = document.createElement('input');
    depthInput.type = 'number';
    depthInput.id = 'pg-traverse-depth';
    depthInput.className = 'pg-input';
    depthInput.value = '2';
    depthInput.min = '1';
    depthInput.max = '5';
    inputContainer.appendChild(createInputGroup('Max Traversal Depth', depthInput));

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
  if (!output) return;

  if (mode === 'create') {
    const content = document.getElementById('pg-create-content')?.value || 'Sample memory';
    const type = document.getElementById('pg-create-type')?.value || 'decision';
    const ws = document.getElementById('pg-create-workspace')?.value || 'default';
    const newId = 'mem_' + Math.random().toString(36).substr(2, 6);

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
          entities_extracted: ['AES-256-GCM', 'Snapshot', 'Encryption']
        }
      }
    };
    output.textContent = JSON.stringify(res, null, 2);
  } else if (mode === 'search') {
    const query = document.getElementById('pg-search-query')?.value || 'query';
    const res = {
      jsonrpc: '2.0',
      result: {
        query: query,
        matches_found: 2,
        fusion_algorithm: 'Reciprocal Rank Fusion (RRF)',
        results: [
          {
            id: 'mem_01',
            content: 'Database migration: PostgreSQL connection pool size set to 25 with 5s timeout.',
            relevance_score: 0.964,
            breakdown: { bm25: 0.92, vector: 0.98, fuzzy: 0.89 },
            salience: 0.95,
            workspace: 'prod'
          },
          {
            id: 'mem_02',
            content: 'API authentication uses JWT with RS256 signing keys stored in Vault.',
            relevance_score: 0.412,
            breakdown: { bm25: 0.12, vector: 0.54, fuzzy: 0.30 },
            salience: 0.92,
            workspace: 'prod'
          }
        ]
      }
    };
    output.textContent = JSON.stringify(res, null, 2);
  } else if (mode === 'traverse') {
    const entity = document.getElementById('pg-traverse-entity')?.value || 'AuthService';
    const res = {
      jsonrpc: '2.0',
      result: {
        root_entity: entity,
        canonical_id: 'ident_' + entity.toLowerCase(),
        aliases: [entity, entity.toLowerCase() + '-core', 'idp-handler'],
        relations: [
          { type: 'uses_secret_store', target: 'Vault', confidence: 0.99, source_memory: 'mem_02' },
          { type: 'validates_tokens_for', target: 'GatewayWorker', confidence: 0.94, source_memory: 'mem_77d1e4' },
          { type: 'persists_to', target: 'PostgreSQL', confidence: 0.97, source_memory: 'mem_01' }
        ],
        graph_diameter: 2,
        latency_ms: 0.74
      }
    };
    output.textContent = JSON.stringify(res, null, 2);
  } else if (mode === 'contradiction') {
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

    // 3-way hybrid search (BM25 + HNSW Vectors + Fuzzy)
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
        memory = await client.create_memory(
            content="API keys are rotated daily via HashiCorp Vault",
            workspace="security",
            memory_type="decision"
        )
        print(f"Stored memory ID: {memory.id}")

        # Execute hybrid search with typo tolerance
        results = await client.search_memory(
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
  const memory = await client.memories.create({
    content: 'User prefers dark mode and JetBrains Mono code font',
    workspace: 'user-preferences',
    memoryType: 'permanent'
  });

  // Hybrid search with entity resolution
  const searchResults = await client.search.query({
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
          btn.classList.add('copied');
          setTimeout(() => {
            btn.classList.remove('copied');
          }, 2000);
        });
      }
    });
  });
}
