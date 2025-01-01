## validate wit file

`cat ./aws.wit | wasm-tools component wit`

## generate js types

npx @bytecodealliance/jco types ./ -o bindings

## build component
