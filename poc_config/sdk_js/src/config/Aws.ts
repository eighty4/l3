import {LambdaPlatform} from './LambdaPlatform'

export class ApiGateway {
    readonly tag: string = 'api-gateway'
}

export class LoadBalancer {
    readonly tag: string = 'application-load-balancer'
}

export type HttpRouting = ApiGateway | LoadBalancer

export class Platform extends LambdaPlatform<HttpRouting> {
    httpRouting(routing: HttpRouting) {
    }
}
