window.BENCHMARK_DATA = {
  "lastUpdate": 1780664604322,
  "repoUrl": "https://github.com/aiconnai/engram",
  "entries": {
    "Engram Performance": [
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dba53664c78ff17b4a2bd31956eb4b072ccf074a",
          "message": "chore(ci): update checkout action for node 24 (#54)",
          "timestamp": "2026-06-05T09:48:57-03:00",
          "tree_id": "5610e52650744239bd44796b05acc174390df0ef",
          "url": "https://github.com/aiconnai/engram/commit/dba53664c78ff17b4a2bd31956eb4b072ccf074a"
        },
        "date": 1780664603291,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5379359,
            "range": "± 6218",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3584,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 9269,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 208311,
            "range": "± 4249",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 527364,
            "range": "± 6410",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 286543,
            "range": "± 1872",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117350,
            "range": "± 1051",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 162,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 211747,
            "range": "± 7679",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47121,
            "range": "± 1352",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114772,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 290400,
            "range": "± 2189",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 442150,
            "range": "± 3567",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686023,
            "range": "± 2590",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 846259,
            "range": "± 5371",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1070714,
            "range": "± 12643",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35433,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22012,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321817,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 257994,
            "range": "± 2574",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 296217,
            "range": "± 1364",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 332628,
            "range": "± 1488",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 375957,
            "range": "± 1504",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 52345,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 529234,
            "range": "± 6758",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 74324,
            "range": "± 2110",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 858,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2305,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5287,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192802,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18470,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17838,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18807,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 228316,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 530777,
            "range": "± 4532",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2094765,
            "range": "± 24874",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6605,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34172,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137720,
            "range": "± 1044",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 572,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 547,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 549,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2914,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10700,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 41642,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13282,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67785,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80434202,
            "range": "± 98460",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5167170,
            "range": "± 97884",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18909702,
            "range": "± 61107",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1244162,
            "range": "± 25974",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}