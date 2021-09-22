window.BENCHMARK_DATA = {
  "lastUpdate": 1632347726041,
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
          "id": "318ed8897d0f4f2614db93d290d2c7f046067050",
          "message": "Reversed direction of vertical arrow keys for navigation.\n\nFixes #54.",
          "timestamp": "2021-04-30T17:50:06+02:00",
          "tree_id": "436532d7d8668349f39632ac6d7e0cde99695d85",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/318ed8897d0f4f2614db93d290d2c7f046067050"
        },
        "date": 1619798548645,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 16.138,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 16.609,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 126.28,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.43,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 130.83,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 162.51,
            "range": "+/- 0.280",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 115.03,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 804.37,
            "range": "+/- 9.010",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.1623,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 11.181,
            "range": "+/- 0.244",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 32.376,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 93.649,
            "range": "+/- 0.129",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 262.36,
            "range": "+/- 0.320",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 707.16,
            "range": "+/- 3.240",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 3.6831,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 39.477,
            "range": "+/- 0.696",
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
          "id": "5ee6f428dc2a3e8e5d628334ead6a10d0677315e",
          "message": "Fixed #69.",
          "timestamp": "2021-05-02T13:54:31+02:00",
          "tree_id": "7308050f91fc59c32e20d6d35fc7f9dce137a8d7",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/5ee6f428dc2a3e8e5d628334ead6a10d0677315e"
        },
        "date": 1619957135556,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 19.671,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 20.347,
            "range": "+/- 0.141",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 132.02,
            "range": "+/- 0.740",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 153.16,
            "range": "+/- 0.810",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 137.99,
            "range": "+/- 0.810",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 167.89,
            "range": "+/- 0.770",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 116.9,
            "range": "+/- 0.750",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 782.26,
            "range": "+/- 4.880",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.193,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 10.642,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 32.155,
            "range": "+/- 0.145",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 92.517,
            "range": "+/- 0.441",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 259.96,
            "range": "+/- 1.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 696.82,
            "range": "+/- 2.120",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.2684,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 41.861,
            "range": "+/- 0.253",
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
          "id": "f6451b2f3f4609bf8667c3d55582be52f89433d2",
          "message": "Performance improvements in normalization.\n\nUsing the cached normalizations we can check if a diagram is normalised,\nwhich we can use to check if an arrow in the sink is an identity in full\nnormalisation. This way we can short-circuit when there is an identity\narrow in the sink for full normalisation as well.",
          "timestamp": "2021-05-02T14:30:20+02:00",
          "tree_id": "cf50967e52fd59f865c23b7790d63a9b1fb06e78",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f6451b2f3f4609bf8667c3d55582be52f89433d2"
        },
        "date": 1619959350176,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 13.205,
            "range": "+/- 0.452",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.589,
            "range": "+/- 0.453",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 110.99,
            "range": "+/- 3.260",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 125.48,
            "range": "+/- 2.780",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 116.1,
            "range": "+/- 4.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 126.32,
            "range": "+/- 3.750",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 89.295,
            "range": "+/- 2.953",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 560.44,
            "range": "+/- 9.760",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.2953,
            "range": "+/- 0.041",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.7083,
            "range": "+/- 0.163",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 23.247,
            "range": "+/- 0.355",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 64.173,
            "range": "+/- 0.866",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 196.07,
            "range": "+/- 4.980",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 522.99,
            "range": "+/- 10.100",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.8284,
            "range": "+/- 0.149",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 34.489,
            "range": "+/- 0.573",
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
          "id": "820cb7b724d90fdf52e66e1174c849fb42d14c2f",
          "message": "More performance for normalization.\n\nIt turns out we can even cache normalization results when the sink is\nnot empty.",
          "timestamp": "2021-05-02T14:49:02+02:00",
          "tree_id": "76739ccb099afaf5f158ea82442d116a987e8d09",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/820cb7b724d90fdf52e66e1174c849fb42d14c2f"
        },
        "date": 1619960355768,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.354,
            "range": "+/- 0.155",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 12.712,
            "range": "+/- 0.149",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 96.44,
            "range": "+/- 0.951",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 115.26,
            "range": "+/- 1.340",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 95.92,
            "range": "+/- 1.105",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 131.61,
            "range": "+/- 1.480",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 84.245,
            "range": "+/- 0.947",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 559.46,
            "range": "+/- 6.960",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.2434,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.8652,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.106,
            "range": "+/- 0.202",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 44.45,
            "range": "+/- 0.412",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 104.54,
            "range": "+/- 0.860",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 240.12,
            "range": "+/- 1.680",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.8312,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 37.277,
            "range": "+/- 0.478",
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
          "id": "144fde7a49e5bd309350d6ae309f5cd0014b0da4",
          "message": "Homotopies on touch devices.",
          "timestamp": "2021-05-02T16:35:57+02:00",
          "tree_id": "391c69cf3f49eaec8fd7f847d0ad1f4028f09e92",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/144fde7a49e5bd309350d6ae309f5cd0014b0da4"
        },
        "date": 1619966771040,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 13.523,
            "range": "+/- 0.318",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.775,
            "range": "+/- 0.308",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 111.33,
            "range": "+/- 3.280",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 127.38,
            "range": "+/- 4.100",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 107.05,
            "range": "+/- 2.730",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 142.59,
            "range": "+/- 5.700",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 98.737,
            "range": "+/- 3.925",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 644.11,
            "range": "+/- 18.550",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.6816,
            "range": "+/- 0.083",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 8.1219,
            "range": "+/- 0.231",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 20.345,
            "range": "+/- 0.461",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 48.868,
            "range": "+/- 0.640",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 120,
            "range": "+/- 4.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 301.18,
            "range": "+/- 9.400",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.9743,
            "range": "+/- 0.572",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 38.852,
            "range": "+/- 0.531",
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
          "id": "26e9e82fe83d64a469f670e86e8523dff1d44c1b",
          "message": "Generalised slice controls (closes #58).",
          "timestamp": "2021-05-03T17:59:22+02:00",
          "tree_id": "1f3e925c478dd7efb146a14f1207d906c534b323",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/26e9e82fe83d64a469f670e86e8523dff1d44c1b"
        },
        "date": 1620058660449,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 14.101,
            "range": "+/- 0.274",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 14.311,
            "range": "+/- 0.335",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 112.26,
            "range": "+/- 1.930",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 129.4,
            "range": "+/- 2.670",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 108.29,
            "range": "+/- 1.650",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 144.47,
            "range": "+/- 2.450",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 97.402,
            "range": "+/- 1.994",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 655.92,
            "range": "+/- 11.410",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.5548,
            "range": "+/- 0.050",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.8473,
            "range": "+/- 0.176",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 20.724,
            "range": "+/- 0.360",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 51.716,
            "range": "+/- 1.290",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 121.87,
            "range": "+/- 2.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 279.68,
            "range": "+/- 3.020",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.5002,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 40.858,
            "range": "+/- 0.885",
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
          "id": "4b56ae3910fbcb30edc09998073190fa4d6762aa",
          "message": "Fixed panic in expansion code.\n\nThere was an assertion in the expansion code which I suspected would\nalways hold since the JS version seemed to assume that. In fact this\nassertion can be broken, so we can just return an error in this case to\nprevent a crash.",
          "timestamp": "2021-05-06T23:59:58+02:00",
          "tree_id": "d4ab2f9f25f6130cc5c34fb80031d297785b6ddf",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/4b56ae3910fbcb30edc09998073190fa4d6762aa"
        },
        "date": 1620339839967,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.974,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.054,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 103.88,
            "range": "+/- 1.490",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 118.01,
            "range": "+/- 0.660",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 99.706,
            "range": "+/- 0.476",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 134.06,
            "range": "+/- 0.870",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 92.946,
            "range": "+/- 0.558",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 604.11,
            "range": "+/- 10.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.3595,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.1796,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 19.114,
            "range": "+/- 0.170",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 47.164,
            "range": "+/- 0.348",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 110.72,
            "range": "+/- 0.860",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 252.74,
            "range": "+/- 1.080",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.074,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 37.037,
            "range": "+/- 0.298",
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
          "id": "7d08c1f0c861660887d10bcf127df37981fd2ab3",
          "message": "Updated serialization format.\n\nThe hashes that identify diagrams and their parts are now based on the\ndata and not on the id which incidentally has been assigned by the\nhash-consing implementation. Together with sorting this should guarantee\nthat the same diagram serializes to exactly the same string. The\nintegers in those structures that are addressed by their hash are made\nto be u32 instead of usize so that their hash is constant among\narchitectures.\n\nChanged to (gzipped) JSON from messagepack.  JSON is human-readable, so\ndebugging and exploring the file format is simpler.  This already\nuncovered a bug that left half of the 128 bit keys zero, which would not\nhave been clear at all in messagepack.",
          "timestamp": "2021-05-16T19:38:49+02:00",
          "tree_id": "462d3145e03e8e64e7458e77488e27c384933dc2",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/7d08c1f0c861660887d10bcf127df37981fd2ab3"
        },
        "date": 1621377718127,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 13.409,
            "range": "+/- 0.640",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.179,
            "range": "+/- 0.235",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 105.16,
            "range": "+/- 1.570",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 119.01,
            "range": "+/- 1.230",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 101.26,
            "range": "+/- 1.620",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 135.72,
            "range": "+/- 1.230",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 94.611,
            "range": "+/- 2.173",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 608.86,
            "range": "+/- 8.650",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.356,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.2536,
            "range": "+/- 0.127",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 19.297,
            "range": "+/- 0.279",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 47.946,
            "range": "+/- 0.840",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 112.23,
            "range": "+/- 0.840",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 261.95,
            "range": "+/- 2.450",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.1362,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 38.038,
            "range": "+/- 0.457",
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
          "id": "79cee37e35974c99bfe40d63e2a2143f8fd39508",
          "message": "More work on the serialization.\n\nSwitched back to msgpack to avoid non-determinism in compression.\n\nWe now make sure to use u32 to hash the length of vectors to avoid\ndifferences across architectures.\n\nKeys are now `[u64;2]` instead of `u128`.",
          "timestamp": "2021-05-19T20:01:14+02:00",
          "tree_id": "5c55682b11debd747fca6bfbe1dd30487428f857",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/79cee37e35974c99bfe40d63e2a2143f8fd39508"
        },
        "date": 1621448625038,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 13.002,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.203,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 100.24,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 117.47,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 96.39,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 134.25,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 86.893,
            "range": "+/- 0.119",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 581.58,
            "range": "+/- 0.940",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.2913,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.0098,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.694,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 46.214,
            "range": "+/- 0.116",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 109.18,
            "range": "+/- 0.390",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 255.22,
            "range": "+/- 0.910",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.1445,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 37.662,
            "range": "+/- 0.075",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e3874094f458cc914769b461d0f85a6627a508eb",
          "message": "Merge pull request #92 from doctorn/smallrefactor\n\nRefactor sidebar buttons",
          "timestamp": "2021-06-15T18:23:18+02:00",
          "tree_id": "4e90e69deab33a414c7ff32ab1cb5375daa10937",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/e3874094f458cc914769b461d0f85a6627a508eb"
        },
        "date": 1623775337399,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.343,
            "range": "+/- 0.175",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.721,
            "range": "+/- 0.195",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 93.417,
            "range": "+/- 1.710",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 105.28,
            "range": "+/- 2.190",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 89.526,
            "range": "+/- 0.970",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 119.68,
            "range": "+/- 1.940",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 80.625,
            "range": "+/- 0.685",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 537.04,
            "range": "+/- 6.360",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1074,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.5177,
            "range": "+/- 0.127",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.027,
            "range": "+/- 0.135",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 42.399,
            "range": "+/- 0.467",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 99.912,
            "range": "+/- 1.378",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 229.24,
            "range": "+/- 2.050",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.2725,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 33.649,
            "range": "+/- 0.446",
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
          "id": "edf28534d7486f93e6cb5bdd262767f81ee2179f",
          "message": "fix path control typo, change hamburger icon to star",
          "timestamp": "2021-06-21T17:12:59+01:00",
          "tree_id": "60d37bd7de55eda1264347af8361a6b84e407fc7",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/edf28534d7486f93e6cb5bdd262767f81ee2179f"
        },
        "date": 1624292905301,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.935,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.117,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 99.078,
            "range": "+/- 0.478",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 117.29,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 95.496,
            "range": "+/- 0.189",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 133.32,
            "range": "+/- 0.190",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 86,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 580.32,
            "range": "+/- 0.860",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.279,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.9823,
            "range": "+/- 0.009",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.675,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 45.987,
            "range": "+/- 0.117",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 108.51,
            "range": "+/- 0.170",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 250.47,
            "range": "+/- 0.440",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.1373,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 37.339,
            "range": "+/- 0.095",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "894ed8711f255a729e648426fe723c5b370b0aa1",
          "message": "Merge pull request #94 from doctorn/toasts\n\nExtract toaster component",
          "timestamp": "2021-06-21T17:46:07+01:00",
          "tree_id": "168899d599c64fcddfe2bd951f726db04b67adf0",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/894ed8711f255a729e648426fe723c5b370b0aa1"
        },
        "date": 1624294525171,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 13.268,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 13.323,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 98.753,
            "range": "+/- 0.131",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 117.17,
            "range": "+/- 1.450",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 97.228,
            "range": "+/- 1.684",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 134.32,
            "range": "+/- 1.630",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 84.816,
            "range": "+/- 0.489",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 574.63,
            "range": "+/- 4.020",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.2735,
            "range": "+/- 0.012",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.9551,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.783,
            "range": "+/- 0.165",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 45.703,
            "range": "+/- 0.117",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 107.07,
            "range": "+/- 0.370",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 248.61,
            "range": "+/- 0.750",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.0044,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 36.685,
            "range": "+/- 0.168",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ceb633bf06c778afe7c5cae53a6a7b9c07bdc203",
          "message": "Merge pull request #98 from doctorn/settings\n\nAdd settings subsystem",
          "timestamp": "2021-06-21T17:57:13+01:00",
          "tree_id": "686a63591eb8b0451727f78c3396a5dc894bed57",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/ceb633bf06c778afe7c5cae53a6a7b9c07bdc203"
        },
        "date": 1624295196996,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.362,
            "range": "+/- 0.133",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 12.766,
            "range": "+/- 0.137",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 95.503,
            "range": "+/- 0.988",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 114.15,
            "range": "+/- 1.290",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 92.426,
            "range": "+/- 1.076",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 127.8,
            "range": "+/- 1.240",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 82.826,
            "range": "+/- 1.248",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 561.11,
            "range": "+/- 5.860",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1887,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.6976,
            "range": "+/- 0.065",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.169,
            "range": "+/- 0.119",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 44.252,
            "range": "+/- 0.328",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 103.47,
            "range": "+/- 0.800",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 240.28,
            "range": "+/- 1.920",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.7958,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 35.729,
            "range": "+/- 0.368",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b70b0d3045e2f3fc3be39ef65bd25b740cf166b4",
          "message": "Add build status to README.md",
          "timestamp": "2021-06-22T10:30:19+01:00",
          "tree_id": "0bc9da13843d07c39d66a21ae7d18ef3d3cac05b",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/b70b0d3045e2f3fc3be39ef65bd25b740cf166b4"
        },
        "date": 1624354756655,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.828,
            "range": "+/- 0.007",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.035,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.112,
            "range": "+/- 0.398",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.713,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.383,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.9,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.179,
            "range": "+/- 0.226",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 487.31,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9068,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8593,
            "range": "+/- 0.005",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.696,
            "range": "+/- 0.009",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.806,
            "range": "+/- 0.145",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 93.54,
            "range": "+/- 0.293",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 220.19,
            "range": "+/- 0.630",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.9476,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.234,
            "range": "+/- 0.024",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d642a455673807d116b97f01502359bedce90f75",
          "message": "[CI] Use stable `clippy` (#102)",
          "timestamp": "2021-06-24T16:53:32+01:00",
          "tree_id": "4b9258ed9a80c505cb4b20f759ed5edcffb1c942",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/d642a455673807d116b97f01502359bedce90f75"
        },
        "date": 1624550545624,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.867,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.048,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.098,
            "range": "+/- 0.495",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 97.987,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.076,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.44,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.713,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 483.02,
            "range": "+/- 0.230",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.912,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.849,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.622,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.494,
            "range": "+/- 0.136",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 91.496,
            "range": "+/- 0.331",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 212.05,
            "range": "+/- 0.890",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.9191,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.087,
            "range": "+/- 0.020",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5c46dc47febee0c9cae60cdcbf1a60980f27783a",
          "message": "[CI] Hack deploy (#103)",
          "timestamp": "2021-06-24T17:19:15+01:00",
          "tree_id": "9c0401524c6f5cdc851d8dde9942f3bd2b14ebc8",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/5c46dc47febee0c9cae60cdcbf1a60980f27783a"
        },
        "date": 1624552171501,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 11.547,
            "range": "+/- 0.238",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.438,
            "range": "+/- 0.346",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 87.911,
            "range": "+/- 2.680",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 109.07,
            "range": "+/- 2.540",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 88.608,
            "range": "+/- 2.123",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 117.3,
            "range": "+/- 2.720",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 80.084,
            "range": "+/- 1.189",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 537.41,
            "range": "+/- 12.140",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.0788,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.1428,
            "range": "+/- 0.135",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.773,
            "range": "+/- 0.208",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 40.688,
            "range": "+/- 0.974",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 98.093,
            "range": "+/- 2.327",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 230.09,
            "range": "+/- 4.190",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.8299,
            "range": "+/- 0.156",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.702,
            "range": "+/- 1.315",
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
          "id": "e57989ee6f81426242b6560454f4e354e00484b1",
          "message": "[CI] Tidy up steps",
          "timestamp": "2021-06-24T18:06:39+01:00",
          "tree_id": "9282c7ac49acca9446ccab562af8ed0fabfcf765",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/e57989ee6f81426242b6560454f4e354e00484b1"
        },
        "date": 1624554962759,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.531,
            "range": "+/- 0.325",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 10.321,
            "range": "+/- 0.204",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 87.058,
            "range": "+/- 3.334",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 113.01,
            "range": "+/- 4.450",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 88.005,
            "range": "+/- 3.515",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 106.87,
            "range": "+/- 2.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 69.93,
            "range": "+/- 2.131",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 560.73,
            "range": "+/- 4.390",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.4251,
            "range": "+/- 0.083",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.8476,
            "range": "+/- 0.081",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 16.871,
            "range": "+/- 0.381",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 40.642,
            "range": "+/- 0.568",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 96.058,
            "range": "+/- 2.786",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 218.36,
            "range": "+/- 6.850",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.987,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 33.753,
            "range": "+/- 1.535",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "257dea587177592cf7960bd7a08376db143f0a18",
          "message": "Extract sidebar from `App` (#101)\n\n* Make settings drawer track global settings state\r\n\r\n* Remove redundant dispatcher\r\n\r\n* Add `IdxVec`\r\n\r\n* Reimplement history tree\r\n\r\n* Extract sidebar (I)\r\n\r\n* Extract sidebar (II)\r\n\r\nFix signature drawer open\r\n\r\n* Make `RawHtml` a component\r\n\r\n* Fix undo and redo visibility\r\n\r\n* `cargo clippy --fix`\r\n\r\n* Placate clippy\r\n\r\n* Fix nits",
          "timestamp": "2021-06-25T12:59:30+01:00",
          "tree_id": "16715c9115ab908547d1f446f6abfbe4fcff9616",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/257dea587177592cf7960bd7a08376db143f0a18"
        },
        "date": 1624622889872,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.935,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.03,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 82.544,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 97.835,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.341,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.79,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.644,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 427.3,
            "range": "+/- 0.210",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.8987,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8225,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.577,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.433,
            "range": "+/- 0.064",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 91.201,
            "range": "+/- 0.145",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 211.45,
            "range": "+/- 0.320",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.3543,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 27.507,
            "range": "+/- 0.012",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "akvile.val1206@gmail.com",
            "name": "Akvilė Valentukonytė",
            "username": "Akvile1206"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "24ead89ef18056d38b0ec72911f848897ff96853",
          "message": "Merge pull request #106 from homotopy-io/color\n\nAdded a simple color picker",
          "timestamp": "2021-07-01T18:06:49+01:00",
          "tree_id": "aa97a372690c0ec5b4141c62a17899b13179accf",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/24ead89ef18056d38b0ec72911f848897ff96853"
        },
        "date": 1625159717954,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.881,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.085,
            "range": "+/- 0.011",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 82.86,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 97.797,
            "range": "+/- 0.052",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.282,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.6,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 71.832,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 484.03,
            "range": "+/- 0.250",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9085,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8438,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.61,
            "range": "+/- 0.005",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.454,
            "range": "+/- 0.144",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 91.863,
            "range": "+/- 0.135",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 213.43,
            "range": "+/- 0.750",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.0011,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.229,
            "range": "+/- 0.015",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "91dc3c8590296ea6121e715ecee9dbdf26c86d7f",
          "message": "Fix #42 & #105 (#111)\n\n* Make enter complete rename\r\n\r\n* Remove unused variables\r\n\r\n* Make sidebar manage shortcuts\r\n\r\n* `cargo fmt`\r\n\r\n* Placate `clippy`",
          "timestamp": "2021-07-06T19:38:29+01:00",
          "tree_id": "a1cfaf9f375dd8278ad0166b59d46ea76e90c22b",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/91dc3c8590296ea6121e715ecee9dbdf26c86d7f"
        },
        "date": 1625597209796,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.872,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.038,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 82.707,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.301,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.219,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.78,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.075,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 484.84,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9145,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8383,
            "range": "+/- 0.005",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.628,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.508,
            "range": "+/- 0.063",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 91.099,
            "range": "+/- 0.283",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 212.64,
            "range": "+/- 0.900",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.9835,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.154,
            "range": "+/- 0.012",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "387cde40a2bf8e15e4e9f41840281d8bc35c07f2",
          "message": "Fix #108 (#115)\n\n* Fix singular expansion crash\r\n\r\n* `cargo fmt`",
          "timestamp": "2021-07-07T15:08:07+01:00",
          "tree_id": "a28390031a00771411251e55da610f8317d849a5",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/387cde40a2bf8e15e4e9f41840281d8bc35c07f2"
        },
        "date": 1625667368166,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 9.711,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.102,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.251,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 86.612,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 70.824,
            "range": "+/- 0.180",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 98.296,
            "range": "+/- 0.205",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 64.002,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 427.29,
            "range": "+/- 0.390",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.6865,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.1713,
            "range": "+/- 0.008",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 13.771,
            "range": "+/- 0.018",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 33.84,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 79.955,
            "range": "+/- 0.061",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 187.53,
            "range": "+/- 0.670",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.2258,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 27.36,
            "range": "+/- 0.012",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0025095754a20ab77de85e743548e8b751965bec",
          "message": "Add page leave warning (#116)",
          "timestamp": "2021-07-07T16:32:11+01:00",
          "tree_id": "29b54f8d3576d3258f408c2b4265db5e0f7b900b",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/0025095754a20ab77de85e743548e8b751965bec"
        },
        "date": 1625672916625,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.321,
            "range": "+/- 0.318",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 12.611,
            "range": "+/- 0.214",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 93.923,
            "range": "+/- 1.016",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 110.34,
            "range": "+/- 2.070",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 94.538,
            "range": "+/- 2.837",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 124.62,
            "range": "+/- 1.640",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 82.52,
            "range": "+/- 0.794",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 539.12,
            "range": "+/- 7.960",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1221,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.4976,
            "range": "+/- 0.079",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.364,
            "range": "+/- 0.201",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 42.753,
            "range": "+/- 0.521",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 101.22,
            "range": "+/- 1.220",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 235.64,
            "range": "+/- 2.360",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.7111,
            "range": "+/- 0.086",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 34.929,
            "range": "+/- 0.418",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0025095754a20ab77de85e743548e8b751965bec",
          "message": "Add page leave warning (#116)",
          "timestamp": "2021-07-07T16:32:11+01:00",
          "tree_id": "29b54f8d3576d3258f408c2b4265db5e0f7b900b",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/0025095754a20ab77de85e743548e8b751965bec"
        },
        "date": 1625676102147,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 9.7016,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 9.8348,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 73.651,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.716,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 81.077,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 112.48,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 64.301,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 428.28,
            "range": "+/- 0.180",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.6959,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.1758,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 13.814,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 34.272,
            "range": "+/- 0.074",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 81.982,
            "range": "+/- 0.282",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 194.85,
            "range": "+/- 0.690",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.3451,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 30.967,
            "range": "+/- 0.015",
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
          "id": "9b6e29280cd48fe209ddede273022e0155eaba73",
          "message": "add missing factorization failure case; fixes #117",
          "timestamp": "2021-07-09T13:51:02+01:00",
          "tree_id": "cd80df853210eec376002643ae7cb29d4e25d5e5",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/9b6e29280cd48fe209ddede273022e0155eaba73"
        },
        "date": 1625835705279,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.854,
            "range": "+/- 0.007",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.015,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.411,
            "range": "+/- 0.288",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.6,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.277,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.57,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 73.031,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 484.63,
            "range": "+/- 0.300",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9122,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.865,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.602,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.453,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 91.852,
            "range": "+/- 0.272",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 216.85,
            "range": "+/- 3.420",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.0414,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.401,
            "range": "+/- 0.018",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "146dfbc5d2614d566514b06bcb95a05bd3992c5d",
          "message": "Add extensible data to rewrites (#124)\n\n* Add labels to rewrites\r\n\r\n* Rename `LabelledX` to `GenericX`\r\n\r\n* Rename `LabelledX` to `GenericX`\r\n\r\n* Placate `clippy`\r\n\r\n* Add getter for payloads",
          "timestamp": "2021-07-26T12:39:17+01:00",
          "tree_id": "5b1866e1a12e1d51e3d77907724be3aa2e3cbc53",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/146dfbc5d2614d566514b06bcb95a05bd3992c5d"
        },
        "date": 1627300810001,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 11.688,
            "range": "+/- 0.270",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 11.469,
            "range": "+/- 0.254",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 88.777,
            "range": "+/- 1.692",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 99.942,
            "range": "+/- 2.325",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 78.045,
            "range": "+/- 1.614",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 119.55,
            "range": "+/- 3.100",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 78.382,
            "range": "+/- 2.163",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 548.85,
            "range": "+/- 6.750",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1479,
            "range": "+/- 0.035",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.5024,
            "range": "+/- 0.068",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.113,
            "range": "+/- 0.289",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 42.851,
            "range": "+/- 0.423",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 99.111,
            "range": "+/- 1.267",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 230.96,
            "range": "+/- 3.500",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.8721,
            "range": "+/- 0.069",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 36.816,
            "range": "+/- 0.121",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8c2cd87bfb27194c4e5bb7a933ef72c026e983c0",
          "message": "Add a variety of panzoom improvements (#131)\n\n* Add reset command\r\n\r\n* Add a variety of panzoom improvements\r\n\r\n* Placate clippy",
          "timestamp": "2021-07-27T15:33:18+01:00",
          "tree_id": "4a6abe0ae3ffe5cb985605852b7287a38e3731a4",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/8c2cd87bfb27194c4e5bb7a933ef72c026e983c0"
        },
        "date": 1627396898275,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.664,
            "range": "+/- 0.011",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 10.83,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.057,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 97.697,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 79.711,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 110.65,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.393,
            "range": "+/- 0.167",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 483.12,
            "range": "+/- 0.440",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.8997,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.807,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.449,
            "range": "+/- 0.009",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.307,
            "range": "+/- 0.107",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 91.63,
            "range": "+/- 0.154",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 215.18,
            "range": "+/- 0.400",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.8794,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 30.729,
            "range": "+/- 0.024",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f33ba838b594ee3e1d5fa6d9fb8012a29ec04163",
          "message": "Hotfix zoom logic (#133)",
          "timestamp": "2021-07-27T15:54:29+01:00",
          "tree_id": "b2745bf9738baffde9ad0c506521c8743ceb3c2c",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f33ba838b594ee3e1d5fa6d9fb8012a29ec04163"
        },
        "date": 1627398168032,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 9.3886,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 9.5421,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 72.607,
            "range": "+/- 0.222",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 86.182,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 70.107,
            "range": "+/- 0.032",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 97.503,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 63.796,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 422.98,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.658,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.0849,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 13.552,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 33.49,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 79.293,
            "range": "+/- 0.171",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 185.75,
            "range": "+/- 0.450",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.2511,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 27.243,
            "range": "+/- 0.014",
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
          "id": "911236469a82c6ea70c96667f3c702050516a875",
          "message": "Generalise `MonotoneSequences` to allow for constraints in each digit.\nThis allows the direct generation of *composable* underlying monotone\nfunctions in factorization, as opposed to iterating through all valid\nmonotone sequences (which are vastly more numerous).\nFixes #123.",
          "timestamp": "2021-07-27T21:10:29+01:00",
          "tree_id": "5b41a1ce23fc1ea90528bb3566ee3ed06ca06a24",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/911236469a82c6ea70c96667f3c702050516a875"
        },
        "date": 1627417220735,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.761,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 10.978,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 84.01,
            "range": "+/- 0.437",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.394,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 80.198,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 111.18,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.474,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 481.72,
            "range": "+/- 0.210",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.8905,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.773,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.456,
            "range": "+/- 0.007",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 38.639,
            "range": "+/- 0.089",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 93.61,
            "range": "+/- 0.432",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 219.44,
            "range": "+/- 0.490",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.056,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.18,
            "range": "+/- 0.024",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "99070f125d1f6c9de64210d6802bf92fd58b457d",
          "message": "Add flexible state management solution (#140)\n\n* Add state management solution\r\n\r\n* Fix zooming\r\n\r\n* Placate `clippy`\r\n\r\n* Placate `clippy`",
          "timestamp": "2021-07-30T15:27:14+01:00",
          "tree_id": "9c44f18032a6a6968c87619ea95e43c3c2f62136",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/99070f125d1f6c9de64210d6802bf92fd58b457d"
        },
        "date": 1627656457864,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.293,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 12.606,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 96.491,
            "range": "+/- 0.455",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 115.53,
            "range": "+/- 0.510",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 93.99,
            "range": "+/- 0.193",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 130.36,
            "range": "+/- 0.690",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 85.248,
            "range": "+/- 0.385",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 572.27,
            "range": "+/- 0.770",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.253,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.8861,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.323,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 45.169,
            "range": "+/- 0.067",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 107.51,
            "range": "+/- 0.320",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 248.4,
            "range": "+/- 0.560",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 7.089,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 37.181,
            "range": "+/- 0.048",
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
          "id": "f3fed4b96361d733bfe5e0754d666f74c3763857",
          "message": "migrate from wasm-pack+npm+webpack to trunk",
          "timestamp": "2021-07-30T18:14:14+01:00",
          "tree_id": "2f4efe68c3b504d2d2f062e3b477e5e06ac97b01",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f3fed4b96361d733bfe5e0754d666f74c3763857"
        },
        "date": 1627666259141,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 9.4375,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 9.6361,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 72.739,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 86.281,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 70.397,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 97.775,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 63.657,
            "range": "+/- 0.114",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 424.09,
            "range": "+/- 1.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.6686,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.1024,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.427,
            "range": "+/- 0.007",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 37.877,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 78.766,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 187.84,
            "range": "+/- 0.230",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.2379,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 27.524,
            "range": "+/- 0.014",
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
          "id": "fe71f312348d37d4fc1f71f72da313e8d6f25283",
          "message": "Update build instructions, and make sure CI buils in release mode",
          "timestamp": "2021-08-02T18:45:12+01:00",
          "tree_id": "d900a69755b057285ff9aea784ef70cd66340bae",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/fe71f312348d37d4fc1f71f72da313e8d6f25283"
        },
        "date": 1627926979224,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 9.3481,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 9.502,
            "range": "+/- 0.011",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 72.906,
            "range": "+/- 0.386",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 87.095,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 70.601,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 110.69,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.031,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 426.5,
            "range": "+/- 1.010",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.6721,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.1205,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 13.658,
            "range": "+/- 0.005",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 33.768,
            "range": "+/- 0.092",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 80.381,
            "range": "+/- 0.164",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 211.47,
            "range": "+/- 0.620",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 5.9571,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.185,
            "range": "+/- 0.014",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8dc6ca7f1ca3bcfa0d17af6889e8aa21bfca6401",
          "message": "Update chevrons (#142)\n\n* Merge master\r\n\r\n* Update chevrons",
          "timestamp": "2021-08-04T17:14:16+01:00",
          "tree_id": "7a3fe77ad138d49f6294757469a26111ac686070",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/8dc6ca7f1ca3bcfa0d17af6889e8aa21bfca6401"
        },
        "date": 1628094270574,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.215,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 12.215,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 93.139,
            "range": "+/- 0.805",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 112.78,
            "range": "+/- 0.950",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 90.754,
            "range": "+/- 0.652",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 129.32,
            "range": "+/- 1.010",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 81.9,
            "range": "+/- 0.838",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 553.68,
            "range": "+/- 5.450",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1506,
            "range": "+/- 0.018",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.5819,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.59,
            "range": "+/- 0.152",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 43.419,
            "range": "+/- 0.319",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 102.21,
            "range": "+/- 0.720",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 238.46,
            "range": "+/- 1.190",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.8457,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 35.931,
            "range": "+/- 0.289",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "339f3e0f21434143ac65766446fce73c6f4b41f5",
          "message": "Pipe errors through to interface (#144)",
          "timestamp": "2021-08-05T16:56:29+01:00",
          "tree_id": "f43aff83630e593e5c3cfbc9a6cf3c63a690ce2d",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/339f3e0f21434143ac65766446fce73c6f4b41f5"
        },
        "date": 1628179670737,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 10.663,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 10.799,
            "range": "+/- 0.011",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.844,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.121,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 79.724,
            "range": "+/- 0.032",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 110.93,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.349,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 479.86,
            "range": "+/- 0.220",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.8852,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.7421,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.396,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 37.89,
            "range": "+/- 0.030",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 89.558,
            "range": "+/- 0.077",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 208.25,
            "range": "+/- 0.410",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.0065,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 31.032,
            "range": "+/- 0.014",
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
          "id": "61da3b5e474f7785e9315cb94379efed935c85d6",
          "message": "Add model shim for debugging via homotopy-web",
          "timestamp": "2021-08-06T13:47:08+01:00",
          "tree_id": "d1bab4dbd06cbe18a1e75b59b5e3455a9bbd4b35",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/61da3b5e474f7785e9315cb94379efed935c85d6"
        },
        "date": 1628254818116,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 12.288,
            "range": "+/- 0.102",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 12.583,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 95.322,
            "range": "+/- 0.877",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 112.6,
            "range": "+/- 0.900",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 91.581,
            "range": "+/- 0.832",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 128.19,
            "range": "+/- 1.100",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 83.128,
            "range": "+/- 0.676",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 547.69,
            "range": "+/- 5.300",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1842,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.6432,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.649,
            "range": "+/- 0.121",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 44.318,
            "range": "+/- 0.212",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 103.56,
            "range": "+/- 0.670",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 239.41,
            "range": "+/- 1.790",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 6.7874,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 35.8,
            "range": "+/- 0.299",
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
          "id": "90f526137790816ee44bbfb6ed7eb2c95fadd090",
          "message": "Fixed bug in typechecking that admitted too many diagrams.",
          "timestamp": "2021-08-16T14:08:28+01:00",
          "tree_id": "911a38192d41895ab083ef51227ac4d44b5e0f1c",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/90f526137790816ee44bbfb6ed7eb2c95fadd090"
        },
        "date": 1629119949152,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 65.259,
            "range": "+/- 0.557",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 67.117,
            "range": "+/- 0.691",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 93.004,
            "range": "+/- 1.067",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 115.87,
            "range": "+/- 1.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 142.97,
            "range": "+/- 1.380",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 193.3,
            "range": "+/- 1.830",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 82.173,
            "range": "+/- 0.676",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 563.24,
            "range": "+/- 8.220",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1902,
            "range": "+/- 0.018",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.7003,
            "range": "+/- 0.055",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.05,
            "range": "+/- 0.122",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 43.863,
            "range": "+/- 0.612",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 110.97,
            "range": "+/- 1.510",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 317.52,
            "range": "+/- 3.780",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 63.681,
            "range": "+/- 1.686",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 91.627,
            "range": "+/- 2.167",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a08a5dc116585b2f22923a1e89706e47a22d7704",
          "message": "Merge pull request #146 from homotopy-io/monotone\n\nFix monotone sequences iterator",
          "timestamp": "2021-08-17T14:22:15+01:00",
          "tree_id": "6f348255af7ea7ae72f09c9688dad7c515d2306b",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/a08a5dc116585b2f22923a1e89706e47a22d7704"
        },
        "date": 1629207064965,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 57.264,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 58.379,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.351,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 99.395,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 124.41,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 173.27,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.38,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 490.02,
            "range": "+/- 0.190",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9164,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8975,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.867,
            "range": "+/- 0.008",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 40.663,
            "range": "+/- 0.105",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 107.88,
            "range": "+/- 0.580",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 309.13,
            "range": "+/- 1.270",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 58.435,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 84.581,
            "range": "+/- 0.048",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d6f17e5f9d14d9eb3a637551497c0c2db00550b3",
          "message": "Merge pull request #149 from homotopy-io/commutativity\n\nAdd commutativity checker for rewrites",
          "timestamp": "2021-08-18T09:35:07+01:00",
          "tree_id": "b9c68e6069edc33f8986a054f5c7e2d7cdce162b",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/d6f17e5f9d14d9eb3a637551497c0c2db00550b3"
        },
        "date": 1629276233452,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 57.387,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 51.938,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.887,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 87.169,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 109.14,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 150.99,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 63.985,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 432.53,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.918,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8847,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.831,
            "range": "+/- 0.007",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 39.89,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 105.13,
            "range": "+/- 0.350",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 303.5,
            "range": "+/- 1.050",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 51.001,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 73.843,
            "range": "+/- 0.038",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "95efe1cabe64b47ccccbfbafa112b639115cb577",
          "message": "Merge pull request #150 from homotopy-io/factorisation",
          "timestamp": "2021-08-18T16:16:22+01:00",
          "tree_id": "cceeec51c788ee472eab2d0925b5d7bd68b38958",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/95efe1cabe64b47ccccbfbafa112b639115cb577"
        },
        "date": 1629300346828,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 71.3,
            "range": "+/- 1.078",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 73.942,
            "range": "+/- 1.012",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 106.44,
            "range": "+/- 1.540",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.29,
            "range": "+/- 0.930",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 157.31,
            "range": "+/- 1.760",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 217.29,
            "range": "+/- 2.540",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 93.576,
            "range": "+/- 2.153",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 620.59,
            "range": "+/- 8.580",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.4451,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.5067,
            "range": "+/- 0.078",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 20.179,
            "range": "+/- 0.156",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 52.471,
            "range": "+/- 0.954",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 136.74,
            "range": "+/- 1.160",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 390.09,
            "range": "+/- 3.530",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 74.231,
            "range": "+/- 0.805",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 107.16,
            "range": "+/- 1.720",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8703e0339c535fb33604cdc3234e6a0ac99bed04",
          "message": "Create `homotopy-common` (#151)\n\n* Create `homotopy-common`\r\n\r\n* Fix formatting",
          "timestamp": "2021-08-18T18:13:00+01:00",
          "tree_id": "b4ad0c4816f0c8dbb3f537840656383f334b6278",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/8703e0339c535fb33604cdc3234e6a0ac99bed04"
        },
        "date": 1629307695926,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 67.316,
            "range": "+/- 0.930",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 69.941,
            "range": "+/- 1.113",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 100.5,
            "range": "+/- 1.461",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 118.29,
            "range": "+/- 1.800",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 148.92,
            "range": "+/- 3.890",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 200.47,
            "range": "+/- 2.790",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 85.98,
            "range": "+/- 1.291",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 592.89,
            "range": "+/- 10.630",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.3723,
            "range": "+/- 0.035",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.2959,
            "range": "+/- 0.116",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 19.123,
            "range": "+/- 0.309",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 46.968,
            "range": "+/- 0.505",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 120.47,
            "range": "+/- 1.210",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 330.9,
            "range": "+/- 4.680",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 68.293,
            "range": "+/- 1.459",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 98.288,
            "range": "+/- 1.491",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e1b47279ab36fd6b6ec55659d2717f9584104e2",
          "message": "Add WebGL wrapper to `homotopy-graphics` (#152)\n\n* Stage changes\r\n\r\n* Add render pipeline outline\r\n\r\n* Add test renderer\r\n\r\n* Switch buffering to `euclid`\r\n\r\n* Stage changes\r\n\r\n* Switch to new view of buffers\r\n\r\n* Draw a triangle again\r\n\r\n* Move code into `homotopy_graphics`\r\n\r\n* `Rc` everything important\r\n\r\n* Attributes, uniforms & elements\r\n\r\n* Spinning rabbit\r\n\r\n* Move `tree` into `homotopy-core`\r\n\r\n* Move `tree` into `homotopy-core`\r\n\r\n* Projection\r\n\r\n* Remove use of `yew::services`\r\n\r\n* Remove `bunny.obj`",
          "timestamp": "2021-08-18T19:35:17+01:00",
          "tree_id": "5e0affb2fb8ccb677206e2cd36e51414527aafa4",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/0e1b47279ab36fd6b6ec55659d2717f9584104e2"
        },
        "date": 1629312314111,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 57.282,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 58.615,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 83.556,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.64,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 123.41,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 175.29,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.575,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 490.81,
            "range": "+/- 0.250",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.925,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.8977,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.849,
            "range": "+/- 0.007",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 39.836,
            "range": "+/- 0.164",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 104.65,
            "range": "+/- 0.280",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 299.74,
            "range": "+/- 1.080",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 58.091,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 83.947,
            "range": "+/- 0.042",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "6b79865c81d68922e44ec5c53712a1455720fe89",
          "message": "Fix bug in",
          "timestamp": "2021-08-24T14:54:00+01:00",
          "tree_id": "3ab35af889783086ba4b5ad9c97c3098bd935067",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/6b79865c81d68922e44ec5c53712a1455720fe89"
        },
        "date": 1629814206013,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 70.39,
            "range": "+/- 0.887",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 72.877,
            "range": "+/- 1.558",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 107.01,
            "range": "+/- 2.870",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.36,
            "range": "+/- 2.150",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 154.33,
            "range": "+/- 1.710",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 208.86,
            "range": "+/- 1.710",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 91.319,
            "range": "+/- 1.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 608.36,
            "range": "+/- 7.350",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.3678,
            "range": "+/- 0.014",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.3584,
            "range": "+/- 0.087",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 20.007,
            "range": "+/- 0.466",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 51.156,
            "range": "+/- 1.034",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 134.43,
            "range": "+/- 3.080",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 371.5,
            "range": "+/- 2.670",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 71.852,
            "range": "+/- 0.934",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 101.72,
            "range": "+/- 1.010",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "acea326ebf04cb4fbeba5764205cc27c35a413d1",
          "message": "Fix bug in `DiagramN::is_well_formed`",
          "timestamp": "2021-08-24T14:55:55+01:00",
          "tree_id": "3ab35af889783086ba4b5ad9c97c3098bd935067",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/acea326ebf04cb4fbeba5764205cc27c35a413d1"
        },
        "date": 1629814215600,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 57.398,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 58.451,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 82.53,
            "range": "+/- 0.052",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 98.26,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 124.29,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 175.09,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.528,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 492.28,
            "range": "+/- 0.660",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9361,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.9131,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.901,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 40.016,
            "range": "+/- 0.197",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 104.57,
            "range": "+/- 0.240",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 300.29,
            "range": "+/- 1.110",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 58.469,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 84.62,
            "range": "+/- 0.039",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cfee3cc838e64dc15838ffe8f9a9bc29ba680429",
          "message": "Merge pull request #155 from homotopy-io/import\n\nCheck a diagram is well-formed before importing it",
          "timestamp": "2021-08-24T14:57:27+01:00",
          "tree_id": "4fb847236e1c40e35c10dc53dac1c265e6db1014",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/cfee3cc838e64dc15838ffe8f9a9bc29ba680429"
        },
        "date": 1629814224143,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 60.4,
            "range": "+/- 0.837",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 62.039,
            "range": "+/- 0.981",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 91.744,
            "range": "+/- 2.009",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 102.38,
            "range": "+/- 1.130",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 133.46,
            "range": "+/- 1.840",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 189.02,
            "range": "+/- 3.090",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 80.363,
            "range": "+/- 1.646",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 530.81,
            "range": "+/- 5.370",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.0642,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.4164,
            "range": "+/- 0.105",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.42,
            "range": "+/- 0.346",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 43.002,
            "range": "+/- 0.565",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 109.86,
            "range": "+/- 1.300",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 304.89,
            "range": "+/- 2.030",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 60.922,
            "range": "+/- 0.683",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 87.797,
            "range": "+/- 0.992",
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
          "id": "d943c1be073daed56c677395e353743b3060c90b",
          "message": "CI: prevent deployment race conditions",
          "timestamp": "2021-08-24T16:50:32+01:00",
          "tree_id": "d855b46b75dc1d53278f98c302b2c82a8d3f895d",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/d943c1be073daed56c677395e353743b3060c90b"
        },
        "date": 1629820964417,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 61.621,
            "range": "+/- 0.679",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 63.834,
            "range": "+/- 0.628",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 94.009,
            "range": "+/- 0.769",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 106.51,
            "range": "+/- 1.260",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 142.61,
            "range": "+/- 4.410",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 187.11,
            "range": "+/- 2.170",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 79.936,
            "range": "+/- 0.650",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 560.29,
            "range": "+/- 8.310",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.1299,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.4514,
            "range": "+/- 0.092",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 17.122,
            "range": "+/- 0.111",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 43.376,
            "range": "+/- 0.427",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 110.35,
            "range": "+/- 0.920",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 312.84,
            "range": "+/- 2.600",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 63.3,
            "range": "+/- 0.783",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 90.812,
            "range": "+/- 1.597",
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
          "id": "c9a1dc6b6ece6dfdd06e5892152ad4a9efab5f25",
          "message": "Implement smoothing, where a singular level can be eliminated when it\nis surrounded by identical rewrites. This is triggered by clicking and\ndragging from an adjacent regular level into the singular level.",
          "timestamp": "2021-08-25T11:54:46+01:00",
          "tree_id": "8eb47dd9fff9b5a9089ebec16778457b4a9b60aa",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/c9a1dc6b6ece6dfdd06e5892152ad4a9efab5f25"
        },
        "date": 1629889445150,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 70.122,
            "range": "+/- 1.163",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 74.678,
            "range": "+/- 1.981",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 105.87,
            "range": "+/- 2.360",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 122.93,
            "range": "+/- 2.740",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 156.57,
            "range": "+/- 3.550",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 222.28,
            "range": "+/- 4.830",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 96.506,
            "range": "+/- 3.713",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 644.62,
            "range": "+/- 19.380",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.4648,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 7.837,
            "range": "+/- 0.146",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 20.219,
            "range": "+/- 0.367",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 52.356,
            "range": "+/- 1.157",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 131.28,
            "range": "+/- 1.680",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 378.66,
            "range": "+/- 5.940",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 71.793,
            "range": "+/- 1.779",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 107.68,
            "range": "+/- 2.530",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "distinct": true,
          "id": "1ac984640cf21c63076caa4c2d237b0fe0e811c4",
          "message": "Add error information to well-formed checker",
          "timestamp": "2021-08-25T12:07:42+01:00",
          "tree_id": "d449a7bf8af6735952d19354c0c34aa55eec008a",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/1ac984640cf21c63076caa4c2d237b0fe0e811c4"
        },
        "date": 1629890210787,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 57.047,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 58.701,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 84.447,
            "range": "+/- 0.183",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 100.35,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 123.99,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 174.12,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 73.289,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 493.96,
            "range": "+/- 0.190",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9354,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.9372,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.903,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 39.915,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 102.65,
            "range": "+/- 0.220",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 295.02,
            "range": "+/- 0.830",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 58.235,
            "range": "+/- 0.032",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 84.788,
            "range": "+/- 0.056",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "f86661acd7ce6b47d09216d53d41ed7de2c5c1ab",
          "message": "Fix smoothing on first/last regular level",
          "timestamp": "2021-08-25T16:32:35+01:00",
          "tree_id": "270046ef215e33eb25a4411f08d7dff5c8d619f3",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f86661acd7ce6b47d09216d53d41ed7de2c5c1ab"
        },
        "date": 1629906116185,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 57.279,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 51.53,
            "range": "+/- 0.069",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 82.881,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 87.567,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 123.48,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 175.37,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 72.932,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 431.61,
            "range": "+/- 1.660",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.9278,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.2051,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 14.009,
            "range": "+/- 0.006",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 40.101,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 105.27,
            "range": "+/- 0.260",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 271.79,
            "range": "+/- 1.110",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 51.169,
            "range": "+/- 0.230",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 83.774,
            "range": "+/- 0.030",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "01d4f0c3fcbb0f0d46c97862c5e368f939e0b7db",
          "message": "Merge pull request #156 from homotopy-io/check\n\nCheck for well-formedness on every operation",
          "timestamp": "2021-08-26T12:42:43+01:00",
          "tree_id": "ed4078ea47865e02952f6987912190a993da9fba",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/01d4f0c3fcbb0f0d46c97862c5e368f939e0b7db"
        },
        "date": 1629982344366,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 87.008,
            "range": "+/- 1.180",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 88.589,
            "range": "+/- 1.318",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 133.22,
            "range": "+/- 1.880",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 144.2,
            "range": "+/- 2.260",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 227.51,
            "range": "+/- 2.260",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 278.47,
            "range": "+/- 2.180",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 112.37,
            "range": "+/- 1.900",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 856.81,
            "range": "+/- 11.710",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.9886,
            "range": "+/- 0.096",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.456,
            "range": "+/- 0.236",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 74.266,
            "range": "+/- 1.151",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 417.96,
            "range": "+/- 6.470",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.9642,
            "range": "+/- 0.036",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 26.483,
            "range": "+/- 0.182",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 91.158,
            "range": "+/- 1.436",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 119.3,
            "range": "+/- 2.290",
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
          "id": "5dad7e4584fa7bf665e1f956e84604c9a26437cc",
          "message": "Allow smoothing the last singular level",
          "timestamp": "2021-08-27T12:56:54+01:00",
          "tree_id": "51e42e1f90d883d06c286ed14a478aaf451e543f",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/5dad7e4584fa7bf665e1f956e84604c9a26437cc"
        },
        "date": 1630068818078,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 76.405,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 77.675,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 115.27,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 126.73,
            "range": "+/- 0.410",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 197.11,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 246.35,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 98.03,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 724.01,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3494,
            "range": "+/- 0.007",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.808,
            "range": "+/- 0.056",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.541,
            "range": "+/- 0.122",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 343.45,
            "range": "+/- 0.130",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5146,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.213,
            "range": "+/- 0.041",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 79.398,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 103.51,
            "range": "+/- 0.040",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "efb0b3dcdb7374d627c9527ff921aebfd7c37d5e",
          "message": "Add safe versions of rewrite constructors",
          "timestamp": "2021-08-27T13:45:28+01:00",
          "tree_id": "9f46caf9634172f238020d08002839d8282eced5",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/efb0b3dcdb7374d627c9527ff921aebfd7c37d5e"
        },
        "date": 1630071612968,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 75.774,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 77.158,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 113.83,
            "range": "+/- 0.350",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.58,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 194.81,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 244.46,
            "range": "+/- 1.010",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.552,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 712.08,
            "range": "+/- 0.190",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3108,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.752,
            "range": "+/- 0.054",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.346,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 341.9,
            "range": "+/- 0.210",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.495,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.058,
            "range": "+/- 0.030",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 78.599,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 102.27,
            "range": "+/- 0.030",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "2da470bd358924541777452b9282e9f1b57dceac",
          "message": "Remove leftover error in expansion",
          "timestamp": "2021-08-27T20:58:18+01:00",
          "tree_id": "a38df8fdf430021552bd9fcb85e6b8f889d6df35",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/2da470bd358924541777452b9282e9f1b57dceac"
        },
        "date": 1630097886308,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 86.451,
            "range": "+/- 2.155",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 90.902,
            "range": "+/- 1.828",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 141.99,
            "range": "+/- 2.700",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 153.22,
            "range": "+/- 4.670",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 238.79,
            "range": "+/- 7.160",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 281.16,
            "range": "+/- 8.220",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 113.42,
            "range": "+/- 2.990",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 865.69,
            "range": "+/- 21.460",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.825,
            "range": "+/- 0.084",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 15.931,
            "range": "+/- 0.261",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 70.129,
            "range": "+/- 1.362",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 402.63,
            "range": "+/- 4.620",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.8834,
            "range": "+/- 0.032",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 26.384,
            "range": "+/- 0.179",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 93.087,
            "range": "+/- 2.497",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 118,
            "range": "+/- 3.880",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "24dd156ed4537cde6548c98559c57114c033ca9a",
          "message": "Fix deletion",
          "timestamp": "2021-09-01T21:24:32+01:00",
          "tree_id": "41a70544d9e0d37e5e8ca0a22225920910035ce1",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/24dd156ed4537cde6548c98559c57114c033ca9a"
        },
        "date": 1630531604488,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 80.768,
            "range": "+/- 1.118",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 82.442,
            "range": "+/- 1.650",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 121.93,
            "range": "+/- 1.960",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 129.48,
            "range": "+/- 1.520",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 210.4,
            "range": "+/- 4.520",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 259.9,
            "range": "+/- 3.200",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 104.14,
            "range": "+/- 1.030",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 780.28,
            "range": "+/- 20.460",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.9281,
            "range": "+/- 0.103",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 17.413,
            "range": "+/- 0.742",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 77.146,
            "range": "+/- 1.861",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 430.11,
            "range": "+/- 5.090",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0347,
            "range": "+/- 0.023",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 27.383,
            "range": "+/- 0.262",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 95.667,
            "range": "+/- 1.209",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 124.08,
            "range": "+/- 1.620",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "6b84a5f0f702c40d2861814fe812a05523b1e25d",
          "message": "Fix panzoom",
          "timestamp": "2021-09-02T11:37:38+01:00",
          "tree_id": "e2af02b9c50c0cbf14e7b5a1c43318d32d94ab11",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/6b84a5f0f702c40d2861814fe812a05523b1e25d"
        },
        "date": 1630582147165,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 80.778,
            "range": "+/- 2.264",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 83.434,
            "range": "+/- 2.113",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 113.53,
            "range": "+/- 3.780",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.68,
            "range": "+/- 3.430",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 203.94,
            "range": "+/- 3.870",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 243.08,
            "range": "+/- 5.700",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 103.61,
            "range": "+/- 2.100",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 792.45,
            "range": "+/- 10.780",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.5101,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.45,
            "range": "+/- 0.266",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 59.867,
            "range": "+/- 1.121",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 362.87,
            "range": "+/- 8.940",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.6277,
            "range": "+/- 0.047",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 22.994,
            "range": "+/- 0.333",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 76.524,
            "range": "+/- 1.949",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 98.097,
            "range": "+/- 2.080",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "07219601fda1b4343e8086e66d16bdb80f574dd5",
          "message": "Signatures 2.0 support for non-Mozilla targets",
          "timestamp": "2021-09-02T12:13:19+01:00",
          "tree_id": "08eedd75900af5b21ab8c7a72dde755dbc6359f5",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/07219601fda1b4343e8086e66d16bdb80f574dd5"
        },
        "date": 1630584864845,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 93.721,
            "range": "+/- 2.492",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 96.214,
            "range": "+/- 2.972",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 141.69,
            "range": "+/- 4.900",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 155.82,
            "range": "+/- 6.150",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 246.71,
            "range": "+/- 6.930",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 302.25,
            "range": "+/- 7.700",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 121.99,
            "range": "+/- 2.930",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 897.04,
            "range": "+/- 20.690",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.0849,
            "range": "+/- 0.092",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.801,
            "range": "+/- 0.417",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 74.577,
            "range": "+/- 1.272",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 416.3,
            "range": "+/- 4.470",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0253,
            "range": "+/- 0.018",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 27.701,
            "range": "+/- 0.104",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 96.643,
            "range": "+/- 2.198",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 125.4,
            "range": "+/- 2.440",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "distinct": true,
          "id": "99eaf8552900655f70c7a97990505b258200071d",
          "message": "Make factorisation into an iterator",
          "timestamp": "2021-09-04T17:53:03+03:00",
          "tree_id": "218178460d24dea2c765df9ed3acf185da791e23",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/99eaf8552900655f70c7a97990505b258200071d"
        },
        "date": 1630771024323,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 90.291,
            "range": "+/- 1.502",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 92.851,
            "range": "+/- 1.356",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 140.2,
            "range": "+/- 3.940",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 143.17,
            "range": "+/- 2.610",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 233.43,
            "range": "+/- 5.650",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 284.82,
            "range": "+/- 4.500",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 116.25,
            "range": "+/- 2.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 849.55,
            "range": "+/- 14.630",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.9338,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.035,
            "range": "+/- 0.211",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 72.031,
            "range": "+/- 1.120",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 397.19,
            "range": "+/- 3.480",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.9135,
            "range": "+/- 0.020",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 26.67,
            "range": "+/- 0.087",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 91.918,
            "range": "+/- 2.737",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 118.4,
            "range": "+/- 1.670",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "f75d62c725ca66e866127df4da26235514c6248b",
          "message": "Hotfix drop logic",
          "timestamp": "2021-09-04T17:32:41+01:00",
          "tree_id": "4b93f4a7c62205dd2896e39b0c534adf7c0dbd38",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f75d62c725ca66e866127df4da26235514c6248b"
        },
        "date": 1630776965171,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 94.383,
            "range": "+/- 1.812",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 97.183,
            "range": "+/- 1.717",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 143.28,
            "range": "+/- 2.850",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 150.22,
            "range": "+/- 1.430",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 246.78,
            "range": "+/- 5.280",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 306.26,
            "range": "+/- 5.850",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 123.15,
            "range": "+/- 2.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 903.4,
            "range": "+/- 9.730",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.1623,
            "range": "+/- 0.052",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 17.214,
            "range": "+/- 0.190",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 76.74,
            "range": "+/- 0.879",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 423.39,
            "range": "+/- 3.420",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.1518,
            "range": "+/- 0.016",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 28.551,
            "range": "+/- 0.066",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 100.71,
            "range": "+/- 1.843",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 128.4,
            "range": "+/- 1.530",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "ba397a8261200d0c7599b79d7fd2f3b275e567a9",
          "message": "Fix loop in `render::merge_surfaces`",
          "timestamp": "2021-09-05T21:14:16+03:00",
          "tree_id": "a9544f15d0349686396dd58e2dc8dc4f3931431e",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/ba397a8261200d0c7599b79d7fd2f3b275e567a9"
        },
        "date": 1630868464980,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 76.093,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 77.109,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 112.3,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.77,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 195.76,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 245.81,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.695,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 713.75,
            "range": "+/- 0.270",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.307,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 12.097,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 54.248,
            "range": "+/- 0.021",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 303.6,
            "range": "+/- 0.160",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.223,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 20.388,
            "range": "+/- 0.053",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 68.327,
            "range": "+/- 0.259",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 102.9,
            "range": "+/- 0.020",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "0d2360fc782e2c74444e6a61d05546a72e325a87",
          "message": "Fix formatting",
          "timestamp": "2021-09-05T21:20:51+03:00",
          "tree_id": "6830e48bed20290907c601e1f271205908f23a72",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/0d2360fc782e2c74444e6a61d05546a72e325a87"
        },
        "date": 1630869870398,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 91.666,
            "range": "+/- 1.681",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 95.058,
            "range": "+/- 2.228",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 142.6,
            "range": "+/- 2.220",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.57,
            "range": "+/- 2.990",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 243.77,
            "range": "+/- 4.120",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 310.97,
            "range": "+/- 7.630",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 123.67,
            "range": "+/- 2.730",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 908.77,
            "range": "+/- 21.380",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.1229,
            "range": "+/- 0.075",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 17.765,
            "range": "+/- 0.349",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 77.023,
            "range": "+/- 1.201",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 426.75,
            "range": "+/- 3.780",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0587,
            "range": "+/- 0.020",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 28.677,
            "range": "+/- 0.113",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 96.412,
            "range": "+/- 1.616",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 126.19,
            "range": "+/- 2.570",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "bcc5aebb7e012e42da412a8f097e522199285e56",
          "message": "Do not merge surfaces until that's fixed",
          "timestamp": "2021-09-06T15:22:44+03:00",
          "tree_id": "fe47a5d5bb0605e07b8c3d59add8ad13de887db9",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/bcc5aebb7e012e42da412a8f097e522199285e56"
        },
        "date": 1630934035414,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 75.633,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 76.833,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 113.28,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.71,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 195.72,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 244.17,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.826,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 712.72,
            "range": "+/- 0.300",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3082,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.748,
            "range": "+/- 0.005",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.413,
            "range": "+/- 0.050",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 342.57,
            "range": "+/- 0.160",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5048,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.096,
            "range": "+/- 0.035",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 80.141,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 102.69,
            "range": "+/- 0.040",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "762919074d1bef8f1865f35c3b206dd86d700f59",
          "message": "Move illustration",
          "timestamp": "2021-09-06T13:25:22+01:00",
          "tree_id": "ce6c967e13e4953c465d76ca0ae2aebe49fd782c",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/762919074d1bef8f1865f35c3b206dd86d700f59"
        },
        "date": 1630934816331,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 93.816,
            "range": "+/- 1.244",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 95.158,
            "range": "+/- 1.160",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 140.42,
            "range": "+/- 1.810",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 148.03,
            "range": "+/- 0.840",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 240.29,
            "range": "+/- 4.560",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 297.64,
            "range": "+/- 4.360",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 120.58,
            "range": "+/- 2.110",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 879.72,
            "range": "+/- 11.300",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.1129,
            "range": "+/- 0.072",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 17.553,
            "range": "+/- 0.639",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 74.887,
            "range": "+/- 1.382",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 434.86,
            "range": "+/- 7.120",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0245,
            "range": "+/- 0.014",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 27.523,
            "range": "+/- 0.126",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 97.487,
            "range": "+/- 1.374",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 124.73,
            "range": "+/- 1.760",
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
          "id": "a3a4807419821e6f359e51d75ebec4e0ed9556cb",
          "message": "Move to unstable rustfmt for stricter code formatting",
          "timestamp": "2021-09-07T12:07:53+01:00",
          "tree_id": "cc1cee754851cb162657125f2dfe69f55f1fe044",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/a3a4807419821e6f359e51d75ebec4e0ed9556cb"
        },
        "date": 1631016273070,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 77.123,
            "range": "+/- 2.034",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 79.267,
            "range": "+/- 2.165",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 118.66,
            "range": "+/- 3.320",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 124.08,
            "range": "+/- 1.640",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 205.79,
            "range": "+/- 5.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 257.17,
            "range": "+/- 5.530",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 100.04,
            "range": "+/- 1.833",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 737.29,
            "range": "+/- 13.500",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2723,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 14.008,
            "range": "+/- 0.225",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 62.508,
            "range": "+/- 0.841",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 357.45,
            "range": "+/- 4.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.7124,
            "range": "+/- 0.037",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.827,
            "range": "+/- 0.462",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 88.731,
            "range": "+/- 1.054",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 114.71,
            "range": "+/- 1.580",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "d9203ccc016dade5ec165b91f926a4bb21fe1f9b",
          "message": "Make payload composition safe",
          "timestamp": "2021-09-07T16:52:20+03:00",
          "tree_id": "5369213894217d87655dc2b3835688d9af156e06",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/d9203ccc016dade5ec165b91f926a4bb21fe1f9b"
        },
        "date": 1631026338899,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 76.025,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 77.283,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 113.18,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 124.34,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 196.12,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 248.54,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.869,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 712.09,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3182,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.741,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.52,
            "range": "+/- 0.022",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 343.49,
            "range": "+/- 0.130",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5203,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.103,
            "range": "+/- 0.058",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 80.552,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 103.7,
            "range": "+/- 0.200",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "distinct": true,
          "id": "c07b144149284f0dcafdc32df341f0007ea0d1bd",
          "message": "Cubicalisation",
          "timestamp": "2021-09-07T17:38:48+03:00",
          "tree_id": "539f34926d417fc7ac93801c126f01e56d2bce8e",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/c07b144149284f0dcafdc32df341f0007ea0d1bd"
        },
        "date": 1631028570774,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 73.717,
            "range": "+/- 0.302",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.169,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 110.76,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.87,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 190.61,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 242.99,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 94.291,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 698.91,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2502,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.395,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 60.107,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 335.34,
            "range": "+/- 0.100",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.4525,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 22.707,
            "range": "+/- 0.030",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 76.89,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 101.1,
            "range": "+/- 0.070",
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
          "id": "1680761e96e6c6430597424fd28f10680ab64621",
          "message": "Well-formed checks carry through rewrite composition failures and now\nmay accumulate multiple errors in a single pass rather than the first\none it finds.",
          "timestamp": "2021-09-07T22:50:24+01:00",
          "tree_id": "5d6128f4a0cb70c463df56bd7460554efdecdcd7",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/1680761e96e6c6430597424fd28f10680ab64621"
        },
        "date": 1631055246479,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 91.019,
            "range": "+/- 0.922",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 93.693,
            "range": "+/- 1.431",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 137.61,
            "range": "+/- 2.150",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.94,
            "range": "+/- 1.860",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 239.24,
            "range": "+/- 2.820",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 300.95,
            "range": "+/- 5.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 117.32,
            "range": "+/- 2.120",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 865.38,
            "range": "+/- 10.140",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.9864,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.682,
            "range": "+/- 0.178",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 75.049,
            "range": "+/- 0.646",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 417.61,
            "range": "+/- 2.240",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0735,
            "range": "+/- 0.008",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 28.527,
            "range": "+/- 0.069",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 94.36,
            "range": "+/- 1.011",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 125.53,
            "range": "+/- 2.120",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "62886de39b7622575a12227968dbd4d24c4e8469",
          "message": "Refactor rewrite constructors",
          "timestamp": "2021-09-14T10:07:00+01:00",
          "tree_id": "b87629b3fd6d5a5da884048ff2aea3e7af681e0d",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/62886de39b7622575a12227968dbd4d24c4e8469"
        },
        "date": 1631614260708,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 74.149,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.609,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 111.44,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.14,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 192.89,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 241.78,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.151,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 709.81,
            "range": "+/- 0.910",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2952,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.685,
            "range": "+/- 0.005",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.109,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 341.43,
            "range": "+/- 0.140",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.511,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.134,
            "range": "+/- 0.026",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 79.661,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 102.86,
            "range": "+/- 0.060",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "51a2532a133825768f369036a2c46f500fa89892",
          "message": "Record original position of every node in cubicalisation",
          "timestamp": "2021-09-15T11:34:01+01:00",
          "tree_id": "e4dfa54ff93d81126b4ef63f028ac55899a40c8a",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/51a2532a133825768f369036a2c46f500fa89892"
        },
        "date": 1631705244320,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 74.54,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 76.001,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 111.96,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.3,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 193.21,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 242.42,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.402,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 711.87,
            "range": "+/- 0.420",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.3117,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.764,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 62.313,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 351.56,
            "range": "+/- 0.110",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5576,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.711,
            "range": "+/- 0.093",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 78.263,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 102.04,
            "range": "+/- 0.050",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "f87685df7b97010d4c4d312dbd827e5d2c10710a",
          "message": "Dummy mesh extraction function",
          "timestamp": "2021-09-15T13:40:24+01:00",
          "tree_id": "9fc7dfcf48167eed5747c7afe5f0dd179388c093",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/f87685df7b97010d4c4d312dbd827e5d2c10710a"
        },
        "date": 1631713381254,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 92.192,
            "range": "+/- 1.464",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 94.133,
            "range": "+/- 2.082",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 141.8,
            "range": "+/- 1.690",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.58,
            "range": "+/- 1.980",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 238.81,
            "range": "+/- 2.420",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 291.48,
            "range": "+/- 2.370",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 119.03,
            "range": "+/- 1.700",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 882.28,
            "range": "+/- 9.400",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.1727,
            "range": "+/- 0.139",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.838,
            "range": "+/- 0.240",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 75.227,
            "range": "+/- 1.201",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 416.48,
            "range": "+/- 2.370",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0278,
            "range": "+/- 0.008",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 27.957,
            "range": "+/- 0.039",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 99.829,
            "range": "+/- 1.488",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 127.68,
            "range": "+/- 3.070",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "3446e85ba6086dcc5b0a59ac8caa11fbca106590",
          "message": "Connect `Diagram3D` to dummy mesh builder",
          "timestamp": "2021-09-15T17:19:17+01:00",
          "tree_id": "fd2f423d21d4dae42f1262c5f2304cfb3683e1fa",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/3446e85ba6086dcc5b0a59ac8caa11fbca106590"
        },
        "date": 1631726078500,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 78.568,
            "range": "+/- 1.929",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 81.477,
            "range": "+/- 2.363",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 117.8,
            "range": "+/- 2.510",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 129.46,
            "range": "+/- 3.170",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 197.08,
            "range": "+/- 6.230",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 263.01,
            "range": "+/- 7.220",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 99.964,
            "range": "+/- 2.959",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 736.84,
            "range": "+/- 17.290",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.5709,
            "range": "+/- 0.087",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 14.291,
            "range": "+/- 0.278",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 64.786,
            "range": "+/- 1.281",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 360.55,
            "range": "+/- 5.660",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.6489,
            "range": "+/- 0.029",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 24.681,
            "range": "+/- 0.135",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 80.279,
            "range": "+/- 2.313",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 106.89,
            "range": "+/- 3.230",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "76a3106cf4196a2c463bbe940183173718c4db40",
          "message": "Add generator information to meshes",
          "timestamp": "2021-09-16T13:00:45+01:00",
          "tree_id": "d1c3adc570c9e4011a33845c1689a7bd36bb2fca",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/76a3106cf4196a2c463bbe940183173718c4db40"
        },
        "date": 1631796762140,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 74.395,
            "range": "+/- 0.300",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.616,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 112.01,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.74,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 192.65,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 240.56,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.365,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 711.04,
            "range": "+/- 0.380",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.296,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.676,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.396,
            "range": "+/- 0.056",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 343.54,
            "range": "+/- 0.160",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5159,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.256,
            "range": "+/- 0.041",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 77.84,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 101.02,
            "range": "+/- 0.040",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "79a0c391074f989b73ef2ee95407895f327bea59",
          "message": "Refactor 4D subdivider",
          "timestamp": "2021-09-16T19:09:11+01:00",
          "tree_id": "e01ed39110c190789fe1f99e7e29be5925e3d4b9",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/79a0c391074f989b73ef2ee95407895f327bea59"
        },
        "date": 1631819443906,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 90.637,
            "range": "+/- 0.933",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 92.737,
            "range": "+/- 0.984",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 139.15,
            "range": "+/- 1.880",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.83,
            "range": "+/- 3.330",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 237.7,
            "range": "+/- 2.370",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 293.87,
            "range": "+/- 1.690",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 117.87,
            "range": "+/- 0.710",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 876.52,
            "range": "+/- 10.240",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.0128,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.842,
            "range": "+/- 0.209",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 75.322,
            "range": "+/- 0.722",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 415.62,
            "range": "+/- 1.630",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.0099,
            "range": "+/- 0.005",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 27.63,
            "range": "+/- 0.094",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 92.936,
            "range": "+/- 0.631",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 120.86,
            "range": "+/- 1.300",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "c552d29ac209405d34b541b93d8e91e04a8ee460",
          "message": "Refactor keys in cubicalisation",
          "timestamp": "2021-09-17T10:20:08+01:00",
          "tree_id": "2f2a897623aefa8cf26ad2a75043fa2e05d0ddf2",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/c552d29ac209405d34b541b93d8e91e04a8ee460"
        },
        "date": 1631873737187,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 74.792,
            "range": "+/- 1.221",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.195,
            "range": "+/- 1.363",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 115.49,
            "range": "+/- 2.710",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 123.86,
            "range": "+/- 2.160",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 198.51,
            "range": "+/- 3.700",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 250.04,
            "range": "+/- 4.500",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.637,
            "range": "+/- 1.885",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 717.85,
            "range": "+/- 13.600",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.4675,
            "range": "+/- 0.086",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.656,
            "range": "+/- 0.179",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 69.063,
            "range": "+/- 0.991",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 401.82,
            "range": "+/- 2.820",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.9157,
            "range": "+/- 0.021",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 24.369,
            "range": "+/- 0.404",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 80.95,
            "range": "+/- 1.891",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 105.53,
            "range": "+/- 2.000",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "Calin Tataru",
            "username": "calintat"
          },
          "distinct": true,
          "id": "4a3b6f4b00ef36145dd737ce0bf922704f9d31f9",
          "message": "Mesh extraction",
          "timestamp": "2021-09-17T12:19:24+01:00",
          "tree_id": "c9df52aa1ee0ec606e4d89fb3a0faf27d082d5b3",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/4a3b6f4b00ef36145dd737ce0bf922704f9d31f9"
        },
        "date": 1631880896610,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 81.84,
            "range": "+/- 1.359",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 85.706,
            "range": "+/- 1.314",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 122.2,
            "range": "+/- 1.690",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 129.82,
            "range": "+/- 2.630",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 208.71,
            "range": "+/- 4.670",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 260.21,
            "range": "+/- 5.570",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 103.8,
            "range": "+/- 2.280",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 762.83,
            "range": "+/- 14.870",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.4957,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 15.95,
            "range": "+/- 0.375",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 64.479,
            "range": "+/- 1.020",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 364.36,
            "range": "+/- 4.610",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.6899,
            "range": "+/- 0.023",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 25.2,
            "range": "+/- 0.246",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 85.237,
            "range": "+/- 2.366",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 107.5,
            "range": "+/- 2.640",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "03088dca54cdd3f3d639d5530754367ce362e33a",
          "message": "Remove log message",
          "timestamp": "2021-09-17T12:25:54+01:00",
          "tree_id": "421f1b7fe7f653e599a3fcdf615b49ef98ce376d",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/03088dca54cdd3f3d639d5530754367ce362e33a"
        },
        "date": 1631881137689,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 73.964,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.744,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 111.06,
            "range": "+/- 0.190",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 122.88,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 192.61,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 244.32,
            "range": "+/- 0.430",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 95.211,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 704.79,
            "range": "+/- 0.470",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2883,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.646,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.055,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 343.78,
            "range": "+/- 0.110",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5187,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.221,
            "range": "+/- 0.057",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 77.156,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 100.79,
            "range": "+/- 0.050",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "26f5164f963aeb540597c17de4370623f85ddd40",
          "message": "Do not use map on arrays",
          "timestamp": "2021-09-17T12:55:38+01:00",
          "tree_id": "40dc47cecb0d254412df4996a06e70a605831282",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/26f5164f963aeb540597c17de4370623f85ddd40"
        },
        "date": 1631882881750,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 74.211,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.733,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 112.47,
            "range": "+/- 0.510",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.85,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 192.3,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 243.65,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 94.844,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 711.94,
            "range": "+/- 4.660",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2885,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.683,
            "range": "+/- 0.004",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 61.337,
            "range": "+/- 0.022",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 341.9,
            "range": "+/- 0.230",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5153,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.185,
            "range": "+/- 0.037",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 76.93,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 100.76,
            "range": "+/- 0.030",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "d8de8cc6038068e686e826e05613bbf5c1cac67c",
          "message": "Add 4D lighting",
          "timestamp": "2021-09-17T19:13:28+01:00",
          "tree_id": "51836cf9b2a7fcc6b4d45190b779b03233a32543",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/d8de8cc6038068e686e826e05613bbf5c1cac67c"
        },
        "date": 1631905544851,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 74.594,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 75.861,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 110.99,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 121.78,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 191.81,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 243.64,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 94.334,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 704.1,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.2763,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 13.631,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 60.999,
            "range": "+/- 0.014",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 341.83,
            "range": "+/- 0.090",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.5111,
            "range": "+/- 0.001",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 23.159,
            "range": "+/- 0.044",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 77.202,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 100.43,
            "range": "+/- 0.030",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "607040bc29a803349252c9de993a72c16381dc27",
          "message": "Fix the orientation of elements in the mesh",
          "timestamp": "2021-09-20T14:29:35+01:00",
          "tree_id": "0f409d743e3913c5381db41fb8ccb05938b301a3",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/607040bc29a803349252c9de993a72c16381dc27"
        },
        "date": 1632148337498,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 85.229,
            "range": "+/- 1.117",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 86.141,
            "range": "+/- 0.977",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 131.76,
            "range": "+/- 0.750",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 145.65,
            "range": "+/- 0.840",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 223.85,
            "range": "+/- 1.190",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 278.21,
            "range": "+/- 2.490",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 112.22,
            "range": "+/- 0.780",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 831.24,
            "range": "+/- 5.300",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.8535,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.113,
            "range": "+/- 0.068",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 72.708,
            "range": "+/- 0.321",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 406.12,
            "range": "+/- 1.570",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.9846,
            "range": "+/- 0.009",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 26.985,
            "range": "+/- 0.190",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 92.53,
            "range": "+/- 0.343",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 118.83,
            "range": "+/- 1.070",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "22a21c7839f7f4c5d9ed13b42bbe84a141d37e34",
          "message": "Remove examples",
          "timestamp": "2021-09-20T14:54:34+01:00",
          "tree_id": "a8cb319f8b9a44f59553113822eea0c8e5484065",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/22a21c7839f7f4c5d9ed13b42bbe84a141d37e34"
        },
        "date": 1632149604939,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 84.733,
            "range": "+/- 1.277",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 86.595,
            "range": "+/- 1.636",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 128.7,
            "range": "+/- 1.940",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 141.29,
            "range": "+/- 2.060",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 217.06,
            "range": "+/- 3.900",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 275.24,
            "range": "+/- 4.700",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 107.47,
            "range": "+/- 1.970",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 794.04,
            "range": "+/- 11.400",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 3.721,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 15.349,
            "range": "+/- 0.217",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 71.917,
            "range": "+/- 0.690",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 398.38,
            "range": "+/- 3.850",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 2.839,
            "range": "+/- 0.021",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 26.607,
            "range": "+/- 0.190",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 88.614,
            "range": "+/- 1.369",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 117.19,
            "range": "+/- 1.320",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "committer": {
            "email": "me@nathancorbyn.com",
            "name": "Nathan Corbyn",
            "username": "doctorn"
          },
          "distinct": true,
          "id": "03c026ad73c7090c51439ab4a497939924b26c29",
          "message": "Add line subdivider",
          "timestamp": "2021-09-21T21:13:54+01:00",
          "tree_id": "e12393f5ee08c3657413b55cd36111c9746e1d62",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/03c026ad73c7090c51439ab4a497939924b26c29"
        },
        "date": 1632259030083,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 91.394,
            "range": "+/- 0.735",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 93.599,
            "range": "+/- 0.648",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 139.19,
            "range": "+/- 2.560",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 149.61,
            "range": "+/- 1.140",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 240.48,
            "range": "+/- 2.950",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 299.8,
            "range": "+/- 3.980",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 122.26,
            "range": "+/- 2.400",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 885.96,
            "range": "+/- 7.320",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 4.0866,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 16.805,
            "range": "+/- 0.229",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 74.424,
            "range": "+/- 0.760",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 414.77,
            "range": "+/- 2.260",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 3.007,
            "range": "+/- 0.009",
            "unit": "s"
          },
          {
            "name": "contract high dimensions/9",
            "value": 27.743,
            "range": "+/- 0.060",
            "unit": "s"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 98.665,
            "range": "+/- 1.361",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 126.88,
            "range": "+/- 2.470",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "committer": {
            "email": "calintat@gmail.com",
            "name": "calintat",
            "username": "calintat"
          },
          "distinct": true,
          "id": "5f9e5a808a8f7f7e8e544681edffe9896d7fd129",
          "message": "Disable safety checks",
          "timestamp": "2021-09-22T12:55:37+01:00",
          "tree_id": "971a8a82e99d7e257eee61ff5df6537c7103a1a4",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/5f9e5a808a8f7f7e8e544681edffe9896d7fd129"
        },
        "date": 1632312678180,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 66.686,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 68.466,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 98.423,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 116.77,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 145.11,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 203.35,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 86.868,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 576.21,
            "range": "+/- 0.660",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 2.2633,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 6.9282,
            "range": "+/- 0.007",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 18.685,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 47.028,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 122.97,
            "range": "+/- 0.230",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 349.74,
            "range": "+/- 0.890",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 69.525,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 99.544,
            "range": "+/- 0.079",
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
          "id": "8311b874b698ea9efac3bed3ecc8af859984d35b",
          "message": "Partially fix #168: the highlight computation is still incorrect, but it\nno longer causes a panic",
          "timestamp": "2021-09-22T22:44:40+01:00",
          "tree_id": "93849500100fece443f6741b25d781d5d20a484e",
          "url": "https://github.com/homotopy-io/homotopy-rs/commit/8311b874b698ea9efac3bed3ecc8af859984d35b"
        },
        "date": 1632347710884,
        "tool": "criterion",
        "benches": [
          {
            "name": "contract scalar/left",
            "value": 55.687,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "contract scalar/right",
            "value": 56.758,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "contract beads/contract",
            "value": 81.44,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "contract beads/typecheck",
            "value": 97.032,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "contract stacks/contract",
            "value": 120.82,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract stacks/typecheck",
            "value": 171.53,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/2",
            "value": 70.541,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/3",
            "value": 480.05,
            "range": "+/- 1.050",
            "unit": "us"
          },
          {
            "name": "contract high dimensions/4",
            "value": 1.8872,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/5",
            "value": 5.7875,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/6",
            "value": 15.577,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/7",
            "value": 39.427,
            "range": "+/- 0.093",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/8",
            "value": 105.83,
            "range": "+/- 1.660",
            "unit": "ms"
          },
          {
            "name": "contract high dimensions/9",
            "value": 299.95,
            "range": "+/- 3.610",
            "unit": "ms"
          },
          {
            "name": "expand matchsticks/expand",
            "value": 56.649,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "expand matchsticks/typecheck",
            "value": 82.695,
            "range": "+/- 0.025",
            "unit": "us"
          }
        ]
      }
    ]
  }
}