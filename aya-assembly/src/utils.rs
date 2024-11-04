use crate::lexer::Token;
use crate::parser::error::{EOF_MSG, UNEXPECTED_TOKEN_MSG};

pub fn bail<S: AsRef<str>>(source: S, help: S, message: S, offset: impl Into<miette::SourceSpan>) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(vec![miette::LabeledSpan::at(offset, "this bit")])
            .with_help(help.as_ref()),
    )
    .with_source_code(source.as_ref().to_string())
}

pub fn bail_multi<S: AsRef<str>>(
    source: &str,
    labels: impl IntoIterator<Item = miette::LabeledSpan>,
    message: S,
    help: S,
) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(labels)
            .with_help(help.as_ref()),
    )
    .with_source_code(source.to_string())
}

pub fn unexpected_eof<S: AsRef<str>, T>(source: S, help: S) -> miette::Result<T> {
    let end = source.as_ref().len();
    let start = end.saturating_sub(1);
    Err(bail(source.as_ref(), help.as_ref(), EOF_MSG, start..end))
}

pub fn unexpected_token<S: AsRef<str>, T>(source: S, token: &Token) -> miette::Result<T> {
    Err(bail(
        source.as_ref(),
        &format!("unexpected token {token}"),
        UNEXPECTED_TOKEN_MSG,
        token.offset(),
    ))
}

pub fn unexpected_statement<S: AsRef<str>, T>(
    source: S,
    help: S,
    offset: impl Into<miette::SourceSpan>,
) -> miette::Result<T> {
    Err(bail(
        source.as_ref(),
        help.as_ref(),
        "[SYNTAX_ERROR]: unexpected statement",
        offset,
    ))
}
