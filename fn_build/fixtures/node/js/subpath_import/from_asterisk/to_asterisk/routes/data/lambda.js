import {getData} from '#lib/data/raw.js'
import {getComplexData} from '#lib/data/abstraction/orm.js'

export const GET = () => {
    console.log('got', getData())
    console.log('getting', getComplexData())
}
