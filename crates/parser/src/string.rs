use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_while_m_n};
use nom::character::complete::{char as ch, multispace1};
use nom::combinator::{complete, map, map_opt, map_res, value, verify};
use nom::error::VerboseError;
use nom::IResult;
use nom::multi::{count, fold_many0, many_till};
use nom::sequence::{delimited, pair, preceded};

use crate::expr_token::parse_expr_token_chunk;

fn parse_unicode(input: &str) -> IResult<&str, char, VerboseError<&str>> {
    let parse_1_to_6_hex_num = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_4_hex_num = take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit());

    // eg: u{1F601} => 1F601
    //     uFE0F => FE0F
    let parse_prefixed_hex = preceded(
        ch('u'),
        alt((
            delimited(ch('{'), parse_1_to_6_hex_num, ch('}')),
            parse_4_hex_num,
        )),
    );

    // hex bytes => u32.
    let parse_u32 = map_res(parse_prefixed_hex, move |hex| u32::from_str_radix(hex, 16));

    // Result => Option, because not all u32 values are valid unicode code points
    map_opt(parse_u32, |value| std::char::from_u32(value))(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char(input: &str) -> IResult<&str, char, VerboseError<&str>> {
    preceded(
        ch('\\'),
        alt((
            parse_unicode,
            value('\n', ch('n')),
            value('\r', ch('r')),
            value('\t', ch('t')),
            value('\u{08}', ch('b')),
            value('\u{0C}', ch('f')),
            value('\\', ch('\\')),
            value('/', ch('/')),
            value('"', ch('"')),
        )),
    )(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    preceded(ch('\\'), multispace1)(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
    Value(String),
}

fn parse_normal_string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let parse_literal = verify(is_not(r#"\""#), |s: &str| !s.is_empty());
    let parse_fragment = alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ));
    fold_many0(
        // Our parser function– parses a single string fragment
        parse_fragment,
        // Our init value, an empty string
        String::new,
        // Our folding function. For each fragment, append the fragment to the string.
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                _ => {}
            }
            string
        },
    )(input)
}

fn parse_raw_str(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    // Count number of leading #
    let (remaining, hash_count) = fold_many0(tag("#"), || 0, |acc, _| acc + 1)(input)?;

    // Match " after leading #
    let (remaining, _) = tag(r#"""#)(remaining)?;

    // Take until closing "# (# repeated hash_count times)
    let closing = pair(tag("\""), count(tag("#"), hash_count));
    let (remaining, (inner, _)) = many_till(take(1u8), closing)(remaining)?;

    // Extract inner range
    let offset = hash_count + 1;
    let length = inner.len();

    Ok((remaining, &input[offset..offset + length]))
}

// input: raw string => parse ${} and \ escape => format string
fn parse_format_string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (remaining, raw_str) = parse_raw_str(input)?;

    let parse_literal = verify(is_not(r#"\$"#), |s: &str| !s.is_empty());

    let parse_fragment = alt((
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
        map(parse_expr_token_chunk, |expr| StringFragment::Value(expr.to_string())),
        map(parse_literal, StringFragment::Literal),
    ));

    let parse_string = fold_many0(parse_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::EscapedChar(c) => string.push(c),
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::Value(s) => string.push_str(s.as_str()),
            _ => {}
        }
        string
    });

    let (_remaining_in_f_str, string) = complete(parse_string)(raw_str)?;
    // complete: assert(remaining_in_f_str == "")

    Ok((remaining, string))
}

pub fn parse_string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    alt((
        // " ... ", normal string
        delimited(ch('"'), parse_normal_string, ch('"')),
        // r#" ... "#, row string
        map(preceded(ch('r'), parse_raw_str), String::from),
        // f#" ... "#, format string
        preceded(ch('f'), parse_format_string),
    ))(input)
}

pub fn parse_literal_string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    alt((
        // " ... ", normal string
        delimited(ch('"'), parse_normal_string, ch('"')),
        // r#" ... "#, row string
        map(preceded(ch('r'), parse_raw_str), String::from),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::string::String;

    use super::*;

    #[test]
    fn test_format_string() {
        assert_eq!(
            parse_string(r#"f"${name}""#),
            Ok(("", String::from("Var(name)")))
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
    fn test_string() {
        assert_eq!(parse_string("\"John\""), Ok(("", String::from("John"))));
        assert!(parse_string("\"John").is_err());
        assert_eq!(
            parse_string(
                r#""hello \
        John""#
            ),
            Ok(("", String::from("hello John")))
        );
        assert_eq!(parse_string(r#"r"John""#), Ok(("", String::from("John"))));
        assert_eq!(
            parse_string(r##"r#"John"#"##),
            Ok(("", String::from("John")))
        );
        assert_eq!(
            parse_string(r###"r##"John"##"###),
            Ok(("", String::from("John")))
        );
    }
}
