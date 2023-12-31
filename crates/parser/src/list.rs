use nom::character::complete::char;
use nom::combinator::{cut, opt};
use nom::error::{context, VerboseError};
use nom::IResult;
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated, tuple};

use crate::{eson, eson_literal, EsonLiteralSegment, EsonSegment, sp};

/// some combinators, like `separated_list0` or `many0`, will call a parser repeatedly,
/// accumulating results in a `Vec`, until it encounters an error.
/// If you want more control on the parser application, check out the `iterator`
/// combinator (cf `examples/iterator.rs`)
pub fn parse_lst(i: &str) -> IResult<&str, Vec<EsonSegment>, VerboseError<&str>> {
    context(
        "parse_lst",
        preceded(
            char('['),
            cut(terminated(
                separated_list0(preceded(sp, char(',')), eson),
                tuple((sp, opt(char(',')), sp, char(']'))),
            )),
        ),
    )(i)
}

pub fn parse_literal_lst(i: &str) -> IResult<&str, Vec<EsonLiteralSegment>, VerboseError<&str>> {
    context(
        "parse_literal_lst",
        preceded(
            char('['),
            cut(terminated(
                separated_list0(preceded(sp, char(',')), eson_literal),
                tuple((sp, opt(char(',')), sp, char(']'))),
            )),
        ),
    )(i)
}