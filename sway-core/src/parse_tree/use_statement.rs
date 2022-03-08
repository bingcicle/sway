use crate::Rule;
use pest::iterators::Pair;

pub(crate) fn item_use_parse_from_pair(
    pair: Pair<Rule>,
) -> new_parser_again::ItemUse {
    let pest_span = pair.as_span();
    let src = pest_span.input();
    let start = pest_span.start();
    let end = pest_span.end();
    new_parser_again::lex_and_parse(src, start, end)
}

