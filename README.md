# urouter 

Static (list of routes read once) http router for routing small domains.

## Installation

```shell
git clone https://github.com/ivabus/urouter
cd urouter
```

Edit `alias.json` (or any other JSON file, check `--alias-file` option) and `cargo run`

## `alias.json` example

```json
[
  {
    "uri": "uri",
    "alias": {
      "file":  "somefile"
    }
  },
  {
    "uri": "uri2",
    "alias": {
      "url": "http://example.com"
    }
  },
  {
    "uri": "/",
    "alias": {
      "url": "https://somecoolscript.sh"
    },
    "curl_only": true
  }
]
```

`"curl_only"` thing for `curl https://url | sh` like scripts.

## License

The project is licensed under the terms of the [MIT license](./LICENSE).
