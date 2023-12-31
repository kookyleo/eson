use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while1};
use nom::character::complete::{char as ch, digit1};
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::IResult;
use nom::number::complete::double;
use nom::sequence::{preceded, tuple};

use crate::{EsonLiteralSegment, EsonSegment};

pub(crate) fn parse_numeric(input: &str) -> nom::IResult<&str, EsonSegment, VerboseError<&str>> {
    alt((
        map(parse_bin, |s| {
            EsonSegment::Int(i64::from_str_radix(s, 2).expect("TODO"))
        }),
        map(parse_oct, |s| {
            EsonSegment::Int(i64::from_str_radix(s, 8).expect("TODO"))
        }),
        map(parse_hex, |s| {
            EsonSegment::Int(i64::from_str_radix(s, 16).expect("TODO"))
        }),
        map(
            tuple((
                digit1,
                opt(preceded(ch('.'), digit1)),
                opt(preceded(
                    tag_no_case("e"),
                    tuple((opt(alt((ch('+'), ch('-')))), digit1)),
                )),
            )),
            |(int_part, decimal_part, exp_part): (
                &str,
                Option<&str>,
                Option<(Option<char>, &str)>,
            )| {
                if decimal_part.is_none() && exp_part.is_none() {
                    // 没有小数点或指数部分 => 整数
                    // int_part.parse::<i64>().map(JsonValue::Int)
                    EsonSegment::Int(int_part.parse::<i64>().expect("TODO"))
                } else {
                    let num_str = format!(
                        "{}{}{}",
                        int_part,
                        decimal_part.map_or(String::from(""), |d| format!(".{}", d)),
                        exp_part.map_or(String::from(""), |(sign, e)| format!(
                            "e{}{}",
                            sign.unwrap_or('+'),
                            e
                        ))
                    );
                    // dbg!(num_str.clone());
                    // num_str.parse::<f64>().map(JsonValue::Float)
                    EsonSegment::Float(num_str.parse::<f64>().expect("TODO"))
                }
            },
        ),
        map(tag("Infinity"), |_| EsonSegment::Float(f64::INFINITY)),
        map(tag("-Infinity"), |_| EsonSegment::Float(f64::NEG_INFINITY)),
        map(tag("NaN"), |_| EsonSegment::Float(f64::NAN)),
    ))(input)
}

pub fn parse_literal_number(input: &str) -> IResult<&str, EsonLiteralSegment, VerboseError<&str>> {
    let (remaining, number) = parse_numeric(input)?;
    match number {
        EsonSegment::Int(i) => Ok((remaining, EsonLiteralSegment::Int(i))),
        EsonSegment::Float(f) => Ok((remaining, EsonLiteralSegment::Float(f))),
        _ => unreachable!()
    }
}

fn parse_f64(input: &str) -> nom::IResult<&str, f64, VerboseError<&str>> {
    double(input)
}

fn parse_hex(input: &str) -> nom::IResult<&str, &str, VerboseError<&str>> {
    let is_hex_digit = |c: char| c.is_digit(16);
    let (remaining, _) = tag("0x")(input)?;
    take_while1(is_hex_digit)(remaining)
}

fn parse_oct(input: &str) -> nom::IResult<&str, &str, VerboseError<&str>> {
    let is_oct_digit = |c: char| c.is_digit(8);
    let (remaining, _) = tag("0o")(input)?;
    take_while1(is_oct_digit)(remaining)
}

fn parse_bin(input: &str) -> nom::IResult<&str, &str, VerboseError<&str>> {
    let is_bin_digit = |c: char| c.is_digit(2);
    let (remaining, _) = tag("0b")(input)?;
    take_while1(is_bin_digit)(remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(parse_literal_number("0b1010"), Ok(("", EsonLiteralSegment::Int(10))));
        assert_eq!(parse_literal_number("0o777"), Ok(("", EsonLiteralSegment::Int(511))));
        assert_eq!(parse_literal_number("0x123"), Ok(("", EsonLiteralSegment::Int(291))));
        assert_eq!(
            parse_literal_number("123.456"),
            Ok(("", EsonLiteralSegment::Float(123.456)))
        );
        assert_eq!(
            parse_literal_number("123.456e-10"),
            Ok(("", EsonLiteralSegment::Float(0.0000000123456)))
        );
        assert_eq!(
            parse_literal_number("123.456e+10"),
            Ok(("", EsonLiteralSegment::Float(1234560000000.0)))
        );
        assert_eq!(
            parse_literal_number("123.456e10"),
            Ok(("", EsonLiteralSegment::Float(1234560000000.0)))
        );
        assert_eq!(parse_literal_number("123"), Ok(("", EsonLiteralSegment::Int(123))));

        // let i = 123e2;
    }
}
