import {getData} from '#lib/redis.js'

export const GET = () => {
    console.log('got', getData())
}
