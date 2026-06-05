window.BENCHMARK_DATA = {
  "lastUpdate": 1780687004489,
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
      }
    ]
  }
}