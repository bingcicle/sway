pub fn ident_from(ident: &new_parser_again::Ident) -> sway_types::Ident {
    let span = ident.span();
    let span = span_from(&span);
    sway_types::Ident::new(span)
}

pub fn span_from(span: &new_parser_again::Span) -> sway_types::Span {
    let src = span.src();
    let start = span.start();
    let end = span.end();
    let pest_span = pest::Span::new(src.clone(), start, end).unwrap();
    let span = sway_types::Span {
        span: pest_span,
        path: None,
    };
    span
}
