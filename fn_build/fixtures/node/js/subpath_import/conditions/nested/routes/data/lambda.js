import {getData} from '#data'
import {getLog} from '#log'

export const GET = () => {
    console[getLog()]('got', getData())
}
