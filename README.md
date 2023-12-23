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

- `uri` (string) - of URL after host (e.g., `/`, `some/cool/path`, should not start with `/` (only for root))
- `alias` (set) - set of one field
  - `url` (string) - redirect to URL with HTTP 303 See Other
  - `file` (string) - read file from path `--dir/file` where `--dir` is option (default: `.`, see `--help`) and respond with HTTP 200 OK with `content-type: text/plain`
  - `text` (string) - plain text with HTTP 200 OK with `content-type: text/plain`
  - `external` (set) - download (every time) file using `ureq` HTTP library and response with contents of downloaded resource with HTTP 200 OK and extracted `content-type` from response
    - `url` (string) - URL to download
    - `headers` (set, optional) - headers to include with request
- `agent` (set, optional) - set of one necessary field and one optional
  - `regex` (string) - regular expression to match user-agent HTTP header
  - `only_matching` (bool, optional, false by default) - if false whole alias will be visible for any user agent, if true only for regex matched

#### Set of array of sets (use only for very specific workarounds)

```json
{
  "alias": [
    {
      "uri": "/",
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
    "uri": "/",
    "alias": {
      "url":  "https://somecoolwebsite"
    }
  },
  {
    "uri": "/",
    "alias": {
      "file": "somecoolscript"
    },
    "agent": {
      "regex": "^curl/[0-9].[0-9].[0-9]$",
      "only_matching": false
    }
  },
  {
    "uri": "text",
    "alias": {
      "text": "sometext"
    }
  },
  {
    "uri": "external",
    "alias": {
      "external": {
        "url": "https://somecool.external.link",
        "headers": {
          "user-agent": "curl/8.6.0"
        }
      }
    }
  }
]
```

Agent matching made for `curl https://url | sh` like scripts.

## `alias.json` location

- Passed with `--alias_file`, will look up to this path, if file doesn't exist (or no access to it) will panic
- If urouter started with privileges (EUID = 0), file would be `/etc/urouter/alias.json`
- Otherwise if `XDG_CONFIG_HOME` is set, file would be `$XDG_CONFIG_HOME/urouter/alias.json`
- Otherwise if `$HOME` is set, file would be `$HOME/.config/urouter/alias.json`
- If not matched any, will panic and exit

## License

The project is licensed under the terms of the [MIT license](./LICENSE).
