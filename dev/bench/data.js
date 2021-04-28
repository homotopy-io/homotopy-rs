window.BENCHMARK_DATA = {
  "lastUpdate": 1619651671020,
  "repoUrl": "https://github.com/homotopy-io/homotopy-rs",
  "entries": {
    "Rust Benchmark": [
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
            "name": "",
            "value": 130.95,
            "range": "+/- 3.720",
            "unit": "us"
          },
          {
            "name": "",
            "value": 131.8,
            "range": "+/- 7.940",
            "unit": "us"
          },
          {
            "name": "",
            "value": 155.87,
            "range": "+/- 5.690",
            "unit": "us"
          },
          {
            "name": "",
            "value": 109.99,
            "range": "+/- 3.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 794.03,
            "range": "+/- 71.010",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.3096,
            "range": "+/- 0.252",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 10.379,
            "range": "+/- 0.295",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 30.449,
            "range": "+/- 0.614",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 93.492,
            "range": "+/- 3.040",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 272.42,
            "range": "+/- 8.560",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 703.19,
            "range": "+/- 17.300",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 2.9664,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "",
            "value": 35.084,
            "range": "+/- 1.868",
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
            "name": "",
            "value": 145.85,
            "range": "+/- 0.820",
            "unit": "us"
          },
          {
            "name": "",
            "value": 135.47,
            "range": "+/- 1.470",
            "unit": "us"
          },
          {
            "name": "",
            "value": 169.67,
            "range": "+/- 0.570",
            "unit": "us"
          },
          {
            "name": "",
            "value": 120.59,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "",
            "value": 826.98,
            "range": "+/- 0.940",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.669,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.944,
            "range": "+/- 0.085",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 65.629,
            "range": "+/- 0.272",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 350.17,
            "range": "+/- 1.640",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 2.2825,
            "range": "+/- 0.015",
            "unit": "s"
          },
          {
            "name": "",
            "value": 16.896,
            "range": "+/- 0.142",
            "unit": "s"
          },
          {
            "name": "",
            "value": 3.4466,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "",
            "value": 34.698,
            "range": "+/- 0.398",
            "unit": "us"
          }
        ]
      }
    ]
  }
}