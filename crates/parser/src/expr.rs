use std::fmt::Display;

use nom::character::complete::one_of;
use nom::error::VerboseError;
use nom::IResult;
use nom::multi::many0;

use crate::expr_token::ExprToken;
use crate::util::Iter;

// Resolve valid variable or function identifiers
// The identifier can contain only letters (a to z, A to Z), digits (0 to 9), and underscores (_).
// The first character of the identifier must be a letter or underscore
pub fn legal_id(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    nom::combinator::recognize(nom::sequence::pair(
        one_of("_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        many0(one_of(
            "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        )),
    ))(input)
}

#[derive(Debug, PartialEq)]
pub enum ExprChunk {
    Primary(ExprToken),
    PrefixOp(ExprToken, Box<ExprChunk>),
    InfixOp(ExprToken, Box<ExprChunk>, Box<ExprChunk>),
    PostfixOp(ExprToken, Box<ExprChunk>),
}

impl Display for ExprChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprChunk::Primary(token) => write!(f, "{}", token),
            ExprChunk::PrefixOp(token, rhs) => write!(f, "({}{})", token, rhs),
            ExprChunk::InfixOp(token, lhs, rhs) => write!(f, "({}{}{})", lhs, token, rhs),
            ExprChunk::PostfixOp(token, lhs) => write!(f, "({}{})", lhs, token),
        }
    }
}

struct Parser(Iter<ExprToken>);

impl Parser {
    fn new(tokens: Vec<ExprToken>) -> Self {
        Parser(Iter::from(tokens))
    }

    fn precedence(token: &ExprToken) -> u8 {
        match token {
            ExprToken::Or => 50,
            ExprToken::Val(..) => 0,

            ExprToken::Or => 25,
            ExprToken::And => 30,

            ExprToken::Eq | ExprToken::Ne => 40,
            ExprToken::Lt | ExprToken::Gt | ExprToken::Le | ExprToken::Ge => 50,

            ExprToken::Plus | ExprToken::Minus => 60,
            ExprToken::Mul | ExprToken::Div | ExprToken::Mod => 70,

            ExprToken::Not => 80,

            ExprToken::FnCall(..) => 90, // TODO to be check
            ExprToken::Ref(..) => 90,
            ExprToken::Group(..) => 90,
            _ => 0,
        }
    }

    fn parse(&mut self, prec: u8) -> ExprChunk {
        let token = self.0.take_next().unwrap();
        let mut lhs = match token {
            ExprToken::Val(..) | ExprToken::FnCall(..) | ExprToken::Ref(..) => {
                ExprChunk::Primary(token)
            }
            ExprToken::Group(..) => ExprChunk::Primary(token),
            ExprToken::Not => ExprChunk::PrefixOp(
                token,
                Box::new(self.parse(Self::precedence(&ExprToken::Not))),
            ),
            ExprToken::Plus => ExprChunk::PrefixOp(
                token,
                Box::new(self.parse(Self::precedence(&ExprToken::Plus))),
            ),
            ExprToken::Minus => ExprChunk::PrefixOp(
                token,
                Box::new(self.parse(Self::precedence(&ExprToken::Minus))),
            ),
            _ => panic!("Unexpected prefix token {:?}", &token),
        };
        let mut precedence_r = self.0.peek().map_or(0, Self::precedence);

        while prec < precedence_r {
            let token = self.0.take_next().unwrap();
            lhs = match token {
                ExprToken::Or => ExprChunk::InfixOp(
                    token,
                    Box::new(lhs),
                    Box::new(self.parse(Self::precedence(&ExprToken::Or))),
                ),
                ExprToken::And => ExprChunk::InfixOp(
                    token,
                    Box::new(lhs),
                    Box::new(self.parse(Self::precedence(&ExprToken::And))),
                ),
                ExprToken::Eq | ExprToken::Ne => ExprChunk::InfixOp(
                    token,
                    Box::new(lhs),
                    Box::new(self.parse(Self::precedence(&ExprToken::Eq))),
                ),
                ExprToken::Lt | ExprToken::Gt | ExprToken::Le | ExprToken::Ge => {
                    ExprChunk::InfixOp(
                        token,
                        Box::new(lhs),
                        Box::new(self.parse(Self::precedence(&ExprToken::Lt))),
                    )
                }
                ExprToken::Plus | ExprToken::Minus => ExprChunk::InfixOp(
                    token,
                    Box::new(lhs),
                    Box::new(self.parse(Self::precedence(&ExprToken::Plus))),
                ),
                ExprToken::Mul | ExprToken::Div | ExprToken::Mod => ExprChunk::InfixOp(
                    token,
                    Box::new(lhs),
                    Box::new(self.parse(Self::precedence(&ExprToken::Mul))),
                ),
                ExprToken::Not => ExprChunk::PrefixOp(
                    token,
                    Box::new(self.parse(Self::precedence(&ExprToken::Not))),
                ),
                ExprToken::FnCall(..) => ExprChunk::PostfixOp(token, Box::new(lhs)),
                ExprToken::Ref(..) => ExprChunk::PostfixOp(token, Box::new(lhs)),
                ExprToken::Group(..) => ExprChunk::PostfixOp(token, Box::new(lhs)),
                _ => panic!("Unexpected infix or postfix token {:?}", token),
            };
            precedence_r = self.0.peek().map_or(0, Self::precedence);
        }
        lhs
    }
}

#[cfg(test)]
mod tests {
    use crate::{eson, EsonSegment};
    use crate::expr::ExprChunk;
    use crate::expr_token::chunk::ExprTokenChunk;
    use crate::expr_token::ExprToken;

    #[test]
    fn test_expr() {
        let (remaining, expr) = eson(r#"${ 1 + 2 * 3 }"#).unwrap();
        match expr {
            EsonSegment::Expr(chunk) => {
                assert_eq!(
                    chunk,
                    ExprTokenChunk::from(vec![
                        ExprToken::Val(EsonSegment::Int(1)),
                        ExprToken::Plus,
                        ExprToken::Val(EsonSegment::Int(2)),
                        ExprToken::Mul,
                        ExprToken::Val(EsonSegment::Int(3)),
                    ])
                );

                let mut parser = crate::expr::Parser::new(chunk.into());
                let chunk = parser.parse(0);
                assert_eq!(
                    chunk,
                    ExprChunk::InfixOp(
                        ExprToken::Plus,
                        Box::new(ExprChunk::Primary(ExprToken::Val(EsonSegment::Int(1)))),
                        Box::new(ExprChunk::InfixOp(
                            ExprToken::Mul,
                            Box::new(ExprChunk::Primary(ExprToken::Val(EsonSegment::Int(2)))),
                            Box::new(ExprChunk::Primary(ExprToken::Val(EsonSegment::Int(3)))),
                        )),
                    )
                );
            }
            _ => todo!(),
        }
    }

    #[test]
    fn test_expr_with_fncall() {
        let (remaining, expr) = eson(r#"${ 1 + f(a, b) * 2 }"#).unwrap();
        match expr {
            EsonSegment::Expr(chunk) => {
                assert_eq!(
                    chunk,
                    ExprTokenChunk::from(vec![
                        ExprToken::Val(EsonSegment::Int(1)),
                        ExprToken::Plus,
                        ExprToken::FnCall(
                            String::from("f"),
                            vec![
                                ExprTokenChunk::from(vec![ExprToken::Var("a".to_string())]),
                                ExprTokenChunk::from(vec![ExprToken::Var("b".to_string())]),
                            ],
                        ),
                        ExprToken::Mul,
                        ExprToken::Val(EsonSegment::Int(2)),
                    ])
                );

                let mut parser = crate::expr::Parser::new(chunk.into());
                let chunk = parser.parse(0);
                assert_eq!(
                    chunk,
                    ExprChunk::InfixOp(
                        ExprToken::Plus,
                        Box::new(ExprChunk::Primary(ExprToken::Val(EsonSegment::Int(1)))),
                        Box::new(ExprChunk::InfixOp(
                            ExprToken::Mul,
                            Box::new(ExprChunk::Primary(ExprToken::FnCall(
                                String::from("f"),
                                vec![
                                    ExprTokenChunk::from(vec![ExprToken::Var("a".to_string())]),
                                    ExprTokenChunk::from(vec![ExprToken::Var("b".to_string())]),
                                ],
                            ))),
                            Box::new(ExprChunk::Primary(ExprToken::Val(EsonSegment::Int(2)))),
                        )),
                    )
                );
            }
            _ => todo!(),
        }
    }
}
