import {isModuleSync} from '#not-module-sync'
import {isRequire} from '#not-require'

export const GET = () => {
    console.log(isModuleSync(), isRequire())
}
