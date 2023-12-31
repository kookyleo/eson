use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::VerboseError;
use nom::IResult;

use crate::{EsonLiteralSegment, EsonSegment};

pub fn parse_null(input: &str) -> IResult<&str, EsonSegment, VerboseError<&str>> {
    map(tag("null"), |_| EsonSegment::Null)(input)
}

pub fn parse_literal_null(input: &str) -> IResult<&str, EsonLiteralSegment, VerboseError<&str>> {
    map(tag("null"), |_| EsonLiteralSegment::Null)(input)
}