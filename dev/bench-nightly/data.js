window.BENCHMARK_DATA = {
  "lastUpdate": 1780987915782,
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
      }
    ]
  }
}