window.BENCHMARK_DATA = {
  "lastUpdate": 1619653641015,
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
            "name": "",
            "value": 147.84,
            "range": "+/- 0.410",
            "unit": "us"
          },
          {
            "name": "",
            "value": 131.86,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "",
            "value": 161.49,
            "range": "+/- 0.440",
            "unit": "us"
          },
          {
            "name": "",
            "value": 115.09,
            "range": "+/- 0.220",
            "unit": "us"
          },
          {
            "name": "",
            "value": 760.25,
            "range": "+/- 2.790",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.1957,
            "range": "+/- 0.006",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 10.829,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 33.145,
            "range": "+/- 0.087",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 99.905,
            "range": "+/- 1.494",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 269.23,
            "range": "+/- 0.500",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 729.97,
            "range": "+/- 2.390",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 3.7431,
            "range": "+/- 0.010",
            "unit": "us"
          },
          {
            "name": "",
            "value": 38.829,
            "range": "+/- 0.020",
            "unit": "us"
          }
        ]
      }
    ]
  }
}