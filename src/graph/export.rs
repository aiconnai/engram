//! DOT / GEXF export (RML-894)

use std::collections::HashMap;

use super::types::KnowledgeGraph;

impl KnowledgeGraph {
    /// Export as DOT format for Graphviz
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph knowledge_graph {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=rounded];\n\n");

        // Color mapping for memory types
        let colors: HashMap<&str, &str> = [
            ("note", "#97C2FC"),
            ("todo", "#FFFF00"),
            ("issue", "#FB7E81"),
            ("decision", "#7BE141"),
            ("preference", "#FFA807"),
            ("learning", "#6E6EFD"),
            ("context", "#C2FABC"),
            ("credential", "#FD6A6A"),
        ]
        .into_iter()
        .collect();

        // Write nodes
        for node in &self.nodes {
            let color = colors.get(node.memory_type.as_str()).unwrap_or(&"#CCCCCC");
            let label = node.label.replace('"', "\\\"");
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\", fillcolor=\"{}\", style=\"filled,rounded\"];\n",
                node.id, label, color
            ));
        }

        dot.push('\n');

        // Write edges
        for edge in &self.edges {
            let style = match edge.edge_type.as_str() {
                "related_to" => "solid",
                "part_of" => "dashed",
                "depends_on" => "bold",
                "contradicts" => "dotted",
                "supports" => "solid",
                "references" => "dashed",
                _ => "solid",
            };
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\", style={}, penwidth={}];\n",
                edge.from,
                edge.to,
                edge.edge_type,
                style,
                (edge.score * 2.0 + 0.5).min(3.0)
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Export as GEXF format for Gephi
    pub fn to_gexf(&self) -> String {
        let mut gexf = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<gexf xmlns="http://gexf.net/1.3" version="1.3">
  <meta>
    <creator>Engram</creator>
    <description>Knowledge Graph Export</description>
  </meta>
  <graph mode="static" defaultedgetype="directed">
    <attributes class="node">
      <attribute id="0" title="type" type="string"/>
      <attribute id="1" title="importance" type="float"/>
    </attributes>
    <attributes class="edge">
      <attribute id="0" title="score" type="float"/>
      <attribute id="1" title="confidence" type="float"/>
    </attributes>
    <nodes>
"#,
        );

        for node in &self.nodes {
            let label = node
                .label
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;");
            gexf.push_str(&format!(
                r#"      <node id="{}" label="{}">
        <attvalues>
          <attvalue for="0" value="{}"/>
          <attvalue for="1" value="{}"/>
        </attvalues>
      </node>
"#,
                node.id, label, node.memory_type, node.importance
            ));
        }

        gexf.push_str("    </nodes>\n    <edges>\n");

        for (i, edge) in self.edges.iter().enumerate() {
            gexf.push_str(&format!(
                r#"      <edge id="{}" source="{}" target="{}" label="{}">
        <attvalues>
          <attvalue for="0" value="{}"/>
          <attvalue for="1" value="{}"/>
        </attvalues>
      </edge>
"#,
                i, edge.from, edge.to, edge.edge_type, edge.score, edge.confidence
            ));
        }

        gexf.push_str("    </edges>\n  </graph>\n</gexf>\n");
        gexf
    }
}
