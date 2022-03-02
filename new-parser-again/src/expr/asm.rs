use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct AsmBlock {
    pub asm_token: AsmToken,
    pub registers: Parens<Punctuated<AsmRegisterDeclaration, CommaToken>>,
    pub contents: Braces<AsmBlockContents>,
}

#[derive(Clone, Debug)]
pub struct AsmRegisterDeclaration {
    pub register: Ident,
    pub value_opt: Option<(ColonToken, Box<Expr>)>,
}

#[derive(Clone, Debug)]
pub struct AsmBlockContents {
    pub instructions: Vec<(Instruction, SemicolonToken)>,
    pub final_expr_opt: Option<AsmFinalExpr>,
}

#[derive(Clone, Debug)]
pub struct AsmFinalExpr {
    pub register: Ident,
    pub ty_opt: Option<(ColonToken, Ty)>,
}

#[derive(Clone, Debug)]
pub struct AsmImmediate {
    pub span: Span,
    pub parsed: BigUint,
}

impl Parse for AsmBlock {
    fn parse(parser: &mut Parser) -> ParseResult<AsmBlock> {
        let asm_token = parser.parse()?;
        let registers = parser.parse()?;
        let contents = parser.parse()?;
        Ok(AsmBlock { asm_token, registers, contents })
    }
}

impl Parse for AsmRegisterDeclaration {
    fn parse(parser: &mut Parser) -> ParseResult<AsmRegisterDeclaration> {
        let register = parser.parse()?;
        let value_opt = match parser.take() {
            Some(colon_token) => {
                let value = parser.parse()?;
                Some((colon_token, value))
            },
            None => None,
        };
        Ok(AsmRegisterDeclaration { register, value_opt })
    }
}

impl ParseToEnd for AsmBlockContents {
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(AsmBlockContents, ParserConsumed<'a>)> {
        let mut instructions = Vec::new();
        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }
            let ident = parser.parse()?;
            if let Some(consumed) = parser.check_empty() {
                let final_expr = AsmFinalExpr {
                    register: ident,
                    ty_opt: None,
                };
                break (Some(final_expr), consumed);
            }
            if let Some(colon_token) = parser.take() {
                let ty = parser.parse()?;
                let consumed = match parser.check_empty() {
                    Some(consumed) => consumed,
                    None => {
                        return Err(parser.emit_error("unexpected tokens after final expression in asm block"));
                    },
                };
                let final_expr = AsmFinalExpr {
                    register: ident,
                    ty_opt: Some((colon_token, ty)),
                };
                break (Some(final_expr), consumed);
            }
            let instruction = parse_instruction(ident, &mut parser)?;
            let semicolon_token = parser.parse()?;
            instructions.push((instruction, semicolon_token));
        };
        let contents = AsmBlockContents { instructions, final_expr_opt };
        Ok((contents, consumed))
    }
}

/*
impl ParseToEnd for AsmFinalExpr {
    fn parse_to_end<'a>(parser: Parser<'a>) -> ParseResult<(AsmFinalExpr, ParserConsumed<'a>)> {
        let register = parser.parse()?;
        let ty_opt = match parser.take() {
            Some(colon_token) => {
                let ty = parser.parse()?;
                Some((colon_token, ty))
            },
            None => None,
        };
        let consumed = match parser.check_empty() {
            Some(consumed) => consumed,
            None => {
                return Err(parser.emit_error("unexpected tokens after final expression in asm block"));
            },
        };
        let final_expr = AsmFinalExpr { register, ty_opt };
        Ok((final_expr, consumed))
    }
}

impl Parse for AsmInstruction {
    fn parse(parser: &mut Parser) -> ParseResult<AsmInstruction> {
        let op_code = parser.parse()?;
        let mut args = Vec::new();
        let semicolon_token = loop {
            if let Some(semicolon_token) = parser.take() {
                break semicolon_token;
            }
            let arg = parser.parse()?;
            args.push(arg);
        };
        Ok(AsmInstruction { op_code, args, semicolon_token })
    }
}
*/

impl Parse for AsmImmediate {
    fn parse(parser: &mut Parser) -> ParseResult<AsmImmediate> {
        let ident = parser.parse::<Ident>()?;
        let digits = match ident.as_str().strip_prefix("i") {
            Some(digits) => digits,
            None => return Err(parser.emit_error("immediate values must start with 'i'")),
        };
        let parsed = match BigUint::from_str(&digits).ok() {
            Some(parsed) => parsed,
            None => return Err(parser.emit_error("unable to parse immediate value")),
        };
        Ok(AsmImmediate {
            span: ident.span(),
            parsed,
        })
    }
}

