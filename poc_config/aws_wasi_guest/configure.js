/**
 * @return {import('./bindings/..d.ts').HttpConfig}
 */
export function configure() {
    return {
        tag: 'api-gateway',
        val: {
            api: {
                tag: 'create-new',
            },
        },
    }
}
