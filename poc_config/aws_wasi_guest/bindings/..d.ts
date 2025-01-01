export type ApiInstance = ApiInstanceCreateNew | ApiInstanceUseApiId;
export interface ApiInstanceCreateNew {
  tag: 'create-new',
}
export interface ApiInstanceUseApiId {
  tag: 'use-api-id',
  val: string,
}
export interface ApiGatewayConfig {
  api: ApiInstance,
}
export interface LoadBalancerConfig {
}
export type CloudfrontOrigin = CloudfrontOriginApiGateway | CloudfrontOriginApplicationLoadBalancer;
export interface CloudfrontOriginApiGateway {
  tag: 'api-gateway',
  val: ApiGatewayConfig,
}
export interface CloudfrontOriginApplicationLoadBalancer {
  tag: 'application-load-balancer',
  val: LoadBalancerConfig,
}
export interface CloudfrontConfig {
  origin: CloudfrontOrigin,
}
export type HttpConfig = HttpConfigApiGateway | HttpConfigApplicationLoadBalancer | HttpConfigCloudfrontOrigin;
export interface HttpConfigApiGateway {
  tag: 'api-gateway',
  val: ApiGatewayConfig,
}
export interface HttpConfigApplicationLoadBalancer {
  tag: 'application-load-balancer',
  val: LoadBalancerConfig,
}
export interface HttpConfigCloudfrontOrigin {
  tag: 'cloudfront-origin',
  val: CloudfrontConfig,
}
export function configure(): HttpConfig;
