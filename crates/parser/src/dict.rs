use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{cut, map, opt};
use nom::error::{context, VerboseError};
use nom::IResult;
use nom::multi::separated_list0;
use nom::sequence::{preceded, separated_pair, terminated, tuple};

use crate::{Annotation, eson, eson_literal, EsonLiteralSegment, EsonSegment, sp};
use crate::annotation::parse_annotations;
use crate::expr::legal_id;
use crate::string::parse_string;

#[derive(Debug)]
pub struct Key {
    pub name: String,
    pub annotation: Option<Vec<Annotation>>,
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key {
            name: String::from(s),
            annotation: None,
        }
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Key {
            name: s,
            annotation: None,
        }
    }
}

impl From<Key> for String {
    fn from(k: Key) -> Self {
        k.name
    }
}

fn key(i: &str) -> IResult<&str, Key, VerboseError<&str>> {
    let (remaining, annotation) = opt(parse_annotations)(i)?;
    let (remaining, name) =
        preceded(sp, alt((parse_string, map(legal_id, |s| String::from(s)))))(remaining)?;
    Ok((remaining, Key { name, annotation }))
}

pub fn parse_dict(i: &str) -> IResult<&str, HashMap<Key, EsonSegment>, VerboseError<&str>> {
    fn key_value(i: &str) -> IResult<&str, (Key, EsonSegment), VerboseError<&str>> {
        separated_pair(key, cut(preceded(sp, char(':'))), eson)(i)
    }
    context(
        "parse_dict",
        preceded(
            context("dict_head", preceded(sp, char('{'))),
            cut(terminated(
                context(
                    "dict_body",
                    map(
                        separated_list0(preceded(sp, char(',')), key_value),
                        |tuple_vec| tuple_vec.into_iter().map(|(k, v)| (k, v)).collect(),
                    ),
                ),
                context("dict_tail", tuple((sp, opt(char(',')), sp, char('}')))),
            )),
        ),
    )(i)
}

pub fn parse_literal_dict(
    i: &str,
) -> IResult<&str, HashMap<Key, EsonLiteralSegment>, VerboseError<&str>> {
    fn key_literal_value(i: &str) -> IResult<&str, (Key, EsonLiteralSegment), VerboseError<&str>> {
        separated_pair(key, cut(preceded(sp, char(':'))), eson_literal)(i)
    }
    context(
        "parse_dict_literal",
        preceded(
            context("dict_literal_head", preceded(sp, char('{'))),
            cut(terminated(
                context(
                    "dict_literal_body",
                    map(
                        separated_list0(preceded(sp, char(',')), key_literal_value),
                        |tuple_vec| tuple_vec.into_iter().map(|(k, v)| (k, v)).collect(),
                    ),
                ),
                context(
                    "dict_literal_tail",
                    tuple((sp, opt(char(',')), sp, char('}'))),
                ),
            )),
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key() {
        assert_eq!(
            key("foo"),
            Ok((
                "",
                Key {
                    name: String::from("foo"),
                    annotation: None,
                }
            ))
        );
        assert_eq!(
            key("@bar \n foo"),
            Ok((
                "",
                Key {
                    name: String::from("foo"),
                    annotation: Some(vec![Annotation {
                        name: String::from("bar"),
                        value: None,
                    }]),
                }
            ))
        );
        assert_eq!(
            key("@bar()\nfoo"),
            Ok((
                "",
                Key {
                    name: String::from("foo"),
                    annotation: Some(vec![Annotation {
                        name: String::from("bar"),
                        value: Some(vec![]),
                    }]),
                }
            ))
        );
        assert_eq!(
            key("@bar(1)\nfoo"),
            Ok((
                "",
                Key {
                    name: String::from("foo"),
                    annotation: Some(vec![Annotation {
                        name: String::from("bar"),
                        value: Some(vec![EsonLiteralSegment::Int(1)]),
                    }]),
                }
            ))
        );
        assert_eq!(
            key(" @bar(1, 2)    foo"),
            Ok((
                "",
                Key {
                    name: String::from("foo"),
                    annotation: Some(vec![Annotation {
                        name: String::from("bar"),
                        value: Some(vec![EsonLiteralSegment::Int(1), EsonLiteralSegment::Int(2)]),
                    }]),
                }
            ))
        );
        assert_eq!(
            key("@bar(1, 2, 3) \t\nfoo"),
            Ok((
                "",
                Key {
                    name: String::from("foo"),
                    annotation: Some(vec![Annotation {
                        name: String::from("bar"),
                        value: Some(vec![
                            EsonLiteralSegment::Int(1),
                            EsonLiteralSegment::Int(2),
                            EsonLiteralSegment::Int(3),
                        ]),
                    }]),
                }
            ))
        );
        // todo: fix this
        // assert_eq!(
        //     key("@bar(1, 2, 3,) foo "),
        //     Ok((
        //         "",
        //         Key {
        //             name: String::from("foo"),
        //             annotation: Some(vec![Annotation {
        //                 name: String::from("bar"),
        //                 value: Some(vec![
        //                     EsonLiteralSegment::Int(1),
        //                     EsonLiteralSegment::Int(2),
        //                     EsonLiteralSegment::Int(3)
        //                 ])
        //             }])
        //         }
        //     ))
        // );
        // assert_eq!(
        //     key("@bar(1, 2, 3,) @baz\n foo "),
        //     Ok((
        //         "",
        //         Key {
        //             name: String::from("foo"),
        //             annotation: Some(vec![
        //                 Annotation {
        //                     name: String::from("bar"),
        //                     value: Some(vec![
        //                         EsonLiteralSegment::Int(1),
        //                         EsonLiteralSegment::Int(2),
        //                         EsonLiteralSegment::Int(3)
        //                     ])
        //                 },
        //                 Annotation {
        //                     name: String::from("baz"),
        //                     value: None
        //                 }
        //             ])
        //         }
        //     ))
        // );
    }

    #[test]
    fn test_parse_dict() {
        assert_eq!(parse_dict("{}"), Ok(("", HashMap::new())));
        assert_eq!(
            parse_dict("{foo: 1}"),
            Ok((
                "",
                vec![(
                    Key {
                        name: String::from("foo"),
                        annotation: None,
                    },
                    EsonSegment::Int(1)
                )]
                    .into_iter()
                    .collect()
            ))
        );
        assert_eq!(
            parse_dict("{foo: 1, bar: 2}"),
            Ok((
                "",
                vec![
                    (
                        Key {
                            name: String::from("foo"),
                            annotation: None,
                        },
                        EsonSegment::Int(1)
                    ),
                    (
                        Key {
                            name: String::from("bar"),
                            annotation: None,
                        },
                        EsonSegment::Int(2)
                    ),
                ]
                    .into_iter()
                    .collect()
            ))
        );
        assert_eq!(
            parse_dict("{foo: 1, bar: 2,}"),
            Ok((
                "",
                vec![
                    (
                        Key {
                            name: String::from("foo"),
                            annotation: None,
                        },
                        EsonSegment::Int(1)
                    ),
                    (
                        Key {
                            name: String::from("bar"),
                            annotation: None,
                        },
                        EsonSegment::Int(2)
                    ),
                ]
                    .into_iter()
                    .collect()
            ))
        );
        // todo: check this
        // assert_eq!(
        //     parse_dict("{foo: 1, bar: 2,} "),
        //     Ok((
        //         "",
        //         vec![
        //             (
        //                 Key {
        //                     name: String::from("foo"),
        //                     annotation: None
        //                 },
        //                 EsonSegment::Int(1)
        //             ),
        //             (
        //                 Key {
        //                     name: String::from("bar"),
        //                     annotation: None
        //                 },
        //                 EsonSegment::Int(2)
        //             )
        //         ]
        //         .into_iter()
        //         .collect()
        //     ))
        // );
        // assert_eq!(
        //     parse_dict("{foo: 1, bar: 2,} // comment"),
        //     Ok((
        //         "",
        //         vec![
        //             (
        //                 Key {
        //                     name: String::from("foo"),
        //                     annotation: None
        //                 },
        //                 EsonSegment::Int(1)
        //             ),
        //             (
        //                 Key {
        //                     name: String::from("bar"),
        //                     annotation: None
        //                 },
        //                 EsonSegment::Int(2)
        //             )
        //         ]
        //         .into_iter()
        //         .collect()
        //     ))
        // );
        // assert_eq!(
        //     parse_dict("{foo: 1, bar: 2,} // comment\n"),
        //     Ok((
        //         "",
        //         vec![
        //             (
        //                 Key {
        //                     name: String::from("foo"),
        //                     annotation: None
        //                 },
        //                 EsonSegment::Int(1)
        //             ),
        //             (
        //                 Key {
        //                     name: String::from("bar"),
        //                     annotation: None
        //                 },
        //                 EsonSegment::Int(2)
        //             )
        //         ]
        //         .into_iter()
        //         .collect()
        //     ))
        // );
    }
}
