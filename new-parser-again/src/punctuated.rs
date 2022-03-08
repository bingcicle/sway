use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Punctuated<T, P> {
    pub value_separator_pairs: Vec<(T, P)>,
    pub final_value_opt: Option<Box<T>>,
}

impl<T, P> ParseToEnd for Punctuated<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(Punctuated<T, P>, ParserConsumed<'a>)> {
        let mut value_separator_pairs = Vec::new();
        loop {
            if let Some(consumed) = parser.check_empty() {
                let punctuated = Punctuated {
                    value_separator_pairs,
                    final_value_opt: None,
                };
                return Ok((punctuated, consumed));
            }
            let value = parser.parse()?;
            if let Some(consumed) = parser.check_empty() {
                let punctuated = Punctuated {
                    value_separator_pairs,
                    final_value_opt: Some(Box::new(value)),
                };
                return Ok((punctuated, consumed));
            }
            let separator = parser.parse()?;
            value_separator_pairs.push((value, separator));
        }
    }
}

impl<'a, T, P> IntoIterator for &'a Punctuated<T, P> {
    type Item = &'a T;
    type IntoIter = PunctuatedRefIter<'a, T, P>;
    fn into_iter(self) -> PunctuatedRefIter<'a, T, P> {
        PunctuatedRefIter {
            punctuated: self,
            index: 0,
        }
    }
}

pub struct PunctuatedRefIter<'a, T, P> {
    punctuated: &'a Punctuated<T, P>,
    index: usize,
}

impl<'a, T, P> Iterator for PunctuatedRefIter<'a, T, P> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.index > self.punctuated.value_separator_pairs.len() {
            return None;
        } else {
            match self.punctuated.value_separator_pairs.get(self.index) {
                None => {
                    match &self.punctuated.final_value_opt {
                        Some(value) => {
                            self.index += 1;
                            Some(value)
                        },
                        None => None,
                    }
                },
                Some((value, _separator)) => {
                    self.index += 1;
                    Some(value)
                },
            }
        }
    }
}

