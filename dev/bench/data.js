window.BENCHMARK_DATA = {
  "lastUpdate": 1619796287977,
  "repoUrl": "https://github.com/homotopy-io/homotopy-rs",
  "entries": {
    "Rust Benchmark": [
      {
        "commit": {
          "author": {
            "email": "me@nickhu.co.uk",
            "name": "Nick Hu",
            "username": "NickHu"
          },
          "committer": {
            "email": "me@nickhu.co.uk",
            "name": "Nick Hu",
            "username": "NickHu"
          },
          "distinct": true,
          "id": "650b01862819914900ed3585dab2ab5db7fcf494",
          "message": "CI benchmarking fix (again)",
          "timestamp": "2021-04-28T22:36:57+01:00",
          "tree_id": "af9f869bad88f257666d6b0d2957b701f89dc185",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/650b01862819914900ed3585dab2ab5db7fcf494"
        },
        "date": 1619648308720,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 15.461,
            "range": "+/- 0.306",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 15.563,
            "range": "+/- 0.517",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 122.77,
            "range": "+/- 2.640",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 133.89,
            "range": "+/- 2.310",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 128.01,
            "range": "+/- 2.440",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 151.95,
            "range": "+/- 2.320",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 110.82,
            "range": "+/- 1.990",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 741.51,
            "range": "+/- 12.170",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3125,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.343,
            "range": "+/- 0.224",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 58.447,
            "range": "+/- 1.092",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 313.33,
            "range": "+/- 2.480",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2060.5,
            "range": "+/- 11",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 15690,
            "range": "+/- 71",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.9507,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 33.124,
            "range": "+/- 0.610",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nickhu.co.uk",
            "name": "Nick Hu",
            "username": "NickHu"
          },
          "committer": {
            "email": "me@nickhu.co.uk",
            "name": "Nick Hu",
            "username": "NickHu"
          },
          "distinct": true,
          "id": "073f6db11bcc476d1861a070461a1f0408b0197d",
          "message": "Hashconsing for Cones\n\nPreviously, we were not able to hashcons cones because many cones with\nlogically identical content appeared at different places in a diagram,\nas indicated by `cone.index`. The sharable cone fields have been\nfactored out into a ConeInternal type, behind a hashcons smart pointer\nas with the rest of the hashconsed types. This change is expected to be\nbreaking with respect to serialisation format.\n\nCloses #63",
          "timestamp": "2021-04-28T23:31:23+01:00",
          "tree_id": "cff54faae0af03e2e52e650a36a1c5720418899e",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/073f6db11bcc476d1861a070461a1f0408b0197d"
        },
        "date": 1619651662211,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 15.749,
            "range": "+/- 0.277",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 16.567,
            "range": "+/- 0.212",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 119.65,
            "range": "+/- 1.280",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 145.85,
            "range": "+/- 0.820",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 135.47,
            "range": "+/- 1.470",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 169.67,
            "range": "+/- 0.570",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 120.59,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 826.98,
            "range": "+/- 0.940",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.669,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 14.944,
            "range": "+/- 0.085",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 65.629,
            "range": "+/- 0.272",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 350.17,
            "range": "+/- 1.640",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2282.5,
            "range": "+/- 15",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 16896,
            "range": "+/- 142",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.4466,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 34.698,
            "range": "+/- 0.398",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "5b444dbaedb36bf3654ada7debd1109659168de4",
          "message": "Cache rewrite restrictions during type checking.",
          "timestamp": "2021-04-29T00:41:31+02:00",
          "tree_id": "e3aae8eab44330c133584063c99b09de37c21107",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/5b444dbaedb36bf3654ada7debd1109659168de4"
        },
        "date": 1619650679157,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 17.61,
            "range": "+/- 0.352",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 18.155,
            "range": "+/- 0.503",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 147.78,
            "range": "+/- 5.840",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 155.99,
            "range": "+/- 4.860",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 148.01,
            "range": "+/- 2.880",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 173.29,
            "range": "+/- 4.900",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 128.87,
            "range": "+/- 3.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 867.6,
            "range": "+/- 15.060",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.6159,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 12.492,
            "range": "+/- 0.314",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 37.773,
            "range": "+/- 1.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 109.19,
            "range": "+/- 2.580",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 301.93,
            "range": "+/- 3.750",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 817.66,
            "range": "+/- 6.810",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.7828,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 42.057,
            "range": "+/- 1.818",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "dcc5b35717d1625e303c77230d511b4c06b6b0e0",
          "message": "Show state update time in console.",
          "timestamp": "2021-04-29T00:59:55+02:00",
          "tree_id": "ed5f835338d395e21ea2b2d2545f5b58e9aa54c7",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/dcc5b35717d1625e303c77230d511b4c06b6b0e0"
        },
        "date": 1619651337278,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 16.489,
            "range": "+/- 0.342",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 17.432,
            "range": "+/- 0.443",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 119.49,
            "range": "+/- 3.930",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 130.95,
            "range": "+/- 3.720",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 131.8,
            "range": "+/- 7.940",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 155.87,
            "range": "+/- 5.690",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 109.99,
            "range": "+/- 3.100",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 794.03,
            "range": "+/- 71.010",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3096,
            "range": "+/- 0.252",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 10.379,
            "range": "+/- 0.295",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 30.449,
            "range": "+/- 0.614",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 93.492,
            "range": "+/- 3.040",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 272.42,
            "range": "+/- 8.560",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 703.19,
            "range": "+/- 17.300",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 2.9664,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 35.084,
            "range": "+/- 1.868",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "18e936802ae443c19ac572e017286738e24c5236",
          "message": "Slight optimization for contraction.\n\nSwitching to fxhash and making sure that the graph we are running\nTarjan's algorithm on uses integer node indices instead of hashed ones.",
          "timestamp": "2021-04-29T01:36:43+02:00",
          "tree_id": "bca7abf9756e50b2bf462ca8bc0bac14c5bf393f",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/18e936802ae443c19ac572e017286738e24c5236"
        },
        "date": 1619653631467,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 15.189,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 15.475,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 125.88,
            "range": "+/- 0.210",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 147.84,
            "range": "+/- 0.410",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 131.86,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 161.49,
            "range": "+/- 0.440",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 115.09,
            "range": "+/- 0.220",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 760.25,
            "range": "+/- 2.790",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.1957,
            "range": "+/- 0.006",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 10.829,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 33.145,
            "range": "+/- 0.087",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 99.905,
            "range": "+/- 1.494",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 269.23,
            "range": "+/- 0.500",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 729.97,
            "range": "+/- 2.390",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.7431,
            "range": "+/- 0.010",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 38.829,
            "range": "+/- 0.020",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "4d223a20548225e0a51e70bc642fa87ad889e7ba",
          "message": "Avoid unnecessary allocations in rewriting.",
          "timestamp": "2021-04-29T03:18:51+02:00",
          "tree_id": "f6fdd689215cf649ce7c1031bfeff99b339de9ca",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/4d223a20548225e0a51e70bc642fa87ad889e7ba"
        },
        "date": 1619661318081,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 14.435,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 14.859,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 120.13,
            "range": "+/- 0.900",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 142.2,
            "range": "+/- 1.200",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 123.65,
            "range": "+/- 0.940",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 153.94,
            "range": "+/- 1.180",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 109.09,
            "range": "+/- 0.630",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 731.02,
            "range": "+/- 1.650",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.0369,
            "range": "+/- 0.022",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 10.399,
            "range": "+/- 0.063",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 31.802,
            "range": "+/- 0.148",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 92.057,
            "range": "+/- 0.443",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 258,
            "range": "+/- 0.860",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 697.18,
            "range": "+/- 2.100",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.4966,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 37.348,
            "range": "+/- 0.323",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "e54af63b9e2b713c0640e19529a1ef2b76320215",
          "message": "Switched to iterative SCC algorithm due to stack overflow.",
          "timestamp": "2021-04-29T03:38:09+02:00",
          "tree_id": "5362469ad798a8f00ebc69ae409e3d49a9095bda",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/e54af63b9e2b713c0640e19529a1ef2b76320215"
        },
        "date": 1619661650608,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.633,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.005,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 103.19,
            "range": "+/- 0.530",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.45,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 107.13,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 132.39,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 93.744,
            "range": "+/- 0.253",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 625.75,
            "range": "+/- 0.340",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.6324,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 8.9907,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 27.485,
            "range": "+/- 0.014",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 79.431,
            "range": "+/- 0.509",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 221.28,
            "range": "+/- 0.390",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 597.88,
            "range": "+/- 0.440",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.1218,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.752,
            "range": "+/- 0.014",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "21b4ed42fbf13b404f68785391b83b1aef4ecee1",
          "message": "Removed wee_alloc, added hashcons GC.\n\nSee #61 and #65.",
          "timestamp": "2021-04-29T13:34:20+02:00",
          "tree_id": "da715a864d58be61029957a7938890f5a6b7e867",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/21b4ed42fbf13b404f68785391b83b1aef4ecee1"
        },
        "date": 1619696754169,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.929,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.782,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 92.106,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 108.65,
            "range": "+/- 0.410",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 96.4,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 118.42,
            "range": "+/- 0.210",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 84.212,
            "range": "+/- 0.185",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 560.25,
            "range": "+/- 1.290",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.3454,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.9837,
            "range": "+/- 0.012",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 27.855,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 71.322,
            "range": "+/- 0.145",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 213.66,
            "range": "+/- 5.450",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 574.98,
            "range": "+/- 15.110",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 2.6968,
            "range": "+/- 0.007",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 32.051,
            "range": "+/- 0.014",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "d30281470c224b4a8ff75465784e11bfec3a69cf",
          "message": "Avoid printing entire imported workspace.\n\nFixes #66.",
          "timestamp": "2021-04-29T14:07:08+02:00",
          "tree_id": "446d1274b70f0fb434c9bd3b2581b5f0759a8c31",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/d30281470c224b4a8ff75465784e11bfec3a69cf"
        },
        "date": 1619698720521,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 14.013,
            "range": "+/- 0.394",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 14.916,
            "range": "+/- 0.407",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 128.66,
            "range": "+/- 3.260",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.74,
            "range": "+/- 3.150",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 134.04,
            "range": "+/- 2.580",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 153.8,
            "range": "+/- 4.730",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 111.44,
            "range": "+/- 3.150",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 731.14,
            "range": "+/- 17.670",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2189,
            "range": "+/- 0.075",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 11.407,
            "range": "+/- 0.237",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 33.102,
            "range": "+/- 0.574",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 93.927,
            "range": "+/- 1.702",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 260.29,
            "range": "+/- 3.340",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 729.74,
            "range": "+/- 10.930",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.6858,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 39.716,
            "range": "+/- 0.737",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "2d04c8707f6af3ddc953a2556a8ddca3c68e1630",
          "message": "Simplify recursive contraction problems.\n\nThe recursive calls to the colimit computation in the contraction\nprocedure often receive problems with a lot of identity spans; these\nproblems can be simplified which leads to considerable performance\nincreases in some cases.",
          "timestamp": "2021-04-30T16:08:05+02:00",
          "tree_id": "f0b115f71b6cbad6225c8ccf816041199f9f16c1",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/2d04c8707f6af3ddc953a2556a8ddca3c68e1630"
        },
        "date": 1619792522620,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 14.953,
            "range": "+/- 0.201",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 15.62,
            "range": "+/- 0.432",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 132.9,
            "range": "+/- 4.260",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 146.09,
            "range": "+/- 3.770",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 133.04,
            "range": "+/- 3.470",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 150.84,
            "range": "+/- 2.830",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 111.52,
            "range": "+/- 2.200",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 797.62,
            "range": "+/- 12.500",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.267,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 11.13,
            "range": "+/- 0.188",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 32.789,
            "range": "+/- 0.883",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 94.055,
            "range": "+/- 1.810",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 263.12,
            "range": "+/- 3.300",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 693.84,
            "range": "+/- 6.170",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.4699,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 36.67,
            "range": "+/- 0.557",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "committer": {
            "email": "lukas@heidemann.me",
            "name": "Lukas Heidemann",
            "username": "zrho"
          },
          "distinct": true,
          "id": "f3306f41db18828ff7ef0ca52d3f20786297c351",
          "message": "Added ordering instances to diagrams and rewrites.\n\nThe ordering does not have any semantic meaning but could be used to\ncanonicalise subproblems through sorting. The ordering is determined by\nthe hash consing id and is not meant to be stable across runs.",
          "timestamp": "2021-04-30T17:07:23+02:00",
          "tree_id": "d90252505c09a4c504c720a98ad9638d3ccdb501",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f3306f41db18828ff7ef0ca52d3f20786297c351"
        },
        "date": 1619796274298,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 16.727,
            "range": "+/- 0.368",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 17.457,
            "range": "+/- 0.436",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 134.89,
            "range": "+/- 1.830",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 152.7,
            "range": "+/- 2.290",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 138.14,
            "range": "+/- 2.350",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 166.35,
            "range": "+/- 2.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 121.81,
            "range": "+/- 1.830",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 821.94,
            "range": "+/- 18.120",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2844,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 10.962,
            "range": "+/- 0.176",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 33.314,
            "range": "+/- 0.300",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 96.337,
            "range": "+/- 0.774",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 270.04,
            "range": "+/- 1.500",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 731.63,
            "range": "+/- 4.390",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.6988,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 39.096,
            "range": "+/- 0.416",
            "unit": "us"
          }
        ]
      }
    ]
  }
}