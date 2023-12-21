# urouter

Static (list of routes read once) http router for routing small domains.

## Installation

```shell
cargo install urouter
```

Edit `alias.json` (or any other JSON file, check `--alias-file` option) and `cargo run`

## `alias.json` specification

JSON file with array of sets (or set with one field of arrays of sets with `--alias-file-is-set-not-a-list`, may be useful i.e. [Nix packaging](https://github.com/ivabus/nixos/blob/master/roles/server/urouter.nix)).

Each set contains 2 necessary elements and 1 optional.

- Necessary
  - `uri` (string) - of url after host (e.g., `/`, `some/cool/path`, should not start with `/` (only for root))
  - `alias` (set) - set of one field
    - `url` (string) - redirect to url with HTTP 303 See Other
    - `file` (string) - read file from path `--dir/file` where `--dir` is option (default: `.`, see `--help`) and respond with HTTP 200 OK `content-type: text/plain; charset=utf-8`
    - `text` (string) - plain text
- Optional
  - `agent` (set) - set of one necessary field and one optional
    - `regex` (string) - regular expression to match user-agent HTTP header
    - `only_matching` (bool, optional, false by default) - if false whole alias will be visible for any user agent, if true only for regex matched

#### Set of array of sets

```json
{
  "alias": [
    {
      "uri":"/",
      "alias": {
        "url":  "https://somecoolwebsite"
      }
    }
  ]
}
```

### `alias.json` example

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
    "uri":"text",
    "alias": {
      "text": "sometext"
    }
  }
]
```

Agent matching made for `curl https://url | sh` like scripts.

## License

The project is licensed under the terms of the [MIT license](./LICENSE).
