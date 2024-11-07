import {getData} from '#lib/data/api.js'

export const GET = () => {
    console.log('got', getData())
}
