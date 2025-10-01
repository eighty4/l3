use std::collections::HashMap;

use crate::merge::merge_env_vars;

#[test]
fn test_merge_env_vars_without_merging() {
    assert_eq!(merge_env_vars(Vec::new()), HashMap::new());
    assert_eq!(
        merge_env_vars(vec![Some(HashMap::from([(
            "GREETING".into(),
            "goodbye".into()
        )]))]),
        HashMap::from([("GREETING".into(), "goodbye".into())])
    );
}

#[test]
fn test_merge_env_vars() {
    assert_eq!(
        merge_env_vars(vec![
            Some(HashMap::from([
                ("GREETING".into(), "yo".into()),
                ("SALUTATION".into(), "sup".into()),
            ])),
            Some(HashMap::from([("GREETING".into(), "goodbye".into())])),
        ]),
        HashMap::from([
            ("GREETING".into(), "goodbye".into()),
            ("SALUTATION".into(), "sup".into())
        ])
    );
}
