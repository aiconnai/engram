window.BENCHMARK_DATA = {
  "lastUpdate": 1782027332425,
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
      },
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
          "id": "443278b467bb1a5c80b4ed6975ff6cd20756528c",
          "message": "docs(harness): enforce security reference boundary (#60)",
          "timestamp": "2026-06-06T05:27:37Z",
          "url": "https://github.com/aiconnai/engram/commit/443278b467bb1a5c80b4ed6975ff6cd20756528c"
        },
        "date": 1780727515874,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5941649,
            "range": "± 1689",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3603,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8801,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 322353,
            "range": "± 10310",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 515025,
            "range": "± 2032",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283890,
            "range": "± 5980",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103144,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 204,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 281171,
            "range": "± 15574",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40572,
            "range": "± 888",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115458,
            "range": "± 1770",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 265722,
            "range": "± 2203",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 465786,
            "range": "± 2918",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 677253,
            "range": "± 9423",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 897570,
            "range": "± 10583",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1082975,
            "range": "± 12301",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33435,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22513,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 299256,
            "range": "± 1473",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229594,
            "range": "± 1326",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 265707,
            "range": "± 1625",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 300494,
            "range": "± 2558",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 343443,
            "range": "± 1658",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 41013,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 491312,
            "range": "± 2830",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 68474,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 981,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2665,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 6069,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201808,
            "range": "± 710",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19121,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17860,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18986,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 201398,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 489405,
            "range": "± 2395",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2149375,
            "range": "± 7678",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6658,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34878,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141746,
            "range": "± 1226",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 554,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 549,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 544,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2842,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10742,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 41835,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13418,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69379,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83022889,
            "range": "± 149033",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4759435,
            "range": "± 35399",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16904006,
            "range": "± 61388",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1184231,
            "range": "± 42592",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Ronaldo Martins",
            "username": "limaronaldo",
            "email": "ron@ldinho.com.br"
          },
          "committer": {
            "name": "Ronaldo Martins",
            "username": "limaronaldo",
            "email": "ron@ldinho.com.br"
          },
          "id": "52810094ae072807f5c2590b2b4fbfe1c518370e",
          "message": "ci: update artifact upload action",
          "timestamp": "2026-06-06T17:18:37Z",
          "url": "https://github.com/aiconnai/engram/commit/52810094ae072807f5c2590b2b4fbfe1c518370e"
        },
        "date": 1780816314513,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5928109,
            "range": "± 9288",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3632,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8927,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 321012,
            "range": "± 12581",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513811,
            "range": "± 2712",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 286372,
            "range": "± 4852",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102310,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 201,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 278225,
            "range": "± 12540",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40084,
            "range": "± 846",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115295,
            "range": "± 875",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 269296,
            "range": "± 1907",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 460694,
            "range": "± 4837",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 681956,
            "range": "± 3826",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 888355,
            "range": "± 4722",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1101667,
            "range": "± 16830",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33940,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22470,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 297338,
            "range": "± 907",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 234047,
            "range": "± 6801",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 272516,
            "range": "± 3900",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 307525,
            "range": "± 3466",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353124,
            "range": "± 2807",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 41366,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 500532,
            "range": "± 12378",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 71215,
            "range": "± 4157",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 984,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2656,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 6094,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201085,
            "range": "± 835",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19059,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17990,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19119,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/100",
            "value": 200410,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/1000",
            "value": 500042,
            "range": "± 6382",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/memories/10000",
            "value": 2221955,
            "range": "± 20598",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/100",
            "value": 6657,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34613,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140323,
            "range": "± 6438",
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
            "value": 546,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 545,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2726,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10117,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38792,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13378,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68971,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82291932,
            "range": "± 224339",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4776095,
            "range": "± 61555",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17125785,
            "range": "± 94683",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1185422,
            "range": "± 22334",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "b2aa1ec256a07a3537a001166bee3124803d13a2",
          "message": "fix(engra-84): harden MCP HTTP rate-limit contract for ENGRA-84 (#62)",
          "timestamp": "2026-06-08T04:14:21Z",
          "url": "https://github.com/aiconnai/engram/commit/b2aa1ec256a07a3537a001166bee3124803d13a2"
        },
        "date": 1780904208650,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5307111,
            "range": "± 23016",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3502,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8151,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 369188,
            "range": "± 7612",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 534583,
            "range": "± 1984",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289806,
            "range": "± 3095",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117177,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 222,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 314776,
            "range": "± 10065",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47516,
            "range": "± 2460",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115830,
            "range": "± 1273",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 288187,
            "range": "± 1129",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 449537,
            "range": "± 6786",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 681275,
            "range": "± 7649",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 862870,
            "range": "± 6611",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1071176,
            "range": "± 19568",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35092,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 20937,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330523,
            "range": "± 2188",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 258304,
            "range": "± 1407",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 295239,
            "range": "± 1427",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 327059,
            "range": "± 2050",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 364866,
            "range": "± 4075",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45121,
            "range": "± 632",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12732187,
            "range": "± 111225",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12156857,
            "range": "± 280605",
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
            "value": 2315,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5214,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192791,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18464,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17443,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18289,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1366017,
            "range": "± 13857",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1095943,
            "range": "± 7020",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11194188,
            "range": "± 28442",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10643891,
            "range": "± 48246",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119065070,
            "range": "± 698848",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114600046,
            "range": "± 637388",
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
            "value": 6508,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33006,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134406,
            "range": "± 2664",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 575,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 556,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 560,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2803,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9891,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40241,
            "range": "± 3054",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12905,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66057,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79532891,
            "range": "± 121737",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5191374,
            "range": "± 36567",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18982496,
            "range": "± 62268",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1240139,
            "range": "± 11459",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "b2aa1ec256a07a3537a001166bee3124803d13a2",
          "message": "fix(engra-84): harden MCP HTTP rate-limit contract for ENGRA-84 (#62)",
          "timestamp": "2026-06-08T04:14:21Z",
          "url": "https://github.com/aiconnai/engram/commit/b2aa1ec256a07a3537a001166bee3124803d13a2"
        },
        "date": 1780987915267,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5374915,
            "range": "± 85913",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3491,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8151,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 382588,
            "range": "± 16430",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 543362,
            "range": "± 4820",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 292840,
            "range": "± 2435",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118146,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 220,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 330022,
            "range": "± 12955",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47605,
            "range": "± 1370",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115106,
            "range": "± 705",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 291338,
            "range": "± 2042",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 448781,
            "range": "± 6633",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 690581,
            "range": "± 19474",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 861254,
            "range": "± 13674",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1074000,
            "range": "± 20830",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35297,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21014,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 325721,
            "range": "± 9527",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 256400,
            "range": "± 1382",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 292642,
            "range": "± 1372",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 322729,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 355308,
            "range": "± 1650",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45061,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12938441,
            "range": "± 418028",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12190454,
            "range": "± 237842",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 859,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2391,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5262,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 196881,
            "range": "± 7707",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19162,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18072,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19070,
            "range": "± 2641",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1359009,
            "range": "± 6698",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1095740,
            "range": "± 21164",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11144986,
            "range": "± 21414",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10587205,
            "range": "± 313486",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119055723,
            "range": "± 2525778",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116056987,
            "range": "± 687221",
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
            "value": 6461,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 32842,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 133748,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 562,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 559,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 564,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2772,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9977,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38814,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12981,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 65936,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80076406,
            "range": "± 332760",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5273167,
            "range": "± 19390",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19014873,
            "range": "± 115697",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1253681,
            "range": "± 31170",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a0709821d71dfb1f82d1b38691a573d5204451a6",
          "message": "feat(engra-74): add explicit context artifact retrieval (#64)",
          "timestamp": "2026-06-09T12:13:27Z",
          "url": "https://github.com/aiconnai/engram/commit/a0709821d71dfb1f82d1b38691a573d5204451a6"
        },
        "date": 1781075302590,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5329232,
            "range": "± 28516",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3515,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8227,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 364099,
            "range": "± 8371",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 532408,
            "range": "± 1838",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289471,
            "range": "± 1545",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118049,
            "range": "± 1944",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 151,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 318012,
            "range": "± 9894",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47394,
            "range": "± 1278",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112961,
            "range": "± 2435",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 288405,
            "range": "± 11263",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 438982,
            "range": "± 2192",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686587,
            "range": "± 6996",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 842734,
            "range": "± 7092",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1070978,
            "range": "± 31152",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35422,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21240,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330387,
            "range": "± 6581",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 251513,
            "range": "± 1549",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 288329,
            "range": "± 5017",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319090,
            "range": "± 4781",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353001,
            "range": "± 3755",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44560,
            "range": "± 743",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12638857,
            "range": "± 79543",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12113881,
            "range": "± 109269",
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
            "value": 2309,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5171,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191531,
            "range": "± 2709",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18479,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17294,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18351,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1358967,
            "range": "± 6149",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1092976,
            "range": "± 9753",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11204929,
            "range": "± 46585",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10572112,
            "range": "± 170197",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 112100005,
            "range": "± 1946211",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 108133813,
            "range": "± 755628",
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
            "value": 6520,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33152,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134421,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 563,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 540,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 545,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2757,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9700,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38680,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12983,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66160,
            "range": "± 783",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80611064,
            "range": "± 253748",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5305961,
            "range": "± 59446",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19187642,
            "range": "± 65227",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1247997,
            "range": "± 31037",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a0709821d71dfb1f82d1b38691a573d5204451a6",
          "message": "feat(engra-74): add explicit context artifact retrieval (#64)",
          "timestamp": "2026-06-09T12:13:27Z",
          "url": "https://github.com/aiconnai/engram/commit/a0709821d71dfb1f82d1b38691a573d5204451a6"
        },
        "date": 1781163249996,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5462220,
            "range": "± 22648",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3564,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8189,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 368110,
            "range": "± 17334",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 534289,
            "range": "± 2239",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287111,
            "range": "± 2663",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120227,
            "range": "± 1179",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 196,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 316338,
            "range": "± 10668",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47581,
            "range": "± 1531",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114025,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 287783,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 443894,
            "range": "± 6453",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 685281,
            "range": "± 6231",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 850567,
            "range": "± 8111",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1066376,
            "range": "± 17986",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34849,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21063,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 327238,
            "range": "± 1621",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252455,
            "range": "± 4405",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 285367,
            "range": "± 4502",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 315323,
            "range": "± 2014",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353089,
            "range": "± 2084",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45236,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12675740,
            "range": "± 83263",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12161725,
            "range": "± 178743",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 847,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2305,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5225,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191827,
            "range": "± 1566",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18529,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17364,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18350,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1349164,
            "range": "± 11992",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1083187,
            "range": "± 8537",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11286981,
            "range": "± 138606",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10685369,
            "range": "± 67653",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117129192,
            "range": "± 759249",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113592606,
            "range": "± 737771",
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
            "value": 6568,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33697,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134005,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 561,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 554,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 554,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2819,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10100,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37841,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12976,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66078,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80100894,
            "range": "± 723814",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5219576,
            "range": "± 18338",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19188879,
            "range": "± 154784",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1241039,
            "range": "± 24620",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a0709821d71dfb1f82d1b38691a573d5204451a6",
          "message": "feat(engra-74): add explicit context artifact retrieval (#64)",
          "timestamp": "2026-06-09T12:13:27Z",
          "url": "https://github.com/aiconnai/engram/commit/a0709821d71dfb1f82d1b38691a573d5204451a6"
        },
        "date": 1781249023585,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5319125,
            "range": "± 59647",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3643,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8117,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 371627,
            "range": "± 8657",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 533608,
            "range": "± 2254",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288217,
            "range": "± 1943",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 116597,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 200,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 313717,
            "range": "± 9523",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46609,
            "range": "± 1154",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113762,
            "range": "± 932",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 289936,
            "range": "± 2241",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 446994,
            "range": "± 4137",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 681102,
            "range": "± 6883",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 858243,
            "range": "± 2635",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1074947,
            "range": "± 11864",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34870,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21123,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 325757,
            "range": "± 4721",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252791,
            "range": "± 1373",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289095,
            "range": "± 2278",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319437,
            "range": "± 1840",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 350279,
            "range": "± 2679",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45463,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12597827,
            "range": "± 291241",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11946547,
            "range": "± 89832",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 848,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2298,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5232,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191062,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19127,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18033,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19271,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1344884,
            "range": "± 4578",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1081866,
            "range": "± 97343",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11401404,
            "range": "± 322327",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10482616,
            "range": "± 44977",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117421126,
            "range": "± 1289457",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114235428,
            "range": "± 840027",
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
            "value": 6466,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33516,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 133239,
            "range": "± 644",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 584,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 559,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 568,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2664,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9558,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 36799,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12946,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66369,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79253125,
            "range": "± 150995",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5187377,
            "range": "± 13494",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18873598,
            "range": "± 145105",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1235070,
            "range": "± 13414",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "8d3ce98a805cb21698afa198058013f4ba62339e",
          "message": "test(mcp): add parity harness and model routing RFC (#71)\n\n* test(mcp): cover operational context search and bundle\n\n* test(mcp): add deterministic parity harness\n\n* docs(intelligence): draft model routing contract",
          "timestamp": "2026-06-12T18:58:56Z",
          "url": "https://github.com/aiconnai/engram/commit/8d3ce98a805cb21698afa198058013f4ba62339e"
        },
        "date": 1781334113923,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5375525,
            "range": "± 17923",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3493,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8986,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 366193,
            "range": "± 9667",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536611,
            "range": "± 3695",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289970,
            "range": "± 6560",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117254,
            "range": "± 2553",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 348,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 322211,
            "range": "± 9278",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47953,
            "range": "± 1229",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114075,
            "range": "± 962",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 295711,
            "range": "± 4896",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 443018,
            "range": "± 6045",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 689351,
            "range": "± 13952",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 845898,
            "range": "± 10883",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1080270,
            "range": "± 21566",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34966,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21497,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 335266,
            "range": "± 1566",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 256504,
            "range": "± 3050",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 292261,
            "range": "± 1289",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 325389,
            "range": "± 3413",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 360148,
            "range": "± 5988",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45155,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12658584,
            "range": "± 103704",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12061919,
            "range": "± 89266",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 853,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2305,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5254,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192307,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18501,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17360,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18408,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1370207,
            "range": "± 8063",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1096607,
            "range": "± 3297",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11261236,
            "range": "± 67924",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10671690,
            "range": "± 87579",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118149284,
            "range": "± 1315311",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115107724,
            "range": "± 878822",
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
            "value": 6485,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33435,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134464,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 570,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 555,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 557,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2794,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10302,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40464,
            "range": "± 1047",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12882,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66332,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79506143,
            "range": "± 1231220",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5285456,
            "range": "± 21717",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19202206,
            "range": "± 82311",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1247336,
            "range": "± 27634",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "8d3ce98a805cb21698afa198058013f4ba62339e",
          "message": "test(mcp): add parity harness and model routing RFC (#71)\n\n* test(mcp): cover operational context search and bundle\n\n* test(mcp): add deterministic parity harness\n\n* docs(intelligence): draft model routing contract",
          "timestamp": "2026-06-12T18:58:56Z",
          "url": "https://github.com/aiconnai/engram/commit/8d3ce98a805cb21698afa198058013f4ba62339e"
        },
        "date": 1781421926225,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5409135,
            "range": "± 12021",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3502,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 9218,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 360532,
            "range": "± 8070",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 532258,
            "range": "± 3307",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 284297,
            "range": "± 1285",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120177,
            "range": "± 1659",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 351,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 311976,
            "range": "± 10417",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47001,
            "range": "± 1336",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112575,
            "range": "± 641",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 293370,
            "range": "± 1536",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 440010,
            "range": "± 3933",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 685857,
            "range": "± 3273",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 842108,
            "range": "± 3461",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1074243,
            "range": "± 24490",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34863,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21242,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 332459,
            "range": "± 2213",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 246964,
            "range": "± 1690",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 283350,
            "range": "± 15384",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 313537,
            "range": "± 3294",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 344730,
            "range": "± 2303",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43497,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12503389,
            "range": "± 107246",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11941739,
            "range": "± 155041",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 857,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2326,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5248,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192287,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18441,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17357,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18260,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1340152,
            "range": "± 7037",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1076373,
            "range": "± 6524",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10991008,
            "range": "± 56099",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10417766,
            "range": "± 87429",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 114020832,
            "range": "± 1391228",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 109527693,
            "range": "± 1274307",
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
            "value": 6715,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33581,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134583,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 566,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 558,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 564,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2985,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11102,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43218,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13026,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66209,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 78494350,
            "range": "± 432981",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5152237,
            "range": "± 19187",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19019420,
            "range": "± 90338",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1232924,
            "range": "± 19727",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d5a2a7b43fbd13248023b65eb5e47e50753b0907",
          "message": "ci(github): always report required PR checks\n\nRemove the pull_request path filter so branch-protection checks are reported for every PR.",
          "timestamp": "2026-06-14T17:47:43Z",
          "url": "https://github.com/aiconnai/engram/commit/d5a2a7b43fbd13248023b65eb5e47e50753b0907"
        },
        "date": 1781513871377,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5448606,
            "range": "± 66348",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3495,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8999,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 373455,
            "range": "± 13798",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537439,
            "range": "± 4472",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290147,
            "range": "± 2388",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118623,
            "range": "± 3828",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 351,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 327844,
            "range": "± 12779",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48120,
            "range": "± 1818",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114109,
            "range": "± 7440",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 296871,
            "range": "± 2375",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 446304,
            "range": "± 2875",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 703782,
            "range": "± 8364",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 848370,
            "range": "± 6342",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1096305,
            "range": "± 17847",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35502,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21222,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330024,
            "range": "± 3527",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 253272,
            "range": "± 1603",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290175,
            "range": "± 2505",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319874,
            "range": "± 2034",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 351671,
            "range": "± 2386",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 46625,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12893108,
            "range": "± 198094",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12408556,
            "range": "± 151098",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 851,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2314,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5270,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192695,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19551,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17465,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18318,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1349753,
            "range": "± 21253",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1088058,
            "range": "± 26536",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11228727,
            "range": "± 199802",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10701760,
            "range": "± 175297",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118205787,
            "range": "± 1373576",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113844002,
            "range": "± 799313",
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
            "value": 6528,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33301,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135511,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 570,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 580,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 568,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2797,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10217,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39936,
            "range": "± 941",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12961,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66168,
            "range": "± 4391",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80059617,
            "range": "± 154482",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5310373,
            "range": "± 23925",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19270916,
            "range": "± 177362",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1242697,
            "range": "± 20143",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "f1baf2168d350f85e1b9cbe852bc955c7d42108e",
          "message": "[codex] qualify public documentation claims\n\nQualify public documentation claims, hosted/cloud references, package-channel lag, and schema/performance caveats.",
          "timestamp": "2026-06-16T02:36:57Z",
          "url": "https://github.com/aiconnai/engram/commit/f1baf2168d350f85e1b9cbe852bc955c7d42108e"
        },
        "date": 1781600022739,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5862894,
            "range": "± 7071",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3559,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8991,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 318025,
            "range": "± 8308",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 514919,
            "range": "± 6632",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 282814,
            "range": "± 1661",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103010,
            "range": "± 762",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 334,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 273044,
            "range": "± 11795",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39656,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 111224,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 270750,
            "range": "± 1525",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 443453,
            "range": "± 2809",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 673999,
            "range": "± 14026",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 860061,
            "range": "± 7466",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1078515,
            "range": "± 28300",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33133,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22049,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 317862,
            "range": "± 6436",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 225213,
            "range": "± 5975",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 262224,
            "range": "± 3439",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 292307,
            "range": "± 1651",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 329197,
            "range": "± 4036",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36311,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12802803,
            "range": "± 90688",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12348191,
            "range": "± 75294",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 949,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2444,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5533,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201972,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19131,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18011,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19138,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1317056,
            "range": "± 6232",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1081933,
            "range": "± 7533",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11097047,
            "range": "± 77542",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10601364,
            "range": "± 58892",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117084748,
            "range": "± 1109319",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112836415,
            "range": "± 1101988",
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
            "value": 6672,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34832,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140757,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 547,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 536,
            "range": "± 32",
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
            "value": 2683,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10134,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38062,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13428,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69957,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82523219,
            "range": "± 113894",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4656935,
            "range": "± 19272",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16662706,
            "range": "± 56374",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1199187,
            "range": "± 26659",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d1b46f1698adb9460a509ccf538cd4627c9509c4",
          "message": "ci(harness): add agentshield loop (#89)",
          "timestamp": "2026-06-16T11:09:57Z",
          "url": "https://github.com/aiconnai/engram/commit/d1b46f1698adb9460a509ccf538cd4627c9509c4"
        },
        "date": 1781685127262,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5897045,
            "range": "± 14263",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3557,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8863,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325085,
            "range": "± 9299",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 509254,
            "range": "± 2382",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 285337,
            "range": "± 2375",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 104229,
            "range": "± 1369",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 333,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 274529,
            "range": "± 13911",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39841,
            "range": "± 845",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 110969,
            "range": "± 803",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 270651,
            "range": "± 1636",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 443288,
            "range": "± 4350",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 676819,
            "range": "± 3729",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 871846,
            "range": "± 9271",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1077957,
            "range": "± 17521",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33050,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22098,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 314995,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224839,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 262025,
            "range": "± 4235",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 289826,
            "range": "± 3797",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 325570,
            "range": "± 2074",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36508,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12796579,
            "range": "± 59058",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12352992,
            "range": "± 169805",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 934,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2450,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5528,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 200740,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19220,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18072,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19130,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1319245,
            "range": "± 13303",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1078580,
            "range": "± 6713",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11154020,
            "range": "± 252217",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10675170,
            "range": "± 142770",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118129944,
            "range": "± 675185",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114434694,
            "range": "± 2129027",
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
            "value": 6621,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34909,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 142169,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 544,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 549,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 549,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2606,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9847,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38067,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13341,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69513,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82471058,
            "range": "± 707525",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4625671,
            "range": "± 27900",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16603091,
            "range": "± 44186",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1190653,
            "range": "± 19588",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d1b46f1698adb9460a509ccf538cd4627c9509c4",
          "message": "ci(harness): add agentshield loop (#89)",
          "timestamp": "2026-06-16T11:09:57Z",
          "url": "https://github.com/aiconnai/engram/commit/d1b46f1698adb9460a509ccf538cd4627c9509c4"
        },
        "date": 1781768164610,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5879937,
            "range": "± 7264",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3587,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8842,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 326246,
            "range": "± 8466",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 519922,
            "range": "± 4257",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 286343,
            "range": "± 1552",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103395,
            "range": "± 578",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 332,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 279833,
            "range": "± 13215",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39742,
            "range": "± 1363",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 110781,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 267870,
            "range": "± 2418",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 448896,
            "range": "± 2934",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 670591,
            "range": "± 17123",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 869991,
            "range": "± 2561",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1068024,
            "range": "± 12298",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32559,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22157,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 315536,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229135,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 263766,
            "range": "± 1576",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 293491,
            "range": "± 2178",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 326914,
            "range": "± 2764",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35940,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13129104,
            "range": "± 120789",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12775878,
            "range": "± 112467",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 931,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2468,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5545,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203352,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19099,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17949,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19070,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1350901,
            "range": "± 7054",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1109347,
            "range": "± 9275",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11686465,
            "range": "± 429616",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11139383,
            "range": "± 86773",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 123236725,
            "range": "± 1012216",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118334254,
            "range": "± 856288",
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
            "value": 6633,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34755,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140384,
            "range": "± 1320",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 533,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 540,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 534,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2834,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9253,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 35200,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13441,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69465,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 84826062,
            "range": "± 113692",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4754376,
            "range": "± 37330",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17077551,
            "range": "± 736342",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1201404,
            "range": "± 24653",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d1b46f1698adb9460a509ccf538cd4627c9509c4",
          "message": "ci(harness): add agentshield loop (#89)",
          "timestamp": "2026-06-16T11:09:57Z",
          "url": "https://github.com/aiconnai/engram/commit/d1b46f1698adb9460a509ccf538cd4627c9509c4"
        },
        "date": 1781858202880,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5886566,
            "range": "± 6691",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3632,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 8847,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 326264,
            "range": "± 10072",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 517584,
            "range": "± 3362",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 285600,
            "range": "± 6981",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102644,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 334,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 278116,
            "range": "± 11492",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39195,
            "range": "± 1036",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113343,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 268517,
            "range": "± 1169",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447812,
            "range": "± 2123",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 666334,
            "range": "± 10704",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 858474,
            "range": "± 19454",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1064981,
            "range": "± 10969",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32797,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21609,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 315315,
            "range": "± 4911",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224683,
            "range": "± 4671",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 259176,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 289166,
            "range": "± 1678",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 322257,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35800,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12780379,
            "range": "± 44160",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12477008,
            "range": "± 84835",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 942,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2437,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5551,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201817,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19187,
            "range": "± 814",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17928,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19184,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1329888,
            "range": "± 30679",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1079846,
            "range": "± 7906",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11177784,
            "range": "± 45594",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10740472,
            "range": "± 85495",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117447654,
            "range": "± 1477653",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115177671,
            "range": "± 1237524",
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
            "value": 6711,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34731,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140864,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 546,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 544,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 546,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2690,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10155,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39242,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13400,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69448,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82843154,
            "range": "± 83470",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4621770,
            "range": "± 12832",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16522780,
            "range": "± 100679",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1197633,
            "range": "± 24687",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "bf45c0bb8590d1027026b0b1a259141c1b008094",
          "message": "chore(intelligence): complete code quality maintenance cleanup (#90)\n\n* chore(intelligence): complete code quality maintenance cleanup\n\n* chore(harness): add L1 daily triage starter\n\n* chore(harness): record code-quality review blocker\n\n* chore(harness): close code-quality post-review\n\n* fix(sdk-python): close post-review follow-ups\n\n* fix(intelligence): address review blockers",
          "timestamp": "2026-06-19T18:55:43Z",
          "url": "https://github.com/aiconnai/engram/commit/bf45c0bb8590d1027026b0b1a259141c1b008094"
        },
        "date": 1781939348156,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5927371,
            "range": "± 5237",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3617,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384123,
            "range": "± 8453",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325996,
            "range": "± 11144",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 514684,
            "range": "± 21496",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 282150,
            "range": "± 4538",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102239,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 330,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 275785,
            "range": "± 13074",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 38995,
            "range": "± 1392",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114046,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 271203,
            "range": "± 1923",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 449153,
            "range": "± 3819",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 669729,
            "range": "± 5681",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 862605,
            "range": "± 11269",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1060793,
            "range": "± 14291",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33167,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21988,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 314636,
            "range": "± 7228",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229320,
            "range": "± 3856",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 263244,
            "range": "± 1993",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 294234,
            "range": "± 1328",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 330018,
            "range": "± 6769",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36736,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13277457,
            "range": "± 119114",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12842071,
            "range": "± 258958",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 949,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2525,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5636,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 207350,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19275,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18196,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19313,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1375814,
            "range": "± 5429",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1121170,
            "range": "± 4615",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11503268,
            "range": "± 210735",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11055329,
            "range": "± 81974",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 122789950,
            "range": "± 860776",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118999659,
            "range": "± 618687",
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
            "value": 6704,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34732,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140119,
            "range": "± 268",
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
            "value": 543,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 546,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2691,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9816,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38816,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13432,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68943,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82756424,
            "range": "± 125798",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4695119,
            "range": "± 149543",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16822104,
            "range": "± 76726",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1206384,
            "range": "± 37246",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "7b5cbeacaddb141f0fecbe3c8cca3bd09f1e1782",
          "message": "fix(storage): make extension semantics explicit",
          "timestamp": "2026-06-20T16:07:09Z",
          "url": "https://github.com/aiconnai/engram/commit/7b5cbeacaddb141f0fecbe3c8cca3bd09f1e1782"
        },
        "date": 1782027331945,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5380581,
            "range": "± 4396",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3580,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384184,
            "range": "± 5098",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 359438,
            "range": "± 6877",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 534369,
            "range": "± 4704",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287367,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117374,
            "range": "± 1316",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 343,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 315501,
            "range": "± 11144",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47275,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114646,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 295597,
            "range": "± 2289",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 448237,
            "range": "± 6088",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 693464,
            "range": "± 4509",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 849008,
            "range": "± 8560",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1080210,
            "range": "± 15895",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34412,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21501,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 332330,
            "range": "± 3543",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 255865,
            "range": "± 5644",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 293178,
            "range": "± 2207",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 324327,
            "range": "± 2500",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 359336,
            "range": "± 1516",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44573,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13064060,
            "range": "± 154426",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12326669,
            "range": "± 74767",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 849,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2307,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5310,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192727,
            "range": "± 2915",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17813,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16758,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17911,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1389565,
            "range": "± 11330",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1120935,
            "range": "± 4842",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11449237,
            "range": "± 168511",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10897852,
            "range": "± 32706",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119115845,
            "range": "± 651073",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115723220,
            "range": "± 930347",
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
            "value": 6656,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34710,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137681,
            "range": "± 1283",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 566,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 554,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 553,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2547,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9693,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37157,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13473,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68538,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80635148,
            "range": "± 373330",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5225338,
            "range": "± 21328",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19112293,
            "range": "± 76463",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1255373,
            "range": "± 11460",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}