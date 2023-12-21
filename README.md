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
    "uri":"/",
    "alias": {
      "url":  "https://somecoolwebsite"
    }
  },
  {
    "uri":"/",
    "alias": {
      "file": "somecoolscript"
    },
    "agent": {
      "regex": "^curl/[0-9].[0-9].[0-9]$",
      "only_matching": false
    }
  },
  {
    "uri":"/text",
    "alias": {
      "text": "sometext"
    }
  }
]
```

Agent matching made for `curl https://url | sh` like scripts.

## License

The project is licensed under the terms of the [MIT license](./LICENSE).
