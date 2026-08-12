window.BENCHMARK_DATA = {
  "lastUpdate": 1786509934555,
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
          "id": "9e3ccbc7ed291e813840510c7706c307b744f83e",
          "message": "docs(harness): point gemini reviewer to zed (#105)",
          "timestamp": "2026-06-22T03:26:09Z",
          "url": "https://github.com/aiconnai/engram/commit/9e3ccbc7ed291e813840510c7706c307b744f83e"
        },
        "date": 1782118790741,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5365664,
            "range": "± 5876",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3586,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 393828,
            "range": "± 7771",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 370935,
            "range": "± 6249",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 539329,
            "range": "± 20142",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289930,
            "range": "± 4238",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119354,
            "range": "± 2830",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 361,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 316812,
            "range": "± 10047",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48751,
            "range": "± 1412",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115081,
            "range": "± 1023",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 299080,
            "range": "± 2118",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 450705,
            "range": "± 1881",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 700962,
            "range": "± 2457",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 863855,
            "range": "± 3839",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1098366,
            "range": "± 35878",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35899,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22054,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 334045,
            "range": "± 1282",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 254450,
            "range": "± 1402",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290241,
            "range": "± 2057",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319663,
            "range": "± 1534",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353544,
            "range": "± 1560",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43270,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12786800,
            "range": "± 505946",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12206381,
            "range": "± 59327",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 854,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2290,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5151,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190780,
            "range": "± 4602",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18145,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17155,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18037,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1371973,
            "range": "± 4068",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1103977,
            "range": "± 19209",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11289430,
            "range": "± 51078",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10710406,
            "range": "± 46157",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 116598539,
            "range": "± 917329",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113538792,
            "range": "± 975024",
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
            "value": 6600,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34184,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137446,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 610,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 606,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 604,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3084,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11176,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43767,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13344,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68165,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81083863,
            "range": "± 178440",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5227570,
            "range": "± 16221",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19077352,
            "range": "± 50930",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1255183,
            "range": "± 20594",
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
          "id": "9e3ccbc7ed291e813840510c7706c307b744f83e",
          "message": "docs(harness): point gemini reviewer to zed (#105)",
          "timestamp": "2026-06-22T03:26:09Z",
          "url": "https://github.com/aiconnai/engram/commit/9e3ccbc7ed291e813840510c7706c307b744f83e"
        },
        "date": 1782197438945,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5953137,
            "range": "± 7494",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3632,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382874,
            "range": "± 1279",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 321850,
            "range": "± 8479",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513486,
            "range": "± 9017",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 284684,
            "range": "± 6894",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 104371,
            "range": "± 1746",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 340,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 285610,
            "range": "± 14814",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39494,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 111876,
            "range": "± 1557",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 269612,
            "range": "± 2085",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447378,
            "range": "± 1421",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 667668,
            "range": "± 8720",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 866617,
            "range": "± 3823",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1063855,
            "range": "± 28414",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32765,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22112,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 316335,
            "range": "± 6514",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 229014,
            "range": "± 3775",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 263331,
            "range": "± 3526",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 293441,
            "range": "± 3407",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 328316,
            "range": "± 2220",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35473,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13375730,
            "range": "± 306432",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13087428,
            "range": "± 253110",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 925,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2475,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5528,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 200735,
            "range": "± 3328",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19110,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17956,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19364,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1356866,
            "range": "± 5471",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1108811,
            "range": "± 11109",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11511598,
            "range": "± 220744",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11284757,
            "range": "± 180112",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 122363201,
            "range": "± 1514001",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118627264,
            "range": "± 830302",
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
            "value": 6613,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34824,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141761,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 585,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 552,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 533,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2892,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10259,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39172,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13498,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 70272,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 84314502,
            "range": "± 177952",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4709804,
            "range": "± 67595",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16823479,
            "range": "± 49159",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1183507,
            "range": "± 60026",
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
          "id": "9e3ccbc7ed291e813840510c7706c307b744f83e",
          "message": "docs(harness): point gemini reviewer to zed (#105)",
          "timestamp": "2026-06-22T03:26:09Z",
          "url": "https://github.com/aiconnai/engram/commit/9e3ccbc7ed291e813840510c7706c307b744f83e"
        },
        "date": 1782283657608,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5350159,
            "range": "± 8254",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3519,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 389419,
            "range": "± 37174",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 365796,
            "range": "± 9879",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536296,
            "range": "± 4086",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289697,
            "range": "± 3071",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118202,
            "range": "± 954",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 360,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 317451,
            "range": "± 10192",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47727,
            "range": "± 6458",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115449,
            "range": "± 3081",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 300709,
            "range": "± 12187",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 450217,
            "range": "± 8073",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 702747,
            "range": "± 11621",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 865227,
            "range": "± 15221",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1098514,
            "range": "± 9039",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35351,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21688,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330138,
            "range": "± 2700",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 255254,
            "range": "± 2798",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 293685,
            "range": "± 5840",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 324469,
            "range": "± 6582",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 358168,
            "range": "± 4824",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 46290,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12796901,
            "range": "± 295126",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12255181,
            "range": "± 72885",
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
            "value": 2405,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5318,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 189402,
            "range": "± 5365",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18186,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17114,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18038,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1374826,
            "range": "± 3376",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1103300,
            "range": "± 20596",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11293913,
            "range": "± 53808",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10671408,
            "range": "± 228604",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 112675626,
            "range": "± 2286387",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 108603892,
            "range": "± 1117074",
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
            "value": 6673,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34284,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137601,
            "range": "± 2518",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 637,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 619,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 592,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3094,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11494,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44403,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13362,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67659,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81134400,
            "range": "± 1249681",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5264028,
            "range": "± 18668",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19268837,
            "range": "± 78749",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1246674,
            "range": "± 34160",
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
          "id": "9e3ccbc7ed291e813840510c7706c307b744f83e",
          "message": "docs(harness): point gemini reviewer to zed (#105)",
          "timestamp": "2026-06-22T03:26:09Z",
          "url": "https://github.com/aiconnai/engram/commit/9e3ccbc7ed291e813840510c7706c307b744f83e"
        },
        "date": 1782370256656,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5298465,
            "range": "± 25546",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3520,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 391022,
            "range": "± 2702",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 362418,
            "range": "± 6325",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 531961,
            "range": "± 3959",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287874,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 116328,
            "range": "± 584",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 359,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 310957,
            "range": "± 10019",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46457,
            "range": "± 1153",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113987,
            "range": "± 1187",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 297518,
            "range": "± 2099",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 445143,
            "range": "± 3375",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 695145,
            "range": "± 4439",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 857993,
            "range": "± 8655",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1088174,
            "range": "± 10893",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35314,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21803,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 344607,
            "range": "± 2800",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 251038,
            "range": "± 1750",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 287294,
            "range": "± 1758",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 316055,
            "range": "± 1399",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 350607,
            "range": "± 3357",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44729,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12712533,
            "range": "± 68559",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12324285,
            "range": "± 95704",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 862,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2297,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5147,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191258,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18174,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17131,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18010,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1365056,
            "range": "± 21079",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1099262,
            "range": "± 6999",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11316372,
            "range": "± 204135",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10746275,
            "range": "± 188917",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120395921,
            "range": "± 917422",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115077958,
            "range": "± 669208",
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
            "value": 6590,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34296,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138171,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 599,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 577,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 583,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3250,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10985,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 42290,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13329,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68108,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81022277,
            "range": "± 131418",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5210064,
            "range": "± 16902",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19002134,
            "range": "± 52353",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1245933,
            "range": "± 39502",
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
          "id": "9e3ccbc7ed291e813840510c7706c307b744f83e",
          "message": "docs(harness): point gemini reviewer to zed (#105)",
          "timestamp": "2026-06-22T03:26:09Z",
          "url": "https://github.com/aiconnai/engram/commit/9e3ccbc7ed291e813840510c7706c307b744f83e"
        },
        "date": 1782456875200,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5362244,
            "range": "± 4009",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3497,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 391842,
            "range": "± 16144",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 370818,
            "range": "± 6492",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537666,
            "range": "± 3904",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289313,
            "range": "± 3936",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 116969,
            "range": "± 2668",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 359,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 314902,
            "range": "± 14727",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48198,
            "range": "± 1426",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114579,
            "range": "± 509",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 296866,
            "range": "± 5667",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447036,
            "range": "± 2825",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 695079,
            "range": "± 2254",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 857851,
            "range": "± 3102",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1086680,
            "range": "± 21561",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35089,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21690,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330925,
            "range": "± 2841",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 250900,
            "range": "± 6328",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 287312,
            "range": "± 12868",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 317099,
            "range": "± 1649",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 351085,
            "range": "± 2299",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45134,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12800612,
            "range": "± 128471",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12257736,
            "range": "± 171408",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 893,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2313,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5285,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 193228,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18166,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17158,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18081,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1368432,
            "range": "± 3809",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1102252,
            "range": "± 3094",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11245250,
            "range": "± 24227",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10683662,
            "range": "± 43193",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 114573813,
            "range": "± 2410707",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112581807,
            "range": "± 1578126",
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
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34018,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137563,
            "range": "± 1738",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 613,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 588,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 584,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3062,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11254,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43040,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13379,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68388,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81136568,
            "range": "± 281502",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5159175,
            "range": "± 60334",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18817878,
            "range": "± 102499",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1254566,
            "range": "± 22959",
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
          "id": "9e3ccbc7ed291e813840510c7706c307b744f83e",
          "message": "docs(harness): point gemini reviewer to zed (#105)",
          "timestamp": "2026-06-22T03:26:09Z",
          "url": "https://github.com/aiconnai/engram/commit/9e3ccbc7ed291e813840510c7706c307b744f83e"
        },
        "date": 1782541964208,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5356290,
            "range": "± 2716",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3558,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 390078,
            "range": "± 18643",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 371557,
            "range": "± 8210",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 542613,
            "range": "± 8608",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290440,
            "range": "± 1387",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119931,
            "range": "± 3448",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 359,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 315946,
            "range": "± 12631",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47340,
            "range": "± 1472",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114346,
            "range": "± 2376",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 298454,
            "range": "± 8663",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 445989,
            "range": "± 2241",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 703311,
            "range": "± 12679",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 857845,
            "range": "± 26758",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1095254,
            "range": "± 20013",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35064,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21843,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 327985,
            "range": "± 2802",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 255118,
            "range": "± 1329",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 293310,
            "range": "± 1523",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 322250,
            "range": "± 1984",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 357758,
            "range": "± 1940",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44775,
            "range": "± 2871",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12774903,
            "range": "± 63140",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12232543,
            "range": "± 79805",
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
            "value": 2300,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5192,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191359,
            "range": "± 839",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18126,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17215,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18223,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1377284,
            "range": "± 104305",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1100635,
            "range": "± 16744",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11304862,
            "range": "± 69028",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10773452,
            "range": "± 68227",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121514894,
            "range": "± 1648348",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115513006,
            "range": "± 1016070",
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
            "value": 6598,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33929,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138321,
            "range": "± 574",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 610,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 589,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 611,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2987,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10415,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 41206,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13172,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67908,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81396957,
            "range": "± 243684",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5209767,
            "range": "± 38204",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19049442,
            "range": "± 308582",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1253440,
            "range": "± 36653",
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
          "id": "b201a2abcc90ae7ae7747e9cbbedfd3f36e513ca",
          "message": "test(sdk): regression test for _mcp_call after close; dev deps and README parity (#110)\n\n* test(sdk): add _mcp_call regression after close; fix dev deps and README parity\n\n- Add TestClose::test_mcp_call_raises_after_close to verify EngramError\n  (not AttributeError) is raised when _mcp_call is called on a closed client\n- Add [project.optional-dependencies.dev] with pytest, pytest-asyncio, httpx\n  so contributors can run the test suite with pip install -e '.[dev]'\n- Add asyncio_mode = 'auto' to [tool.pytest.ini_options] so all async tests\n  run without the pytest-asyncio mark needing an explicit plugin load\n- Expand README API reference table to cover all 60+ public methods that\n  exist in client.py but were previously undocumented (graph, temporal,\n  scopes, identity, gardening, compression, federation, etc.)\n\n* fix(sdk): remove httpx duplicate from dev deps; fix link() default in README\n\n- httpx is already a core dependency; listing it in [dev] extras creates\n  misleading docs and a drift risk if the constraint is ever tightened\n- link(from_id, to_id, edge_type) now shows the default value edge_type='related_to'\n  so users reading the README know the parameter is optional",
          "timestamp": "2026-06-27T16:27:16Z",
          "url": "https://github.com/aiconnai/engram/commit/b201a2abcc90ae7ae7747e9cbbedfd3f36e513ca"
        },
        "date": 1782630397839,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5297451,
            "range": "± 8277",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3558,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382297,
            "range": "± 10249",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 372303,
            "range": "± 8579",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 542261,
            "range": "± 5399",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 293722,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118934,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 348,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 325375,
            "range": "± 12595",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 49120,
            "range": "± 1322",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 116549,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 300557,
            "range": "± 2667",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 454597,
            "range": "± 3259",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 704549,
            "range": "± 2596",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 871233,
            "range": "± 4502",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1103064,
            "range": "± 29575",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35868,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21823,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 333638,
            "range": "± 1710",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 257697,
            "range": "± 1770",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 297736,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 329029,
            "range": "± 5177",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 365038,
            "range": "± 1661",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45165,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12916513,
            "range": "± 89058",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12331929,
            "range": "± 26842",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 866,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2376,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5284,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 194734,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18495,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17323,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18365,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1392683,
            "range": "± 7840",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1119067,
            "range": "± 2792",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11458038,
            "range": "± 30899",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10825001,
            "range": "± 25978",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121135678,
            "range": "± 1001664",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116645391,
            "range": "± 1107159",
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
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34199,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139354,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 567,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 566,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 572,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2544,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9454,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 36767,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13085,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67638,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81433088,
            "range": "± 189450",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5287418,
            "range": "± 11939",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19309601,
            "range": "± 107212",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1266415,
            "range": "± 26993",
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
          "id": "c4faad3e30f56fff3df1a64f8d344a4e3bc7b5ee",
          "message": "fix(embedding,sync): surface DB write failures instead of silencing them (#111)\n\nEmbeddingWorker.process_batch now returns Result<()>; callers log errors\nvia tracing::error! and keep the worker alive. Previously all SQL writes\nused let _ = and failures were invisible.\n\nSyncWorker sync_state writes are extracted into mark_sync_started /\nrecord_sync_success / record_sync_failure helpers; failures warn! with\nphase and direction context instead of being dropped silently.\n\nCloses ENGRA-150, ENGRA-151",
          "timestamp": "2026-06-28T14:17:27Z",
          "url": "https://github.com/aiconnai/engram/commit/c4faad3e30f56fff3df1a64f8d344a4e3bc7b5ee"
        },
        "date": 1782718522748,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5705727,
            "range": "± 5216",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3266,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 347408,
            "range": "± 2166",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 349140,
            "range": "± 9997",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 453271,
            "range": "± 6173",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 250908,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 98990,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 313,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 300487,
            "range": "± 14944",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 37876,
            "range": "± 1063",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 98512,
            "range": "± 2825",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 248869,
            "range": "± 2961",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 378257,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 591197,
            "range": "± 9848",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 722516,
            "range": "± 3986",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 927129,
            "range": "± 13733",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 29813,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 19163,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 271312,
            "range": "± 1129",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 201629,
            "range": "± 2630",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 238448,
            "range": "± 2533",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 268022,
            "range": "± 2712",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 298031,
            "range": "± 1897",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 37746,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 11584526,
            "range": "± 112426",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11037878,
            "range": "± 90405",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 898,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2483,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5391,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201189,
            "range": "± 1283",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 16018,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 15278,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 15738,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1163772,
            "range": "± 3023",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 942446,
            "range": "± 1664",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 9841843,
            "range": "± 26220",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 9293053,
            "range": "± 39190",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 104388691,
            "range": "± 1619118",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 99472356,
            "range": "± 924113",
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
            "value": 6495,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33319,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 132275,
            "range": "± 1790",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 611,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 604,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 602,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2906,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10293,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40478,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13199,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66243,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 78947544,
            "range": "± 83306",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4306018,
            "range": "± 11137",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 15608707,
            "range": "± 27407",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1040359,
            "range": "± 21440",
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
          "id": "64b24170da37e16534dccf3fd021e3dd93b89565",
          "message": "ci: add security-focused workflows and harden gitleaks (#112)\n\n* ci: add security-focused workflows (codeql, gitleaks, semgrep)\n\n* ci: fix gitleaks false positives for workflow\n\n* chore: polish gitleaks allowlist comments\n\n* fix(policy): allowlist private-key fixture for gitleaks",
          "timestamp": "2026-06-30T03:36:18Z",
          "url": "https://github.com/aiconnai/engram/commit/64b24170da37e16534dccf3fd021e3dd93b89565"
        },
        "date": 1782802841513,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5950013,
            "range": "± 18090",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3565,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 383790,
            "range": "± 5091",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 325694,
            "range": "± 13796",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 503563,
            "range": "± 3772",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 280852,
            "range": "± 1378",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 100117,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 328,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 277339,
            "range": "± 15412",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39642,
            "range": "± 1202",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113462,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 277969,
            "range": "± 8068",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 453442,
            "range": "± 2273",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686568,
            "range": "± 12203",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 876316,
            "range": "± 4266",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1099025,
            "range": "± 10651",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32602,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22189,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 314794,
            "range": "± 1690",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 227039,
            "range": "± 1717",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 260430,
            "range": "± 1099",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 291222,
            "range": "± 4687",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 328110,
            "range": "± 2357",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36352,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13040454,
            "range": "± 73545",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12549817,
            "range": "± 451732",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 935,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2463,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5597,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203205,
            "range": "± 673",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19163,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18396,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19380,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1350623,
            "range": "± 6435",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1107367,
            "range": "± 18668",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11389910,
            "range": "± 179390",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10938198,
            "range": "± 287463",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121774296,
            "range": "± 1116289",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116692469,
            "range": "± 803669",
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
            "value": 6725,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34700,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140502,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 569,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 564,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 570,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2816,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10329,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40771,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13374,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69119,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83703266,
            "range": "± 1041527",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4700211,
            "range": "± 14763",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16869033,
            "range": "± 69685",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1214739,
            "range": "± 33680",
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
          "id": "64b24170da37e16534dccf3fd021e3dd93b89565",
          "message": "ci: add security-focused workflows and harden gitleaks (#112)\n\n* ci: add security-focused workflows (codeql, gitleaks, semgrep)\n\n* ci: fix gitleaks false positives for workflow\n\n* chore: polish gitleaks allowlist comments\n\n* fix(policy): allowlist private-key fixture for gitleaks",
          "timestamp": "2026-06-30T03:36:18Z",
          "url": "https://github.com/aiconnai/engram/commit/64b24170da37e16534dccf3fd021e3dd93b89565"
        },
        "date": 1782890267352,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5380352,
            "range": "± 7755",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3560,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384036,
            "range": "± 1244",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 364959,
            "range": "± 7142",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537372,
            "range": "± 7892",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 291990,
            "range": "± 2502",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118664,
            "range": "± 3147",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 283,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 320407,
            "range": "± 10094",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47378,
            "range": "± 1508",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113778,
            "range": "± 768",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 292565,
            "range": "± 2879",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 439359,
            "range": "± 4329",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 691449,
            "range": "± 2533",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 844700,
            "range": "± 7154",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1075337,
            "range": "± 9717",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34800,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21229,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 328114,
            "range": "± 1546",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 250114,
            "range": "± 4626",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 288670,
            "range": "± 1475",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321048,
            "range": "± 6189",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 357145,
            "range": "± 1819",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45457,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12455105,
            "range": "± 62056",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11919820,
            "range": "± 44629",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 871,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2302,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5201,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191156,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18022,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17041,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17951,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1344204,
            "range": "± 5135",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1077433,
            "range": "± 5627",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11084582,
            "range": "± 64776",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10569285,
            "range": "± 53869",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 111784904,
            "range": "± 1150159",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 109235536,
            "range": "± 2494147",
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
            "value": 6660,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34145,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138129,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 569,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 557,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 556,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2915,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10315,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39789,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13301,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68022,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80615642,
            "range": "± 143798",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5262136,
            "range": "± 120782",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19245721,
            "range": "± 163543",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1261881,
            "range": "± 15111",
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
          "id": "64b24170da37e16534dccf3fd021e3dd93b89565",
          "message": "ci: add security-focused workflows and harden gitleaks (#112)\n\n* ci: add security-focused workflows (codeql, gitleaks, semgrep)\n\n* ci: fix gitleaks false positives for workflow\n\n* chore: polish gitleaks allowlist comments\n\n* fix(policy): allowlist private-key fixture for gitleaks",
          "timestamp": "2026-06-30T03:36:18Z",
          "url": "https://github.com/aiconnai/engram/commit/64b24170da37e16534dccf3fd021e3dd93b89565"
        },
        "date": 1782974516291,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5426657,
            "range": "± 49568",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3492,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385136,
            "range": "± 10834",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 374894,
            "range": "± 9056",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537696,
            "range": "± 8872",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 287370,
            "range": "± 1879",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119017,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 345,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 321572,
            "range": "± 14294",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47784,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113381,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 289284,
            "range": "± 3793",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 441726,
            "range": "± 2363",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 679735,
            "range": "± 3174",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 846003,
            "range": "± 14291",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1057071,
            "range": "± 18735",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34479,
            "range": "± 657",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21333,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 322284,
            "range": "± 9677",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 253086,
            "range": "± 3376",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 291322,
            "range": "± 1859",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321700,
            "range": "± 1615",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 354434,
            "range": "± 2837",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44628,
            "range": "± 945",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12595646,
            "range": "± 128027",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12076432,
            "range": "± 113806",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 848,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2289,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5173,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190554,
            "range": "± 2650",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17975,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17060,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17959,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1352398,
            "range": "± 6765",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1085431,
            "range": "± 5811",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11146393,
            "range": "± 130410",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10602844,
            "range": "± 103513",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118082665,
            "range": "± 1609699",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112734293,
            "range": "± 1130535",
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
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34198,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138819,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 558,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 553,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 559,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2672,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9909,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38496,
            "range": "± 1018",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13257,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67984,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81107577,
            "range": "± 400597",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5268097,
            "range": "± 23465",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19186367,
            "range": "± 129043",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1257840,
            "range": "± 11696",
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
          "id": "74c7404dba080f24c6e1b9b5bb11af68c83a6e36",
          "message": "refactor(mcp): consolidate tool registry definitions\n\nConsolidate MCP tool definitions into registry.rs, make the default tools/list standard-tier, preserve advanced/all opt-in behavior, and update docs/tests/gate evidence.",
          "timestamp": "2026-07-03T03:07:53Z",
          "url": "https://github.com/aiconnai/engram/commit/74c7404dba080f24c6e1b9b5bb11af68c83a6e36"
        },
        "date": 1783060398422,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5497792,
            "range": "± 46055",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3531,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385941,
            "range": "± 8399",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 382497,
            "range": "± 11118",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537983,
            "range": "± 8734",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 292013,
            "range": "± 5598",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120526,
            "range": "± 2782",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 344,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 320621,
            "range": "± 14085",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47544,
            "range": "± 1221",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113535,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 284265,
            "range": "± 3375",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 441415,
            "range": "± 8697",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 672175,
            "range": "± 17665",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 840884,
            "range": "± 13768",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1056379,
            "range": "± 25140",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35354,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21159,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 325264,
            "range": "± 2083",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 250157,
            "range": "± 2618",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 288426,
            "range": "± 9924",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 317472,
            "range": "± 2567",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 352189,
            "range": "± 4949",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45152,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12624924,
            "range": "± 237444",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12100206,
            "range": "± 362122",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 864,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2309,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5203,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191976,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18064,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17012,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18099,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1323833,
            "range": "± 5433",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1063019,
            "range": "± 4730",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10923350,
            "range": "± 77038",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10347840,
            "range": "± 103201",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 116497027,
            "range": "± 1127915",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 111228326,
            "range": "± 2492115",
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
            "value": 6681,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33918,
            "range": "± 527",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138755,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 558,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 555,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 559,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2548,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10270,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37540,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13344,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68027,
            "range": "± 1217",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80961175,
            "range": "± 1193025",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5281815,
            "range": "± 147505",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19049723,
            "range": "± 94478",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1248975,
            "range": "± 53080",
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
          "id": "881976a976f71a49dd8b4746f93ce926550015eb",
          "message": "Add pending agent writeback candidates (#115)\n\n* feat(mcp): add pending agent writeback candidates\n\n* fix(mcp): harden agent writeback candidates",
          "timestamp": "2026-07-03T13:38:07Z",
          "url": "https://github.com/aiconnai/engram/commit/881976a976f71a49dd8b4746f93ce926550015eb"
        },
        "date": 1783145939274,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5655947,
            "range": "± 3742",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3279,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 347119,
            "range": "± 1206",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 342740,
            "range": "± 10759",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 446923,
            "range": "± 4205",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 245583,
            "range": "± 1889",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 99625,
            "range": "± 648",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 262,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 295729,
            "range": "± 15446",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 37710,
            "range": "± 1125",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 97667,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 240561,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 368889,
            "range": "± 5705",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 573254,
            "range": "± 3678",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 706906,
            "range": "± 3468",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 905898,
            "range": "± 3757",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 29337,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 18268,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 266512,
            "range": "± 873",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 199650,
            "range": "± 2859",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 233449,
            "range": "± 1212",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 260534,
            "range": "± 2121",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 289946,
            "range": "± 1047",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 37097,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 11384790,
            "range": "± 91307",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 10921936,
            "range": "± 102410",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 894,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2465,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5330,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 196617,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 16351,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 14998,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 15750,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1144430,
            "range": "± 5333",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 925121,
            "range": "± 2184",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 9688151,
            "range": "± 46395",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 9155352,
            "range": "± 22948",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 101826246,
            "range": "± 1542159",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 97587708,
            "range": "± 1045602",
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
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33504,
            "range": "± 601",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 132715,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 564,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 559,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 539,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2838,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10068,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 40419,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13348,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66551,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 78716579,
            "range": "± 83501",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4296053,
            "range": "± 21078",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 15591094,
            "range": "± 113365",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1036077,
            "range": "± 19135",
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
          "id": "881976a976f71a49dd8b4746f93ce926550015eb",
          "message": "Add pending agent writeback candidates (#115)\n\n* feat(mcp): add pending agent writeback candidates\n\n* fix(mcp): harden agent writeback candidates",
          "timestamp": "2026-07-03T13:38:07Z",
          "url": "https://github.com/aiconnai/engram/commit/881976a976f71a49dd8b4746f93ce926550015eb"
        },
        "date": 1783233705866,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5396917,
            "range": "± 17236",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3611,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 386669,
            "range": "± 2027",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 391393,
            "range": "± 14267",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536958,
            "range": "± 2241",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288073,
            "range": "± 4432",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118646,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 278,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 330002,
            "range": "± 12146",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48170,
            "range": "± 1316",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 116098,
            "range": "± 1407",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 290078,
            "range": "± 3131",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 453768,
            "range": "± 2416",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 696340,
            "range": "± 2858",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 871624,
            "range": "± 6689",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1097993,
            "range": "± 15700",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35724,
            "range": "± 1298",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21831,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 316557,
            "range": "± 4102",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 256095,
            "range": "± 1653",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 291135,
            "range": "± 1729",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321203,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 355708,
            "range": "± 3649",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45223,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13404420,
            "range": "± 178210",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12758865,
            "range": "± 186227",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 858,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2286,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5129,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 188699,
            "range": "± 6269",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18003,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16953,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17853,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1389141,
            "range": "± 7263",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1116728,
            "range": "± 16876",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11466061,
            "range": "± 83945",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11036275,
            "range": "± 265055",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121026532,
            "range": "± 780907",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 117327985,
            "range": "± 945404",
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
            "value": 6597,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34078,
            "range": "± 1740",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 136679,
            "range": "± 304",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 587,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 583,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 580,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2999,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11238,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43958,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13226,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67458,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79468956,
            "range": "± 863031",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5265330,
            "range": "± 35918",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19135753,
            "range": "± 99779",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1251407,
            "range": "± 12407",
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
          "id": "881976a976f71a49dd8b4746f93ce926550015eb",
          "message": "Add pending agent writeback candidates (#115)\n\n* feat(mcp): add pending agent writeback candidates\n\n* fix(mcp): harden agent writeback candidates",
          "timestamp": "2026-07-03T13:38:07Z",
          "url": "https://github.com/aiconnai/engram/commit/881976a976f71a49dd8b4746f93ce926550015eb"
        },
        "date": 1783321923092,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5392721,
            "range": "± 11582",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3515,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 383775,
            "range": "± 1799",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 390602,
            "range": "± 15506",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 539595,
            "range": "± 2825",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290225,
            "range": "± 1413",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118116,
            "range": "± 1503",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 280,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 329067,
            "range": "± 14541",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47483,
            "range": "± 1216",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117367,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 292530,
            "range": "± 2090",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 457384,
            "range": "± 3455",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 699800,
            "range": "± 6225",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 876160,
            "range": "± 4613",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1103986,
            "range": "± 15614",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35092,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21844,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 316935,
            "range": "± 1726",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 255106,
            "range": "± 2171",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289275,
            "range": "± 2377",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 318502,
            "range": "± 3128",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 350152,
            "range": "± 2102",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45688,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13493840,
            "range": "± 240127",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12897093,
            "range": "± 131530",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 848,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2280,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5148,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191006,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17970,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16933,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18037,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1360940,
            "range": "± 18625",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1091170,
            "range": "± 20789",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11968605,
            "range": "± 129811",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11237088,
            "range": "± 77621",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120796654,
            "range": "± 1057177",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116754031,
            "range": "± 1062003",
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
            "value": 6625,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33954,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137450,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 600,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 579,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 581,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3098,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11503,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44723,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13293,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67902,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79867643,
            "range": "± 261186",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5305464,
            "range": "± 28586",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19332216,
            "range": "± 144674",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1255394,
            "range": "± 18675",
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
          "id": "881976a976f71a49dd8b4746f93ce926550015eb",
          "message": "Add pending agent writeback candidates (#115)\n\n* feat(mcp): add pending agent writeback candidates\n\n* fix(mcp): harden agent writeback candidates",
          "timestamp": "2026-07-03T13:38:07Z",
          "url": "https://github.com/aiconnai/engram/commit/881976a976f71a49dd8b4746f93ce926550015eb"
        },
        "date": 1783406816097,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5404550,
            "range": "± 14304",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3545,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 389138,
            "range": "± 13664",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 385681,
            "range": "± 10780",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537246,
            "range": "± 3709",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289233,
            "range": "± 8986",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120678,
            "range": "± 2642",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 280,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 340253,
            "range": "± 15563",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47889,
            "range": "± 1374",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 118602,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 291008,
            "range": "± 4024",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 459559,
            "range": "± 3938",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 695642,
            "range": "± 9061",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 885359,
            "range": "± 3885",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1101815,
            "range": "± 33224",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35798,
            "range": "± 1245",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22522,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 319640,
            "range": "± 2609",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 258806,
            "range": "± 3691",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 292852,
            "range": "± 2835",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 323565,
            "range": "± 1921",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 359094,
            "range": "± 3321",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45956,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13129059,
            "range": "± 205168",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12703858,
            "range": "± 472790",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 882,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2297,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5138,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 188921,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18016,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17086,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18042,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1393158,
            "range": "± 5952",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1116494,
            "range": "± 6363",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11745683,
            "range": "± 155931",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10937645,
            "range": "± 158281",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 123104622,
            "range": "± 854641",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 117518526,
            "range": "± 804583",
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
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33897,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138011,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 570,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 573,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 575,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3337,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11538,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44726,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13250,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67741,
            "range": "± 1161",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80232533,
            "range": "± 1391870",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5266705,
            "range": "± 34496",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19342359,
            "range": "± 106418",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1268119,
            "range": "± 17791",
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
          "id": "67c07fc8ba029ede4f4c7658abff0868d96b686b",
          "message": "refactor(storage): split query tests by domain (#126)",
          "timestamp": "2026-07-07T22:29:34Z",
          "url": "https://github.com/aiconnai/engram/commit/67c07fc8ba029ede4f4c7658abff0868d96b686b"
        },
        "date": 1783490250096,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5412648,
            "range": "± 27493",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3499,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 383831,
            "range": "± 2467",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 374235,
            "range": "± 10005",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 537881,
            "range": "± 12455",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289337,
            "range": "± 20654",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117668,
            "range": "± 1226",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 290,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 336217,
            "range": "± 11787",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48970,
            "range": "± 1506",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 118851,
            "range": "± 1523",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 293719,
            "range": "± 2308",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 459559,
            "range": "± 14887",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 700757,
            "range": "± 4517",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 880128,
            "range": "± 6016",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1103094,
            "range": "± 48118",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 36254,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21784,
            "range": "± 1221",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 320988,
            "range": "± 2417",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 258721,
            "range": "± 3116",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 298728,
            "range": "± 3734",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 330308,
            "range": "± 3053",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 367343,
            "range": "± 5347",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 46378,
            "range": "± 1004",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13398027,
            "range": "± 209069",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12719476,
            "range": "± 264527",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 854,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2310,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5197,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190453,
            "range": "± 709",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17972,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17099,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17939,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1390917,
            "range": "± 6938",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1117915,
            "range": "± 33488",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11834036,
            "range": "± 182998",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10916517,
            "range": "± 118654",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 122226973,
            "range": "± 1121223",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 117461329,
            "range": "± 828738",
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
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34260,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138135,
            "range": "± 3020",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 579,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 547,
            "range": "± 10",
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
            "value": 3039,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11123,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43146,
            "range": "± 1181",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13509,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68242,
            "range": "± 1038",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81398124,
            "range": "± 2788914",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5336619,
            "range": "± 33410",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19546649,
            "range": "± 112136",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1259677,
            "range": "± 23651",
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
          "id": "f1e5986625555a77761a00654ec1dec287969c73",
          "message": "refactor(mcp): split mcp/handlers/harness.rs by tool family (ADR-CLEANUP-20260708-2 row 4) (#182)\n\n* refactor(mcp): split harness.rs by tool family (ADR-CLEANUP-20260708-2 row 5)\n\nMechanical split of src/mcp/handlers/harness.rs (~1507 lines) into\nharness/ module directory: record.rs, status.rs, handoff.rs, verify.rs,\nwith shared helpers (run_command, kind_to_memory_type), VALID_KINDS and\ntests kept in mod.rs. No behavior changes.\n\n* docs(harness): add harness-split post-gate review artifacts",
          "timestamp": "2026-07-09T01:33:08Z",
          "url": "https://github.com/aiconnai/engram/commit/f1e5986625555a77761a00654ec1dec287969c73"
        },
        "date": 1783579504463,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5380733,
            "range": "± 16279",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3516,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 381597,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 380152,
            "range": "± 14433",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 535567,
            "range": "± 5405",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290257,
            "range": "± 4361",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119169,
            "range": "± 1061",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 297,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 326955,
            "range": "± 12594",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48049,
            "range": "± 1326",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117281,
            "range": "± 1626",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 291153,
            "range": "± 7759",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 456960,
            "range": "± 4359",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 699469,
            "range": "± 5621",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 873937,
            "range": "± 8076",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1111625,
            "range": "± 25587",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 36083,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21988,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 322778,
            "range": "± 11640",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 261233,
            "range": "± 4339",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 296900,
            "range": "± 1915",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 328279,
            "range": "± 1246",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 362186,
            "range": "± 3726",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45029,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12850652,
            "range": "± 168095",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12358273,
            "range": "± 108384",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 859,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2302,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5176,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191128,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18063,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16925,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17989,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1388337,
            "range": "± 7014",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1124866,
            "range": "± 21050",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11492718,
            "range": "± 99367",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10909405,
            "range": "± 119252",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120299253,
            "range": "± 1264467",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116381752,
            "range": "± 1562461",
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
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34156,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137647,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 601,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 565,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 564,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2962,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11102,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43395,
            "range": "± 666",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13222,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68097,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80500494,
            "range": "± 260771",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5276301,
            "range": "± 39293",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19206447,
            "range": "± 297402",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1268242,
            "range": "± 14345",
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
          "id": "843fd520cbd0eb4c2b1885fe11c997198beb2ca1",
          "message": "fix(deps): update anyhow for RUSTSEC-2026-0190 (#186)",
          "timestamp": "2026-07-10T00:55:14Z",
          "url": "https://github.com/aiconnai/engram/commit/843fd520cbd0eb4c2b1885fe11c997198beb2ca1"
        },
        "date": 1783666099595,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5330240,
            "range": "± 5652",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3514,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 391012,
            "range": "± 6017",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 375639,
            "range": "± 8702",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 538245,
            "range": "± 2301",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288486,
            "range": "± 2687",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119894,
            "range": "± 1989",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 295,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 324673,
            "range": "± 11315",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47949,
            "range": "± 1628",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114630,
            "range": "± 649",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 292282,
            "range": "± 1775",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447909,
            "range": "± 7921",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686767,
            "range": "± 4269",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 850267,
            "range": "± 9074",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1075426,
            "range": "± 21451",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 36699,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21715,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321787,
            "range": "± 6492",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 255616,
            "range": "± 2327",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 296047,
            "range": "± 3519",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 326145,
            "range": "± 1415",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 361328,
            "range": "± 4419",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45768,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12720344,
            "range": "± 34243",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12217522,
            "range": "± 102544",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 908,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2440,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5390,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 196161,
            "range": "± 1897",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18449,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17435,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18396,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1366332,
            "range": "± 3525",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1099111,
            "range": "± 7093",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11195705,
            "range": "± 23117",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10636605,
            "range": "± 73855",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119060992,
            "range": "± 3602622",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115259709,
            "range": "± 944066",
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
            "value": 6515,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33343,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134879,
            "range": "± 379",
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
            "value": 526,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 527,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2793,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10170,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39159,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12967,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66019,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80588856,
            "range": "± 238979",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5260519,
            "range": "± 17662",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19243947,
            "range": "± 220526",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1255238,
            "range": "± 28964",
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
          "id": "843fd520cbd0eb4c2b1885fe11c997198beb2ca1",
          "message": "fix(deps): update anyhow for RUSTSEC-2026-0190 (#186)",
          "timestamp": "2026-07-10T00:55:14Z",
          "url": "https://github.com/aiconnai/engram/commit/843fd520cbd0eb4c2b1885fe11c997198beb2ca1"
        },
        "date": 1783748650690,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5318004,
            "range": "± 154530",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3506,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 391880,
            "range": "± 8888",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 361915,
            "range": "± 15153",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 530013,
            "range": "± 4098",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288430,
            "range": "± 1543",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117921,
            "range": "± 1326",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 296,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 329421,
            "range": "± 13186",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47169,
            "range": "± 1435",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115005,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 288321,
            "range": "± 4156",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 444664,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 682405,
            "range": "± 4478",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 852497,
            "range": "± 2368",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1072566,
            "range": "± 12173",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35217,
            "range": "± 1288",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21241,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 320539,
            "range": "± 3490",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 249395,
            "range": "± 1314",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 288595,
            "range": "± 1809",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 317814,
            "range": "± 1556",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 351318,
            "range": "± 1362",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43390,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12728590,
            "range": "± 122440",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12076766,
            "range": "± 336802",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 885,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2288,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5177,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190886,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18468,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17352,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18307,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1344628,
            "range": "± 26837",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1080451,
            "range": "± 4418",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11062849,
            "range": "± 46982",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10510333,
            "range": "± 99772",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 115947897,
            "range": "± 1090519",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112446905,
            "range": "± 1026594",
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
            "value": 6561,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33469,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134648,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 557,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 546,
            "range": "± 3",
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
            "value": 2737,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9836,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38397,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13126,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 65993,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79572532,
            "range": "± 178798",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5242278,
            "range": "± 23450",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18949512,
            "range": "± 61673",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1245365,
            "range": "± 27915",
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
          "id": "843fd520cbd0eb4c2b1885fe11c997198beb2ca1",
          "message": "fix(deps): update anyhow for RUSTSEC-2026-0190 (#186)",
          "timestamp": "2026-07-10T00:55:14Z",
          "url": "https://github.com/aiconnai/engram/commit/843fd520cbd0eb4c2b1885fe11c997198beb2ca1"
        },
        "date": 1783835965030,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5312948,
            "range": "± 19743",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3576,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 389519,
            "range": "± 6105",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 369158,
            "range": "± 12418",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 529832,
            "range": "± 2167",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288383,
            "range": "± 1184",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118718,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 293,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 317852,
            "range": "± 10025",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47687,
            "range": "± 1289",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114480,
            "range": "± 734",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 289664,
            "range": "± 1703",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 445902,
            "range": "± 5039",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686455,
            "range": "± 2325",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 856060,
            "range": "± 5403",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1077070,
            "range": "± 10031",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35407,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21670,
            "range": "± 918",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321676,
            "range": "± 1329",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 254663,
            "range": "± 6711",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290709,
            "range": "± 2115",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 323591,
            "range": "± 1331",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 356275,
            "range": "± 1747",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43691,
            "range": "± 403",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12781120,
            "range": "± 132247",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12181889,
            "range": "± 113680",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 826,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2274,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5195,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 189290,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18485,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17351,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18445,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1371599,
            "range": "± 13944",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1095305,
            "range": "± 4067",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11249345,
            "range": "± 51567",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10619703,
            "range": "± 148473",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118178137,
            "range": "± 1412689",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112884856,
            "range": "± 974155",
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
            "value": 6493,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33211,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134451,
            "range": "± 2240",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 549,
            "range": "± 10",
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
            "value": 538,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2756,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10058,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39323,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12956,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 65892,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80106268,
            "range": "± 372631",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5282306,
            "range": "± 26067",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19223455,
            "range": "± 60904",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1254667,
            "range": "± 21626",
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
          "id": "13e7462d9a182efc4369b07e943f9c1797a9332f",
          "message": "refactor(graph): split duckdb_graph module (#190)",
          "timestamp": "2026-07-12T16:50:04Z",
          "url": "https://github.com/aiconnai/engram/commit/13e7462d9a182efc4369b07e943f9c1797a9332f"
        },
        "date": 1783923238683,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5239879,
            "range": "± 10050",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3505,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385598,
            "range": "± 1683",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 366554,
            "range": "± 5564",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 539498,
            "range": "± 2657",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289298,
            "range": "± 2961",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 136250,
            "range": "± 1607",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 297,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 318605,
            "range": "± 8755",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47467,
            "range": "± 1382",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 118376,
            "range": "± 1671",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 301339,
            "range": "± 1216",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 460832,
            "range": "± 5713",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 709920,
            "range": "± 5268",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 890279,
            "range": "± 5159",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1129301,
            "range": "± 19094",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35893,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22206,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 337046,
            "range": "± 3350",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 255013,
            "range": "± 2807",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 293595,
            "range": "± 2029",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 325509,
            "range": "± 1526",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 358941,
            "range": "± 2521",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 46000,
            "range": "± 2287",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13041984,
            "range": "± 87905",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12399726,
            "range": "± 46810",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 947,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2342,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5276,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192033,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18376,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17512,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18362,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1396705,
            "range": "± 5075",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1122300,
            "range": "± 6287",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11406288,
            "range": "± 28683",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10853103,
            "range": "± 54332",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 114097996,
            "range": "± 1756006",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 110190017,
            "range": "± 923465",
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
            "value": 6693,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34133,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137531,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 587,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 583,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 581,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2645,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9890,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38561,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13273,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 67836,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80085816,
            "range": "± 214892",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5362889,
            "range": "± 19979",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19527926,
            "range": "± 87537",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1265461,
            "range": "± 18734",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784007184413,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5221034,
            "range": "± 6622",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3552,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384792,
            "range": "± 1730",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 384402,
            "range": "± 13512",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 544112,
            "range": "± 2672",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290856,
            "range": "± 1555",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119667,
            "range": "± 851",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 299,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 325203,
            "range": "± 13108",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47788,
            "range": "± 1527",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117443,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 300663,
            "range": "± 1803",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 464099,
            "range": "± 6390",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 708380,
            "range": "± 3154",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 876549,
            "range": "± 4480",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1108422,
            "range": "± 18484",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35807,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22466,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 330563,
            "range": "± 2527",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 253570,
            "range": "± 5219",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289706,
            "range": "± 1432",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321588,
            "range": "± 1523",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 357789,
            "range": "± 2299",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44537,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13367123,
            "range": "± 274128",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13004685,
            "range": "± 237098",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 855,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2304,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5243,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190721,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18382,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17396,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18520,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1394700,
            "range": "± 6832",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1115523,
            "range": "± 4367",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 12371118,
            "range": "± 59002",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11354675,
            "range": "± 89978",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 122502947,
            "range": "± 1228660",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 117201426,
            "range": "± 1158249",
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
            "value": 6658,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34077,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 139331,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 580,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 583,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 564,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3016,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11263,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 42856,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13235,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68090,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80462245,
            "range": "± 164518",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5352094,
            "range": "± 16612",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19572643,
            "range": "± 137835",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1257827,
            "range": "± 10472",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784093590150,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5812031,
            "range": "± 12572",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3576,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 380862,
            "range": "± 1196",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 327370,
            "range": "± 11536",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 520424,
            "range": "± 3349",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283603,
            "range": "± 1813",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102948,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 291,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 279464,
            "range": "± 13576",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39725,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113455,
            "range": "± 1361",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 270978,
            "range": "± 3381",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 458453,
            "range": "± 3088",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 672522,
            "range": "± 3334",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 878479,
            "range": "± 7582",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1070937,
            "range": "± 15970",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32900,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22533,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 313558,
            "range": "± 4412",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 230323,
            "range": "± 2409",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 263463,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 291024,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 325258,
            "range": "± 4246",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35727,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13936383,
            "range": "± 255916",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13028734,
            "range": "± 224537",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 949,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2455,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5549,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 200593,
            "range": "± 5957",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19172,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18035,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19671,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1376743,
            "range": "± 47848",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1122587,
            "range": "± 23705",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 12011168,
            "range": "± 84209",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11512054,
            "range": "± 203647",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121535255,
            "range": "± 3228603",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118359581,
            "range": "± 2072180",
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
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34987,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141239,
            "range": "± 1565",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 552,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 550,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 551,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2664,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10057,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38989,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13483,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69520,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82667257,
            "range": "± 700331",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4715444,
            "range": "± 7678",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 16905254,
            "range": "± 53437",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1204402,
            "range": "± 13384",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784180573066,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 4387544,
            "range": "± 50009",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3047,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 324672,
            "range": "± 10687",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 306878,
            "range": "± 11688",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 420608,
            "range": "± 9923",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 240203,
            "range": "± 8170",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 87561,
            "range": "± 2400",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 225,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 263977,
            "range": "± 16057",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 33346,
            "range": "± 1338",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 100828,
            "range": "± 1608",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 234379,
            "range": "± 3695",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 375503,
            "range": "± 12309",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 551070,
            "range": "± 14329",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 711903,
            "range": "± 18578",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 871228,
            "range": "± 22990",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 27531,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 19331,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 243258,
            "range": "± 14629",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 184123,
            "range": "± 5544",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 211274,
            "range": "± 5569",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 238658,
            "range": "± 4867",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 262094,
            "range": "± 5157",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 32070,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 10160271,
            "range": "± 250388",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 9842720,
            "range": "± 253127",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 623,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2008,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 4827,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 166219,
            "range": "± 3227",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 15460,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 14424,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 15282,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1114401,
            "range": "± 11847",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 902221,
            "range": "± 32073",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 9202701,
            "range": "± 358870",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 8666793,
            "range": "± 205401",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 96394322,
            "range": "± 2275834",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 93130845,
            "range": "± 2539820",
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
            "value": 5607,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 28670,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 115184,
            "range": "± 3502",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 486,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 479,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 478,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2404,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 8680,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 35117,
            "range": "± 909",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 11084,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 56954,
            "range": "± 1119",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 74720335,
            "range": "± 1623591",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 3861377,
            "range": "± 42159",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 14139242,
            "range": "± 94108",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1032537,
            "range": "± 28864",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784267329367,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5837928,
            "range": "± 32170",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3618,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 381126,
            "range": "± 4572",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 318456,
            "range": "± 8039",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 511814,
            "range": "± 4583",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 284075,
            "range": "± 2741",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101703,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 285,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 286779,
            "range": "± 13416",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39821,
            "range": "± 1022",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112530,
            "range": "± 527",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 267213,
            "range": "± 1985",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 443750,
            "range": "± 3659",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 667856,
            "range": "± 2994",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 860551,
            "range": "± 11385",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1065028,
            "range": "± 12090",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32956,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22093,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 297051,
            "range": "± 1366",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 231578,
            "range": "± 4803",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 266028,
            "range": "± 2118",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 296331,
            "range": "± 1866",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 328466,
            "range": "± 1721",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 37229,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13026541,
            "range": "± 156450",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12713455,
            "range": "± 114534",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 940,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2547,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5719,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 204083,
            "range": "± 704",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19382,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18015,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19045,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1331379,
            "range": "± 3597",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1103520,
            "range": "± 6498",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11212802,
            "range": "± 59248",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10827959,
            "range": "± 119476",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120246385,
            "range": "± 1307948",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116495954,
            "range": "± 1377040",
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
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34820,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141519,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 541,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 537,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 540,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2863,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10073,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 35899,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13467,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69344,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82315905,
            "range": "± 107802",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4794773,
            "range": "± 12689",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17271293,
            "range": "± 116608",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1174598,
            "range": "± 25253",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784352505048,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5271200,
            "range": "± 10837",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3563,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385037,
            "range": "± 1300",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 386103,
            "range": "± 10985",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 539290,
            "range": "± 4514",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 293653,
            "range": "± 2009",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 119427,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 334877,
            "range": "± 13958",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48011,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114290,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 289859,
            "range": "± 2380",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 442160,
            "range": "± 1588",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 681129,
            "range": "± 2411",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 847431,
            "range": "± 4023",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1069285,
            "range": "± 9038",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35526,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21405,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 317580,
            "range": "± 2620",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 253430,
            "range": "± 2391",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 291183,
            "range": "± 1363",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 322536,
            "range": "± 2559",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 357076,
            "range": "± 4281",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 46915,
            "range": "± 1164",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13236119,
            "range": "± 120366",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12581481,
            "range": "± 252977",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 843,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2295,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5266,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 193142,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18070,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16975,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18020,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1350152,
            "range": "± 2858",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1083854,
            "range": "± 3482",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11606993,
            "range": "± 109408",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10936951,
            "range": "± 122168",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119810550,
            "range": "± 566903",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114653246,
            "range": "± 708263",
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
            "value": 6546,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33597,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137084,
            "range": "± 3095",
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
            "value": 578,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 572,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3099,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11469,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 45157,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 14461,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 73652,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80531276,
            "range": "± 254068",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5533143,
            "range": "± 32353",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 20077837,
            "range": "± 47989",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1265974,
            "range": "± 22873",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784440437978,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5243623,
            "range": "± 36716",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3511,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385277,
            "range": "± 3392",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 389876,
            "range": "± 13516",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 538529,
            "range": "± 9292",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 292584,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 121672,
            "range": "± 732",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 284,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 335617,
            "range": "± 18982",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48964,
            "range": "± 3111",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115728,
            "range": "± 1891",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 290985,
            "range": "± 5201",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 451229,
            "range": "± 1826",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686109,
            "range": "± 3786",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 865950,
            "range": "± 5334",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1080212,
            "range": "± 13576",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35594,
            "range": "± 1628",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21653,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 325622,
            "range": "± 1503",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 254766,
            "range": "± 1292",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 292049,
            "range": "± 4515",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321329,
            "range": "± 8751",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 355654,
            "range": "± 3909",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44617,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13418355,
            "range": "± 175296",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12646751,
            "range": "± 268998",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 854,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2306,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5276,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192153,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18036,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17022,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17963,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1353701,
            "range": "± 25631",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1082565,
            "range": "± 3764",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11466379,
            "range": "± 153655",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10879464,
            "range": "± 120567",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118826585,
            "range": "± 903259",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113856711,
            "range": "± 2296163",
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
            "value": 6538,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33375,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137062,
            "range": "± 2595",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 584,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 578,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 555,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3140,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11643,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44995,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12910,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66694,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79857550,
            "range": "± 433017",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5493100,
            "range": "± 16782",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19869761,
            "range": "± 50138",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1251225,
            "range": "± 14206",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784527895103,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5250396,
            "range": "± 5560",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3587,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385900,
            "range": "± 2261",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 377319,
            "range": "± 13318",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 539709,
            "range": "± 4243",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 293092,
            "range": "± 1613",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117900,
            "range": "± 845",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 283,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 328138,
            "range": "± 12060",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47902,
            "range": "± 1394",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115979,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 291831,
            "range": "± 1336",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 449931,
            "range": "± 1975",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 691238,
            "range": "± 8604",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 867335,
            "range": "± 3940",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1087032,
            "range": "± 27254",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35762,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22144,
            "range": "± 991",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 320144,
            "range": "± 2989",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 251749,
            "range": "± 1555",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 288836,
            "range": "± 6073",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 318767,
            "range": "± 2672",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353802,
            "range": "± 2149",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45335,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12859449,
            "range": "± 163111",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12446012,
            "range": "± 181231",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 846,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2301,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5215,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191987,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18464,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17578,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18402,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1360587,
            "range": "± 6359",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1098668,
            "range": "± 6835",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11321979,
            "range": "± 272289",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10897816,
            "range": "± 135198",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120961046,
            "range": "± 899047",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116061367,
            "range": "± 1409804",
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
            "value": 6525,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33155,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135378,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 579,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 572,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 575,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3101,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11728,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44511,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13016,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66370,
            "range": "± 1139",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80247307,
            "range": "± 824798",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5430781,
            "range": "± 59455",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19631936,
            "range": "± 83026",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1274623,
            "range": "± 26672",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784613201486,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5834612,
            "range": "± 3030",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3606,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 380466,
            "range": "± 1854",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 327155,
            "range": "± 9094",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513870,
            "range": "± 3459",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283808,
            "range": "± 1756",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101463,
            "range": "± 530",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 288259,
            "range": "± 20467",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39576,
            "range": "± 1575",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113704,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 266862,
            "range": "± 2327",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 452287,
            "range": "± 8109",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 667821,
            "range": "± 5054",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 867120,
            "range": "± 9166",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1070242,
            "range": "± 28238",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32642,
            "range": "± 1309",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21361,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 299399,
            "range": "± 1842",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224188,
            "range": "± 7381",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 258556,
            "range": "± 6385",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 286970,
            "range": "± 4704",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 318774,
            "range": "± 4040",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35601,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13204068,
            "range": "± 542247",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12811258,
            "range": "± 516251",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 907,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2474,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5619,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 200494,
            "range": "± 1922",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19168,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17880,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19119,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1318824,
            "range": "± 5312",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1073827,
            "range": "± 8706",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11129812,
            "range": "± 211865",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10613023,
            "range": "± 59966",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117911791,
            "range": "± 1288664",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113690877,
            "range": "± 1350973",
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
            "value": 6696,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34764,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140890,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 537,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 516,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 524,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2614,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9816,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38322,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13534,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 70448,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82906911,
            "range": "± 749759",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4769517,
            "range": "± 20832",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17252590,
            "range": "± 118230",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1171338,
            "range": "± 106267",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784699503100,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5223692,
            "range": "± 13581",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3568,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385895,
            "range": "± 2302",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 373109,
            "range": "± 8837",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 548049,
            "range": "± 2460",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 291842,
            "range": "± 4741",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120830,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 285,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 328656,
            "range": "± 23331",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 49049,
            "range": "± 1365",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 114692,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 292914,
            "range": "± 1900",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447340,
            "range": "± 4072",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 696149,
            "range": "± 4731",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 856138,
            "range": "± 8189",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1080647,
            "range": "± 20333",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35948,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22022,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 317842,
            "range": "± 3314",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252503,
            "range": "± 1999",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290348,
            "range": "± 1702",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321635,
            "range": "± 3097",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 356784,
            "range": "± 7655",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 46119,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13650012,
            "range": "± 103359",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12894852,
            "range": "± 90620",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 835,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2272,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5299,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192194,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18143,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17135,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18118,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1364298,
            "range": "± 4077",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1090252,
            "range": "± 4583",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11451614,
            "range": "± 114729",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10745733,
            "range": "± 140705",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121687027,
            "range": "± 797135",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115746278,
            "range": "± 903059",
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
            "value": 6486,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33370,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137400,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 580,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 578,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 578,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3004,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11376,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43999,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12908,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66076,
            "range": "± 896",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80780566,
            "range": "± 202168",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5398345,
            "range": "± 13241",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19540424,
            "range": "± 124241",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1257609,
            "range": "± 17539",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784786177677,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5816831,
            "range": "± 7726",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3604,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 381455,
            "range": "± 4233",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 318275,
            "range": "± 29972",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 518307,
            "range": "± 2815",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 281327,
            "range": "± 2352",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 100876,
            "range": "± 1323",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 291,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 277919,
            "range": "± 11978",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39654,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113356,
            "range": "± 2495",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 264152,
            "range": "± 2010",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 453982,
            "range": "± 3763",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 662459,
            "range": "± 7724",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 862047,
            "range": "± 3011",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1056476,
            "range": "± 18230",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33109,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21673,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 301768,
            "range": "± 12414",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224287,
            "range": "± 1782",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 260105,
            "range": "± 2096",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 289448,
            "range": "± 2021",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 321142,
            "range": "± 2794",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35671,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12977149,
            "range": "± 607374",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12295618,
            "range": "± 67931",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 941,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2467,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5582,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201647,
            "range": "± 3455",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19106,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18013,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18989,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1323364,
            "range": "± 5229",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1076690,
            "range": "± 7005",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11169795,
            "range": "± 55959",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10765584,
            "range": "± 50789",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 117869156,
            "range": "± 1119685",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114479314,
            "range": "± 1034106",
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
            "value": 6647,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34937,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141544,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 540,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 539,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 539,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2702,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9783,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39451,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13422,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69801,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82772978,
            "range": "± 156566",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4751387,
            "range": "± 14793",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17089186,
            "range": "± 98418",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1171148,
            "range": "± 14293",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784872378342,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5372309,
            "range": "± 14984",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3550,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384641,
            "range": "± 3954",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 392871,
            "range": "± 12610",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 542439,
            "range": "± 3626",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 295759,
            "range": "± 4864",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118742,
            "range": "± 924",
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
            "value": 329564,
            "range": "± 11117",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47905,
            "range": "± 1393",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115091,
            "range": "± 1006",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 290763,
            "range": "± 2569",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 448614,
            "range": "± 4404",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 688890,
            "range": "± 3664",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 867247,
            "range": "± 36513",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1077714,
            "range": "± 24244",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35526,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21387,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 319519,
            "range": "± 2179",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 251550,
            "range": "± 3334",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289149,
            "range": "± 5053",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319994,
            "range": "± 4194",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353155,
            "range": "± 2277",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44427,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13191031,
            "range": "± 346180",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12650019,
            "range": "± 221508",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 843,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2289,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5286,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190691,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18077,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17064,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18070,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1354097,
            "range": "± 28861",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1087286,
            "range": "± 3901",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11525288,
            "range": "± 122111",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10908129,
            "range": "± 117700",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 119303995,
            "range": "± 753300",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 113783300,
            "range": "± 915369",
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
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33156,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135425,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 576,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 578,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 563,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3247,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11965,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 45826,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12932,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66687,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79829624,
            "range": "± 144985",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5361428,
            "range": "± 21055",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19650180,
            "range": "± 57531",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1255683,
            "range": "± 80298",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1784958410341,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5237616,
            "range": "± 20120",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3591,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 386337,
            "range": "± 2721",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 372822,
            "range": "± 9609",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 550779,
            "range": "± 4756",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 293122,
            "range": "± 4499",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118318,
            "range": "± 1137",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 283,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 330818,
            "range": "± 10949",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48148,
            "range": "± 1359",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117741,
            "range": "± 2514",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 294194,
            "range": "± 8630",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 461148,
            "range": "± 3424",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 702735,
            "range": "± 4039",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 882217,
            "range": "± 5049",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1102508,
            "range": "± 27109",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35654,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21993,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 325889,
            "range": "± 1972",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 251475,
            "range": "± 9635",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 287899,
            "range": "± 2014",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319040,
            "range": "± 16826",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 353245,
            "range": "± 2890",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45694,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12791568,
            "range": "± 112318",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12211516,
            "range": "± 118996",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 831,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2300,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5243,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 193619,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18032,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16993,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17990,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1344909,
            "range": "± 3234",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1075814,
            "range": "± 4516",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11160015,
            "range": "± 43972",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10541446,
            "range": "± 113096",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120250510,
            "range": "± 1281288",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 116363235,
            "range": "± 804707",
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
            "value": 6555,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33329,
            "range": "± 1476",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 137086,
            "range": "± 686",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 608,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 606,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 561,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3130,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11721,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44360,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12933,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66395,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81471951,
            "range": "± 1018648",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5562890,
            "range": "± 49741",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 20583245,
            "range": "± 285139",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1260415,
            "range": "± 8300",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785045779106,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5294326,
            "range": "± 52862",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3523,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384456,
            "range": "± 2820",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 385590,
            "range": "± 14406",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536482,
            "range": "± 3017",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 292985,
            "range": "± 2323",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117614,
            "range": "± 7056",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 285,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 319342,
            "range": "± 14969",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48266,
            "range": "± 3320",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115233,
            "range": "± 3196",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 293074,
            "range": "± 1953",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 454729,
            "range": "± 13066",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 687411,
            "range": "± 12507",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 856654,
            "range": "± 6485",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1072588,
            "range": "± 12164",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35021,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21754,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 318509,
            "range": "± 1342",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252377,
            "range": "± 10354",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290841,
            "range": "± 3496",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 321805,
            "range": "± 3221",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 356844,
            "range": "± 2532",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45197,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12561512,
            "range": "± 585358",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12026266,
            "range": "± 181879",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 853,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2296,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5300,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192722,
            "range": "± 1728",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17982,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17008,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18153,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1352992,
            "range": "± 12788",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1081084,
            "range": "± 28832",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11608019,
            "range": "± 229701",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10541697,
            "range": "± 112422",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118771128,
            "range": "± 819123",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114350374,
            "range": "± 1899968",
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
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33426,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135602,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 579,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 574,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 573,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3032,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11445,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44271,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12893,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 65901,
            "range": "± 2883",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79728168,
            "range": "± 1000412",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5385869,
            "range": "± 82391",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19692588,
            "range": "± 204682",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1265143,
            "range": "± 15899",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785133714330,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5198014,
            "range": "± 32457",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3524,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385768,
            "range": "± 1344",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 360056,
            "range": "± 9846",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 533479,
            "range": "± 7159",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 290789,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117109,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 288,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 327420,
            "range": "± 10495",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48337,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115355,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 292989,
            "range": "± 1504",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 444147,
            "range": "± 4826",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 687986,
            "range": "± 7809",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 850575,
            "range": "± 8740",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1074742,
            "range": "± 19701",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 37466,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21423,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321010,
            "range": "± 2465",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252368,
            "range": "± 6298",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289087,
            "range": "± 4544",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 319781,
            "range": "± 8011",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 351519,
            "range": "± 1725",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44332,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12599802,
            "range": "± 46564",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12009398,
            "range": "± 46584",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 860,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2310,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5248,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191138,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18015,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17133,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18039,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1348043,
            "range": "± 18637",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1080917,
            "range": "± 3270",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11079794,
            "range": "± 20859",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10493354,
            "range": "± 50522",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 113299466,
            "range": "± 973176",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 109291047,
            "range": "± 2758822",
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
            "value": 6525,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 32995,
            "range": "± 691",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135037,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 580,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 574,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 573,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3140,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11606,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 43607,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12925,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66285,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79855482,
            "range": "± 146717",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5296912,
            "range": "± 13881",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19182739,
            "range": "± 52643",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1243000,
            "range": "± 17486",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785217761485,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5231574,
            "range": "± 6264",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3509,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 391045,
            "range": "± 18300",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 380816,
            "range": "± 9351",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 541535,
            "range": "± 4411",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 291563,
            "range": "± 2458",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 116908,
            "range": "± 1739",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 328909,
            "range": "± 10387",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47473,
            "range": "± 1312",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117366,
            "range": "± 1804",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 293774,
            "range": "± 4676",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 456333,
            "range": "± 4024",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 690029,
            "range": "± 4387",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 877855,
            "range": "± 6045",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1083283,
            "range": "± 12283",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35443,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21895,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321250,
            "range": "± 3086",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 256181,
            "range": "± 1649",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 293665,
            "range": "± 1307",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 323807,
            "range": "± 2319",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 358317,
            "range": "± 2636",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45350,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12760506,
            "range": "± 247330",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12311020,
            "range": "± 361650",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 830,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2273,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5262,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191808,
            "range": "± 676",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18302,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17075,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18016,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1352877,
            "range": "± 4233",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1081506,
            "range": "± 8347",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11116987,
            "range": "± 35446",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10540689,
            "range": "± 62473",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 113801347,
            "range": "± 1477042",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 109300001,
            "range": "± 1421822",
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
            "value": 6563,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33378,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 136061,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 578,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 575,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 560,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3243,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11389,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44374,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12996,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66108,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80162262,
            "range": "± 413558",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5398567,
            "range": "± 24608",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19804576,
            "range": "± 105175",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1251633,
            "range": "± 13290",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785304440869,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5231518,
            "range": "± 13618",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3566,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 385342,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 367875,
            "range": "± 6125",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 540670,
            "range": "± 6824",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289704,
            "range": "± 1670",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 116527,
            "range": "± 759",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 284,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 318872,
            "range": "± 11778",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47172,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115853,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 289199,
            "range": "± 1719",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 445673,
            "range": "± 8331",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 681457,
            "range": "± 3338",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 863845,
            "range": "± 4589",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1068148,
            "range": "± 14944",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35230,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21671,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 321244,
            "range": "± 7626",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 249595,
            "range": "± 4346",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 283884,
            "range": "± 8321",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 316283,
            "range": "± 1882",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 351192,
            "range": "± 1593",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43694,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12441690,
            "range": "± 108489",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11860761,
            "range": "± 117006",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 865,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2276,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5243,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192081,
            "range": "± 2443",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17999,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17044,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17969,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1338328,
            "range": "± 6118",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1071290,
            "range": "± 5314",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10935035,
            "range": "± 26893",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10398960,
            "range": "± 95115",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 114344876,
            "range": "± 1682390",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 109052733,
            "range": "± 1499557",
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
            "value": 6497,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33228,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 136881,
            "range": "± 1056",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 583,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 579,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 575,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3055,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11404,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44214,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12926,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66515,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79210344,
            "range": "± 149478",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5292518,
            "range": "± 31279",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19258857,
            "range": "± 104005",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1241275,
            "range": "± 33501",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785390190043,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5840335,
            "range": "± 24426",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3562,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382071,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 321450,
            "range": "± 8114",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 513571,
            "range": "± 2755",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 282314,
            "range": "± 2077",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 100777,
            "range": "± 1340",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 286,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 286285,
            "range": "± 11804",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39852,
            "range": "± 949",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113546,
            "range": "± 1713",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 266034,
            "range": "± 18875",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 447784,
            "range": "± 5420",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 659460,
            "range": "± 3990",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 866724,
            "range": "± 27128",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1048723,
            "range": "± 25339",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32381,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21382,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 295444,
            "range": "± 3197",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224353,
            "range": "± 1299",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 260983,
            "range": "± 9242",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 289090,
            "range": "± 2173",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 321279,
            "range": "± 1873",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 36686,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12869725,
            "range": "± 139272",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12531556,
            "range": "± 665793",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 952,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2487,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5580,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202389,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19128,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18291,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19120,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1311007,
            "range": "± 4754",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1087274,
            "range": "± 13318",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11419129,
            "range": "± 212582",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10691323,
            "range": "± 144011",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 120112927,
            "range": "± 787505",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 115543155,
            "range": "± 1045834",
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
            "value": 6748,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 35227,
            "range": "± 851",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141936,
            "range": "± 1456",
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
            "value": 537,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 544,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2724,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10182,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39185,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13373,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69396,
            "range": "± 2210",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83521414,
            "range": "± 478885",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4805702,
            "range": "± 17357",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17322516,
            "range": "± 63959",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1174031,
            "range": "± 35017",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785477983502,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 3619292,
            "range": "± 24120",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 2304,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 246125,
            "range": "± 5225",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 226222,
            "range": "± 10512",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 311197,
            "range": "± 1287",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 184031,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 64823,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 172,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 219794,
            "range": "± 17117",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 24555,
            "range": "± 1740",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 72523,
            "range": "± 1742",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 169738,
            "range": "± 6523",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 268684,
            "range": "± 2999",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 407636,
            "range": "± 2434",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 511861,
            "range": "± 14121",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 642564,
            "range": "± 30095",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 20402,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 12804,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 177873,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 137322,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 160587,
            "range": "± 1750",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 177087,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 197727,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 24377,
            "range": "± 1759",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 7733236,
            "range": "± 292049",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 7316126,
            "range": "± 94460",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 497,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 1563,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 3687,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 126626,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 11679,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 10915,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 11542,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 828858,
            "range": "± 15158",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 654566,
            "range": "± 30409",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 6719169,
            "range": "± 155031",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 6430696,
            "range": "± 185279",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 71399589,
            "range": "± 3093875",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 69635355,
            "range": "± 1609199",
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
            "value": 4032,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 21036,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 85210,
            "range": "± 1980",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 356,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 352,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 363,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 1755,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 6365,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 25488,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 8173,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 41850,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 59691045,
            "range": "± 1813544",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 2981226,
            "range": "± 79534",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 10558467,
            "range": "± 279661",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 742909,
            "range": "± 34855",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785563896253,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5834787,
            "range": "± 3193",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3629,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382826,
            "range": "± 2029",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 320108,
            "range": "± 6359",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 516267,
            "range": "± 5408",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 281941,
            "range": "± 1585",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 100847,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 277123,
            "range": "± 12165",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39105,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112596,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 265562,
            "range": "± 1713",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 459372,
            "range": "± 2265",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 676079,
            "range": "± 5030",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 888110,
            "range": "± 3206",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1081162,
            "range": "± 32424",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32926,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21976,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 294868,
            "range": "± 1460",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 223104,
            "range": "± 3670",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 258581,
            "range": "± 1247",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 287418,
            "range": "± 1512",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 320755,
            "range": "± 1325",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35663,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12717101,
            "range": "± 43461",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12306948,
            "range": "± 47694",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 904,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2479,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5586,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201112,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19006,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17862,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19018,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1319454,
            "range": "± 3483",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1078366,
            "range": "± 2812",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11093472,
            "range": "± 72883",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10759892,
            "range": "± 69266",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 115998533,
            "range": "± 1012076",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 112620026,
            "range": "± 1346104",
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
            "value": 6676,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34849,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140433,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 544,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 541,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 547,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2577,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9835,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38546,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13420,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69673,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 83103125,
            "range": "± 922256",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4726522,
            "range": "± 11264",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17029928,
            "range": "± 60275",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1178532,
            "range": "± 14150",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785650336067,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5241265,
            "range": "± 9210",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3588,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 386175,
            "range": "± 3160",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 360188,
            "range": "± 7292",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 531621,
            "range": "± 3030",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289032,
            "range": "± 4497",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118701,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 284,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 313416,
            "range": "± 9893",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 46897,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113953,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 286252,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 446022,
            "range": "± 1846",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 678737,
            "range": "± 3031",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 856979,
            "range": "± 5194",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1065903,
            "range": "± 11997",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34707,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21148,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 318594,
            "range": "± 1692",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 247504,
            "range": "± 1724",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 284613,
            "range": "± 1342",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 315075,
            "range": "± 1383",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 346400,
            "range": "± 3163",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43553,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12393131,
            "range": "± 38484",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11908446,
            "range": "± 76907",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 848,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2294,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5249,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191398,
            "range": "± 421",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18080,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17119,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18086,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1334720,
            "range": "± 4871",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1079574,
            "range": "± 5354",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10925106,
            "range": "± 80025",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10411707,
            "range": "± 102092",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 110912029,
            "range": "± 1600661",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 108024875,
            "range": "± 1632036",
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
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33343,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135076,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 578,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 571,
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
            "value": 3185,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11453,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 45061,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12814,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66157,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79881261,
            "range": "± 354536",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5206086,
            "range": "± 12763",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 18996300,
            "range": "± 62239",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1240027,
            "range": "± 118710",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785737658854,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 4458628,
            "range": "± 48338",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3035,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 322329,
            "range": "± 5310",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 287475,
            "range": "± 13109",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 412547,
            "range": "± 11563",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 244780,
            "range": "± 6530",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 86374,
            "range": "± 1568",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 232,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 249579,
            "range": "± 15881",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 32774,
            "range": "± 1237",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 97289,
            "range": "± 2216",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 224530,
            "range": "± 6786",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 380908,
            "range": "± 6636",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 543901,
            "range": "± 11655",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 704328,
            "range": "± 12978",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 853022,
            "range": "± 21934",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 27382,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 18649,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 231886,
            "range": "± 7969",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 180806,
            "range": "± 4193",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 209823,
            "range": "± 5310",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 233995,
            "range": "± 6891",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 253033,
            "range": "± 5250",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 30483,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 9914034,
            "range": "± 137044",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 9524347,
            "range": "± 332623",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 618,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 1945,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 4633,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 160474,
            "range": "± 4448",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 15431,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 14257,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 15266,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1102551,
            "range": "± 13536",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 881806,
            "range": "± 26943",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 9281048,
            "range": "± 129468",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 8772331,
            "range": "± 228558",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 97693461,
            "range": "± 2196310",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 94574191,
            "range": "± 1678941",
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
            "value": 5313,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 27890,
            "range": "± 1635",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 114631,
            "range": "± 3688",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 485,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 495,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 497,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2387,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 8736,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 36612,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 10956,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 55491,
            "range": "± 1171",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 75775019,
            "range": "± 766668",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 3879543,
            "range": "± 48330",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 13747146,
            "range": "± 249331",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1003927,
            "range": "± 28812",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785822623771,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5237562,
            "range": "± 6483",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3605,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 386376,
            "range": "± 1712",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 362146,
            "range": "± 6917",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 531899,
            "range": "± 5183",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289094,
            "range": "± 2133",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 115527,
            "range": "± 606",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 306,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 321144,
            "range": "± 9480",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47792,
            "range": "± 2752",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115205,
            "range": "± 2511",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 290934,
            "range": "± 3735",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 450811,
            "range": "± 3638",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 687932,
            "range": "± 6817",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 875688,
            "range": "± 14385",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1078199,
            "range": "± 36254",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35953,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21825,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 320760,
            "range": "± 2850",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 248338,
            "range": "± 1664",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 287446,
            "range": "± 951",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 318308,
            "range": "± 2068",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 352388,
            "range": "± 1647",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44247,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12477638,
            "range": "± 126801",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11906152,
            "range": "± 147436",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 856,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2298,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5218,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191742,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18015,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16954,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18009,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1329004,
            "range": "± 5668",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1065339,
            "range": "± 4808",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10935728,
            "range": "± 110909",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10400901,
            "range": "± 49846",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 116048723,
            "range": "± 1328393",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 111796292,
            "range": "± 804117",
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
            "value": 6477,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33213,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 134446,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 571,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 560,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 560,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3086,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11406,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44479,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12931,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66251,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80041573,
            "range": "± 270297",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5292337,
            "range": "± 26233",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19594365,
            "range": "± 209156",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1242668,
            "range": "± 11448",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785908936781,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5238114,
            "range": "± 48223",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3543,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 388334,
            "range": "± 3052",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 371734,
            "range": "± 9263",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 536105,
            "range": "± 4045",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 291366,
            "range": "± 1311",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 117128,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 311,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 319888,
            "range": "± 19361",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47300,
            "range": "± 1563",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113516,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 288543,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 437041,
            "range": "± 3117",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 679841,
            "range": "± 2807",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 842508,
            "range": "± 7841",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1067010,
            "range": "± 30477",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 34986,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21496,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 317773,
            "range": "± 2710",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 254371,
            "range": "± 6556",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 289885,
            "range": "± 4242",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 320677,
            "range": "± 1485",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 352809,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44473,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12671217,
            "range": "± 104553",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12159893,
            "range": "± 289567",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 853,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2292,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5435,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 191896,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18084,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17030,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18020,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1355072,
            "range": "± 3743",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1090278,
            "range": "± 5439",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11191077,
            "range": "± 103606",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10597982,
            "range": "± 86724",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 118255812,
            "range": "± 856920",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 114005716,
            "range": "± 806605",
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
            "value": 6566,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33570,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135506,
            "range": "± 621",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 572,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 574,
            "range": "± 22",
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
            "value": 3116,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11626,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 46917,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12870,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66681,
            "range": "± 830",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 79586651,
            "range": "± 211333",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5330450,
            "range": "± 17426",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19424130,
            "range": "± 71966",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1232097,
            "range": "± 12405",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1785995463729,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5838111,
            "range": "± 8237",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3595,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382328,
            "range": "± 7826",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 332175,
            "range": "± 11459",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 520345,
            "range": "± 63135",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283629,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102959,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 290,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 281053,
            "range": "± 12125",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39641,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112194,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 263215,
            "range": "± 14524",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 446150,
            "range": "± 2641",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 663056,
            "range": "± 5364",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 865090,
            "range": "± 4071",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1051042,
            "range": "± 21164",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32987,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21317,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 301673,
            "range": "± 2846",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224995,
            "range": "± 1387",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 259890,
            "range": "± 1727",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 289380,
            "range": "± 2864",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 322659,
            "range": "± 3035",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35748,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13855191,
            "range": "± 265547",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 13485395,
            "range": "± 102311",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 959,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2487,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5512,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 201236,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19045,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17996,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19234,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1319590,
            "range": "± 7203",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1075361,
            "range": "± 5338",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11984919,
            "range": "± 255292",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 11639996,
            "range": "± 144039",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 127059320,
            "range": "± 1636867",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 119806131,
            "range": "± 1520701",
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
            "value": 6681,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34824,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141529,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 546,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 562,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 552,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2833,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9848,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39372,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13493,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69259,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82465178,
            "range": "± 1126805",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4833226,
            "range": "± 39107",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17402172,
            "range": "± 62270",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1200657,
            "range": "± 28438",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1786078791178,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5228769,
            "range": "± 2675",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3778,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 381366,
            "range": "± 1419",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 372743,
            "range": "± 8114",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 531514,
            "range": "± 4433",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 288918,
            "range": "± 1940",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 118270,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 284,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 325867,
            "range": "± 13426",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48095,
            "range": "± 1218",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 113800,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 285782,
            "range": "± 2823",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 439541,
            "range": "± 3192",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 682672,
            "range": "± 3590",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 846000,
            "range": "± 18163",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1064502,
            "range": "± 39097",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35038,
            "range": "± 1000",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21598,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 317093,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 249937,
            "range": "± 9753",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 284870,
            "range": "± 1061",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 316174,
            "range": "± 1411",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 347681,
            "range": "± 3499",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 43586,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12442121,
            "range": "± 214817",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11831002,
            "range": "± 118610",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 861,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2323,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5294,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 192580,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 17995,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 16951,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17940,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1330010,
            "range": "± 4971",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1068893,
            "range": "± 48725",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 10898931,
            "range": "± 366035",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10365481,
            "range": "± 316936",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 112957630,
            "range": "± 2648720",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 106816315,
            "range": "± 2497527",
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
            "value": 6521,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33240,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 133832,
            "range": "± 1438",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 575,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 577,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 569,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2657,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10038,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 39824,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12909,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 66017,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80032954,
            "range": "± 320288",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5312700,
            "range": "± 35106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19305016,
            "range": "± 176758",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1245249,
            "range": "± 29840",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1786162488511,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5863906,
            "range": "± 26838",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3560,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 381800,
            "range": "± 1228",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 322693,
            "range": "± 8263",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 511624,
            "range": "± 2634",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289272,
            "range": "± 7210",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 102981,
            "range": "± 1210",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 277499,
            "range": "± 11663",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 40474,
            "range": "± 1022",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 117318,
            "range": "± 538",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 270942,
            "range": "± 1481",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 463735,
            "range": "± 2080",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 684558,
            "range": "± 5240",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 888022,
            "range": "± 9193",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1098875,
            "range": "± 12562",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33839,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22177,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 296203,
            "range": "± 1277",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 228110,
            "range": "± 2719",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 260715,
            "range": "± 1664",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 290032,
            "range": "± 15239",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 322595,
            "range": "± 2013",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35764,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 13245473,
            "range": "± 282071",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12828432,
            "range": "± 302739",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 924,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2492,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5604,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203118,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19199,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18111,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19356,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1323003,
            "range": "± 18935",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1084436,
            "range": "± 19366",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11222514,
            "range": "± 49967",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10690085,
            "range": "± 114001",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121063290,
            "range": "± 2468005",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 117538145,
            "range": "± 1795168",
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
            "value": 6660,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34776,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141765,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 548,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 528,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 527,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2769,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 10131,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 42844,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13399,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69532,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82476197,
            "range": "± 182991",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4769626,
            "range": "± 73971",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17220141,
            "range": "± 104978",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1190659,
            "range": "± 13569",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1786249285779,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5840569,
            "range": "± 17708",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3562,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 382993,
            "range": "± 804",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 315185,
            "range": "± 7299",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 511852,
            "range": "± 3570",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 284245,
            "range": "± 2071",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 103013,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 274662,
            "range": "± 10925",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39396,
            "range": "± 1065",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115045,
            "range": "± 887",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 264523,
            "range": "± 3365",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 449116,
            "range": "± 1444",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 661140,
            "range": "± 4410",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 877665,
            "range": "± 7405",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1054395,
            "range": "± 12017",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 33825,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21735,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 299544,
            "range": "± 1366",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224419,
            "range": "± 2543",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 258748,
            "range": "± 4148",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 288388,
            "range": "± 3094",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 321804,
            "range": "± 1562",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35883,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12692705,
            "range": "± 52889",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12249695,
            "range": "± 81666",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 922,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2486,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5612,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 202298,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 19189,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 18001,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19213,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1307421,
            "range": "± 4632",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1087029,
            "range": "± 4473",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11057041,
            "range": "± 38196",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10583314,
            "range": "± 73963",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 116236750,
            "range": "± 642946",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 111218560,
            "range": "± 899708",
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
            "value": 6687,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34700,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 141307,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 543,
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
            "value": 534,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2656,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9640,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 38469,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13471,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69722,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82649307,
            "range": "± 858215",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4797209,
            "range": "± 18216",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17173514,
            "range": "± 62680",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1176978,
            "range": "± 17546",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1786336444136,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5239797,
            "range": "± 89291",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3518,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 384931,
            "range": "± 2426",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 366280,
            "range": "± 9479",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 530275,
            "range": "± 2248",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 289605,
            "range": "± 1120",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 120006,
            "range": "± 1989",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 286,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 320492,
            "range": "± 10097",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 47742,
            "range": "± 1478",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115243,
            "range": "± 831",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 288724,
            "range": "± 4033",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 450956,
            "range": "± 3012",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 686338,
            "range": "± 8192",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 873666,
            "range": "± 8745",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1079700,
            "range": "± 14959",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35322,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21430,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 324696,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 249227,
            "range": "± 2203",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 285678,
            "range": "± 1880",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 315901,
            "range": "± 7494",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 347760,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 44006,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12549430,
            "range": "± 73131",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 11961913,
            "range": "± 38611",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 827,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2278,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5250,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 190946,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18105,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17387,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 17941,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1346742,
            "range": "± 5321",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1082461,
            "range": "± 17630",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11070189,
            "range": "± 48069",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10421157,
            "range": "± 167821",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 115681428,
            "range": "± 1272244",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 110655855,
            "range": "± 1803682",
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
            "value": 6507,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 33059,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 135612,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 574,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 575,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 571,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3108,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11546,
            "range": "± 563",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 45848,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 12905,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 65698,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 80503216,
            "range": "± 148089",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5287706,
            "range": "± 37082",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19333018,
            "range": "± 93604",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1267326,
            "range": "± 15563",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1786422170044,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5234171,
            "range": "± 17164",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3566,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 393505,
            "range": "± 5655",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 392936,
            "range": "± 11974",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 549456,
            "range": "± 3448",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 294290,
            "range": "± 6988",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 121632,
            "range": "± 920",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 290,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 341288,
            "range": "± 17716",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 48453,
            "range": "± 1454",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 115609,
            "range": "± 2753",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 295861,
            "range": "± 4566",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 449064,
            "range": "± 4575",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 690812,
            "range": "± 4220",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 858719,
            "range": "± 45545",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1081237,
            "range": "± 20254",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 35066,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 21813,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 324864,
            "range": "± 4943",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 252234,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 290749,
            "range": "± 3140",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 322336,
            "range": "± 3894",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 355706,
            "range": "± 2539",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 45782,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12565394,
            "range": "± 41906",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12004427,
            "range": "± 74955",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/short",
            "value": 838,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/medium",
            "value": 2353,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5416,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 194756,
            "range": "± 1611",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18376,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17540,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 18459,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1356462,
            "range": "± 4675",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1085971,
            "range": "± 3890",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11143503,
            "range": "± 100173",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10605171,
            "range": "± 36813",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 114996483,
            "range": "± 1402664",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 109205366,
            "range": "± 1595000",
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
            "value": 6781,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34104,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 138073,
            "range": "± 2957",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 591,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 581,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 575,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 3238,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 11514,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 44289,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13368,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 68331,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 81251098,
            "range": "± 192278",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 5328553,
            "range": "± 20453",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 19489086,
            "range": "± 184147",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1269728,
            "range": "± 25060",
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
          "id": "962655a90f9bb05497fcc54fd78952c87ce19712",
          "message": "feat(security): land Engram 10/10 wave 3 foundations (#191)\n\n* docs(harness): start wave 3 execution\n\n* fix(http): bound proxy and request resources\n\n* ci(security): require aggregate supply-chain gate\n\n* test(mcp): prove canonical memory journey\n\n* test(search): add frozen relevance corpus\n\n* test(examples): smoke public integrations\n\n* docs(harness): record wave 3 integration\n\n* test(mcp): share state across canonical transports\n\n* docs(harness): close wave 3 review findings\n\n* fix(ci): restore Semgrep scanner image\n\n* fix(ci): pin Semgrep-scanned actions",
          "timestamp": "2026-07-13T16:50:17Z",
          "url": "https://github.com/aiconnai/engram/commit/962655a90f9bb05497fcc54fd78952c87ce19712"
        },
        "date": 1786509933740,
        "tool": "cargo",
        "benches": [
          {
            "name": "community_detection/detect_communities_500_nodes",
            "value": 5847745,
            "range": "± 32373",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extractor_new/default",
            "value": 3739,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "entity_extraction/extract_mixed",
            "value": 381742,
            "range": "± 1896",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_create/memory_create",
            "value": 333596,
            "range": "± 12976",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_search/memory_search",
            "value": 516717,
            "range": "± 4010",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_list/memory_list",
            "value": 283797,
            "range": "± 4038",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_memory_stats/memory_stats",
            "value": 101084,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "mcp_dispatch_error_path/unknown_tool",
            "value": 287,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "memory_create/no_embedding",
            "value": 284052,
            "range": "± 12985",
            "unit": "ns/iter"
          },
          {
            "name": "memory_get/by_id",
            "value": 39472,
            "range": "± 859",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/10",
            "value": 112894,
            "range": "± 639",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/10",
            "value": 265282,
            "range": "± 2911",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/50",
            "value": 446729,
            "range": "± 3342",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/50",
            "value": 669471,
            "range": "± 3915",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/limit/100",
            "value": 879116,
            "range": "± 16072",
            "unit": "ns/iter"
          },
          {
            "name": "memory_list/with_tag_filter/100",
            "value": 1057164,
            "range": "± 13360",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/create",
            "value": 32808,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "crossref/get_related",
            "value": 22252,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "get_stats",
            "value": 297751,
            "range": "± 6273",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/authentication",
            "value": 224038,
            "range": "± 4544",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/database migration",
            "value": 257214,
            "range": "± 1773",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/React hooks optimization",
            "value": 287991,
            "range": "± 1416",
            "unit": "ns/iter"
          },
          {
            "name": "bm25_search/query/API rate limiting Redis",
            "value": 319693,
            "range": "± 6191",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/short",
            "value": 35714,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/medium",
            "value": 12765263,
            "range": "± 126174",
            "unit": "ns/iter"
          },
          {
            "name": "hybrid_search/query_type/long",
            "value": 12543732,
            "range": "± 210663",
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
            "value": 2461,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/text_length/long",
            "value": 5595,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "tfidf_embedding/batch_100",
            "value": 203092,
            "range": "± 994",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/1_char_typo",
            "value": 18955,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/2_char_typo",
            "value": 17922,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "fuzzy_search/typo_type/transposition",
            "value": 19424,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/100",
            "value": 1311506,
            "range": "± 7621",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/100",
            "value": 1067367,
            "range": "± 5225",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/1000",
            "value": 11306068,
            "range": "± 253529",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/1000",
            "value": 10763030,
            "range": "± 109163",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/hybrid_memories/10000",
            "value": 121955182,
            "range": "± 1110653",
            "unit": "ns/iter"
          },
          {
            "name": "search_scale/semantic_only_memories/10000",
            "value": 118110075,
            "range": "± 1081317",
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
            "value": 6721,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/500",
            "value": 34862,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/cargo/lines/2000",
            "value": 140285,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/20",
            "value": 547,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/100",
            "value": 543,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/output_filter/git/commits/500",
            "value": 553,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/tight_500/500",
            "value": 2576,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/medium_2k/2000",
            "value": 9753,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/truncation_engine/loose_8k/8000",
            "value": 37229,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/200",
            "value": 13450,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/full_pipeline/cargo_lines/1000",
            "value": 69661,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/semantic_compression/fixed_corpus_ratio_recall",
            "value": 82570085,
            "range": "± 620575",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/50",
            "value": 4852609,
            "range": "± 27288",
            "unit": "ns/iter"
          },
          {
            "name": "token_reduction/consolidation/memories/200",
            "value": 17471580,
            "range": "± 253535",
            "unit": "ns/iter"
          },
          {
            "name": "traversal/bfs_depth_3",
            "value": 1193842,
            "range": "± 26954",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}