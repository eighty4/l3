use crate::aws::resources::parse_fn_name_from_arn;

#[test]
fn test_fn_arn_to_fn_name() {
    let arn = "arn:aws:lambda:us-east-2:1234:function:l3-install_sh-asdf-get".to_string();
    assert_eq!(parse_fn_name_from_arn(&arn), "l3-install_sh-asdf-get");
}
