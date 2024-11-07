import {getData} from '#lib/redis/data'

export const GET = () => {
    console.log('got', getData())
}
