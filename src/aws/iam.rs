pub async fn get_account_id(iam: &aws_sdk_iam::Client) -> Result<String, anyhow::Error> {
    let user_arn = iam.get_user().send().await.unwrap().user.unwrap().arn;
    Ok(user_arn.split(':').nth(4).unwrap().to_string())
}
