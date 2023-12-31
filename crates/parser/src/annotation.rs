// @some
// @some1("value")
// @some2("value", "value2")
// @some3([1, 2, 5])
// @some4({
//    "key": "value",
// })

use nom::bytes::complete::{tag, take_while};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, terminated};

use crate::{eson_literal, EsonLiteralSegment, sp};
use crate::legal_id;

#[derive(Debug, PartialEq)]
pub struct Annotation {
    pub name: String,
    pub value: Option<Vec<EsonLiteralSegment>>,
}

fn annotation(input: &str) -> nom::IResult<&str, Annotation, VerboseError<&str>> {
    let (remaining, _) = tag("@")(input)?;
    let (remaining, name) = terminated(legal_id, sp_without_br0)(remaining)?;
    let (remaining, value) = opt(preceded(
        delimited(sp_without_br0, tag("("), sp_without_br0),
        delimited(
            multispace0,
            separated_list0(delimited(multispace0, tag(","), multispace0), eson_literal),
            delimited(multispace0, tag(")"), sp_without_br0),
        ),
    ))(remaining)?;
    Ok((
        remaining,
        Annotation {
            name: name.to_string(),
            value,
        },
    ))
}

pub(crate) fn parse_annotations(
    input: &str,
) -> nom::IResult<&str, Vec<Annotation>, VerboseError<&str>> {
    let (remaining, _) = multispace0(input)?;
    let (remaining, annotations) = many0(delimited(sp, annotation, sp))(remaining)?;
    Ok((remaining, annotations))
}

fn sp_without_br0(input: &str) -> nom::IResult<&str, &str, VerboseError<&str>> {
    let chars = " \t\r";
    map(take_while(move |c| chars.contains(c)), |_s| "")(input)
}

#[cfg(test)]
mod tests {
    use nom::character::complete::{alphanumeric1, char};
    use nom::IResult;

    use crate::EsonLiteralSegment;

    use super::*;

    #[test]
    fn test_devel() {
        let s = r#"
            // foo annotation
            @foo
        "#;
        assert_eq!(sp(s), Ok(("@foo\n        ", "")));

        assert_eq!(
            annotation("@foo\n        "),
            Ok((
                "\n        ",
                Annotation {
                    name: "foo".to_string(),
                    value: None,
                }
            ))
        );

        assert_eq!(
            preceded(sp, annotation)(s),
            Ok((
                "\n        ",
                Annotation {
                    name: "foo".to_string(),
                    value: None,
                }
            ))
        );

        let (remaining, _) = multispace0::<&str, VerboseError<&str>>(s).unwrap();
        // dbg!(remaining);

        let (remaining, annotations) =
            separated_list0(char('\n'), delimited(sp, annotation, sp))(remaining).unwrap();
        // dbg!(remaining, annotations);
        assert_eq!(remaining, "");
        assert_eq!(
            annotations,
            vec![Annotation {
                name: "foo".to_string(),
                value: None,
            }]
        );

        let s = r#"
            // foo annotation
            @foo
            @bar
        "#;
        let (remaining, _) = multispace0::<&str, VerboseError<&str>>(s).unwrap();
        let (remaining, annotations) = many0(delimited(sp, annotation, sp))(remaining).unwrap();
        // separated_list0(char('\n'), delimited(sp, annotation, sp))(s).unwrap();
        // dbg!(remaining, annotations);
        assert_eq!(remaining, "");
        assert_eq!(
            annotations,
            vec![
                Annotation {
                    name: "foo".to_string(),
                    value: None,
                },
                Annotation {
                    name: "bar".to_string(),
                    value: None,
                },
            ]
        );
    }

    #[test]
    fn test_annotation_after_comment() {
        let s = r###"
            // foo annotation
            @foo
        "###;
        assert_eq!(
            parse_annotations(s),
            Ok((
                "",
                vec![Annotation {
                    name: "foo".to_string(),
                    value: None,
                }]
            ))
        );
    }

    #[test]
    fn test_annotation3() {
        let s = "        @world";
        assert_eq!(
            parse_annotations(s),
            Ok((
                "",
                Vec::<Annotation>::from(vec![Annotation {
                    name: "world".to_string(),
                    value: None,
                }])
            ))
        );
    }

    #[test]
    fn test_annotation() {
        assert_eq!(
            annotation("@DEF\n"),
            Ok((
                "\n",
                Annotation {
                    name: "DEF".to_string(),
                    value: None,
                }
            ))
        );

        assert_eq!(
            parse_annotations("@DEF\n"),
            Ok((
                "",
                Vec::<Annotation>::from(vec![Annotation {
                    name: "DEF".to_string(),
                    value: None,
                }])
            ))
        );
    }

    #[test]
    fn test_sp_without_br0() {
        assert_eq!(sp_without_br0("   "), Ok(("", "")));
        assert_eq!(sp_without_br0(" \t \r  "), Ok(("", "")));
        assert_eq!(sp_without_br0(""), Ok(("", "")));
        assert_eq!(sp_without_br0("   \n"), Ok(("\n", "")));
        assert_eq!(sp_without_br0("   \n  "), Ok(("\n  ", "")));
    }

    #[test]
    fn test_sp() {
        fn parser(s: &str) -> IResult<&str, Vec<&str>, VerboseError<&str>> {
            separated_list0(
                char('\n'),
                delimited(sp_without_br0, alphanumeric1, sp_without_br0),
            )(s)
        }
        assert_eq!(parser("   ABC  \n  DEF"), Ok(("", vec!["ABC", "DEF"])));
    }

    #[test]
    fn test_sep_annotation() {
        fn parser(s: &str) -> IResult<&str, Vec<Annotation>, VerboseError<&str>> {
            separated_list0(
                char('\n'),
                delimited(sp_without_br0, annotation, sp_without_br0),
            )(s)
        }

        assert_eq!(
            parser("   @ABC  \n  @DEF "),
            Ok((
                "",
                vec![
                    Annotation {
                        name: "ABC".to_string(),
                        value: None,
                    },
                    Annotation {
                        name: "DEF".to_string(),
                        value: None,
                    },
                ]
            ))
        );
    }

    #[test]
    fn test_annotations() {
        let anno_str = r##"
            @some
            @some1("value")
            @some2("value", "value2")
        "##;

        assert_eq!(
            parse_annotations(anno_str),
            Ok((
                "",
                vec![
                    Annotation {
                        name: "some".to_string(),
                        value: None,
                    },
                    Annotation {
                        name: "some1".to_string(),
                        value: Some(vec![EsonLiteralSegment::Str("value".to_string())]),
                    },
                    Annotation {
                        name: "some2".to_string(),
                        value: Some(vec![
                            EsonLiteralSegment::Str("value".to_string()),
                            EsonLiteralSegment::Str("value2".to_string()),
                        ]),
                    },
                ]
            ))
        );
    }

    #[test]
    fn test_annotation2() {
        assert_eq!(
            annotation("@DEF   "),
            Ok((
                "",
                Annotation {
                    name: "DEF".to_string(),
                    value: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            annotation("@some"),
            Ok((
                "",
                Annotation {
                    name: "some".to_string(),
                    value: None,
                }
            ))
        );
        assert_eq!(
            annotation("@some()"),
            Ok((
                "",
                Annotation {
                    name: "some".to_string(),
                    value: Some(vec![]),
                }
            ))
        );
        assert_eq!(
            annotation("@some(1, 2, 3)"),
            Ok((
                "",
                Annotation {
                    name: "some".to_string(),
                    value: Some(vec![
                        EsonLiteralSegment::Int(1),
                        EsonLiteralSegment::Int(2),
                        EsonLiteralSegment::Int(3),
                    ]),
                }
            ))
        );
        assert_eq!(
            annotation("@some(1, 2, 3, 4)"),
            Ok((
                "",
                Annotation {
                    name: "some".to_string(),
                    value: Some(vec![
                        EsonLiteralSegment::Int(1),
                        EsonLiteralSegment::Int(2),
                        EsonLiteralSegment::Int(3),
                        EsonLiteralSegment::Int(4),
                    ]),
                }
            ))
        );
        assert_eq!(
            annotation(r#"@some("foo", "bar")"#),
            Ok((
                "",
                Annotation {
                    name: "some".to_string(),
                    value: Some(vec![
                        EsonLiteralSegment::Str("foo".to_string()),
                        EsonLiteralSegment::Str("bar".to_string()),
                    ]),
                }
            ))
        );
    }
}
