window.BENCHMARK_DATA = {
  "lastUpdate": 1782094094096,
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
      },
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
          "id": "f8668eb9776cf206b509a5349e4cabb6f001bc74",
          "message": "feat(audit): emit CRUD audit events and add temporal replay edges (ENGRA-62/61/63) (#55)\n\n* chore(ci): update checkout action for node 24\n\n* feat(audit): emit CRUD audit events and add temporal replay edges (ENGRA-62/61/63)\n\n- memory_create/update/delete now call emit_best_effort for audit trail coverage\n- memory_replay_at_time response includes temporal_edges active at requested timestamp\n- Protocol tests: assert memory_replay_at_time in tools/list, add 4 integration tests\n  covering replay structure, error validation, and CRUD audit emit end-to-end\n\n* fix(audit): address 5 code-review findings in CRUD audit and temporal replay\n\n- cascade delete: emit one event per deleted member (not just root) so\n  memory_enrichment_timeline returns results for every id in the chain\n- emit inside with_transaction for atomicity: audit event and data write\n  now commit or roll back together in memory_create/update/delete\n- workspace captured inside transaction for memory_update/delete (was None)\n- snapshot_at full-scan replaced by edges_for_memory_at: SQL-filtered query\n  O(K) instead of O(N_total_edges) for memory_replay_at_time\n- snapshot_at error now logs a tracing::warn instead of silently returning []\n\n* style(audit): apply rustfmt to mcp_protocol_tests",
          "timestamp": "2026-06-05T12:48:55-03:00",
          "tree_id": "3af3409f011ee64beed5d17611725f104f5176f7",
          "url": "https://github.com/aiconnai/engram/commit/f8668eb9776cf206b509a5349e4cabb6f001bc74"
        },
        "date": 1780675404483,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5925794,
            "range": "± 4605",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3575,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 10027,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 233088,
            "range": "± 7382",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 510439,
            "range": "± 2673",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 279025,
            "range": "± 6642",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102611,
            "range": "± 5538",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 160,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 191502,
            "range": "± 10706",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39362,
            "range": "± 1222",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112794,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 269851,
            "range": "± 4430",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 444988,
            "range": "± 2761",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 674103,
            "range": "± 7219",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 858542,
            "range": "± 6242",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1074008,
            "range": "± 17037",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32710,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21839,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 310143,
            "range": "± 4713",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 231561,
            "range": "± 2017",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 268753,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 304944,
            "range": "± 1856",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 346459,
            "range": "± 1724",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 42726,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 496846,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69291,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 920,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2486,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5624,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 205971,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19326,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18628,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19623,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 202742,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 496769,
            "range": "± 1675",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2138683,
            "range": "± 7016",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6654,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 35333,
            "range": "± 647",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141462,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 556,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 551,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 555,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2590,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10174,
            "range": "± 567",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 36823,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13596,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 70413,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 84509132,
            "range": "± 492157",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4655495,
            "range": "± 94493",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16780497,
            "range": "± 82660",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1190312,
            "range": "± 12297",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "4bd990cec9af06ea3edd4937c46b8cdf8783ff33",
          "message": "feat: align product framing, harness docs, and operational context (#56)\n\n* feat(engram): align product framing and harness docs\n\n* chore: implement ENGRA-58-60 MCP HTTP validation and harness alignment",
          "timestamp": "2026-06-05T16:02:13-03:00",
          "tree_id": "aa1f19b87cedd5a5ddbe8876c3567e2234cd044f",
          "url": "https://github.com/aiconnai/engram/commit/4bd990cec9af06ea3edd4937c46b8cdf8783ff33"
        },
        "date": 1780687004113,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5973087,
            "range": "± 22488",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3542,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 9791,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 234569,
            "range": "± 7408",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 511132,
            "range": "± 3077",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 285958,
            "range": "± 1765",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102964,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 158,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 190212,
            "range": "± 10076",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39449,
            "range": "± 902",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115247,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 273627,
            "range": "± 1761",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 454142,
            "range": "± 4428",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 676833,
            "range": "± 8740",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 870874,
            "range": "± 6278",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1087312,
            "range": "± 18953",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32831,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22100,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 308362,
            "range": "± 4695",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229203,
            "range": "± 1489",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 268691,
            "range": "± 1910",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 303211,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 349521,
            "range": "± 6322",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43541,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 498461,
            "range": "± 2852",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69257,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 922,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2447,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5590,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201225,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19068,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17885,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19095,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 204617,
            "range": "± 1959",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 499268,
            "range": "± 3941",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2159788,
            "range": "± 10952",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6650,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34685,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141488,
            "range": "± 1434",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 545,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 537,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 536,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2702,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10183,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39578,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13502,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69898,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83872302,
            "range": "± 129273",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4614820,
            "range": "± 14332",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16662549,
            "range": "± 108044",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1182699,
            "range": "± 21657",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "6f1af3faa4032eacea47d5c44d00bc48eaff3c4e",
          "message": "feat(context): add operational context memory (#57)\n\n* chore(ci): update checkout action for node 24\n\n* feat(audit): emit CRUD audit events and add temporal replay edges (ENGRA-62/61/63)\n\n- memory_create/update/delete now call emit_best_effort for audit trail coverage\n- memory_replay_at_time response includes temporal_edges active at requested timestamp\n- Protocol tests: assert memory_replay_at_time in tools/list, add 4 integration tests\n  covering replay structure, error validation, and CRUD audit emit end-to-end\n\n* fix(audit): address 5 code-review findings in CRUD audit and temporal replay\n\n- cascade delete: emit one event per deleted member (not just root) so\n  memory_enrichment_timeline returns results for every id in the chain\n- emit inside with_transaction for atomicity: audit event and data write\n  now commit or roll back together in memory_create/update/delete\n- workspace captured inside transaction for memory_update/delete (was None)\n- snapshot_at full-scan replaced by edges_for_memory_at: SQL-filtered query\n  O(K) instead of O(N_total_edges) for memory_replay_at_time\n- snapshot_at error now logs a tracing::warn instead of silently returning []\n\n* style(audit): apply rustfmt to mcp_protocol_tests\n\n* feat(context): add operational context memory",
          "timestamp": "2026-06-05T17:21:50-03:00",
          "tree_id": "aa8090f34edf96ae3ee0509e776e244fd17fe681",
          "url": "https://github.com/aiconnai/engram/commit/6f1af3faa4032eacea47d5c44d00bc48eaff3c4e"
        },
        "date": 1780691794433,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5389168,
            "range": "± 29189",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3538,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8119,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 269245,
            "range": "± 6133",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 525889,
            "range": "± 3665",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289330,
            "range": "± 7507",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118269,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 162,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 209604,
            "range": "± 8307",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46709,
            "range": "± 1426",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114192,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 287292,
            "range": "± 2331",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 441603,
            "range": "± 3138",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 676097,
            "range": "± 3829",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 847225,
            "range": "± 3017",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1058431,
            "range": "± 26528",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34791,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21303,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321456,
            "range": "± 1613",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 251371,
            "range": "± 2223",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290294,
            "range": "± 1857",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 325562,
            "range": "± 1988",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 366229,
            "range": "± 2557",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 52063,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 521411,
            "range": "± 6772",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 74422,
            "range": "± 824",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 908,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2325,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5260,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 193382,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18240,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17181,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18090,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 228251,
            "range": "± 2888",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 520526,
            "range": "± 3975",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2068522,
            "range": "± 14123",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6746,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34173,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139138,
            "range": "± 764",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 578,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 550,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 552,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3042,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11332,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44567,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13290,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67872,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80300890,
            "range": "± 241575",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5252459,
            "range": "± 30772",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19131727,
            "range": "± 118725",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1227695,
            "range": "± 22914",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "c77cc6bcbd3f9ca0247ad65bfed3710a93f203c9",
          "message": "feat(intelligence): add deterministic memory policy layer (#58)\n\n* feat(engram): align product framing and harness docs\n\n* chore: implement ENGRA-58-60 MCP HTTP validation and harness alignment\n\n* feat(intelligence): add deterministic memory policy layer\n\n* fix(context): remove duplicate context module file\n\n* fix(storage): format memory policy migration",
          "timestamp": "2026-06-06T00:56:30-03:00",
          "tree_id": "613080decc1521f0f53033aae03973307936eff2",
          "url": "https://github.com/aiconnai/engram/commit/c77cc6bcbd3f9ca0247ad65bfed3710a93f203c9"
        },
        "date": 1780719065470,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5932232,
            "range": "± 35639",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3572,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8950,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325272,
            "range": "± 33390",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513962,
            "range": "± 3042",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 284646,
            "range": "± 1722",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102627,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 202,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 281169,
            "range": "± 11928",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40472,
            "range": "± 880",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115316,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 267234,
            "range": "± 1941",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 458640,
            "range": "± 4753",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 674759,
            "range": "± 4588",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 897869,
            "range": "± 7659",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1088508,
            "range": "± 16504",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33447,
            "range": "± 727",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22569,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 294689,
            "range": "± 972",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 231211,
            "range": "± 1355",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 270102,
            "range": "± 1583",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 305553,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 345953,
            "range": "± 1597",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 42880,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 499963,
            "range": "± 2598",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69039,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 931,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2425,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5545,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203283,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19138,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17927,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19026,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 201762,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 500455,
            "range": "± 2142",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2152742,
            "range": "± 16257",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6647,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34682,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140265,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 561,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 547,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 541,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2619,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9833,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38265,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13329,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69191,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82612232,
            "range": "± 111886",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4698199,
            "range": "± 19782",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16965832,
            "range": "± 45786",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1200405,
            "range": "± 19239",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "665b73a38a340a93d6d03e1939d9f35ebec27e6b",
          "message": "docs: move cloud reference docs under docs (#59)\n\n* docs: move cloud reference docs under docs\n\n* docs: update references for relocated docs",
          "timestamp": "2026-06-06T01:50:41-03:00",
          "tree_id": "a9398ab439c20218e463b7279669d6e91b2d000c",
          "url": "https://github.com/aiconnai/engram/commit/665b73a38a340a93d6d03e1939d9f35ebec27e6b"
        },
        "date": 1780722307654,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5944082,
            "range": "± 11366",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3618,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8941,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 339013,
            "range": "± 15207",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513642,
            "range": "± 3812",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 285408,
            "range": "± 2479",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101991,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 203,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 275760,
            "range": "± 13046",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40034,
            "range": "± 906",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114726,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 266044,
            "range": "± 1413",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 459395,
            "range": "± 4281",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 675960,
            "range": "± 7521",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 896422,
            "range": "± 16197",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1082763,
            "range": "± 11754",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33791,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22689,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 294368,
            "range": "± 1600",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229300,
            "range": "± 1086",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 266096,
            "range": "± 1450",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 303788,
            "range": "± 2958",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 346854,
            "range": "± 2544",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 42073,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 495784,
            "range": "± 3500",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69347,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 938,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2450,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5610,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201480,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19141,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17895,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19003,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 202333,
            "range": "± 1229",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 496342,
            "range": "± 2362",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2168285,
            "range": "± 17717",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6648,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34925,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139670,
            "range": "± 2155",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 553,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 552,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 545,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2798,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10308,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 36098,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13327,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69013,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82870969,
            "range": "± 758504",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4693966,
            "range": "± 14401",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16924202,
            "range": "± 76372",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1204887,
            "range": "± 12344",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "443278b467bb1a5c80b4ed6975ff6cd20756528c",
          "message": "docs(harness): enforce security reference boundary (#60)",
          "timestamp": "2026-06-06T02:27:37-03:00",
          "tree_id": "4dc0b60fc734f66c3096f26e8b9c24449d603113",
          "url": "https://github.com/aiconnai/engram/commit/443278b467bb1a5c80b4ed6975ff6cd20756528c"
        },
        "date": 1780724527579,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5936131,
            "range": "± 4640",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3601,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8817,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 321939,
            "range": "± 7617",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 512101,
            "range": "± 3082",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 280204,
            "range": "± 1902",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103086,
            "range": "± 878",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 206,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 284529,
            "range": "± 15318",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39471,
            "range": "± 994",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114577,
            "range": "± 1111",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 264755,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 461057,
            "range": "± 4992",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 678212,
            "range": "± 5020",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 906232,
            "range": "± 8993",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1087914,
            "range": "± 22355",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33459,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22412,
            "range": "± 2982",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 294125,
            "range": "± 20275",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 231334,
            "range": "± 3319",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 269195,
            "range": "± 2202",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 304538,
            "range": "± 6487",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 345663,
            "range": "± 1728",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43045,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 495039,
            "range": "± 9399",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69194,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 957,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2456,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5612,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203749,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19407,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17898,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19043,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 202478,
            "range": "± 3159",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 496722,
            "range": "± 2876",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2151836,
            "range": "± 47661",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6634,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34576,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140455,
            "range": "± 4728",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 547,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 529,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 530,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2661,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9857,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38263,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13401,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69779,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82946548,
            "range": "± 129799",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4654384,
            "range": "± 73010",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16732031,
            "range": "± 459761",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1193351,
            "range": "± 59268",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "be37912f3ee1a635b54f5096172548f8fb0370a4",
          "message": "ci: split required and extended Rust checks",
          "timestamp": "2026-06-06T10:39:54-03:00",
          "tree_id": "90ff6b35b62bb8d2e42333fffd7678e60d3d32f8",
          "url": "https://github.com/aiconnai/engram/commit/be37912f3ee1a635b54f5096172548f8fb0370a4"
        },
        "date": 1780754336168,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5928196,
            "range": "± 74743",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3604,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8816,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 339364,
            "range": "± 13468",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513156,
            "range": "± 2504",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 280489,
            "range": "± 5249",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102875,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 202,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 280124,
            "range": "± 11510",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39885,
            "range": "± 953",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114848,
            "range": "± 1035",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 262071,
            "range": "± 3785",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 463406,
            "range": "± 3456",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 669404,
            "range": "± 49181",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 895909,
            "range": "± 4793",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1082225,
            "range": "± 14271",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33543,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22128,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 295250,
            "range": "± 1233",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 228989,
            "range": "± 1631",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 267790,
            "range": "± 1783",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 304986,
            "range": "± 2771",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 350251,
            "range": "± 6953",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43572,
            "range": "± 2306",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 493538,
            "range": "± 2280",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69216,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 952,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2471,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5679,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201403,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19720,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18427,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19796,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 201586,
            "range": "± 1502",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 501006,
            "range": "± 2335",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2187758,
            "range": "± 15227",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6592,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34724,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140931,
            "range": "± 2019",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 549,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 544,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 546,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2648,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9798,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38303,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13362,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69176,
            "range": "± 2037",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82192267,
            "range": "± 139677",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4646009,
            "range": "± 12536",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16787968,
            "range": "± 62709",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1190976,
            "range": "± 22553",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "fec2f77e22adefe075007b2a70868d78afd2de04",
          "message": "ci: install protoc without deprecated action",
          "timestamp": "2026-06-06T11:19:15-03:00",
          "tree_id": "6fc3953302bf42e31f07cb9c731aace15f0fd51c",
          "url": "https://github.com/aiconnai/engram/commit/fec2f77e22adefe075007b2a70868d78afd2de04"
        },
        "date": 1780756436603,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5982103,
            "range": "± 50691",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3581,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8754,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 327223,
            "range": "± 14589",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513298,
            "range": "± 9156",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283367,
            "range": "± 4200",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103248,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 204,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 285070,
            "range": "± 14001",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40159,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 116262,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 267632,
            "range": "± 3706",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 470886,
            "range": "± 3521",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 677979,
            "range": "± 11590",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 893881,
            "range": "± 12733",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1073238,
            "range": "± 44741",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33396,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21944,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 301534,
            "range": "± 2892",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229980,
            "range": "± 1595",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 265050,
            "range": "± 6567",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 301355,
            "range": "± 1625",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 343914,
            "range": "± 6896",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 42890,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 496154,
            "range": "± 5211",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 69663,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 960,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2508,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5599,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201786,
            "range": "± 2057",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19226,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17984,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19302,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 202282,
            "range": "± 3421",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 491468,
            "range": "± 1862",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2114851,
            "range": "± 146061",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6674,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34488,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139761,
            "range": "± 1371",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 544,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 541,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 542,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2854,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9749,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40061,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13400,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69438,
            "range": "± 1536",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82365759,
            "range": "± 227856",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4673758,
            "range": "± 16404",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16877082,
            "range": "± 62109",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1193178,
            "range": "± 14753",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "52810094ae072807f5c2590b2b4fbfe1c518370e",
          "message": "ci: update artifact upload action",
          "timestamp": "2026-06-06T14:18:37-03:00",
          "tree_id": "fbff0686765d67d7a1a7dd64a45fcb77fae52d45",
          "url": "https://github.com/aiconnai/engram/commit/52810094ae072807f5c2590b2b4fbfe1c518370e"
        },
        "date": 1780767224916,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5968468,
            "range": "± 95108",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3596,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8824,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 333611,
            "range": "± 10858",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 511426,
            "range": "± 3363",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283552,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102090,
            "range": "± 1411",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 202,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 280802,
            "range": "± 13576",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40040,
            "range": "± 874",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 116091,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 266982,
            "range": "± 3872",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 457656,
            "range": "± 2566",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 676894,
            "range": "± 3087",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 902458,
            "range": "± 2818",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1101655,
            "range": "± 13853",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33561,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22477,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 296312,
            "range": "± 1619",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 231408,
            "range": "± 2149",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 269842,
            "range": "± 17370",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 305252,
            "range": "± 1697",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 347428,
            "range": "± 2065",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 41719,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 494881,
            "range": "± 5929",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 68820,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 931,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2473,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5582,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203101,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19071,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17873,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18980,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 201987,
            "range": "± 3008",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 493038,
            "range": "± 4286",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2147912,
            "range": "± 25025",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6615,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34521,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140591,
            "range": "± 1212",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 598,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 581,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 583,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2760,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9768,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37426,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13415,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69673,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82419286,
            "range": "± 123657",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4713146,
            "range": "± 74792",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16757593,
            "range": "± 65406",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1181940,
            "range": "± 17275",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "ed4daa8b4b64a5248182378e1000555cbbdb7f42",
          "message": "ci: move coverage off main push",
          "timestamp": "2026-06-07T13:22:13-03:00",
          "tree_id": "d7a5015ff7d022531ba62f7405d4439f0800d963",
          "url": "https://github.com/aiconnai/engram/commit/ed4daa8b4b64a5248182378e1000555cbbdb7f42"
        },
        "date": 1780850198669,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5931272,
            "range": "± 32342",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3634,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8810,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325302,
            "range": "± 8058",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 514073,
            "range": "± 2780",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283035,
            "range": "± 1837",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102457,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 202,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 277589,
            "range": "± 12665",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39846,
            "range": "± 911",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 118185,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 266777,
            "range": "± 1541",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 464980,
            "range": "± 5930",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 674546,
            "range": "± 9764",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 901852,
            "range": "± 10059",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1076037,
            "range": "± 9935",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33879,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22633,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 297143,
            "range": "± 3825",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 230204,
            "range": "± 3805",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 267557,
            "range": "± 4108",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 305851,
            "range": "± 3723",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 343704,
            "range": "± 1222",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 42476,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 490296,
            "range": "± 4583",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 68331,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 903,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2451,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5592,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201385,
            "range": "± 1910",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19183,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17971,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19050,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 200594,
            "range": "± 945",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 497530,
            "range": "± 9749",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2147192,
            "range": "± 25929",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6649,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 35031,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140974,
            "range": "± 1392",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 545,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 551,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 554,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2718,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10143,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39201,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13366,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69549,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82206753,
            "range": "± 1111664",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4707641,
            "range": "± 10909",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16964396,
            "range": "± 59249",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1181795,
            "range": "± 17813",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "9fcafcfa338bfb174c890569334d0369b8802244",
          "message": "ci: decouple neural rerank smoke from openai",
          "timestamp": "2026-06-07T15:00:38-03:00",
          "tree_id": "cef3b2385f742dcbb7d21b865e0bfa93d358f974",
          "url": "https://github.com/aiconnai/engram/commit/9fcafcfa338bfb174c890569334d0369b8802244"
        },
        "date": 1780856153715,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5897292,
            "range": "± 59441",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3758,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8972,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 319217,
            "range": "± 17352",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 510790,
            "range": "± 8862",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 282826,
            "range": "± 1895",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103812,
            "range": "± 1220",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 206,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 282620,
            "range": "± 44875",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40312,
            "range": "± 1819",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114385,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 263577,
            "range": "± 7489",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 453911,
            "range": "± 2371",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 667902,
            "range": "± 7069",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 879348,
            "range": "± 16303",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1070250,
            "range": "± 19935",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33547,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22442,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 296444,
            "range": "± 1527",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 235975,
            "range": "± 4165",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 271060,
            "range": "± 1515",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 301014,
            "range": "± 3077",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 333184,
            "range": "± 4089",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36392,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13153401,
            "range": "± 84473",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12716923,
            "range": "± 97941",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 974,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2518,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5637,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202651,
            "range": "± 835",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19207,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17949,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19578,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1360308,
            "range": "± 10245",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1111405,
            "range": "± 13322",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11554449,
            "range": "± 198137",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11061486,
            "range": "± 62763",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120946240,
            "range": "± 1244253",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115925606,
            "range": "± 1081688",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6678,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34893,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141802,
            "range": "± 3380",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 531,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 517,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 520,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2736,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9940,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39014,
            "range": "± 1167",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13336,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69701,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82703453,
            "range": "± 204473",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4669892,
            "range": "± 44861",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16855736,
            "range": "± 53124",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1194516,
            "range": "± 37513",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "ed1ecb5800ae1458c405418aceb26cfd893c8be4",
          "message": "chore: ignore local project skills\n\nKeep .claude/skills/ project-scoped but out of version control: local\nskills can carry internal operational detail (Huly workspace, tokens)\nthat should not ship in a public repo.",
          "timestamp": "2026-06-07T15:17:25-03:00",
          "tree_id": "5d9434c6018ca1ecf836315f10d661cab175ef40",
          "url": "https://github.com/aiconnai/engram/commit/ed1ecb5800ae1458c405418aceb26cfd893c8be4"
        },
        "date": 1780857342191,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5314650,
            "range": "± 7089",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3568,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8171,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 364889,
            "range": "± 8419",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 531906,
            "range": "± 2460",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290771,
            "range": "± 1577",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119997,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 215,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 318545,
            "range": "± 10582",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48782,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 118058,
            "range": "± 1098",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 290649,
            "range": "± 1820",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 465493,
            "range": "± 8247",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 709332,
            "range": "± 3285",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 895635,
            "range": "± 9014",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1102048,
            "range": "± 25879",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 37057,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22293,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 316310,
            "range": "± 1300",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 253052,
            "range": "± 1406",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 291823,
            "range": "± 2903",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 322694,
            "range": "± 2754",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 357393,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44730,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12926954,
            "range": "± 97361",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12442366,
            "range": "± 95634",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 885,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2359,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5339,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 195279,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18628,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17681,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18498,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1383253,
            "range": "± 6226",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1114019,
            "range": "± 3713",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11381973,
            "range": "± 41641",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10864859,
            "range": "± 99242",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119141278,
            "range": "± 887238",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114889420,
            "range": "± 757948",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6675,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34209,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139014,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 567,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 566,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 566,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2616,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9581,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37654,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13285,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67909,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80119065,
            "range": "± 181415",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5298775,
            "range": "± 11118",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19287332,
            "range": "± 33917",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1248196,
            "range": "± 35701",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "0ece5db3d85f993fba35647ec9f118af35e25b62",
          "message": "ci: move full feature tests off main push",
          "timestamp": "2026-06-07T15:26:39-03:00",
          "tree_id": "99ad293655124ca699b69e4db9068a487be30a29",
          "url": "https://github.com/aiconnai/engram/commit/0ece5db3d85f993fba35647ec9f118af35e25b62"
        },
        "date": 1780857725911,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5907501,
            "range": "± 6942",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3602,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8732,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 324816,
            "range": "± 9057",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 518964,
            "range": "± 3581",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 285336,
            "range": "± 2495",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102823,
            "range": "± 893",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 205,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 288841,
            "range": "± 13435",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40233,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117156,
            "range": "± 881",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 270002,
            "range": "± 7829",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 463828,
            "range": "± 5520",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 681843,
            "range": "± 3835",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 901491,
            "range": "± 6224",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1087311,
            "range": "± 13831",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33042,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22300,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 296451,
            "range": "± 1087",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 226501,
            "range": "± 1224",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 261070,
            "range": "± 1624",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 289405,
            "range": "± 1479",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 323630,
            "range": "± 3256",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35921,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13536579,
            "range": "± 282969",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13062787,
            "range": "± 253284",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 928,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2462,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5640,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202820,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19128,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18034,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 20447,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1357884,
            "range": "± 4803",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1110746,
            "range": "± 8493",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11656988,
            "range": "± 160105",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11267844,
            "range": "± 143210",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 125694526,
            "range": "± 1258065",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 121853426,
            "range": "± 628409",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6769,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34880,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141841,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 540,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 538,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 539,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2778,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10155,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39849,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13474,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69665,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82412927,
            "range": "± 106829",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4748374,
            "range": "± 4188",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16954212,
            "range": "± 114669",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1192695,
            "range": "± 25351",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "c5ea6e9514132bfe5de24abbb297c61ad5d3b28a",
          "message": "ci: skip full feature tests on main push",
          "timestamp": "2026-06-07T15:27:51-03:00",
          "tree_id": "33e50f843da7cb833a86e10923cdd5a23a9ec7cc",
          "url": "https://github.com/aiconnai/engram/commit/c5ea6e9514132bfe5de24abbb297c61ad5d3b28a"
        },
        "date": 1780857774420,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5664989,
            "range": "± 6954",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3292,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 7137,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 346148,
            "range": "± 10519",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 453601,
            "range": "± 2995",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 251954,
            "range": "± 3783",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 98498,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 197,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 298839,
            "range": "± 15790",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 37789,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 97167,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 241465,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 366118,
            "range": "± 2712",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 573831,
            "range": "± 4310",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 700342,
            "range": "± 2548",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 903119,
            "range": "± 3405",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 29211,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 18381,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 264475,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 200688,
            "range": "± 5035",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 234767,
            "range": "± 3865",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 262286,
            "range": "± 3952",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 292774,
            "range": "± 1619",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 37765,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 11543589,
            "range": "± 81399",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11048016,
            "range": "± 41725",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 905,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2472,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5441,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 198102,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 15729,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 14935,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 15750,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1163951,
            "range": "± 2174",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 936930,
            "range": "± 6869",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 9915774,
            "range": "± 68908",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 9322912,
            "range": "± 35436",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 104465394,
            "range": "± 821453",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 99292838,
            "range": "± 665706",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6587,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33578,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 132846,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 565,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 566,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 563,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2796,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9745,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38336,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13193,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66634,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79197590,
            "range": "± 93280",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4292937,
            "range": "± 25782",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 15488544,
            "range": "± 70509",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1026011,
            "range": "± 19343",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "committer": {
            "email": "ron@ldinho.com.br",
            "name": "Ronaldo Martins",
            "username": "limaronaldo"
          },
          "distinct": true,
          "id": "ee0117662ab0d67e009261ddda9dec930eee2e1c",
          "message": "ci: move concurrency comment next to its block",
          "timestamp": "2026-06-07T15:35:10-03:00",
          "tree_id": "0640eb1481ef37bdad31c27416682523e55ccb18",
          "url": "https://github.com/aiconnai/engram/commit/ee0117662ab0d67e009261ddda9dec930eee2e1c"
        },
        "date": 1780858175971,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 4546100,
            "range": "± 5270",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 2858,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 6749,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 248882,
            "range": "± 5411",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 390272,
            "range": "± 1242",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 219245,
            "range": "± 1649",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 79759,
            "range": "± 719",
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
            "value": 213887,
            "range": "± 10885",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 30380,
            "range": "± 919",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 87412,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 203745,
            "range": "± 900",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 353263,
            "range": "± 3160",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 518897,
            "range": "± 2581",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 688334,
            "range": "± 6797",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 837817,
            "range": "± 3702",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 25733,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 17093,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 226169,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 176754,
            "range": "± 8460",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 203553,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 227774,
            "range": "± 1118",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 253368,
            "range": "± 1636",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 28086,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 10265522,
            "range": "± 90371",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 9825792,
            "range": "± 109106",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 719,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 1905,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 4396,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 160188,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 14868,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 14030,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 14944,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1054268,
            "range": "± 6829",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 875322,
            "range": "± 5550",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 8955129,
            "range": "± 49961",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 8606803,
            "range": "± 56366",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 94822906,
            "range": "± 953754",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 91556867,
            "range": "± 1024892",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 5190,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 26917,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 108977,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 417,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 420,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 415,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2303,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 7926,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 31219,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 10410,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 53742,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 63932157,
            "range": "± 65557",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 3619030,
            "range": "± 11595",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 13037663,
            "range": "± 82004",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 925966,
            "range": "± 4981",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "e0ac9f2807fee7896dc75ba2f3a1510473157e57",
          "message": "feat(dream): reviewable dream snapshot pipeline (ENGRA-94..100) + apply contract fix (#61)\n\n* docs(harness): plan dream snapshot implementation\n\n* docs(engra-94): define dream snapshot review contract\n\n* docs(engra-100): add dream snapshot eval scaffolding\n\n* feat(engra-95): add dream snapshot storage\n\n* feat(engra-96): add deterministic dream candidate generator\n\n* feat(engra-97): add dream snapshot mcp review tools\n\n* fix(dream): harden candidate apply contract\n\n* fix(dream): complete eval contract and merge provenance\n\n* docs(harness): record dream snapshot post review gate",
          "timestamp": "2026-06-07T23:46:50-03:00",
          "tree_id": "03badf9f7793e5a45ce0fdda22f09156e8a5dd43",
          "url": "https://github.com/aiconnai/engram/commit/e0ac9f2807fee7896dc75ba2f3a1510473157e57"
        },
        "date": 1780887754575,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5915786,
            "range": "± 6540",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3577,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8749,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 319779,
            "range": "± 10525",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 508797,
            "range": "± 6010",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 278206,
            "range": "± 1222",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102326,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 159,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 278461,
            "range": "± 12123",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39413,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113277,
            "range": "± 2024",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 264175,
            "range": "± 1467",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 457697,
            "range": "± 9694",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 667822,
            "range": "± 4731",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 885292,
            "range": "± 9702",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1070403,
            "range": "± 20231",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33679,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22202,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 309974,
            "range": "± 6498",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 225542,
            "range": "± 4017",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 261975,
            "range": "± 1939",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 290955,
            "range": "± 1483",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 324053,
            "range": "± 1694",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36870,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12994739,
            "range": "± 70380",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12464486,
            "range": "± 84707",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 930,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2457,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5598,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 200490,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19426,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18466,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19380,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1337342,
            "range": "± 16179",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1090353,
            "range": "± 4378",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11332601,
            "range": "± 38122",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10872187,
            "range": "± 63140",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119035422,
            "range": "± 809412",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114832903,
            "range": "± 999891",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6638,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34545,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139810,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 529,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 532,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 519,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2609,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9977,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38858,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13288,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68794,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82055788,
            "range": "± 1303398",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4723694,
            "range": "± 12776",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16958566,
            "range": "± 63498",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1175799,
            "range": "± 41598",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "b2aa1ec256a07a3537a001166bee3124803d13a2",
          "message": "fix(engra-84): harden MCP HTTP rate-limit contract for ENGRA-84 (#62)",
          "timestamp": "2026-06-08T01:14:21-03:00",
          "tree_id": "e093d8bfdde6977233d3aa62fadcd053c9d0aee6",
          "url": "https://github.com/aiconnai/engram/commit/b2aa1ec256a07a3537a001166bee3124803d13a2"
        },
        "date": 1780892935034,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 4570960,
            "range": "± 4628",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 2830,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 6742,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 246817,
            "range": "± 11769",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 383527,
            "range": "± 2728",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 211710,
            "range": "± 1303",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 79259,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 155,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 215703,
            "range": "± 11453",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 30393,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 85871,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 206413,
            "range": "± 1033",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 345449,
            "range": "± 1440",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 516388,
            "range": "± 2843",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 667593,
            "range": "± 2563",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 831352,
            "range": "± 4169",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 25537,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 16942,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 234331,
            "range": "± 824",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 173844,
            "range": "± 1426",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 200400,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 223002,
            "range": "± 1004",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 249072,
            "range": "± 5175",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 27576,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 10132470,
            "range": "± 60314",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 9726960,
            "range": "± 67756",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 734,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 1926,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 4308,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 156442,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 15013,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 14369,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 15115,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1047040,
            "range": "± 3468",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 850525,
            "range": "± 6700",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 8831050,
            "range": "± 53904",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 8434547,
            "range": "± 38935",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 96984259,
            "range": "± 1259623",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 92964413,
            "range": "± 1029986",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 5173,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 26848,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 108767,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 409,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 408,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 410,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2006,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 7448,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 29525,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 10389,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 53724,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 63665553,
            "range": "± 129003",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 3611714,
            "range": "± 11293",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 12985663,
            "range": "± 36443",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 912575,
            "range": "± 5720",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "76d1089b6906336b0504ccade2b0a6fad727c098",
          "message": "feat(engra-103): add read-only memory_digest tool (#63)\n\n* docs(engra-103): add memory_digest RFC, plan, and canvas\n\n* feat(engra-103): add read-only memory_digest tool\n\n* docs(engra-103): cite self-correction rationale",
          "timestamp": "2026-06-09T08:38:17-03:00",
          "tree_id": "97aed72b031dcc888485ad4f5b4661e249673b65",
          "url": "https://github.com/aiconnai/engram/commit/76d1089b6906336b0504ccade2b0a6fad727c098"
        },
        "date": 1781006031944,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5537652,
            "range": "± 43241",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3467,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8158,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 378621,
            "range": "± 12202",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536980,
            "range": "± 2554",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287456,
            "range": "± 1595",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120425,
            "range": "± 1664",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 195,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 316018,
            "range": "± 10939",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46871,
            "range": "± 1368",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 111792,
            "range": "± 838",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 285558,
            "range": "± 1343",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 437984,
            "range": "± 4764",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 676130,
            "range": "± 6577",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 840583,
            "range": "± 3679",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1063549,
            "range": "± 17714",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34546,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21235,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 325103,
            "range": "± 3051",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 248461,
            "range": "± 4018",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 285437,
            "range": "± 3491",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 314150,
            "range": "± 4190",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 346217,
            "range": "± 1371",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45643,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12524234,
            "range": "± 92396",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12062030,
            "range": "± 86735",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 847,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2315,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5239,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 193812,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19070,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18087,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18892,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1344185,
            "range": "± 27646",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1082331,
            "range": "± 8418",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11123219,
            "range": "± 59086",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10598140,
            "range": "± 41228",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 115314269,
            "range": "± 1124878",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112638833,
            "range": "± 1314130",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6465,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33410,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 133425,
            "range": "± 2952",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 565,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 566,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 562,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2793,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11181,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 42436,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12936,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66007,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79310557,
            "range": "± 182624",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5184598,
            "range": "± 10406",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18836708,
            "range": "± 66013",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1237154,
            "range": "± 10035",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a0709821d71dfb1f82d1b38691a573d5204451a6",
          "message": "feat(engra-74): add explicit context artifact retrieval (#64)",
          "timestamp": "2026-06-09T09:13:27-03:00",
          "tree_id": "188e985d1e558e3f68f53d073f8facdcb783c0f6",
          "url": "https://github.com/aiconnai/engram/commit/a0709821d71dfb1f82d1b38691a573d5204451a6"
        },
        "date": 1781008146737,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5928430,
            "range": "± 17585",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3594,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8751,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325022,
            "range": "± 9450",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 510282,
            "range": "± 18055",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 281027,
            "range": "± 7711",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101450,
            "range": "± 2606",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 209,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 285895,
            "range": "± 15083",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39256,
            "range": "± 907",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113117,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 264035,
            "range": "± 4429",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 452046,
            "range": "± 3642",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 660315,
            "range": "± 6320",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 886190,
            "range": "± 8827",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1051979,
            "range": "± 25181",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32957,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22287,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 312502,
            "range": "± 5058",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 225771,
            "range": "± 4077",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 261127,
            "range": "± 4515",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 290544,
            "range": "± 1456",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 326361,
            "range": "± 15011",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36898,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13090002,
            "range": "± 237967",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12535206,
            "range": "± 131347",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 944,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2525,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5573,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 204452,
            "range": "± 786",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19264,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18205,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19095,
            "range": "± 786",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1348016,
            "range": "± 5605",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1097533,
            "range": "± 27476",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11374727,
            "range": "± 156790",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10791279,
            "range": "± 67170",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119654779,
            "range": "± 969363",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116145245,
            "range": "± 1150702",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6602,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34953,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141155,
            "range": "± 2463",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 528,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 527,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 528,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2694,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9783,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38354,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13340,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 70083,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82446294,
            "range": "± 317042",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4780198,
            "range": "± 27316",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17089050,
            "range": "± 56744",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1185282,
            "range": "± 24668",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "8d3ce98a805cb21698afa198058013f4ba62339e",
          "message": "test(mcp): add parity harness and model routing RFC (#71)\n\n* test(mcp): cover operational context search and bundle\n\n* test(mcp): add deterministic parity harness\n\n* docs(intelligence): draft model routing contract",
          "timestamp": "2026-06-12T15:58:56-03:00",
          "tree_id": "3fc2da6e2d5b2dafcf56d1ebc0dcc55034661555",
          "url": "https://github.com/aiconnai/engram/commit/8d3ce98a805cb21698afa198058013f4ba62339e"
        },
        "date": 1781291882209,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5940109,
            "range": "± 7486",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3607,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 10193,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 322699,
            "range": "± 12808",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 518430,
            "range": "± 4089",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288770,
            "range": "± 2705",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103101,
            "range": "± 1166",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 333,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 279172,
            "range": "± 12931",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39495,
            "range": "± 1010",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 110887,
            "range": "± 1051",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 268067,
            "range": "± 4549",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447539,
            "range": "± 2728",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 662839,
            "range": "± 4181",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 859675,
            "range": "± 11103",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1054308,
            "range": "± 15524",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32879,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21786,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 313841,
            "range": "± 1667",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 228494,
            "range": "± 4068",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 264277,
            "range": "± 1499",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 292417,
            "range": "± 1625",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 326576,
            "range": "± 1797",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36267,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 14278114,
            "range": "± 106693",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13649062,
            "range": "± 79674",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 934,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2454,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5596,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202333,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19348,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18085,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19278,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1373361,
            "range": "± 4715",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1112949,
            "range": "± 11495",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11652713,
            "range": "± 167450",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11031311,
            "range": "± 94504",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 123050787,
            "range": "± 3890994",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118671461,
            "range": "± 3083756",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6657,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34528,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140996,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 547,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 526,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 534,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2635,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9420,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 35657,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13356,
            "range": "± 831",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69265,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82177802,
            "range": "± 92451",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4658221,
            "range": "± 9970",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16722950,
            "range": "± 54580",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1196939,
            "range": "± 17952",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d5a2a7b43fbd13248023b65eb5e47e50753b0907",
          "message": "ci(github): always report required PR checks\n\nRemove the pull_request path filter so branch-protection checks are reported for every PR.",
          "timestamp": "2026-06-14T14:47:43-03:00",
          "tree_id": "de4708ad5479169b8e889c322346b53ddaee8f74",
          "url": "https://github.com/aiconnai/engram/commit/d5a2a7b43fbd13248023b65eb5e47e50753b0907"
        },
        "date": 1781460204256,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5368742,
            "range": "± 16159",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3543,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8995,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 369785,
            "range": "± 7509",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 534204,
            "range": "± 7994",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289511,
            "range": "± 1535",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118670,
            "range": "± 1231",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 348,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 318789,
            "range": "± 9554",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48105,
            "range": "± 1345",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115256,
            "range": "± 1015",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 300790,
            "range": "± 1940",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 451825,
            "range": "± 5079",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 704929,
            "range": "± 20713",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 868077,
            "range": "± 8353",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1097690,
            "range": "± 10066",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35140,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22504,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 341224,
            "range": "± 3651",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 254573,
            "range": "± 1311",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290329,
            "range": "± 956",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319829,
            "range": "± 1597",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 352786,
            "range": "± 2807",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45301,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12560636,
            "range": "± 38325",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12009679,
            "range": "± 52669",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 854,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2306,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5231,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192069,
            "range": "± 8896",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18380,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17390,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18386,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1354410,
            "range": "± 8804",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1085708,
            "range": "± 7379",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11157723,
            "range": "± 103363",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10552690,
            "range": "± 65934",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 115927346,
            "range": "± 1337587",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112636311,
            "range": "± 1681016",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6509,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33425,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 136137,
            "range": "± 1072",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 554,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 561,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 554,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2722,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9885,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37475,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12893,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66532,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79857114,
            "range": "± 116698",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5311183,
            "range": "± 65228",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19325815,
            "range": "± 438545",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1257791,
            "range": "± 14589",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "c00326be45b1f5aa3710b2724c3e107088a15455",
          "message": "chore(docs): organize docs and benchmark artifacts\n\nMove integration guides under docs/integrations and benchmark snapshots under benches/results.",
          "timestamp": "2026-06-15T14:05:42-03:00",
          "tree_id": "1029c2bc495ac23e7fd2be3acae9831df833a3cf",
          "url": "https://github.com/aiconnai/engram/commit/c00326be45b1f5aa3710b2724c3e107088a15455"
        },
        "date": 1781544069652,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5401354,
            "range": "± 31924",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3572,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 9017,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 375696,
            "range": "± 11915",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 528830,
            "range": "± 3352",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287571,
            "range": "± 4290",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119527,
            "range": "± 802",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 353,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 318951,
            "range": "± 11231",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46994,
            "range": "± 1308",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114717,
            "range": "± 1622",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 298380,
            "range": "± 6543",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 448835,
            "range": "± 3908",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 697273,
            "range": "± 6453",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 855241,
            "range": "± 7595",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1092258,
            "range": "± 19845",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35597,
            "range": "± 1063",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21672,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 329378,
            "range": "± 3671",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 249397,
            "range": "± 3247",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 285345,
            "range": "± 15747",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 314595,
            "range": "± 5244",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 348138,
            "range": "± 2061",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45414,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12806136,
            "range": "± 237244",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12009859,
            "range": "± 120625",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 901,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2301,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5228,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192435,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18421,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17445,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18363,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1354406,
            "range": "± 5164",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1089963,
            "range": "± 10149",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11109933,
            "range": "± 186807",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10548649,
            "range": "± 83419",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119336864,
            "range": "± 1008583",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113589303,
            "range": "± 2664173",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6468,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33421,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135168,
            "range": "± 743",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 559,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 555,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 556,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2880,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9743,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40967,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12968,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 65924,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79817681,
            "range": "± 347767",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5262568,
            "range": "± 98682",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19045954,
            "range": "± 99182",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1261594,
            "range": "± 33454",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "60c36b8d0f8f6f36a4069c3b8f287ee1b72fe78e",
          "message": "chore(release): prepare 0.21.1 (#81)",
          "timestamp": "2026-06-15T15:12:30-03:00",
          "tree_id": "90c300368f77dceb8c72401f512ded7f97150c6c",
          "url": "https://github.com/aiconnai/engram/commit/60c36b8d0f8f6f36a4069c3b8f287ee1b72fe78e"
        },
        "date": 1781548110760,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5945296,
            "range": "± 45107",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3659,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8976,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 323206,
            "range": "± 11913",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 512008,
            "range": "± 3338",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288980,
            "range": "± 2479",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103456,
            "range": "± 657",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 338,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 276640,
            "range": "± 16060",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39476,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113471,
            "range": "± 14932",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 269823,
            "range": "± 3012",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 461156,
            "range": "± 14529",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 678406,
            "range": "± 16971",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 877373,
            "range": "± 8124",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1081493,
            "range": "± 17052",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33128,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22542,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 311864,
            "range": "± 2249",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 225965,
            "range": "± 2896",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 260475,
            "range": "± 1409",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 288943,
            "range": "± 1962",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 322944,
            "range": "± 2738",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35995,
            "range": "± 600",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12670229,
            "range": "± 68815",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12142845,
            "range": "± 219251",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 956,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2520,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5687,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 206254,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19184,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18137,
            "range": "± 518",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19252,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1318450,
            "range": "± 5974",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1064272,
            "range": "± 19583",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11058424,
            "range": "± 28854",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10618482,
            "range": "± 188866",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 116339140,
            "range": "± 3014132",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113070131,
            "range": "± 879908",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6650,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34989,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140351,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 548,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 525,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 526,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2672,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9930,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38216,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13362,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69245,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82339606,
            "range": "± 144153",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4717881,
            "range": "± 21966",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17059406,
            "range": "± 196573",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1188688,
            "range": "± 54109",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "00f916a8bbac49156f7b42f484f793d2333e716d",
          "message": "ci(release): use node24 action versions (#83)",
          "timestamp": "2026-06-15T15:42:25-03:00",
          "tree_id": "6b92d233cecf4c271293a1402616d5e5473f8ada",
          "url": "https://github.com/aiconnai/engram/commit/00f916a8bbac49156f7b42f484f793d2333e716d"
        },
        "date": 1781549936617,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5458461,
            "range": "± 43208",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3519,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8216,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 380708,
            "range": "± 15962",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 531404,
            "range": "± 7105",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287818,
            "range": "± 2430",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118491,
            "range": "± 1872",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 338,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 331088,
            "range": "± 15464",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48463,
            "range": "± 1722",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114119,
            "range": "± 2391",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 297296,
            "range": "± 2778",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 443009,
            "range": "± 3104",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 695316,
            "range": "± 3925",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 867090,
            "range": "± 7759",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1089175,
            "range": "± 14652",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34767,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21592,
            "range": "± 512",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 333314,
            "range": "± 2342",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 250792,
            "range": "± 4791",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 286774,
            "range": "± 5450",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 315937,
            "range": "± 2489",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 350736,
            "range": "± 10325",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45242,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12813376,
            "range": "± 226927",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12164202,
            "range": "± 227961",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 828,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2281,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5198,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192733,
            "range": "± 2575",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18464,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17533,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18523,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1345209,
            "range": "± 32527",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1078528,
            "range": "± 4457",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11400008,
            "range": "± 190984",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10814427,
            "range": "± 248873",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118197872,
            "range": "± 2286262",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112885208,
            "range": "± 806142",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6750,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33919,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138091,
            "range": "± 2452",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 570,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 573,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 570,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2916,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10512,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 41698,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13409,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67859,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80370304,
            "range": "± 762990",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5331017,
            "range": "± 13765",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19331510,
            "range": "± 539510",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1257689,
            "range": "± 10033",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "475fe848b61a3c179df093bcc98bd6c0d8774196",
          "message": "ci(release): make homebrew update idempotent",
          "timestamp": "2026-06-15T20:38:55-03:00",
          "tree_id": "ee68fe58a5be7daf3205b960e040eae0e4d30881",
          "url": "https://github.com/aiconnai/engram/commit/475fe848b61a3c179df093bcc98bd6c0d8774196"
        },
        "date": 1781567683204,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5327727,
            "range": "± 18746",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3511,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8160,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 367982,
            "range": "± 6505",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 539704,
            "range": "± 4074",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 291142,
            "range": "± 1591",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118798,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 336,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 313499,
            "range": "± 12375",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47363,
            "range": "± 1354",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114453,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 292900,
            "range": "± 3401",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 442357,
            "range": "± 1762",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 688045,
            "range": "± 7492",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 851563,
            "range": "± 7269",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1079110,
            "range": "± 19950",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34648,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21580,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 332213,
            "range": "± 6275",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 247196,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 284837,
            "range": "± 2721",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 316384,
            "range": "± 1975",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 347081,
            "range": "± 1925",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45072,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12502112,
            "range": "± 218221",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11930873,
            "range": "± 119428",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 846,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2289,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5214,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192261,
            "range": "± 1100",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18591,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17385,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18620,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1337165,
            "range": "± 13144",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1077506,
            "range": "± 3877",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10958435,
            "range": "± 42156",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10416703,
            "range": "± 63382",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 114090308,
            "range": "± 1731542",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 110387701,
            "range": "± 1539911",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6788,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33887,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 136495,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 574,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 557,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 549,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3004,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10283,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39916,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13022,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68068,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79230437,
            "range": "± 188765",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5206192,
            "range": "± 16127",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18979720,
            "range": "± 74569",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1256292,
            "range": "± 26333",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "78dde4d3aee3df8265886a8cfa2ea268a770b2a3",
          "message": "ci(ci): guard existing release assets",
          "timestamp": "2026-06-15T20:59:37-03:00",
          "tree_id": "5b4bac015a14e48b5520fb16c39e51ad22fede19",
          "url": "https://github.com/aiconnai/engram/commit/78dde4d3aee3df8265886a8cfa2ea268a770b2a3"
        },
        "date": 1781568897067,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5942769,
            "range": "± 13455",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3613,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8786,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 326454,
            "range": "± 8263",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 512799,
            "range": "± 3447",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 286768,
            "range": "± 4042",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103108,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 290,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 278290,
            "range": "± 10941",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39824,
            "range": "± 895",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114315,
            "range": "± 851",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 274414,
            "range": "± 1803",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 463555,
            "range": "± 2077",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 684029,
            "range": "± 4033",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 895132,
            "range": "± 42345",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1091928,
            "range": "± 19467",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32970,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22405,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 315122,
            "range": "± 3864",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 228709,
            "range": "± 2157",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 266756,
            "range": "± 3500",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 297775,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 330654,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35834,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12966684,
            "range": "± 170280",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12461165,
            "range": "± 166527",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 939,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2437,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5542,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201021,
            "range": "± 14921",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19215,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18036,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19148,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1333303,
            "range": "± 7000",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1084747,
            "range": "± 11668",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11360704,
            "range": "± 620260",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10774088,
            "range": "± 84698",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118304857,
            "range": "± 1121424",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116894087,
            "range": "± 1079970",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6644,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34622,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140978,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 537,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 523,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 524,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2711,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10365,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 36761,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13287,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69764,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82457008,
            "range": "± 153171",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4904389,
            "range": "± 25040",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17579739,
            "range": "± 47034",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1193896,
            "range": "± 33124",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "145ac7abc7f9bbf50f5d4cb30523de4d6f88b32f",
          "message": "ci(ci): control release notes (#86)",
          "timestamp": "2026-06-15T21:54:25-03:00",
          "tree_id": "4bcd97098309cf2b7e2966b1b1887ec755254852",
          "url": "https://github.com/aiconnai/engram/commit/145ac7abc7f9bbf50f5d4cb30523de4d6f88b32f"
        },
        "date": 1781572191707,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5938074,
            "range": "± 5933",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3575,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8747,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 326702,
            "range": "± 11532",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513850,
            "range": "± 3316",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 284174,
            "range": "± 1529",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103259,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 337,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 279671,
            "range": "± 11697",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40222,
            "range": "± 826",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115473,
            "range": "± 1188",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 279723,
            "range": "± 4648",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 465628,
            "range": "± 1908",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 687639,
            "range": "± 13926",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 894660,
            "range": "± 5491",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1097382,
            "range": "± 24885",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33544,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 23478,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 312468,
            "range": "± 1648",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 226042,
            "range": "± 1662",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 263114,
            "range": "± 1575",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 291122,
            "range": "± 2865",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 324147,
            "range": "± 1179",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36783,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12777646,
            "range": "± 53585",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12378579,
            "range": "± 102717",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 929,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2466,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5540,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 204322,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19072,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17957,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19136,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1319173,
            "range": "± 6809",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1083210,
            "range": "± 4747",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11152856,
            "range": "± 286417",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10670069,
            "range": "± 253553",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118125856,
            "range": "± 1548972",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115281217,
            "range": "± 1314370",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6624,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34700,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140200,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 573,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 528,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 514,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2676,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10150,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39020,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13413,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69013,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82317784,
            "range": "± 1037192",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4669514,
            "range": "± 26505",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16867213,
            "range": "± 97033",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1197344,
            "range": "± 20266",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a11337aa5cea8e3e4284f3e1a827298e21cf4c3c",
          "message": "chore(ci): prepare v0.21.2 (#87)",
          "timestamp": "2026-06-15T22:18:53-03:00",
          "tree_id": "bd4b930fc99163670f370bb621238b2e2c91e2aa",
          "url": "https://github.com/aiconnai/engram/commit/a11337aa5cea8e3e4284f3e1a827298e21cf4c3c"
        },
        "date": 1781573689761,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5401874,
            "range": "± 32715",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3522,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8441,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 379446,
            "range": "± 27051",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 540198,
            "range": "± 2688",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 291790,
            "range": "± 19888",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 121066,
            "range": "± 4192",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 363,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 319017,
            "range": "± 10299",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47705,
            "range": "± 1204",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 116722,
            "range": "± 567",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 299670,
            "range": "± 1939",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 451734,
            "range": "± 2438",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 701715,
            "range": "± 4975",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 876157,
            "range": "± 8593",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1095231,
            "range": "± 9902",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35013,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22033,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 333066,
            "range": "± 2636",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 250935,
            "range": "± 1485",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289178,
            "range": "± 2901",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319152,
            "range": "± 6230",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 352554,
            "range": "± 2804",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44841,
            "range": "± 1334",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12567440,
            "range": "± 141222",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12012237,
            "range": "± 434112",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 861,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2300,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5141,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190416,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18075,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17112,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18171,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1349508,
            "range": "± 105302",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1074947,
            "range": "± 4783",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11113320,
            "range": "± 115202",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10435456,
            "range": "± 210540",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117968737,
            "range": "± 2849043",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113511894,
            "range": "± 3281771",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6511,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33227,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135476,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 554,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 556,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 538,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2602,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9492,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37305,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12930,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66080,
            "range": "± 945",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81059941,
            "range": "± 1082332",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5297840,
            "range": "± 27676",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19254180,
            "range": "± 88145",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1259972,
            "range": "± 21362",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d1b46f1698adb9460a509ccf538cd4627c9509c4",
          "message": "ci(harness): add agentshield loop (#89)",
          "timestamp": "2026-06-16T08:09:57-03:00",
          "tree_id": "0cb4ba13ea762ad981f3603259c50145b0d35070",
          "url": "https://github.com/aiconnai/engram/commit/d1b46f1698adb9460a509ccf538cd4627c9509c4"
        },
        "date": 1781609128325,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5403321,
            "range": "± 15725",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3493,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8128,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 364396,
            "range": "± 6357",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 528963,
            "range": "± 7686",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 286634,
            "range": "± 1458",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119730,
            "range": "± 1583",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 357,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 321099,
            "range": "± 12736",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47224,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112172,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 294700,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 445546,
            "range": "± 5698",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 691330,
            "range": "± 6002",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 854930,
            "range": "± 9633",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1081700,
            "range": "± 23762",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35229,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21886,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 332488,
            "range": "± 1891",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 249751,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 283688,
            "range": "± 1605",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 313703,
            "range": "± 4141",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 345383,
            "range": "± 2529",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44387,
            "range": "± 889",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12556219,
            "range": "± 142192",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11997953,
            "range": "± 92671",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 867,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2311,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5120,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 189850,
            "range": "± 854",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18140,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17106,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18126,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1338684,
            "range": "± 18730",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1079468,
            "range": "± 5962",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10999013,
            "range": "± 36435",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10532188,
            "range": "± 118602",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 116122009,
            "range": "± 2372731",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 111833806,
            "range": "± 1085726",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6449,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33489,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 136108,
            "range": "± 506",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 556,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 556,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 557,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2713,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10045,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39412,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12932,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66310,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79804091,
            "range": "± 273541",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5167910,
            "range": "± 17310",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18781153,
            "range": "± 94790",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1238995,
            "range": "± 25569",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "bf45c0bb8590d1027026b0b1a259141c1b008094",
          "message": "chore(intelligence): complete code quality maintenance cleanup (#90)\n\n* chore(intelligence): complete code quality maintenance cleanup\n\n* chore(harness): add L1 daily triage starter\n\n* chore(harness): record code-quality review blocker\n\n* chore(harness): close code-quality post-review\n\n* fix(sdk-python): close post-review follow-ups\n\n* fix(intelligence): address review blockers",
          "timestamp": "2026-06-19T15:55:43-03:00",
          "tree_id": "4f940f84819a78be9a1027c83d7438c5d6d366b4",
          "url": "https://github.com/aiconnai/engram/commit/bf45c0bb8590d1027026b0b1a259141c1b008094"
        },
        "date": 1781896393922,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5951095,
            "range": "± 269007",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3582,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382402,
            "range": "± 5445",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325476,
            "range": "± 7257",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 507308,
            "range": "± 2474",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 277512,
            "range": "± 2397",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101812,
            "range": "± 1004",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 333,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 277286,
            "range": "± 11518",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 38985,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113664,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 268395,
            "range": "± 3027",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 450689,
            "range": "± 2532",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 666906,
            "range": "± 3288",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 870072,
            "range": "± 6693",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1059857,
            "range": "± 12968",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33045,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21845,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 316891,
            "range": "± 1366",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 227781,
            "range": "± 1977",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 261443,
            "range": "± 2046",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 292340,
            "range": "± 2965",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 325590,
            "range": "± 2276",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35878,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13228398,
            "range": "± 75365",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12745331,
            "range": "± 63965",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 929,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2459,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5631,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202403,
            "range": "± 3448",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19100,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17929,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19157,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1365002,
            "range": "± 47896",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1120414,
            "range": "± 10823",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11583482,
            "range": "± 50637",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11037349,
            "range": "± 78430",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 123551438,
            "range": "± 1972475",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 120942233,
            "range": "± 1168011",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6607,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34678,
            "range": "± 2381",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140685,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 524,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 527,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 533,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2577,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9823,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39870,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13529,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69405,
            "range": "± 5532",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82535759,
            "range": "± 79385",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4662850,
            "range": "± 56753",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16764689,
            "range": "± 32063",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1191002,
            "range": "± 20268",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "7b5cbeacaddb141f0fecbe3c8cca3bd09f1e1782",
          "message": "fix(storage): make extension semantics explicit",
          "timestamp": "2026-06-20T13:07:09-03:00",
          "tree_id": "f80e756ee03908e8a42bc3765ac5d797761e6dc8",
          "url": "https://github.com/aiconnai/engram/commit/7b5cbeacaddb141f0fecbe3c8cca3bd09f1e1782"
        },
        "date": 1781972599787,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5948629,
            "range": "± 72954",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3663,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 383112,
            "range": "± 7963",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 321504,
            "range": "± 7735",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 506127,
            "range": "± 2359",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 279646,
            "range": "± 3697",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103337,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 335,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 281613,
            "range": "± 12232",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39328,
            "range": "± 989",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112145,
            "range": "± 3914",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 267225,
            "range": "± 2999",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 444035,
            "range": "± 3284",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 671927,
            "range": "± 6861",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 869108,
            "range": "± 6148",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1058153,
            "range": "± 22020",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32599,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22039,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 312671,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 228042,
            "range": "± 2672",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 261374,
            "range": "± 2094",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 292740,
            "range": "± 14950",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 326974,
            "range": "± 2516",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36631,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13604255,
            "range": "± 157263",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13225910,
            "range": "± 157354",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 969,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2483,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5644,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 205537,
            "range": "± 2655",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19236,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17967,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19156,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1367393,
            "range": "± 14293",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1123472,
            "range": "± 8116",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11628095,
            "range": "± 167105",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11048333,
            "range": "± 62435",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121803683,
            "range": "± 749920",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118535281,
            "range": "± 968285",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6655,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34713,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140029,
            "range": "± 4939",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 555,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 525,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 514,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2886,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10541,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 41657,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13316,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69675,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82536455,
            "range": "± 202655",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4648224,
            "range": "± 16323",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16686043,
            "range": "± 180294",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1204038,
            "range": "± 45321",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "079d13385c47ec55e256ae9c11d81f7a4e953d15",
          "message": "fix: harden API key validation and wasm clippy (#95)\n\n* fix(auth): use constant-time comparison for API key hash\n\nAPI key validation compared the recomputed SHA-256 hash against the\nstored hash with plain string ==, which is not constant-time and can\nleak the stored hash byte-by-byte via a timing oracle. Switch to\nsubtle::ConstantTimeEq::ct_eq, mirroring the existing pattern in\nattestation/chain.rs. Both operands are fixed-length hex SHA-256\ndigests, so no length is leaked.\n\n* fix(wasm): resolve clippy -D warnings in engram-wasm\n\nengram-wasm failed the cargo clippy --workspace -- -D warnings done-gate\nwith three errors: two manual_is_multiple_of (% 2 == 0) in tfidf.rs and\none unnecessary_sort_by in graph.rs. Apply the clippy-suggested rewrites\n(is_multiple_of, sort_by_key + Reverse). Behavior unchanged.",
          "timestamp": "2026-06-21T16:28:33-03:00",
          "tree_id": "44306255bc9c6e1031f0ede52001a918558c8a5c",
          "url": "https://github.com/aiconnai/engram/commit/079d13385c47ec55e256ae9c11d81f7a4e953d15"
        },
        "date": 1782071056804,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5370638,
            "range": "± 10894",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3599,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382368,
            "range": "± 6258",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 367090,
            "range": "± 10722",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 535988,
            "range": "± 3021",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289388,
            "range": "± 2358",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117804,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 344,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 324437,
            "range": "± 14035",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47950,
            "range": "± 1347",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 116934,
            "range": "± 1071",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 297953,
            "range": "± 2621",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 444243,
            "range": "± 4750",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 693473,
            "range": "± 9333",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 851431,
            "range": "± 7316",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1081674,
            "range": "± 10554",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34857,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21503,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 361250,
            "range": "± 3806",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 253888,
            "range": "± 3263",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 293432,
            "range": "± 1952",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 326334,
            "range": "± 2706",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 360537,
            "range": "± 2267",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44986,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12811513,
            "range": "± 102275",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12291171,
            "range": "± 119994",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 849,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2305,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5314,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 194132,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17867,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16699,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17925,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1381031,
            "range": "± 15636",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1106116,
            "range": "± 6577",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11343130,
            "range": "± 114493",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10845036,
            "range": "± 136362",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119167773,
            "range": "± 1082586",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115126312,
            "range": "± 1347200",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6526,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33943,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137127,
            "range": "± 2135",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 558,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 555,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 554,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2569,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10256,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39265,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13155,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67534,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80335893,
            "range": "± 227194",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5205724,
            "range": "± 21116",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18971526,
            "range": "± 109192",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1244282,
            "range": "± 15367",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "4a27698fc769e9d3b72526940bad480aff5c9bc0",
          "message": "fix: constant-time API key compare + clean up engram-wasm clippy\n\nMerged PR #96 after resolving conflicts with origin/main. Required checks passed: Format, Clippy, Documentation, and Test (ubuntu-latest).",
          "timestamp": "2026-06-21T17:25:39-03:00",
          "tree_id": "c408d834ebc7253e0a602472f8d308a187a52d2c",
          "url": "https://github.com/aiconnai/engram/commit/4a27698fc769e9d3b72526940bad480aff5c9bc0"
        },
        "date": 1782074487878,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5939340,
            "range": "± 16360",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3638,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384022,
            "range": "± 10272",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 322974,
            "range": "± 8957",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 516575,
            "range": "± 6912",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288698,
            "range": "± 3304",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101678,
            "range": "± 2410",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 336,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 278250,
            "range": "± 10817",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39363,
            "range": "± 910",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112247,
            "range": "± 1946",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 272031,
            "range": "± 7124",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 445926,
            "range": "± 3117",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 670476,
            "range": "± 6435",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 872498,
            "range": "± 15760",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1067342,
            "range": "± 18998",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33084,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21436,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 316142,
            "range": "± 5642",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 225854,
            "range": "± 2535",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 262595,
            "range": "± 1448",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 290490,
            "range": "± 19383",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 323380,
            "range": "± 2056",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36113,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13597148,
            "range": "± 125824",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13099704,
            "range": "± 545099",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 930,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2453,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5544,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202103,
            "range": "± 3899",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19259,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18003,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19072,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1336481,
            "range": "± 24300",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1110254,
            "range": "± 38643",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11335867,
            "range": "± 204175",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10939271,
            "range": "± 182714",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121111574,
            "range": "± 1315799",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 117491251,
            "range": "± 1072599",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6669,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 35222,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141657,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 568,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 529,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 536,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2753,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9882,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38147,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13440,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69777,
            "range": "± 566",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83777030,
            "range": "± 217286",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4682522,
            "range": "± 46079",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16810615,
            "range": "± 66617",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1199453,
            "range": "± 35207",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "e9ac318b8aeaef0e747d37eb74cd82cddeff0491",
          "message": "Harness parity policy and final review evidence (#97)\n\n* fix(harness): align PR title policy scripts\n\n* docs(harness): record final full gate evidence\n\n* docs(harness): add final harness parity review evidence",
          "timestamp": "2026-06-21T18:51:03-03:00",
          "tree_id": "2bb8504658e962761ec1e4d762169ab70160655f",
          "url": "https://github.com/aiconnai/engram/commit/e9ac318b8aeaef0e747d37eb74cd82cddeff0491"
        },
        "date": 1782079588714,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5419857,
            "range": "± 60085",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3649,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 390909,
            "range": "± 5523",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 364214,
            "range": "± 12643",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 535647,
            "range": "± 3995",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290300,
            "range": "± 2005",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118647,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 326,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 313935,
            "range": "± 10858",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46450,
            "range": "± 1219",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114572,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 294653,
            "range": "± 4224",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 441744,
            "range": "± 2182",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686518,
            "range": "± 6172",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 848544,
            "range": "± 4300",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1076569,
            "range": "± 8899",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34930,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21496,
            "range": "± 890",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 331719,
            "range": "± 3175",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 248482,
            "range": "± 4808",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 284166,
            "range": "± 1957",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 316779,
            "range": "± 9262",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 347361,
            "range": "± 1302",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44488,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12618833,
            "range": "± 137803",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12088744,
            "range": "± 230171",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 860,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2300,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5128,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 189512,
            "range": "± 2502",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18095,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17064,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18064,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1348061,
            "range": "± 2666",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1085135,
            "range": "± 3175",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11158641,
            "range": "± 33651",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10579553,
            "range": "± 51165",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119457440,
            "range": "± 1032710",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114190926,
            "range": "± 879479",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6592,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33956,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137347,
            "range": "± 413",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 607,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 603,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 604,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2895,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9972,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39771,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13172,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68334,
            "range": "± 3768",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81181743,
            "range": "± 91521",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5233539,
            "range": "± 9524",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19067621,
            "range": "± 57088",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1252061,
            "range": "± 39548",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "3325e7695078bfa8350c3dd6fcf3d0ee1df6d62f",
          "message": "fix: preserve subsecond replay cutoff (#98)\n\n* fix(mcp): preserve subsecond replay cutoff\n\n* docs(harness): add enrichment replay post-review evidence",
          "timestamp": "2026-06-21T20:03:56-03:00",
          "tree_id": "6a852ddd01323cf06a754abb54d0fbeb72cfadec",
          "url": "https://github.com/aiconnai/engram/commit/3325e7695078bfa8350c3dd6fcf3d0ee1df6d62f"
        },
        "date": 1782083999036,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5371700,
            "range": "± 145036",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3543,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 391826,
            "range": "± 1486",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 375159,
            "range": "± 11293",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536714,
            "range": "± 4116",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 293700,
            "range": "± 2290",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118821,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 325,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 322353,
            "range": "± 12407",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47950,
            "range": "± 1242",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115903,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 302052,
            "range": "± 2431",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 449389,
            "range": "± 2436",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 702411,
            "range": "± 4136",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 866269,
            "range": "± 8140",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1100368,
            "range": "± 54857",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34956,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21643,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330472,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 256801,
            "range": "± 1633",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 291142,
            "range": "± 1571",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 324348,
            "range": "± 2165",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 355896,
            "range": "± 3069",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44778,
            "range": "± 880",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13028451,
            "range": "± 244663",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12396271,
            "range": "± 193707",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 833,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2277,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5172,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191395,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18190,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17178,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18072,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1390087,
            "range": "± 22055",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1119451,
            "range": "± 10480",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11439489,
            "range": "± 148683",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10833377,
            "range": "± 158340",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121829038,
            "range": "± 1248581",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116572878,
            "range": "± 895561",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6601,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33961,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137931,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 614,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 584,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 589,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3087,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11250,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44845,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13323,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67913,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81554751,
            "range": "± 148636",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5297963,
            "range": "± 42383",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19362306,
            "range": "± 74710",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1256726,
            "range": "± 20552",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "3c534d9dca960f9a77566f44d1d5fe206b98d8e3",
          "message": "docs(harness): add SKILLS.md policy and loop-engineering skill (#100)\n\n* docs(harness): add SKILLS.md policy and loop-engineering skill\n\n* docs(harness): add B2 loop-skills review evidence\n\n* docs(harness): normalize B2 review artifacts",
          "timestamp": "2026-06-21T20:35:12-03:00",
          "tree_id": "13998ec70e66300c0dc731771ddea8639d229fb7",
          "url": "https://github.com/aiconnai/engram/commit/3c534d9dca960f9a77566f44d1d5fe206b98d8e3"
        },
        "date": 1782085854676,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5930731,
            "range": "± 55460",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3588,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382691,
            "range": "± 1438",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 319307,
            "range": "± 9271",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 510806,
            "range": "± 8720",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 278421,
            "range": "± 2689",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101009,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 288,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 280487,
            "range": "± 10998",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39261,
            "range": "± 919",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114187,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 270664,
            "range": "± 1247",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 460093,
            "range": "± 2781",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 678802,
            "range": "± 4066",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 897883,
            "range": "± 6358",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1085817,
            "range": "± 19302",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33070,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22925,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 315914,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 225745,
            "range": "± 1877",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 260019,
            "range": "± 2002",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 293850,
            "range": "± 5372",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 322888,
            "range": "± 1724",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36113,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12969474,
            "range": "± 278661",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12557803,
            "range": "± 68248",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 925,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2440,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5453,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 199947,
            "range": "± 1622",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19196,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18039,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19142,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1330882,
            "range": "± 5768",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1103696,
            "range": "± 5353",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11322692,
            "range": "± 62768",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10878256,
            "range": "± 66354",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119402411,
            "range": "± 652806",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116089508,
            "range": "± 588814",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6643,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34769,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141323,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 554,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 551,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 549,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2514,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9507,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37318,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13318,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69486,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 84100374,
            "range": "± 309362",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4700866,
            "range": "± 17193",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16867898,
            "range": "± 52240",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1195549,
            "range": "± 16365",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "369781d942dba89addf00ceb274fbaf8c971c1f9",
          "message": "fix(mcp): honor memory export workspace and scope\n\nRecover memory_export workspace filtering and scope round-trip from old stash cleanup.",
          "timestamp": "2026-06-21T22:52:40-03:00",
          "tree_id": "3e253a98c9eda27f7f73802f79b0122ba9ec9b07",
          "url": "https://github.com/aiconnai/engram/commit/369781d942dba89addf00ceb274fbaf8c971c1f9"
        },
        "date": 1782094093163,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5364836,
            "range": "± 34504",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3499,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 393215,
            "range": "± 2602",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 384897,
            "range": "± 10507",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 541302,
            "range": "± 4042",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 294390,
            "range": "± 1754",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 122182,
            "range": "± 1810",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 344,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 342559,
            "range": "± 16286",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48224,
            "range": "± 1342",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117080,
            "range": "± 1875",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 302668,
            "range": "± 4391",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 451815,
            "range": "± 3595",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 708337,
            "range": "± 10482",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 869911,
            "range": "± 10991",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1102281,
            "range": "± 11052",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35579,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22022,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 332235,
            "range": "± 8676",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252886,
            "range": "± 1275",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289032,
            "range": "± 1427",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 318065,
            "range": "± 2989",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 352280,
            "range": "± 6326",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45530,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13395278,
            "range": "± 159992",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12698746,
            "range": "± 180281",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 839,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2321,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5182,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190490,
            "range": "± 1445",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18168,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17159,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18154,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1365102,
            "range": "± 18885",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1094900,
            "range": "± 3697",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11210810,
            "range": "± 60166",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10630119,
            "range": "± 60136",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117700935,
            "range": "± 3029595",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 111719645,
            "range": "± 2446185",
            "unit": "ns/iter"
          },
          {
            "name": "search_index_v2_report/noop",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6572,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34232,
            "range": "± 1260",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138528,
            "range": "± 3516",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 622,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 613,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 610,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3127,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11273,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43586,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13706,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67514,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81347237,
            "range": "± 558766",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5377933,
            "range": "± 94793",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19294489,
            "range": "± 89830",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1270336,
            "range": "± 15416",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}