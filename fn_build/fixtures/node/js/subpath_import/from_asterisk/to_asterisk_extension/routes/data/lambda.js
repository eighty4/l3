import {getData} from '#lib/data/raw'
import {getComplexData} from '#lib/data/abstraction/orm'

export const GET = () => {
    console.log('got', getData())
    console.log('getting', getComplexData())
}
