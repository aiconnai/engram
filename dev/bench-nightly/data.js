window.BENCHMARK_DATA = {
  "lastUpdate": 1780643543235,
  "repoUrl": "https://github.com/aiconnai/engram",
  "entries": {
    "Engram Performance (Nightly)": [
      {
        "commit": {
          "author": {
            "name": "Ronaldo Martins",
            "username": "limaronaldo",
            "email": "ron@ldinho.com.br"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ce658775306d340903e5df61b736fa9923005fa8",
          "message": "fix(mcp): align tool registry and markdown sync (#53)\n\n* refactor(mcp): split tool registry by domain\n\n* fix(mcp): align tool registry and markdown sync",
          "timestamp": "2026-06-04T19:04:12Z",
          "url": "https://github.com/aiconnai/engram/commit/ce658775306d340903e5df61b736fa9923005fa8"
        },
        "date": 1780643542405,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5911459,
            "range": "± 9032",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3623,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 9753,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 181777,
            "range": "± 2952",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 510915,
            "range": "± 8201",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 282316,
            "range": "± 1600",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 104426,
            "range": "± 357",
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
            "value": 192423,
            "range": "± 9807",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39215,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 111679,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 272192,
            "range": "± 11004",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 437945,
            "range": "± 3549",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 671585,
            "range": "± 4248",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 842593,
            "range": "± 7666",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1065781,
            "range": "± 37272",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33046,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21879,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 312248,
            "range": "± 4909",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 227803,
            "range": "± 1649",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 268220,
            "range": "± 4461",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 304911,
            "range": "± 2122",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 346226,
            "range": "± 18511",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 42453,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 489369,
            "range": "± 3453",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 68338,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 956,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2461,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5635,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203787,
            "range": "± 2626",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19153,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18079,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19214,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 200552,
            "range": "± 2141",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 493429,
            "range": "± 2338",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2123125,
            "range": "± 9125",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6654,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34868,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141328,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 545,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 532,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 539,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2911,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10031,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39956,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13515,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69613,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83827439,
            "range": "± 196504",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4684089,
            "range": "± 15998",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16886557,
            "range": "± 67321",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1182142,
            "range": "± 11828",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}