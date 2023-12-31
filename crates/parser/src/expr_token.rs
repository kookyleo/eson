use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, multispace0};
use nom::combinator::{map, map_res};
use nom::error::{context, VerboseError};
use nom::IResult;
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, pair, separated_pair};

use crate::{eson, EsonSegment};
use crate::expr::legal_id;
use crate::expr_token::chunk::ExprTokenChunk;
use crate::string::parse_literal_string;

#[derive(PartialEq, Debug)]
pub enum RefIndex {
    Int(i16),
    Str(String),
}

#[derive(PartialEq, Debug)]
pub enum RefPronoun {
    Curr(Vec<RefIndex>),
    Super(Vec<RefIndex>),
    Root(Vec<RefIndex>), // $
}

pub(crate) mod chunk {
    use std::fmt::Display;

    use crate::expr_token::ExprToken;

    #[derive(Debug, PartialEq)]
    pub(crate) struct ExprTokenChunk(Vec<ExprToken>);

    impl Display for ExprTokenChunk {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut s = String::new();
            for token in &self.0 {
                s.push_str(&format!("{}", token));
            }
            write!(f, "{}", s)
        }
    }

    impl From<Vec<ExprToken>> for ExprTokenChunk {
        fn from(tokens: Vec<ExprToken>) -> Self {
            ExprTokenChunk(tokens)
        }
    }

    impl From<ExprTokenChunk> for Vec<ExprToken> {
        fn from(chunk: ExprTokenChunk) -> Self {
            chunk.0
        }
    }
}


#[derive(Debug, PartialEq)]
pub(crate) enum ExprToken {
    None,
    Group(ExprTokenChunk),
    Val(EsonSegment),
    FnCall(String, Vec<ExprTokenChunk>),
    Var(String),
    Ref(RefPronoun), // eg. self, super, $, self.ele, super["ele"], $[0] ..

    Pipe,
    // expr | fn
    Q,
    // ?
    COLON, // :

    Eq,
    // ==
    Ne,
    // !=
    Le,
    // <=
    Ge,
    // >=
    And,
    // &&
    Or, // ||

    Not,
    // !
    Gt,
    // >
    Lt, // <

    Plus,
    // +
    Minus,
    // -
    Mul,
    // *
    Div,
    // /
    Mod, // %

    Eoi, // End of input
}

impl Display for ExprToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprToken::None => write!(f, "None"),
            ExprToken::Group(..) => write!(f, "Group"),
            ExprToken::Val(v) => write!(f, "Val({:?})", v),
            ExprToken::FnCall(id, args) => write!(f, "FnCall({}, {:?})", id, args),
            ExprToken::Var(id) => write!(f, "Var({})", id),
            ExprToken::Ref(RefPronoun::Curr(elements)) => {
                write!(f, "Ref(Curr({:?}))", elements)
            }
            ExprToken::Ref(RefPronoun::Super(elements)) => {
                write!(f, "Ref(Super({:?}))", elements)
            }
            ExprToken::Ref(RefPronoun::Root(elements)) => {
                write!(f, "Ref(Root({:?}))", elements)
            }
            ExprToken::Pipe => write!(f, "Pipe"),
            ExprToken::Eq => write!(f, "Eq"),
            ExprToken::Ne => write!(f, "Ne"),
            ExprToken::Le => write!(f, "Le"),
            ExprToken::Ge => write!(f, "Ge"),
            ExprToken::And => write!(f, "And"),
            ExprToken::Or => write!(f, "Or"),
            ExprToken::Not => write!(f, "Not"),
            ExprToken::Gt => write!(f, "Gt"),
            ExprToken::Lt => write!(f, "Lt"),
            ExprToken::Plus => write!(f, "Plus"),
            ExprToken::Minus => write!(f, "Minus"),
            ExprToken::Mul => write!(f, "Mul"),
            ExprToken::Div => write!(f, "Div"),
            ExprToken::Mod => write!(f, "Mod"),
            ExprToken::Eoi => write!(f, "Eoi"),
            ExprToken::Q => write!(f, "Q"),
            ExprToken::COLON => write!(f, "COLON"),
        }
    }
}

pub(crate) fn expr_token_set(input: &str) -> IResult<&str, ExprTokenChunk, VerboseError<&str>> {
    context(
        "expr_tokens",
        map(
            many1(alt((fn_call, reference, value, var, operator))),
            |tokens| ExprTokenChunk::from(tokens),
        ),
    )(input)
}

fn var(input: &str) -> IResult<&str, ExprToken, VerboseError<&str>> {
    context(
        "var",
        map(delimited(multispace0, legal_id, multispace0), |id: &str| {
            ExprToken::Var(id.to_string())
        }),
    )(input)
}

fn fn_call(input: &str) -> IResult<&str, ExprToken, VerboseError<&str>> {
    context(
        "fn_call",
        map(
            separated_pair(
                legal_id,
                delimited(multispace0, tag("("), multispace0),
                delimited(
                    multispace0,
                    separated_list0(
                        delimited(multispace0, tag(","), multispace0),
                        expr_token_set,
                    ),
                    delimited(multispace0, tag(")"), multispace0),
                ),
            ),
            |(id, args)| ExprToken::FnCall(id.to_string(), args),
        ),
    )(input)
}

fn value(input: &str) -> IResult<&str, ExprToken, VerboseError<&str>> {
    context("value", map(eson, |v| ExprToken::Val(v)))(input)
}

// reference (eg. self.ele, super["ele"], $[0]) => Token::Ref
fn reference(input: &str) -> IResult<&str, ExprToken, VerboseError<&str>> {
    // self, super, $
    let ref_head = alt((
        map(tag("self"), |_| RefPronoun::Curr(vec![])),
        map(tag("super"), |_| RefPronoun::Super(vec![])),
        map(tag("$"), |_| RefPronoun::Root(vec![])),
    ));

    // .ele => RefIndex::Str("ele".to_string())
    // ["ele"] => RefIndex::Str("ele".to_string())
    // [0] => RefIndex::Int(0)
    let ref_element = alt((
        map(
            delimited(
                delimited(multispace0, tag("."), multispace0),
                legal_id,
                multispace0,
            ),
            |s| RefIndex::Str(s.to_string()),
        ),
        map(
            delimited(
                delimited(multispace0, tag("["), multispace0),
                parse_literal_string,
                delimited(multispace0, tag("]"), multispace0),
            ),
            |s| RefIndex::Str(s.to_string()),
        ),
        map(
            delimited(
                delimited(multispace0, tag("["), multispace0),
                map_res(digit1, |s: &str| s.parse::<i16>()),
                delimited(multispace0, tag("]"), multispace0),
            ),
            |i| RefIndex::Int(i),
        ),
    ));

    context(
        "reference",
        map(
            pair(
                ref_head,
                many0(delimited(multispace0, ref_element, multispace0)),
            ),
            |(head, elements)| match head {
                RefPronoun::Curr(_) => ExprToken::Ref(RefPronoun::Curr(elements)),
                RefPronoun::Super(_) => ExprToken::Ref(RefPronoun::Super(elements)),
                RefPronoun::Root(_) => ExprToken::Ref(RefPronoun::Root(elements)),
            },
        ),
    )(input)
}

fn operator(input: &str) -> IResult<&str, ExprToken, VerboseError<&str>> {
    context(
        "symbol",
        map(
            alt((
                delimited(multispace0, tag("=="), multispace0),
                delimited(multispace0, tag("!="), multispace0),
                delimited(multispace0, tag("<="), multispace0),
                delimited(multispace0, tag(">="), multispace0),
                delimited(multispace0, tag("&&"), multispace0),
                delimited(multispace0, tag("||"), multispace0),
                delimited(multispace0, tag("!"), multispace0),
                delimited(multispace0, tag(">"), multispace0),
                delimited(multispace0, tag("<"), multispace0),
                delimited(multispace0, tag("+"), multispace0),
                delimited(multispace0, tag("-"), multispace0),
                delimited(multispace0, tag("*"), multispace0),
                delimited(multispace0, tag("/"), multispace0),
                delimited(multispace0, tag("%"), multispace0),
                delimited(multispace0, tag("^"), multispace0),
                delimited(multispace0, tag("-"), multispace0),
            )),
            |op| match op {
                "==" => ExprToken::Eq,
                "!=" => ExprToken::Ne,
                "<=" => ExprToken::Le,
                ">=" => ExprToken::Ge,
                "&&" => ExprToken::And,
                "||" => ExprToken::Or,
                "!" => ExprToken::Not,
                ">" => ExprToken::Gt,
                "<" => ExprToken::Lt,
                "+" => ExprToken::Plus,
                "-" => ExprToken::Minus,
                "*" => ExprToken::Mul,
                "/" => ExprToken::Div,
                "%" => ExprToken::Mod,
                _ => unreachable!(),
            },
        ),
    )(input)
}

// ${ ... }
pub(crate) fn parse_expr_token_chunk(input: &str) -> IResult<&str, ExprTokenChunk, VerboseError<&str>> {
    context(
        "parse_expr_token_chunk",
        delimited(
            pair(tag("${"), multispace0),
            expr_token_set,
            pair(multispace0, tag("}")),
        ),
    )(input)
}


#[cfg(test)]
mod tests {
    use crate::expr_token::ExprToken::{FnCall, Var};

    use super::*;

    #[test]
    fn test_nil() {
        assert!(fn_call("").is_err());
        assert!(reference("").is_err());
        assert!(var("").is_err());
        assert!(operator("").is_err());
        assert!(value("").is_err());
        assert!(expr_token_set("").is_err());
    }

    #[test]
    fn test_expr_chunk_zero() {
        assert!(expr_token_set(r#""#).is_err());
    }

    #[test]
    fn test_value() {
        let dat = "1";
        assert_eq!(value(dat), Ok(("", ExprToken::Val(EsonSegment::Int(1)))));
    }

    #[test]
    fn test_expr_chunk() {
        assert_eq!(
            expr_token_set(r#"1"#),
            Ok((
                "",
                ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(1))])
            ))
        );
    }

    #[test]
    fn test_separated_list() {
        fn sl(i: &str) -> IResult<&str, Vec<ExprTokenChunk>, VerboseError<&str>> {
            separated_list0(
                delimited(multispace0, tag(","), multispace0),
                expr_token_set,
            )(i)
        }

        assert_eq!(
            sl("1, 2, 3"),
            Ok((
                "",
                vec![
                    ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(1))]),
                    ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(2))]),
                    ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(3))]),
                ],
            ))
        );

        assert_eq!(
            sl("a, b, 1"),
            Ok((
                "",
                vec![
                    ExprTokenChunk::from(vec![Var("a".to_string())]),
                    ExprTokenChunk::from(vec![Var("b".to_string())]),
                    ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(1))]),
                ],
            ))
        );

        assert_eq!(
            sl("1"),
            Ok((
                "",
                vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                    1
                ))])],
            ))
        );

        assert_eq!(sl(""), Ok(("", vec![])));
    }

    #[test]
    fn test_fn_call0() {
        assert_eq!(
            fn_call("fn_call()"),
            Ok(("", FnCall("fn_call".to_string(), vec![])))
        );
        assert_eq!(
            fn_call("fn_call( )"),
            Ok(("", FnCall("fn_call".to_string(), vec![])))
        );
        assert_eq!(
            fn_call("fn_call( ) "),
            Ok(("", FnCall("fn_call".to_string(), vec![])))
        );
        assert_eq!(
            fn_call("fn_call ( )"),
            Ok(("", FnCall("fn_call".to_string(), vec![])))
        );
        assert_eq!(
            fn_call("fn_call(hello, world)"),
            Ok((
                "",
                FnCall(
                    "fn_call".to_string(),
                    vec![
                        ExprTokenChunk::from(vec![Var("hello".to_string())]),
                        ExprTokenChunk::from(vec![Var("world".to_string())]),
                    ],
                )
            ))
        );
        assert_eq!(
            fn_call("fn_call(fn_call())"),
            Ok((
                "",
                FnCall(
                    "fn_call".to_string(),
                    vec![ExprTokenChunk::from(vec![FnCall(
                        "fn_call".to_string(),
                        vec![],
                    )])],
                )
            ))
        );
        assert_eq!(
            fn_call("fn_call(fn_call(), fn_call(1, 2), 3)"),
            Ok((
                "",
                FnCall(
                    "fn_call".to_string(),
                    vec![
                        ExprTokenChunk::from(vec![FnCall("fn_call".to_string(), vec![])]),
                        ExprTokenChunk::from(vec![FnCall(
                            "fn_call".to_string(),
                            vec![
                                ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(1))]),
                                ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(2))]),
                            ],
                        )]),
                        ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(3))]),
                    ],
                )
            ))
        );
    }

    #[test]
    fn test_expr_chunk1() {
        assert_eq!(
            expr_token_set(r#"1 + 2 * 3 / 4 % 5"#),
            Ok((
                "",
                ExprTokenChunk::from(vec![
                    ExprToken::Val(EsonSegment::Int(1)),
                    ExprToken::Plus,
                    ExprToken::Val(EsonSegment::Int(2)),
                    ExprToken::Mul,
                    ExprToken::Val(EsonSegment::Int(3)),
                    ExprToken::Div,
                    ExprToken::Val(EsonSegment::Int(4)),
                    ExprToken::Mod,
                    ExprToken::Val(EsonSegment::Int(5)),
                ])
            ))
        );
    }

    #[test]
    fn test_expr_chunk2() {
        // dbg!(expr_token_set(r#"f(1) + g(2) * h(3)"#));
        assert_eq!(
            expr_token_set(r#"f(1) + g(2) * h(3)"#),
            Ok((
                "",
                ExprTokenChunk::from(vec![
                    FnCall(
                        "f".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            1
                        ))])],
                    ),
                    ExprToken::Plus,
                    FnCall(
                        "g".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            2
                        ))])],
                    ),
                    ExprToken::Mul,
                    FnCall(
                        "h".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            3
                        ))])],
                    ),
                ])
            ))
        );
    }

    #[test]
    fn test_float() {
        assert_eq!(
            value("1.0"),
            Ok(("", ExprToken::Val(EsonSegment::Float(1.0))))
        );
        assert_eq!(
            value("1.0e1"),
            Ok(("", ExprToken::Val(EsonSegment::Float(10.0))))
        );
        assert_eq!(
            value("1.0e-1"),
            Ok(("", ExprToken::Val(EsonSegment::Float(0.1))))
        );
        assert_eq!(
            value("1.0e+1"),
            Ok(("", ExprToken::Val(EsonSegment::Float(10.0))))
        );
        assert_eq!(
            value("1.0E1"),
            Ok(("", ExprToken::Val(EsonSegment::Float(10.0))))
        );
        assert_eq!(
            value("1.0E-1"),
            Ok(("", ExprToken::Val(EsonSegment::Float(0.1))))
        );
        assert_eq!(
            value("1.0E+1"),
            Ok(("", ExprToken::Val(EsonSegment::Float(10.0))))
        );
    }

    #[test]
    fn test_fn_call_arg_float() {
        assert_eq!(
            fn_call("fn_call(1.0)"),
            Ok((
                "",
                FnCall(
                    "fn_call".to_string(),
                    vec![ExprTokenChunk::from(vec![ExprToken::Val(
                        EsonSegment::Float(1.0)
                    )])],
                )
            ))
        );
        assert_eq!(
            fn_call("fn_call(1.0, 2.0)"),
            Ok((
                "",
                FnCall(
                    "fn_call".to_string(),
                    vec![
                        ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Float(1.0))]),
                        ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Float(2.0))]),
                    ],
                )
            ))
        );
    }

    #[test]
    fn test_expr_chunk3() {
        assert_eq!(
            expr_token_set(r#"f(1) + g(2, 2.5) * h(3) + 4"#),
            Ok((
                "",
                ExprTokenChunk::from(vec![
                    FnCall(
                        "f".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            1
                        ))])],
                    ),
                    ExprToken::Plus,
                    FnCall(
                        "g".to_string(),
                        vec![
                            ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(2))]),
                            ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Float(2.5))]),
                        ],
                    ),
                    ExprToken::Mul,
                    FnCall(
                        "h".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            3
                        ))])],
                    ),
                    ExprToken::Plus,
                    ExprToken::Val(EsonSegment::Int(4)),
                ])
            ))
        );
    }

    fn test_expr_chunk4() {
        assert_eq!(
            expr_token_set(r#"f(1) + g(2, k(2 * 7)) * h(3) + 4"#),
            Ok((
                "",
                ExprTokenChunk::from(vec![
                    FnCall(
                        "f".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            1
                        ))])],
                    ),
                    ExprToken::Plus,
                    FnCall(
                        "g".to_string(),
                        vec![
                            ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(2))]),
                            ExprTokenChunk::from(vec![FnCall(
                                "k".to_string(),
                                vec![ExprTokenChunk::from(vec![
                                    ExprToken::Val(EsonSegment::Int(2)),
                                    ExprToken::Mul,
                                    ExprToken::Val(EsonSegment::Int(7)),
                                ])],
                            )]),
                        ],
                    ),
                    ExprToken::Mul,
                    FnCall(
                        "h".to_string(),
                        vec![ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(
                            3
                        ))])],
                    ),
                    ExprToken::Plus,
                    ExprToken::Val(EsonSegment::Int(4)),
                ])
            ))
        );
    }

    #[test]
    fn test_parse_expr_token_chunk() {
        assert!(parse_expr_token_chunk("${}").is_err());
        assert!(parse_expr_token_chunk("${ }").is_err());
        assert_eq!(
            parse_expr_token_chunk("${ 1 }"),
            Ok((
                "",
                ExprTokenChunk::from(vec![ExprToken::Val(EsonSegment::Int(1))])
            ))
        );

        dbg!(parse_expr_token_chunk("${ 1 + 2 * 3 }"));

        // assert_eq!(
        //     parse_expr("${ 1 + 2 }"),
        // );
        // assert_eq!(
        //     parse_expr("${ 1 + 2 * 3 }"),
        //     Ok((
        //         "",
        //
        //     ))
        // );
        // assert_eq!(
        //     parse_expr("${ 1 + 2 * 3 / 4 }"),
        //     Ok((
        //         "",
        //
        //     ))
        // );
        // assert_eq!(
        //     parse_expr(r#"${ {k: "value"} }"#),
        //     Ok((
        //         "",
        //         ExprToken::Val(EsonSegment::Dict(
        //             vec![(
        //                 Key {
        //                     name: String::from("k"),
        //                     annotation: None,
        //                 },
        //                 EsonSegment::Str(String::from("value")),
        //             )]
        //             .into_iter()
        //             .collect()
        //         ))
        //     ))
        // );
    }
}
