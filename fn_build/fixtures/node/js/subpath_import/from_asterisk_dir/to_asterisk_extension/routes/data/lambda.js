import {getData} from '#lib/redis/data.js'

export const GET = () => {
    console.log('got', getData())
}
