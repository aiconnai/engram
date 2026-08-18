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
    chatForget.innerHTML = '';
    chatRemember.innerHTML = '';

    // Step 1: User message appears on both sides
    const userMsg1 = document.createElement('div');
    userMsg1.className = 'chat-bubble user';
    userMsg1.textContent = chatScript.userPrompt;

    const userMsg2 = document.createElement('div');
    userMsg2.className = 'chat-bubble user';
    userMsg2.textContent = chatScript.userPrompt;

    chatForget.appendChild(userMsg1);
    chatRemember.appendChild(userMsg2);

    // Step 2: Agent responses with typing effect
    setTimeout(() => {
      const agentForget = document.createElement('div');
      agentForget.className = 'chat-bubble agent failure';
      agentForget.innerHTML = `<strong>Claude (without memory):</strong><br>${chatScript.forgetAgent}`;
      chatForget.appendChild(agentForget);

      const agentRemember = document.createElement('div');
      agentRemember.className = 'chat-bubble agent success';
      agentRemember.innerHTML = `<strong>Claude (with Engram MCP):</strong><br>${chatScript.rememberAgent.replace(/\n/g, '<br>')}`;
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
  mcp: `<div><span class="t-prompt">$</span> <span class="t-cmd">engram-server --transport stdio</span></div>
<div class="t-banner">  Engram v0.24.0 (local-first persistent memory server)</div>
<div class="t-dim">  Database: ~/.local/share/engram/memories.db [SQLite 3.45 + WAL]</div>
<div class="t-dim">  Search: Hybrid BM25 (FTS5) + Cosine Vectors (MiniLM ONNX) + Fuzzy Levenshtein</div>
<div class="t-dim">  MCP Protocol: JSON-RPC 2.0 active over stdio [243 tools registered]</div>
<br>
<div><span class="t-success">[MCP INITIALIZED]</span> Handshake from host: <span class="t-cyan">Claude Code v1.0.18</span></div>
<div class="t-dim">  Capabilities: tools, resources, prompts, logging, completions</div>
<div><span class="t-success">[READY]</span> Loaded 1,482 project memories across 4 workspaces in 1.4ms</div>`,

  search: `<div><span class="t-prompt">$</span> <span class="t-cmd">engram-cli search "asynch awiat rust" --workspace backend --explain</span></div>
<div class="t-dim">  Executing 3-way hybrid search with Reciprocal Rank Fusion (RRF)...</div>
<br>
<div class="t-highlight">[MATCH 1] <span class="t-cyan">Score: 0.982</span> (BM25: 0.94, Vector: 0.99, Fuzzy: 0.92)</div>
<div class="t-dim">  ID: mem_98f4a1 | Workspace: backend | Type: decision | Salience: 0.95</div>
<div>  <span class="t-cmd">"Use Tokio async/await for all I/O-bound workers; reserve std::thread for CPU compute."</span></div>
<div class="t-dim">  Entities: [Tokio, Rust, Async/Await] · Provenance: session_2026_07_14.jsonl</div>
<br>
<div class="t-highlight">[MATCH 2] <span class="t-cyan">Score: 0.874</span> (BM25: 0.82, Vector: 0.91, Fuzzy: 0.79)</div>
<div class="t-dim">  ID: mem_42c8d9 | Workspace: backend | Type: pattern | Salience: 0.81</div>
<div>  <span class="t-cmd">"Async channel buffer depth must be bounded at 1024 messages to prevent memory ballooning."</span></div>`,

  graph: `<div><span class="t-prompt">$</span> <span class="t-cmd">engram-cli graph traverse --entity "AuthService" --depth 2 --output json</span></div>
<div class="t-dim">  Traversing knowledge graph from canonical identity 'AuthService'...</div>
<br>
<div><span class="t-purple">[ENTITY]</span> <span class="t-cyan">AuthService</span> (Aliases: [auth-api, authentication-worker, idp-client])</div>
<div>  ├── <span class="t-success">(depends_on)</span> ──> <span class="t-cyan">VaultSecrets</span> [mem_12a0f7: "API keys are rotated daily via HashiCorp Vault"]</div>
<div>  ├── <span class="t-success">(stores_in)</span> ───> <span class="t-cyan">PostgreSQL</span> [mem_33b8c2: "User credentials stored with Argon2id hash"]</div>
<div>  └── <span class="t-success">(accessed_by)</span> ─> <span class="t-cyan">GatewayWorker</span> [mem_77d1e4: "JWT bearer validation executed at edge proxy"]</div>
<br>
<div class="t-dim">  Resolved 4 nodes, 3 relations in 0.82ms. Shortest path to DB: 1 hop.</div>`
};

function initTerminalSimulation() {
  const terminalBody = document.getElementById('hero-terminal');
  const tabBtns = document.querySelectorAll('.terminal-tab-btn');
  if (!terminalBody || !tabBtns.length) return;

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      tabBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const target = btn.getAttribute('data-tab');
      if (terminalScenarios[target]) {
        terminalBody.innerHTML = terminalScenarios[target];
      }
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

  if (mode === 'create') {
    actionBtn.textContent = 'Execute memory_create';
    inputContainer.innerHTML = `
      <div class="pg-input-group">
        <label>Memory Content</label>
        <input type="text" id="pg-create-content" class="pg-input" value="Use AES-256-GCM encryption for cloud backup snapshots" />
      </div>
      <div class="pg-input-group">
        <label>Memory Type</label>
        <select id="pg-create-type" class="pg-select">
          <option value="decision" selected>decision (Permanent)</option>
          <option value="architecture">architecture</option>
          <option value="daily">daily (24h Auto-Expire)</option>
        </select>
      </div>
      <div class="pg-input-group">
        <label>Workspace</label>
        <input type="text" id="pg-create-workspace" class="pg-input" value="default" />
      </div>
    `;
  } else if (mode === 'search') {
    actionBtn.textContent = 'Execute memory_search';
    inputContainer.innerHTML = `
      <div class="pg-input-group">
        <label>Search Query (supports typos & semantic concepts)</label>
        <input type="text" id="pg-search-query" class="pg-input" value="postgre connection pooling" />
      </div>
      <div class="pg-input-group">
        <label>Workspace Filter</label>
        <input type="text" id="pg-search-workspace" class="pg-input" value="prod" />
      </div>
    `;
  } else if (mode === 'traverse') {
    actionBtn.textContent = 'Execute memory_traverse';
    inputContainer.innerHTML = `
      <div class="pg-input-group">
        <label>Target Entity / Identity</label>
        <select id="pg-traverse-entity" class="pg-select">
          <option value="AuthService">AuthService</option>
          <option value="PostgreSQL">PostgreSQL</option>
          <option value="Vault">Vault</option>
        </select>
      </div>
      <div class="pg-input-group">
        <label>Max Traversal Depth</label>
        <input type="number" id="pg-traverse-depth" class="pg-input" value="2" min="1" max="5" />
      </div>
    `;
  } else if (mode === 'contradiction') {
    actionBtn.textContent = 'Execute memory_temporal_contradictions';
    inputContainer.innerHTML = `
      <div class="pg-input-group">
        <label>Target Workspace</label>
        <input type="text" id="pg-contra-workspace" class="pg-input" value="prod" />
      </div>
      <div class="pg-input-group">
        <label>Scan Scope</label>
        <select class="pg-select">
          <option value="all">Full Workspace Conflict Analysis</option>
          <option value="recent">Past 7 Days</option>
        </select>
      </div>
    `;
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
          vector_embedding: '384-dim (MiniLM ONNX)',
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
          const originalHTML = btn.innerHTML;
          btn.innerHTML = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>`;
          setTimeout(() => {
            btn.innerHTML = originalHTML;
          }, 2000);
        });
      }
    });
  });
}
