use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{char as ch, multispace0};
use nom::error::VerboseError;
use nom::IResult;
use nom::sequence::{preceded, terminated};

fn sp(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(input)
}

pub(crate) fn comment(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    let (remaining, _) = preceded(multispace0, tag("//"))(input)?;
    preceded(sp, terminated(take_until("\n"), ch('\n')))(remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(comment("// hello\n"), Ok(("", "hello")));
        assert_eq!(comment("// hello\nworld"), Ok(("world", "hello")));
        assert_eq!(
            comment(
                r#"// hello
        @world"#
            ),
            Ok(("        @world", "hello"))
        );
    }
}
