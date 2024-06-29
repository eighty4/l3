pub type FunctionArn = String;
pub type FunctionName = String;
pub type IntegrationId = String;
pub type RouteId = String;

pub fn parse_fn_name_from_arn(fn_arn: &str) -> String {
    fn_arn.split(':').last().unwrap().to_string()
}
