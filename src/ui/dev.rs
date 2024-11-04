use crate::notification::{
    LambdaEventKind, LambdaNotification, LambdaUpdateKind, LambdaUpdateResult,
};

pub fn print_notification(notification: &LambdaNotification) {
    match notification {
        LambdaNotification::Lambda(lambda_event) => match &lambda_event.kind {
            LambdaEventKind::Creating => {
                println!(
                    "Creating lambda fn {}",
                    lambda_event.lambda_fn.route_key.to_route_key_string()
                );
            }
            LambdaEventKind::Created(result) => match result {
                LambdaUpdateResult::Success => {
                    println!(
                        "Created lambda fn {}",
                        lambda_event.lambda_fn.route_key.to_route_key_string()
                    );
                }
                LambdaUpdateResult::Failure(error_msg) => {
                    println!(
                        "Failed creating lambda fn {}: {error_msg}",
                        lambda_event.lambda_fn.route_key.to_route_key_string()
                    );
                }
            },
            LambdaEventKind::Removing => {
                println!(
                    "Removing lambda fn {}",
                    lambda_event.lambda_fn.route_key.to_route_key_string()
                );
            }
            LambdaEventKind::Removed(result) => match result {
                LambdaUpdateResult::Success => {
                    println!(
                        "Removed lambda fn {}",
                        lambda_event.lambda_fn.route_key.to_route_key_string()
                    );
                }
                LambdaUpdateResult::Failure(error_msg) => {
                    println!(
                        "Failed removing lambda fn {}: {error_msg}",
                        lambda_event.lambda_fn.route_key.to_route_key_string()
                    );
                }
            },
            LambdaEventKind::Updating(kind) => {
                println!(
                    "Updating lambda fn {} {}",
                    lambda_event.lambda_fn.route_key.to_route_key_string(),
                    match kind {
                        LambdaUpdateKind::Code => "code",
                        LambdaUpdateKind::Dependencies => "dependencies",
                        LambdaUpdateKind::Env => "env",
                    }
                );
            }
            LambdaEventKind::Updated(kind, result) => match result {
                LambdaUpdateResult::Success => {
                    println!(
                        "Updated lambda fn {} {}",
                        lambda_event.lambda_fn.route_key.to_route_key_string(),
                        match kind {
                            LambdaUpdateKind::Code => "code",
                            LambdaUpdateKind::Dependencies => "dependencies",
                            LambdaUpdateKind::Env => "env",
                        }
                    );
                }
                LambdaUpdateResult::Failure(error_msg) => {
                    println!(
                        "Failed updating lambda fn {} {}: {error_msg}",
                        lambda_event.lambda_fn.route_key.to_route_key_string(),
                        match kind {
                            LambdaUpdateKind::Code => "code",
                            LambdaUpdateKind::Dependencies => "dependencies",
                            LambdaUpdateKind::Env => "env",
                        }
                    );
                }
            },
        },
        LambdaNotification::Log(log_event) => {
            println!(
                "{:?} {}: {}",
                log_event.timestamp,
                log_event.lambda_fn.route_key.to_route_key_string(),
                log_event.message
            );
        }
        _ => {}
    }
}
