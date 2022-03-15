use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemEnum {
    pub visibility: Option<PubToken>,
    pub enum_token: EnumToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub fields: Braces<Punctuated<TypeField, CommaToken>>,
}

impl ItemEnum {
    pub fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.enum_token.span(),
        };
        let end = self.fields.span();
        Span::join(start.clone(), end)
    }
}

impl Parse for ItemEnum {
    fn parse(parser: &mut Parser) -> ParseResult<ItemEnum> {
        let visibility = parser.take();
        let enum_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<LessThanToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let fields = parser.parse()?;
        Ok(ItemEnum { visibility, enum_token, name, generics, fields })
    }
}

