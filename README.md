# l3

## Install from source

The best path for getting started with `l3` is to build from source with `cargo`:

```shell
cargo install --path .
```

## Create a project

This command will generate a config file, a routes directory for HTTP lambdas and an example lambda.

```shell
l3 init
```

## Sync to AWS

`l3` will deploy any `delete.js`, `get.js`, `patch.js`, `post.js` and `put.js` JavaScript or `mjs` modules as lambdas.

```shell
l3 sync
```
