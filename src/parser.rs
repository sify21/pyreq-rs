pub mod requirement_specifier;
pub mod version;

//pub fn pip_option(input: &str) -> IResult<&str, &str> {}
//pub fn requirement_specifier(input: &str) -> IResult<&str, &str> {}
//pub fn archive_url(input: &str) -> IResult<&str, &str> {}
//pub fn archive_path(input: &str) -> IResult<&str, &str> {}
//pub fn local_project_path(input: &str) -> IResult<&str, &str> {}
//pub fn vcs_project_url(input: &str) -> IResult<&str, &str> {}
//pub fn comment(input: &str) -> IResult<&str, &str> {}
//pub fn line(input: &str) -> IResult<&str, &str> {
//escaped(not_line_ending, '\\', line_ending)(input)
//}
//
#[cfg(test)]
mod tests {
    use super::requirement_specifier::specification;
    use super::version::version_scheme;
    use crate::requirements::{
        Comparison, LocalVersionPart, MarkerExpr, MarkerOp, RequirementSpecifier, Version,
    };

    #[test]
    fn test_requirement_specifier() {
        // samples from https://peps.python.org/pep-0508/
        // ```python
        // tests = [
        //     "A",
        //     "A.B-C_D",
        //     "aa",
        //     "name",
        //     "name<=1",
        //     "name>=3",
        //     "name>=3,<2",
        //     "name@http://foo.com",
        //     "name [fred,bar] @ http://foo.com ; python_version=='2.7'",
        //     "name[quux, strange];python_version<'2.7' and platform_version=='2'",
        //     "name; os_name=='a' or os_name=='b'",
        //     # Should parse as (a and b) or c
        //     "name; os_name=='a' and os_name=='b' or os_name=='c'",
        //     # Overriding precedence -> a and (b or c)
        //     "name; os_name=='a' and (os_name=='b' or os_name=='c')",
        //     # should parse as a or (b and c)
        //     "name; os_name=='a' or os_name=='b' and os_name=='c'",
        //     # Overriding precedence -> (a or b) and c
        //     "name; (os_name=='a' or os_name=='b') and os_name=='c'",
        // ]
        // ```
        assert_eq!(
            specification("A"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "A".to_string(),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("A.B-C_D"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "A.B-C_D".to_string(),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("aa"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "aa".to_string(),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name<=1"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    version_specs: vec![(Comparison::LessThanOrEqual, "1".to_string()).into()],
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name>=3"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    version_specs: vec![(Comparison::GreaterThanOrEqual, "3".to_string()).into()],
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name>=3,<2"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    version_specs: vec![
                        (Comparison::GreaterThanOrEqual, "3".to_string()).into(),
                        (Comparison::LessThan, "2".to_string()).into()
                    ],
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name@http://foo.com"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    urlspec: Some("http://foo.com".to_string()),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name [fred,bar] @ http://foo.com ; python_version=='2.7'"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    extras: vec!["fred".to_string(), "bar".to_string()],
                    urlspec: Some("http://foo.com".to_string()),
                    marker_expr: Some(MarkerExpr::Basic(
                        "python_version".to_string(),
                        MarkerOp::Comparison(Comparison::Equal),
                        "2.7".to_string()
                    )),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name[quux, strange];python_version<'2.7' and platform_version=='2'"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    extras: vec!["quux".to_string(), "strange".to_string()],
                    marker_expr: Some(MarkerExpr::And(
                        Box::new(MarkerExpr::Basic(
                            "python_version".to_string(),
                            MarkerOp::Comparison(Comparison::LessThan),
                            "2.7".to_string()
                        )),
                        Box::new(MarkerExpr::Basic(
                            "platform_version".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "2".to_string()
                        ))
                    )),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name; os_name=='a' or os_name=='b'"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    marker_expr: Some(MarkerExpr::Or(
                        Box::new(MarkerExpr::Basic(
                            "os_name".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "a".to_string()
                        )),
                        Box::new(MarkerExpr::Basic(
                            "os_name".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "b".to_string()
                        ))
                    )),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name; os_name=='a' and os_name=='b' or os_name=='c'"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    marker_expr: Some(MarkerExpr::Or(
                        Box::new(MarkerExpr::And(
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "a".to_string()
                            )),
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "b".to_string()
                            ))
                        )),
                        Box::new(MarkerExpr::Basic(
                            "os_name".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "c".to_string()
                        ))
                    )),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name; os_name=='a' and (os_name=='b' or os_name=='c')"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    marker_expr: Some(MarkerExpr::And(
                        Box::new(MarkerExpr::Basic(
                            "os_name".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "a".to_string()
                        )),
                        Box::new(MarkerExpr::Or(
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "b".to_string()
                            )),
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "c".to_string()
                            ))
                        ))
                    )),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name; os_name=='a' or os_name=='b' and os_name=='c'"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    marker_expr: Some(MarkerExpr::Or(
                        Box::new(MarkerExpr::Basic(
                            "os_name".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "a".to_string()
                        )),
                        Box::new(MarkerExpr::And(
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "b".to_string()
                            )),
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "c".to_string()
                            ))
                        ))
                    )),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            specification("name; (os_name=='a' or os_name=='b') and os_name=='c'"),
            Ok((
                "",
                RequirementSpecifier {
                    name: "name".to_string(),
                    marker_expr: Some(MarkerExpr::And(
                        Box::new(MarkerExpr::Or(
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "a".to_string()
                            )),
                            Box::new(MarkerExpr::Basic(
                                "os_name".to_string(),
                                MarkerOp::Comparison(Comparison::Equal),
                                "b".to_string()
                            ))
                        )),
                        Box::new(MarkerExpr::Basic(
                            "os_name".to_string(),
                            MarkerOp::Comparison(Comparison::Equal),
                            "c".to_string()
                        ))
                    )),
                    ..Default::default()
                }
            ))
        );
    }

    #[test]
    fn test_version_scheme() {
        // samples from https://peps.python.org/pep-0440/#examples-of-compliant-version-schemes
        assert_eq!(
            version_scheme("0.1"),
            Ok((
                "",
                Version {
                    epoch: 0,
                    release: vec![0, 1],
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            version_scheme("1.1.0"),
            Ok((
                "",
                Version {
                    epoch: 0,
                    release: vec![1, 1, 0],
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            version_scheme("1.1a1"),
            Ok((
                "",
                Version {
                    epoch: 0,
                    release: vec![1, 1],
                    pre: Some(("a".to_string(), 1)),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            version_scheme("1.1.0.PoSt1"),
            Ok((
                "",
                Version {
                    epoch: 0,
                    release: vec![1, 1, 0],
                    post: Some(("post".to_string(), 1)),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            version_scheme("3!1.1.2-Beta3+Ubuntu.3-release"),
            Ok((
                "",
                Version {
                    epoch: 3,
                    release: vec![1, 1, 2],
                    pre: Some(("b".to_string(), 3)),
                    local: Some(vec![
                        LocalVersionPart::LowerStr("ubuntu".to_string()),
                        LocalVersionPart::Num(3),
                        LocalVersionPart::LowerStr("release".to_string()),
                    ]),
                    ..Default::default()
                }
            ))
        );
    }
}
