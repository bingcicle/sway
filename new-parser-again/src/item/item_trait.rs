use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemTrait {
    pub visibility: Option<PubToken>,
    pub trait_token: TraitToken,
    pub name: Ident,
    pub super_traits: Option<(ColonToken, Traits)>,
    pub trait_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
    pub trait_defs_opt: Option<Braces<Vec<ItemFn>>>,
}

#[derive(Clone, Debug)]
pub struct Traits {
    pub prefix: Ty,
    pub suffixes: Vec<(AddToken, Ty)>,
}

impl Parse for ItemTrait {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTrait> {
        let visibility = parser.take();
        let trait_token = parser.parse()?;
        let name = parser.parse()?;
        let super_traits = match parser.take() {
            Some(colon_token) => {
                let traits = parser.parse()?;
                Some((colon_token, traits))
            },
            None => None,
        };
        let trait_items = parser.parse()?;
        let trait_defs_opt = Braces::try_parse(parser)?;
        Ok(ItemTrait { visibility, trait_token, name, super_traits, trait_items, trait_defs_opt })
    }
}

impl Parse for Traits {
    fn parse(parser: &mut Parser) -> ParseResult<Traits> {
        let prefix = parser.parse()?;
        let mut suffixes = Vec::new();
        loop {
            let add_token = match parser.take() {
                Some(add_token) => add_token,
                None => break,
            };
            let suffix = parser.parse()?;
            suffixes.push((add_token, suffix));
        }
        let traits = Traits { prefix, suffixes };
        Ok(traits)
    }
}

