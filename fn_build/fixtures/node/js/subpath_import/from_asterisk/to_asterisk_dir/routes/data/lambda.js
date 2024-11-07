import {getData} from '#lib/data/redis'

export const GET = () => {
    console.log('got', getData())
}
