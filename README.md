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

`l3` will deploy any `lambda.js` or `lambda.mjs` modules in the project's `./routes` directory with functions `DELETE`, `GET`, `PATCH`, `POST` or `PUT` as HTTP APIs via AWS API Gateway and Lambda services.

```shell
l3 sync
```
