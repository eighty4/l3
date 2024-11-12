import {getData} from '../../lib/api.js'

export const GET = () => {
    console.log('got', getData())
}
