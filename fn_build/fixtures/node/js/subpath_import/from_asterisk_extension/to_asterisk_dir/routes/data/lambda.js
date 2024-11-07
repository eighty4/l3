import {getData} from '#lib/data/redis.js'

export const GET = () => {
    console.log('got', getData())
}
