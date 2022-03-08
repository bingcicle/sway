use std::{path::PathBuf, sync::Arc, borrow::Cow};

/// Represents a span of the source code in a specific file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    ///  A [pest::Span] returned directly from the generated parser.
    span: pest::Span,
    // A reference counted pointer to the file from which this span originated.
    path: Option<Arc<PathBuf>>,
}

impl Span {
    pub fn from_pest(pest_span: pest::Span, path: Option<Arc<PathBuf>>) -> Span {
        Span {
            span: pest_span,
            path,
        }
    }

    /*
    pub fn from_pest_no_path(pest_span: pest::Span) -> Span {
        Span {
            span: pest_span,
            path: None,
        }
    }
    */

    pub fn new(src: Arc<str>, start: usize, end: usize, path: Option<Arc<PathBuf>>) -> Option<Span> {
        Some(Span {
            span: pest::Span::new(src, start, end)?,
            path,
        })
    }

    /*
    pub fn new_no_path(src: Arc<str>, start: usize, end: usize) -> Option<Span> {
        Some(Span {
            span: pest::Span::new(src, start, end)?,
            path: None,
        })
    }
    */

    pub fn src(&self) -> &Arc<str> {
        self.span.input()
    }

    pub fn start(&self) -> usize {
        self.span.start()
    }

    pub fn end(&self) -> usize {
        self.span.end()
    }

    pub fn path(&self) -> Option<&Arc<PathBuf>> {
        self.path.as_ref()
    }

    pub fn path_str(&self) -> Option<Cow<'_, str>> {
        self.path.as_deref().map(|path| path.to_string_lossy())
    }

    pub fn start_pos(&self) -> pest::Position {
        self.span.start_pos()
    }

    pub fn end_pos(&self) -> pest::Position {
        self.span.end_pos()
    }

    pub fn split(&self) -> (pest::Position, pest::Position) {
        self.span.clone().split()
    }

    pub fn str(self) -> String {
        self.span.as_str().to_string()
    }

    pub fn as_str(&self) -> &str {
        self.span.as_str()
    }

    pub fn input(&self) -> &str {
        self.span.input()
    }

    /*
    pub fn path(&self) -> String {
        self.path
            .as_deref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "".to_string())
    }
    */

    pub fn trim(self) -> Span {
        let start_delta = self.as_str().len() - self.as_str().trim_start().len();
        let end_delta = self.as_str().len() - self.as_str().trim_end().len();
        let span = pest::Span::new(
            self.span.input().clone(),
            self.span.start() + start_delta,
            self.span.end() - end_delta,
        )
        .unwrap();
        Span {
            span,
            path: self.path,
        }
    }
}

/// This panics if the spans are not from the same file. This should
/// only be used on spans that are actually next to each other.
pub fn join_spans(s1: Span, s2: Span) -> Span {
    if s1.as_str() == "core" {
        return s2;
    }
    assert!(
        s1.input() == s2.input() && s1.path == s2.path,
        "Spans from different files cannot be joined.",
    );

    let s1_positions = s1.split();
    let s2_positions = s2.split();
    if s1_positions.0 < s2_positions.1 {
        Span {
            span: s1_positions.0.span(&s2_positions.1),
            path: s1.path,
        }
    } else {
        Span {
            span: s2_positions.0.span(&s1_positions.1),
            path: s1.path,
        }
    }
}
