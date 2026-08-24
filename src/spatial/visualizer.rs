//! Mnemonic Palace topological visualizer (Method of Loci).
//! Extracts and renders spatial memory hierarchies into HTML, ASCII, SVG, Mermaid, and JSON.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use crate::error::{EngramError, Result};
use crate::storage::Storage;

/// Visualizer export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PalaceFormat {
    Html,
    Ascii,
    Svg,
    Mermaid,
    Json,
}

impl std::str::FromStr for PalaceFormat {
    type Err = EngramError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "html" => Ok(PalaceFormat::Html),
            "ascii" | "text" | "txt" => Ok(PalaceFormat::Ascii),
            "svg" => Ok(PalaceFormat::Svg),
            "mermaid" | "mmd" => Ok(PalaceFormat::Mermaid),
            "json" => Ok(PalaceFormat::Json),
            other => Err(EngramError::InvalidInput(format!(
                "Invalid palace visualizer format '{}'. Expected: html, ascii, svg, mermaid, json",
                other
            ))),
        }
    }
}

/// An individual memory item inside a spatial room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceDrawer {
    pub id: i64,
    pub title: String,
    pub preview: String,
    pub memory_type: String,
    pub importance: f64,
    pub tags: Vec<String>,
    pub scope_path: String,
    pub created_at: String,
}

/// A room inside a spatial wing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceRoom {
    pub name: String,
    pub full_path: String,
    pub drawer_count: usize,
    pub drawers: Vec<PalaceDrawer>,
}

/// A major functional wing inside the memory palace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceWing {
    pub name: String,
    pub drawer_count: usize,
    pub room_count: usize,
    pub rooms: Vec<PalaceRoom>,
}

/// The complete hierarchical topological palace graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceGraph {
    pub workspace: String,
    pub total_drawers: usize,
    pub wings_count: usize,
    pub rooms_count: usize,
    pub wings: Vec<PalaceWing>,
}

impl PalaceGraph {
    /// Extract the spatial palace graph from storage for a given workspace and optional wing filter.
    pub fn extract(storage: &Storage, workspace: &str, target_wing: Option<&str>) -> Result<Self> {
        storage.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT m.id, m.content, m.memory_type, m.importance,
                        COALESCE(GROUP_CONCAT(t.name, ','), '') as tags_str,
                        m.scope_path, m.scope_type, m.scope_id, m.created_at
                 FROM memories m
                 LEFT JOIN memory_tags mt ON m.id = mt.memory_id
                 LEFT JOIN tags t ON mt.tag_id = t.id
                 WHERE m.workspace = ?1
                   AND COALESCE(m.lifecycle_state, 'active') != 'archived'
                   AND m.valid_to IS NULL
                 GROUP BY m.id
                 ORDER BY COALESCE(m.scope_path, m.scope_id, 'global'), m.importance DESC, m.id DESC",
            )?;

            let rows = stmt.query_map(rusqlite::params![workspace], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                let memory_type: String = row.get(2)?;
                let importance: f64 = row.get(3).unwrap_or(0.5);
                let tags_str: String = row.get(4).unwrap_or_default();
                let scope_path: Option<String> = row.get(5)?;
                let scope_type: Option<String> = row.get(6)?;
                let scope_id: Option<String> = row.get(7)?;
                let created_at: String = row.get(8).unwrap_or_default();

                let tags: Vec<String> = if tags_str.starts_with('[') {
                    serde_json::from_str(&tags_str).unwrap_or_default()
                } else if !tags_str.is_empty() {
                    tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
                } else {
                    Vec::new()
                };

                let effective_scope = match (scope_path.as_deref(), scope_type.as_deref(), scope_id.as_deref()) {
                    (Some(p), _, _) if p != "global" && !p.is_empty() => p.to_string(),
                    (_, Some("custom"), Some(id)) if !id.is_empty() => id.to_string(),
                    (_, _, Some(id)) if id.contains("wing:") || id.contains('/') => id.to_string(),
                    (Some(p), _, _) => p.to_string(),
                    _ => "global".to_string(),
                };

                let title = content.lines().next().unwrap_or("Untitled").chars().take(80).collect();
                let preview = content.chars().take(300).collect();

                Ok(PalaceDrawer {
                    id,
                    title,
                    preview,
                    memory_type,
                    importance,
                    tags,
                    scope_path: effective_scope,
                    created_at,
                })
            })?;

            // Grouping: Wing -> Room -> Vec<PalaceDrawer>
            let mut wings_map: BTreeMap<String, BTreeMap<String, Vec<PalaceDrawer>>> = BTreeMap::new();
            let mut total_drawers = 0;

            for row in rows {
                let drawer = row?;
                total_drawers += 1;

                let clean = drawer.scope_path.trim_start_matches("wing:").trim_start_matches('/');
                let mut parts = clean.split('/');
                let wing = parts.next().unwrap_or("general").to_string();
                let room = parts.next().map(|r| r.trim_start_matches("room:").to_string()).unwrap_or_else(|| "main".to_string());

                if let Some(tw) = target_wing {
                    if !wing.eq_ignore_ascii_case(tw) && !drawer.scope_path.contains(tw) {
                        continue;
                    }
                }

                wings_map.entry(wing).or_default().entry(room).or_default().push(drawer);
            }

            let mut wings = Vec::new();
            let mut rooms_count = 0;

            for (wing_name, rooms_map) in wings_map {
                let mut rooms = Vec::new();
                let mut wing_drawer_count = 0;

                for (room_name, drawers) in rooms_map {
                    rooms_count += 1;
                    wing_drawer_count += drawers.len();
                    rooms.push(PalaceRoom {
                        name: room_name.clone(),
                        full_path: format!("wing:{}/room:{}", wing_name, room_name),
                        drawer_count: drawers.len(),
                        drawers,
                    });
                }

                wings.push(PalaceWing {
                    name: wing_name,
                    drawer_count: wing_drawer_count,
                    room_count: rooms.len(),
                    rooms,
                });
            }

            let wings_count = wings.len();

            Ok(PalaceGraph {
                workspace: workspace.to_string(),
                total_drawers,
                wings_count,
                rooms_count,
                wings,
            })
        })
    }

    /// Render the palace in the specified format.
    pub fn render(&self, format: PalaceFormat) -> String {
        match format {
            PalaceFormat::Html => self.render_html(),
            PalaceFormat::Ascii => self.render_ascii(),
            PalaceFormat::Svg => self.render_svg(),
            PalaceFormat::Mermaid => self.render_mermaid(),
            PalaceFormat::Json => {
                serde_json::to_string_pretty(&self.render_json()).unwrap_or_default()
            }
        }
    }

    /// Render terminal/LLM friendly ASCII floorplan tree.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "🏰 [MEMORY PALACE: {}] -- {} Wings, {} Rooms, {} Total Drawers\n",
            self.workspace, self.wings_count, self.rooms_count, self.total_drawers
        ));
        out.push_str("═".repeat(78).as_str());
        out.push('\n');

        if self.wings.is_empty() {
            out.push_str("  (Palace is currently empty. Mine transcripts or assign scope paths to populate wings/rooms.)\n");
            return out;
        }

        for (w_idx, wing) in self.wings.iter().enumerate() {
            let is_last_wing = w_idx + 1 == self.wings.len();
            let wing_branch = if is_last_wing {
                "└──"
            } else {
                "├──"
            };
            let wing_pipe = if is_last_wing { "   " } else { "│  " };

            let bar_len = wing.drawer_count.clamp(1, 20);
            let meter = "█".repeat(bar_len);

            out.push_str(&format!(
                "{} 🏛️  WING: {:<16} [{} rooms, {:>3} drawers] {}\n",
                wing_branch, wing.name, wing.room_count, wing.drawer_count, meter
            ));

            for (r_idx, room) in wing.rooms.iter().enumerate() {
                let is_last_room = r_idx + 1 == wing.rooms.len();
                let room_branch = if is_last_room {
                    "└──"
                } else {
                    "├──"
                };
                let room_pipe = if is_last_room { "   " } else { "│  " };

                out.push_str(&format!(
                    "{}{} 🚪 Room: {:<14} ({:>2} drawers)\n",
                    wing_pipe, room_branch, room.name, room.drawer_count
                ));

                for (d_idx, drawer) in room.drawers.iter().take(5).enumerate() {
                    let is_last_d = d_idx + 1 == room.drawers.len().min(5);
                    let d_branch = if is_last_d && room.drawers.len() <= 5 {
                        "└──"
                    } else {
                        "├──"
                    };
                    let imp_stars = if drawer.importance >= 0.8 {
                        "★"
                    } else {
                        "·"
                    };
                    out.push_str(&format!(
                        "{}{}{} 📦 [#{:<4} | {:<8}] {} {}\n",
                        wing_pipe,
                        room_pipe,
                        d_branch,
                        drawer.id,
                        drawer.memory_type,
                        imp_stars,
                        drawer.title
                    ));
                }

                if room.drawers.len() > 5 {
                    out.push_str(&format!(
                        "{}{}{} ... (+{} more drawers in room)\n",
                        wing_pipe,
                        room_pipe,
                        "└──",
                        room.drawers.len() - 5
                    ));
                }
            }
            out.push('\n');
        }

        out
    }

    /// Render Mermaid mindmap / flowchart syntax.
    pub fn render_mermaid(&self) -> String {
        let mut out = String::new();
        out.push_str("mindmap\n");
        out.push_str(&format!("  root((🏰 {}))\n", self.workspace));

        for wing in &self.wings {
            let safe_wing = wing.name.replace('(', "[").replace(')', "]");
            out.push_str(&format!(
                "    🏛️ \"{} ({} drawers)\"\n",
                safe_wing, wing.drawer_count
            ));
            for room in &wing.rooms {
                let safe_room = room.name.replace('(', "[").replace(')', "]");
                out.push_str(&format!(
                    "      🚪 \"{} ({})\"\n",
                    safe_room, room.drawer_count
                ));
                for drawer in room.drawers.iter().take(3) {
                    let safe_title = drawer
                        .title
                        .chars()
                        .take(30)
                        .collect::<String>()
                        .replace('"', "'")
                        .replace('(', "[")
                        .replace(')', "]");
                    out.push_str(&format!("        📦 \"#{} - {}\"\n", drawer.id, safe_title));
                }
                if room.drawers.len() > 3 {
                    out.push_str(&format!("        \"+{} more\"\n", room.drawers.len() - 3));
                }
            }
        }

        out
    }

    /// Render SVG diagram of the memory palace topology.
    pub fn render_svg(&self) -> String {
        let width = 1000;
        let wing_height = 140;
        let height = 100 + (self.wings.len().max(1) * wing_height) + 40;

        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="100%" height="100%" style="background-color:#0f172a; font-family:system-ui, -apple-system, sans-serif;">
<defs>
  <linearGradient id="palaceGrad" x1="0%" y1="0%" x2="100%" y2="100%">
    <stop offset="0%" stop-color="#3b82f6"/>
    <stop offset="100%" stop-color="#8b5cf6"/>
  </linearGradient>
  <linearGradient id="wingGrad" x1="0%" y1="0%" x2="100%" y2="0%">
    <stop offset="0%" stop-color="#1e293b"/>
    <stop offset="100%" stop-color="#334155"/>
  </linearGradient>
</defs>
<rect x="20" y="20" width="{header_w}" height="50" rx="10" fill="url(#palaceGrad)"/>
<text x="40" y="52" fill="#ffffff" font-size="20" font-weight="bold">🏰 Palace: {ws} ({total} drawers in {w_cnt} wings)</text>
"##,
            width = width,
            height = height,
            header_w = width - 40,
            ws = self.workspace,
            total = self.total_drawers,
            w_cnt = self.wings_count
        );

        let mut cur_y = 90;
        for wing in &self.wings {
            svg.push_str(&format!(
                r##"<g transform="translate(20, {y})">
  <rect x="0" y="0" width="{w}" height="120" rx="8" fill="url(#wingGrad)" stroke="#475569" stroke-width="1.5"/>
  <text x="20" y="30" fill="#38bdf8" font-size="16" font-weight="bold">🏛️ Wing: {wing_name} ({drawers} drawers)</text>
"##,
                y = cur_y,
                w = width - 40,
                wing_name = wing.name,
                drawers = wing.drawer_count
            ));

            let mut room_x = 20;
            for room in &wing.rooms {
                let room_w = (room.name.len() * 9 + 40).max(100);
                if room_x + room_w > width - 60 {
                    break;
                }
                svg.push_str(&format!(
                    r##"  <g transform="translate({rx}, 45)">
    <rect x="0" y="0" width="{rw}" height="55" rx="6" fill="#0f172a" stroke="#64748b" stroke-width="1"/>
    <text x="10" y="24" fill="#e2e8f0" font-size="13" font-weight="600">🚪 {rname}</text>
    <text x="10" y="44" fill="#94a3b8" font-size="11">📦 {rdrawers} drawers</text>
  </g>
"##,
                    rx = room_x,
                    rw = room_w,
                    rname = room.name,
                    rdrawers = room.drawer_count
                ));
                room_x += room_w + 15;
            }

            svg.push_str("</g>\n");
            cur_y += wing_height;
        }

        svg.push_str("</svg>\n");
        svg
    }

    /// Render Cytoscape / D3 compatible JSON graph schema.
    pub fn render_json(&self) -> Value {
        json!({
            "palace": self.workspace,
            "total_drawers": self.total_drawers,
            "wings_count": self.wings_count,
            "rooms_count": self.rooms_count,
            "wings": self.wings
        })
    }

    /// Render interactive HTML5/Canvas visualization application.
    pub fn render_html(&self) -> String {
        let palace_json = serde_json::to_string(&self).unwrap_or_default();
        format!(
            r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Mnemonic Palace: {}</title>
  <style>
    :root {{
      --bg: #0b0f19;
      --card-bg: #151e2e;
      --card-border: #243248;
      --text: #f1f5f9;
      --text-muted: #94a3b8;
      --accent: #6366f1;
      --accent-glow: rgba(99, 102, 241, 0.2);
      --wing-border: #3b82f6;
      --room-bg: #0f172a;
      --drawer-bg: #1e293b;
      --drawer-hover: #334155;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      overflow-x: hidden;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }}
    header {{
      background: var(--card-bg);
      border-bottom: 1px solid var(--card-border);
      padding: 16px 24px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      position: sticky;
      top: 0;
      z-index: 100;
    }}
    .header-title {{
      display: flex;
      align-items: center;
      gap: 12px;
    }}
    .header-title h1 {{ font-size: 20px; font-weight: 700; color: #fff; }}
    .badge {{
      background: var(--accent-glow);
      color: #818cf8;
      padding: 4px 10px;
      border-radius: 9999px;
      font-size: 12px;
      font-weight: 600;
      border: 1px solid var(--accent);
    }}
    .controls {{
      display: flex;
      gap: 12px;
      align-items: center;
    }}
    .search-input {{
      background: var(--bg);
      border: 1px solid var(--card-border);
      color: var(--text);
      padding: 8px 16px;
      border-radius: 8px;
      font-size: 14px;
      width: 260px;
      outline: none;
      transition: border-color 0.2s;
    }}
    .search-input:focus {{ border-color: var(--accent); }}
    main {{
      flex: 1;
      padding: 24px;
      display: grid;
      grid-template-columns: 280px 1fr;
      gap: 24px;
      max-width: 1600px;
      margin: 0 auto;
      width: 100%;
    }}
    .sidebar {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 16px;
      height: fit-content;
      position: sticky;
      top: 88px;
    }}
    .sidebar h2 {{ font-size: 14px; text-transform: uppercase; color: var(--text-muted); margin-bottom: 12px; letter-spacing: 0.05em; }}
    .nav-item {{
      display: flex;
      justify-content: space-between;
      padding: 10px 12px;
      border-radius: 8px;
      cursor: pointer;
      color: var(--text);
      font-size: 14px;
      font-weight: 500;
      transition: background 0.15s;
      margin-bottom: 4px;
    }}
    .nav-item:hover, .nav-item.active {{ background: var(--drawer-hover); color: #38bdf8; }}
    .nav-count {{ background: var(--bg); padding: 2px 8px; border-radius: 6px; font-size: 12px; color: var(--text-muted); }}
    .wings-container {{
      display: flex;
      flex-direction: column;
      gap: 24px;
    }}
    .wing-card {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-left: 4px solid var(--wing-border);
      border-radius: 12px;
      padding: 20px;
      box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    }}
    .wing-header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
    }}
    .wing-title {{
      font-size: 18px;
      font-weight: 700;
      color: #38bdf8;
      display: flex;
      align-items: center;
      gap: 8px;
    }}
    .rooms-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
      gap: 16px;
    }}
    .room-card {{
      background: var(--room-bg);
      border: 1px solid var(--card-border);
      border-radius: 10px;
      padding: 16px;
    }}
    .room-header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;
      border-bottom: 1px solid rgba(255,255,255,0.05);
      padding-bottom: 8px;
    }}
    .room-title {{ font-size: 15px; font-weight: 600; color: #e2e8f0; }}
    .drawers-list {{
      display: flex;
      flex-direction: column;
      gap: 8px;
    }}
    .drawer-item {{
      background: var(--drawer-bg);
      border: 1px solid rgba(255,255,255,0.04);
      border-radius: 6px;
      padding: 10px 12px;
      cursor: pointer;
      transition: transform 0.15s, border-color 0.15s;
    }}
    .drawer-item:hover {{
      transform: translateX(4px);
      border-color: var(--accent);
      background: var(--drawer-hover);
    }}
    .drawer-header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 4px;
    }}
    .drawer-type {{
      font-size: 11px;
      text-transform: uppercase;
      font-weight: 700;
      color: #a78bfa;
    }}
    .drawer-title {{
      font-size: 13px;
      font-weight: 500;
      color: #f1f5f9;
      line-height: 1.4;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }}
    .drawer-tags {{
      display: flex;
      gap: 4px;
      margin-top: 6px;
      flex-wrap: wrap;
    }}
    .tag {{
      background: rgba(255,255,255,0.06);
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 10px;
      color: var(--text-muted);
    }}
    /* Modal */
    .modal-overlay {{
      position: fixed;
      inset: 0;
      background: rgba(0,0,0,0.7);
      backdrop-filter: blur(4px);
      display: none;
      justify-content: center;
      align-items: center;
      z-index: 1000;
    }}
    .modal-overlay.open {{ display: flex; }}
    .modal {{
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 14px;
      width: 90%;
      max-width: 640px;
      max-height: 85vh;
      display: flex;
      flex-direction: column;
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
    }}
    .modal-header {{
      padding: 16px 20px;
      border-bottom: 1px solid var(--card-border);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .modal-body {{
      padding: 20px;
      overflow-y: auto;
      font-size: 14px;
      line-height: 1.6;
      color: #cbd5e1;
      white-space: pre-wrap;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }}
    .modal-close {{
      background: none;
      border: none;
      color: var(--text-muted);
      font-size: 20px;
      cursor: pointer;
    }}
    .modal-close:hover {{ color: #fff; }}
  </style>
</head>
<body>
  <header>
    <div class="header-title">
      <h1>🏰 Memory Palace: <span style="color:#818cf8;">{}</span></h1>
      <span class="badge">{} Wings · {} Rooms · {} Drawers</span>
    </div>
    <div class="controls">
      <input type="text" id="searchInput" class="search-input" placeholder="🔍 Search palace drawers..." />
    </div>
  </header>

  <main>
    <aside class="sidebar">
      <h2>Wings Directory</h2>
      <div id="sidebarList"></div>
    </aside>

    <section class="wings-container" id="wingsContainer"></section>
  </main>

  <div class="modal-overlay" id="modalOverlay" onclick="closeModal(event)">
    <div class="modal" onclick="event.stopPropagation()">
      <div class="modal-header">
        <h3 id="modalTitle" style="font-size:16px; font-weight:700;"></h3>
        <button class="modal-close" onclick="closeModalDirect()">&times;</button>
      </div>
      <div class="modal-body" id="modalBody"></div>
    </div>
  </div>

  <script>
    const data = {};

    function render(filter = '') {{
      const query = filter.toLowerCase();
      const sidebar = document.getElementById('sidebarList');
      const container = document.getElementById('wingsContainer');
      sidebar.innerHTML = '';
      container.innerHTML = '';

      // All wings option
      const allItem = document.createElement('div');
      allItem.className = 'nav-item active';
      allItem.innerHTML = `<span>🏰 All Wings</span><span class="nav-count">${{data.total_drawers}}</span>`;
      allItem.onclick = () => {{
        document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
        allItem.classList.add('active');
        document.querySelectorAll('.wing-card').forEach(c => c.style.display = 'block');
      }};
      sidebar.appendChild(allItem);

      data.wings.forEach(wing => {{
        // Sidebar item
        const nav = document.createElement('div');
        nav.className = 'nav-item';
        nav.innerHTML = `<span>🏛️ ${{wing.name}}</span><span class="nav-count">${{wing.drawer_count}}</span>`;
        nav.onclick = () => {{
          document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
          nav.classList.add('active');
          document.querySelectorAll('.wing-card').forEach(c => {{
            c.style.display = c.id === `wing-${{wing.name}}` ? 'block' : 'none';
          }});
        }};
        sidebar.appendChild(nav);

        // Wing card
        const wingCard = document.createElement('div');
        wingCard.className = 'wing-card';
        wingCard.id = `wing-${{wing.name}}`;

        let roomsHtml = '';
        let matchingDrawers = 0;

        wing.rooms.forEach(room => {{
          const filteredDrawers = room.drawers.filter(d => {{
            if (!query) return true;
            return d.title.toLowerCase().includes(query) ||
                   d.preview.toLowerCase().includes(query) ||
                   d.tags.some(t => t.toLowerCase().includes(query)) ||
                   d.memory_type.toLowerCase().includes(query);
          }});

          if (filteredDrawers.length === 0 && query) return;
          matchingDrawers += filteredDrawers.length;

          let drawersHtml = filteredDrawers.map(d => `
            <div class="drawer-item" onclick="openDrawer(${{JSON.stringify(d).replace(/"/g, '&quot;')}})">
              <div class="drawer-header">
                <span class="drawer-type">${{d.memory_type}}</span>
                <span style="font-size:11px; color:var(--text-muted);">#${{d.id}}</span>
              </div>
              <div class="drawer-title">${{escapeHtml(d.title)}}</div>
              <div class="drawer-tags">
                ${{d.tags.map(t => `<span class="tag">#${{escapeHtml(t)}}</span>`).join('')}}
              </div>
            </div>
          `).join('');

          roomsHtml += `
            <div class="room-card">
              <div class="room-header">
                <span class="room-title">🚪 Room: ${{escapeHtml(room.name)}}</span>
                <span style="font-size:12px; color:var(--text-muted);">${{filteredDrawers.length}} items</span>
              </div>
              <div class="drawers-list">
                ${{drawersHtml || '<div style="font-size:12px; color:var(--text-muted);">Empty room</div>'}}
              </div>
            </div>
          `;
        }});

        if (query && matchingDrawers === 0) return;

        wingCard.innerHTML = `
          <div class="wing-header">
            <div class="wing-title">🏛️ Wing: ${{escapeHtml(wing.name)}}</div>
            <span style="font-size:13px; color:var(--text-muted);">${{wing.drawer_count}} drawers</span>
          </div>
          <div class="rooms-grid">${{roomsHtml}}</div>
        `;
        container.appendChild(wingCard);
      }});

      if (container.children.length === 0) {{
        container.innerHTML = '<div style="text-align:center; padding:48px; color:var(--text-muted);">No matching drawers found in this palace.</div>';
      }}
    }}

    function openDrawer(d) {{
      document.getElementById('modalTitle').textContent = `📦 Drawer #${{d.id}} [${{d.memory_type}}] - ${{d.scope_path}}`;
      document.getElementById('modalBody').textContent = d.preview;
      document.getElementById('modalOverlay').classList.add('open');
    }}

    function closeModal(e) {{
      if (e.target.id === 'modalOverlay') closeModalDirect();
    }}
    function closeModalDirect() {{
      document.getElementById('modalOverlay').classList.remove('open');
    }}

    function escapeHtml(str) {{
      return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }}

    document.getElementById('searchInput').addEventListener('input', (e) => {{
      render(e.target.value);
    }});

    // Initialize
    render();
  </script>
</body>
</html>
"#,
            self.workspace,
            self.workspace,
            self.wings_count,
            self.rooms_count,
            self.total_drawers,
            palace_json
        )
    }
}
