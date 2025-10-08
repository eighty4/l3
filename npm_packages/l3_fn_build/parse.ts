import { buildFn, parseFn, parseEntrypoint } from "./gen/l3_fn_build.js"

const entrypoint = parseEntrypoint({
    entrypoint: 'routes/data/lambda.js',
    projectDir: '/Users/adam/work/eighty4/l3/fn_build/fixtures/node/js/http_routes/delete_fn',
    runtime: 'node',
})

console.log('parse_entrypoint:', JSON.stringify(entrypoint, null, 4))

const parseManifest = parseFn({
    entrypoint: 'routes/data/lambda.js',
    projectDir: '/Users/adam/work/eighty4/l3/fn_build/fixtures/node/js/http_routes/delete_fn',
    runtime: 'node',
})

console.log('parse_fn:', JSON.stringify(parseManifest, null, 4))

const buildManifest = buildFn({
    mode: 'debug',
    projectDir: '/Users/adam/work/eighty4/l3/fn_build/fixtures/node/js/http_routes/delete_fn',
    handlerFnName: 'DELETE',
    runtime: 'node',
    entrypoint: 'routes/data/lambda.js',
    output: {
        buildRoot: '.l3',
        createArchive: true,
        dirname: 'my-sweet-lambda',
        useBuildMode: true,
    }
})

console.log('build_fn:', JSON.stringify(buildManifest, null, 4))
