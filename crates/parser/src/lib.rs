use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str;

use nom::{
    branch::alt,
    combinator::{map, opt},
    error::VerboseError,
    IResult,
    sequence::{delimited, preceded},
};
use nom::character::complete::multispace1;
use nom::multi::many0;

pub use annotation::Annotation;

use crate::annotation::parse_annotations;
use crate::boolean::{parse_boolean, parse_literal_boolean};
use crate::comments::comment;
use crate::dict::{Key, parse_dict, parse_literal_dict};
use crate::expr::legal_id;
use crate::expr_token::{chunk::ExprTokenChunk, parse_expr_token_chunk};
use crate::list::{parse_literal_lst, parse_lst};
use crate::null::{parse_literal_null, parse_null};
use crate::numeric::{parse_literal_number, parse_numeric};
use crate::string::{parse_literal_string, parse_string};

mod annotation;
mod boolean;
mod comments;
mod dict;
mod expr;
mod expr_token;
mod list;
mod null;
mod numeric;
mod string;
mod util;

#[derive(Debug, PartialEq)]
pub enum EsonSegment {
    Null,
    Str(String),
    Boolean(bool),
    Int(i64),
    Float(f64),
    List(Vec<EsonSegment>),
    Dict(HashMap<Key, EsonSegment>),
    Expr(ExprTokenChunk),
}

#[derive(Debug, PartialEq)]
pub enum EsonLiteralSegment {
    Null,
    Str(String),
    Boolean(bool),
    Int(i64),
    Float(f64),
    List(Vec<EsonLiteralSegment>),
    Dict(HashMap<Key, EsonLiteralSegment>),
}

pub(crate) fn sp(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    // let chars = " \t\r\n";
    // take_while(move |c| chars.contains(c))(input)
    map(many0(alt((multispace1, comment))), |_v| "")(input)
}

pub fn eson(i: &str) -> IResult<&str, EsonSegment, VerboseError<&str>> {
    preceded(
        sp,
        alt((
            map(parse_string, EsonSegment::Str),
            map(parse_numeric, |n| n),
            map(parse_boolean, |b| b),
            map(parse_null, |_| EsonSegment::Null),
            map(parse_lst, EsonSegment::List),
            map(parse_dict, EsonSegment::Dict),
            map(parse_expr_token_chunk, EsonSegment::Expr),
        )),
    )(i)
}

pub fn eson_literal(i: &str) -> IResult<&str, EsonLiteralSegment, VerboseError<&str>> {
    preceded(
        sp,
        alt((
            map(parse_literal_number, |n| n),
            map(parse_literal_boolean, |b| b),
            map(parse_literal_null, |_| EsonLiteralSegment::Null),
            map(parse_literal_string, EsonLiteralSegment::Str),
            map(parse_literal_lst, EsonLiteralSegment::List),
            map(parse_literal_dict, EsonLiteralSegment::Dict),
        )),
    )(i)
}

/// the root element of a JSON parser is either an object or an array
pub fn root(input: &str) -> IResult<&str, EsonSegment, VerboseError<&str>> {
    delimited(
        sp,
        alt((
            // map(hash, EsonValue::Lit),
            // map(array, EsonValue::Lit),
            map(parse_null, |_| EsonSegment::Null),
        )),
        opt(sp),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::EsonLiteralSegment::Dict;
    use crate::string::parse_string;

    use super::*;

    #[test]
    fn test_eson() {
        let dat = r###"
        {
            // hello
            @hello
            "c": {},
            "d": "bar",
        }
        "###;

        // todo
        dbg!(eson(dat));
    }

    #[test]
    fn test_eson2() {
        let data = "1";
        assert_eq!(eson(data), Ok(("", EsonSegment::Int(1))));
    }

    #[test]
    fn test_comment_annotation() {
        let dat = r###"
        // hello
        @hello
        "c": {},
        "###;

        fn parse(s: &str) -> IResult<&str, EsonLiteralSegment, VerboseError<&str>> {
            // dbg!(s);
            // let (rem, comment) = preceded(multispace0, comment)(s)?;
            // dbg!(rem, comment);
            // let (rem, annotation) = parse_annotations(rem)?;
            // dbg!(rem, annotation);

            let (rem, comment) = sp(s)?;
            dbg!(rem, comment);
            let (rem, annotation) = parse_annotations(rem)?;
            dbg!(rem, annotation);

            Ok((rem, EsonLiteralSegment::Null))
            // annotation(s)
        }
        // parse(dat);
    }

    #[test]
    fn test_map1() {
        let dat = r###"
        {
            // foo annotation
            @foo


            "c": "hello",
            "d": "bar",
        }
        "###;

        // dbg!(root(dat));
        // dbg!(hash(dat));

        let r = Dict(
            vec![(
                Key {
                    name: "c".to_string(),
                    annotation: Some(vec![Annotation {
                        name: "foo".to_string(),
                        value: None,
                    }]),
                },
                EsonLiteralSegment::Str("hello".to_string()),
            )]
                .into_iter()
                .collect(),
        );

        // assert_eq!(root(dat), Ok(("", r)));

        // assert_eq!(
        //     root(dat),
        //     Ok((
        //         "",
        //         JsonValue::Object(
        //             vec![
        //                 ("c".into(), JsonValue::Str("hello".to_string())),
        //             ]
        //             .into_iter()
        //             .collect()
        //         )
        //     ))
        // );
    }

    #[test]
    fn test_root0() {
        let dat = r###"
        {
        // hello
            @hello
            "c": {},
            "d": "bar",
        }
        "###;

        // dbg!(root(dat));

        // assert_eq!(
        //     root(dat),
        //     Ok((
        //         "",
        //         JsonValue::Object(
        //             vec![(
        //                 Key {
        //                     name: "c".to_string(),
        //                     annotation: Some(vec![Annotation {
        //                         name: "hello".to_string(),
        //                         value: None
        //                     }])
        //                 },
        //                 JsonValue::Object(HashMap::new())
        //             )]
        //             .into_iter()
        //             .collect()
        //         )
        //     ))
        // );
    }

    #[test]
    fn test_root() {
        let dat = r###"
        // comment1
        {
            "a": 42,
            "b": [
                "x", // 注释2
                "y",
                12
            ],
            // 注释3
            @hello
            "c": {
                "hello": "world",
                "foo": r"bar",
                "bar": f"hello ${name}",
            }
        }
        "###;

        // assert_eq!(
        //     root(dat),
        //     Ok((
        //         "",
        //         EsonLiteral::Object(
        //             vec![
        //                 ("a".into(), EsonLiteral::Int(42)),
        //                 (
        //                     "b".into(),
        //                     EsonLiteral::Array(vec![
        //                         EsonLiteral::Str("x".to_string()),
        //                         EsonLiteral::Str("y".to_string()),
        //                         EsonLiteral::Int(12),
        //                     ])
        //                 ),
        //                 (
        //                     Key {
        //                         name: "c".to_string(),
        //                         annotation: Some(vec![Annotation {
        //                             name: "hello".to_string(),
        //                             value: None,
        //                         }]),
        //                     },
        //                     EsonLiteral::Object(
        //                         vec![
        //                             (
        //                                 Key {
        //                                     name: "hello".to_string(),
        //                                     annotation: None,
        //                                 },
        //                                 EsonLiteral::Str("world".to_string())
        //                             ),
        //                             (
        //                                 Key {
        //                                     name: "foo".to_string(),
        //                                     annotation: None,
        //                                 },
        //                                 EsonLiteral::Str("bar".to_string())
        //                             ),
        //                             (
        //                                 Key {
        //                                     name: "bar".to_string(),
        //                                     annotation: None,
        //                                 },
        //                                 EsonLiteral::Str("hello TODO!".to_string())
        //                             ),
        //                         ]
        //                         .into_iter()
        //                         .collect()
        //                     )
        //                 ),
        //             ]
        //             .into_iter()
        //             .collect()
        //         )
        //     ))
        // );
    }

    #[test]
    fn test_comment() {
        let json = r##"
        // comment1
        {}
        "##;
        // assert_eq!(root(json), Ok(("", EsonLiteral::Object(HashMap::new()))));
    }

    #[test]
    fn test_comment2() {
        let json = r##"
        // comment1
        // comment2
        {}
        "##;
        // assert_eq!(root(json), Ok(("", EsonLiteral::Object(HashMap::new()))));
    }

    #[test]
    fn test_json_with_comments() {
        let json = r##"
        // comment1
        // 注释2
        {
            // comment3
            "a": 42,
            "b": [
                "x", // 注释4
                "y",
                12
            ],
            "c": {
                "hello": "world",
                "foo": r"bar",
                // 注释5
                "bar": f"hello ${name}",
            }
        }
        "##;
        // assert_eq!(
        //     root(json),
        //     Ok((
        //         "",
        //         EsonLiteral::Object(
        //             vec![
        //                 ("a".into(), EsonLiteral::Int(42)),
        //                 (
        //                     "b".into(),
        //                     EsonLiteral::Array(vec![
        //                         EsonLiteral::Str("x".to_string()),
        //                         EsonLiteral::Str("y".to_string()),
        //                         EsonLiteral::Int(12),
        //                     ])
        //                 ),
        //                 (
        //                     "c".into(),
        //                     EsonLiteral::Object(
        //                         vec![
        //                             ("hello".into(), EsonLiteral::Str("world".to_string())),
        //                             ("foo".into(), EsonLiteral::Str("bar".to_string())),
        //                             ("bar".into(), EsonLiteral::Str("hello TODO!".to_string())),
        //                         ]
        //                         .into_iter()
        //                         .collect()
        //                     )
        //                 ),
        //             ]
        //             .into_iter()
        //             .collect()
        //         )
        //     ))
        // );
    }

    #[test]
    fn test_literal_key() {
        // assert_eq!(
        //     key_value(r#""hello": "world""#),
        //     key_value(r#"hello: "world""#),
        // );
    }

    #[test]
    fn test_sp() {
        assert_eq!(sp("  "), Ok(("", "")));
        assert_eq!(sp(" \t\r\n"), Ok(("", "")));
        assert_eq!(sp(""), Ok(("", "")));
    }

    #[test]
    fn test_null() {
        // assert_eq!(null("null"), Ok(("", ())));
    }

    #[test]
    fn test_f_string() {
        assert_eq!(
            parse_string(r#"f"${name}""#),
            Ok(("", String::from("Var(name)"))) // @todo
        );
        assert_eq!(
            parse_string(r#"f"hello ${name}""#),
            Ok(("", String::from("hello TODO!")))
        );
        assert_eq!(
            parse_string(r#"f"hello ${ name }""#),
            Ok(("", String::from("hello TODO!")))
        );
        assert_eq!(
            parse_string(r#"f"hello ${ name } world""#),
            Ok(("", String::from("hello TODO! world")))
        );
        assert_eq!(
            parse_string(r#"f"hello ${ name } world ${ name }""#),
            Ok(("", String::from("hello TODO! world TODO!")))
        );
        assert_eq!(
            parse_string(r####"f#"hello ${ name }"#"####),
            Ok(("", String::from("hello TODO!")))
        );
        assert_eq!(
            parse_string(r####"f#"hello ${ foo(bar) }"#"####),
            Ok(("", String::from("hello TODO!")))
        );
        assert_eq!(
            parse_string(r####"f#"hello ${ foo(bar, foo()) }"#"####),
            Ok(("", String::from("hello TODO!")))
        );
    }

    #[test]
    fn test_f_string2() {
        // let data = r#"f"${hello}" hello "#;
        // dbg!(parse_string(data));

        // let data = r#"[f"${hello}"]"#;
        // dbg!(root(data));
    }

    #[test]
    fn test_json() {
        let data = r#"{
            "a": 42,
            "b": [
                "x",
                "y",
                12
            ],
            "c": {
                "hello": "world",
                "foo": r"bar",
                "bar": f"hello ${name}",
            }
        }"#;

        // dbg!(root(data));
    }
}
