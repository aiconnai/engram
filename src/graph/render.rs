//! vis.js JSON and standalone HTML rendering for the knowledge graph.

use super::types::KnowledgeGraph;

impl KnowledgeGraph {
    /// Export as vis.js compatible JSON
    pub fn to_visjs_json(&self) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = self
            .nodes
            .iter()
            .map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "label": n.label,
                    "group": n.memory_type,
                    "value": (n.importance * 10.0) as i32 + 5,
                    "title": format!("Type: {}\nTags: {}", n.memory_type, n.tags.join(", "))
                })
            })
            .collect();

        let edges: Vec<serde_json::Value> = self
            .edges
            .iter()
            .map(|e| {
                serde_json::json!({
                    "from": e.from,
                    "to": e.to,
                    "label": e.edge_type,
                    "value": (e.score * e.confidence * 5.0) as i32 + 1,
                    "title": format!("Score: {:.2}, Confidence: {:.2}", e.score, e.confidence)
                })
            })
            .collect();

        serde_json::json!({
            "nodes": nodes,
            "edges": edges
        })
    }

    /// Export as standalone HTML with vis.js
    pub fn to_html(&self) -> String {
        let graph_data = self.to_visjs_json();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Engram Knowledge Graph</title>
    <script type="text/javascript" src="https://unpkg.com/vis-network/standalone/umd/vis-network.min.js"></script>
    <style>
        body {{ margin: 0; padding: 0; font-family: system-ui, sans-serif; }}
        #graph {{ width: 100vw; height: 100vh; }}
        #controls {{
            position: absolute;
            top: 10px;
            left: 10px;
            background: white;
            padding: 10px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        #search {{ padding: 8px; width: 200px; border: 1px solid #ddd; border-radius: 4px; }}
        .legend {{ display: flex; gap: 10px; margin-top: 10px; flex-wrap: wrap; }}
        .legend-item {{ display: flex; align-items: center; gap: 5px; font-size: 12px; }}
        .legend-dot {{ width: 12px; height: 12px; border-radius: 50%; }}
    </style>
</head>
<body>
    <div id="controls">
        <input type="text" id="search" placeholder="Search nodes...">
        <div class="legend">
            <div class="legend-item"><span class="legend-dot" style="background: #97C2FC;"></span> note</div>
            <div class="legend-item"><span class="legend-dot" style="background: #FFFF00;"></span> todo</div>
            <div class="legend-item"><span class="legend-dot" style="background: #FB7E81;"></span> issue</div>
            <div class="legend-item"><span class="legend-dot" style="background: #7BE141;"></span> decision</div>
            <div class="legend-item"><span class="legend-dot" style="background: #FFA807;"></span> preference</div>
            <div class="legend-item"><span class="legend-dot" style="background: #6E6EFD;"></span> learning</div>
        </div>
    </div>
    <div id="graph"></div>
    <script>
        const data = {graph_data};

        const options = {{
            nodes: {{
                shape: 'dot',
                scaling: {{ min: 10, max: 30 }},
                font: {{ size: 12, face: 'system-ui' }}
            }},
            edges: {{
                arrows: 'to',
                scaling: {{ min: 1, max: 5 }},
                font: {{ size: 10, align: 'middle' }}
            }},
            groups: {{
                note: {{ color: '#97C2FC' }},
                todo: {{ color: '#FFFF00' }},
                issue: {{ color: '#FB7E81' }},
                decision: {{ color: '#7BE141' }},
                preference: {{ color: '#FFA807' }},
                learning: {{ color: '#6E6EFD' }},
                context: {{ color: '#C2FABC' }},
                credential: {{ color: '#FD6A6A' }}
            }},
            physics: {{
                stabilization: {{ iterations: 100 }},
                barnesHut: {{
                    gravitationalConstant: -2000,
                    springLength: 100
                }}
            }},
            interaction: {{
                hover: true,
                tooltipDelay: 100
            }}
        }};

        const container = document.getElementById('graph');
        const network = new vis.Network(container, data, options);

        // Search functionality
        const searchInput = document.getElementById('search');
        searchInput.addEventListener('input', function() {{
            const query = this.value.toLowerCase();
            if (query) {{
                const matchingNodes = data.nodes.filter(n =>
                    n.label.toLowerCase().includes(query)
                ).map(n => n.id);
                network.selectNodes(matchingNodes);
                if (matchingNodes.length > 0) {{
                    network.focus(matchingNodes[0], {{ scale: 1.5, animation: true }});
                }}
            }} else {{
                network.unselectAll();
            }}
        }});

        // Click to focus
        network.on('click', function(params) {{
            if (params.nodes.length > 0) {{
                network.focus(params.nodes[0], {{ scale: 1.5, animation: true }});
            }}
        }});
    </script>
</body>
</html>"#,
            graph_data = serde_json::to_string(&graph_data).unwrap_or_default()
        )
    }
}
