use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::VerboseError;
use nom::IResult;

use crate::{EsonLiteralSegment, EsonSegment};

pub(crate) fn parse_boolean(input: &str) -> IResult<&str, EsonSegment, VerboseError<&str>> {
    alt((
        map(tag("true"), |_| EsonSegment::Boolean(true)),
        map(tag("false"), |_| EsonSegment::Boolean(false)),
    ))(input)
}

pub(crate) fn parse_literal_boolean(input: &str) -> IResult<&str, EsonLiteralSegment, VerboseError<&str>> {
    alt((
        map(tag("true"), |_| EsonLiteralSegment::Boolean(true)),
        map(tag("false"), |_| EsonLiteralSegment::Boolean(false)),
    ))(input)
}