window.BENCHMARK_DATA = {
  "lastUpdate": 1780887754906,
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
      }
    ]
  }
}