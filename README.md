# urouter 

Fork of [ivabus/aliurl](https://github.com/ivabus/aliurl) that routes statically.

## Installation

```shell
git clone https://github.com/ivabus/urouter
cd urouter
```

Edit `alias.json` and `cargo run`

## `alias.json` example

```json
[
  {
    "uri": "uri",
    "alias": "file"
  },
  {
    "uri": "uri2",
    "alias": "http://example.com",
    "is_url": true
  }
]
```

## License

The project is licensed under the terms of the [MIT license](./LICENSE).%