use crate::ast::ByteOffset;
use crate::lexer::Token;

pub type Result<T> = std::result::Result<T, miette::Error>;

pub static ADDRESS_HELP: &str = "valid addresses takes the form of &FFFF";
pub static ADDRESS_MSG: &str = "[SYNTAX_ERROR]: expected address";

pub static HEX_LIT_HELP: &str = "valid hex literals takes the form of $FFFF";
pub static HEX_LIT_MSG: &str = "[SYNTAX_ERROR]: expected hex literal";

pub static PATH_MSG: &str = "[SYNTAX_ERROR]: expected path string";

pub static IDENT_MSG: &str = "[SYNTAX_ERROR]: expected valid identifier";

pub static EOF_MSG: &str = "[SYNTAX_ERROR]: unexpected end of file [EOF]";

pub static UNEXPECTED_TOKEN_MSG: &str = "[SYNTAX_ERROR]: unexpected token";

pub static COLON_MSG: &str = "[SYNTAX_ERROR]: expected a `:` [COLON]";
pub static DOT_MSG: &str = "[SYNTAX_ERROR]: expected a `.` [DOT]";
pub static PLUS_MSG: &str = "[SYNTAX_ERROR]: expected a `+` [PLUS]";
pub static COMMA_MSG: &str = "[SYNTAX_ERROR]: expected a `,` [COMMA]";
pub static LBRACE_MSG: &str = "[SYNTAX_ERROR]: expected a `{` [LEFT_CURLY]";
pub static RBRACE_MSG: &str = "[SYNTAX_ERROR]: expected a `}` [RIGHT_CURLY]";
pub static LBRACKET_MSG: &str = "[SYNTAX_ERROR]: expected a `[` [LEFT_BRACKET]";
pub static RBRACKET_MSG: &str = "[SYNTAX_ERROR]: expected a `]` [RIGHT_BRACKET]";

pub fn bail<S: AsRef<str>>(source: S, help: S, message: S, offset: impl Into<ByteOffset>) -> miette::Error {
    let offset = offset.into();
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(vec![miette::LabeledSpan::at(offset.start..offset.end, "this bit")])
            .with_help(help.as_ref()),
    )
    .with_source_code(source.as_ref().to_string())
}

pub fn unexpected_eof<S: AsRef<str>, T>(source: S, help: S) -> Result<T> {
    let end = source.as_ref().len();
    let start = end.saturating_sub(1);
    Err(bail(source.as_ref(), help.as_ref(), EOF_MSG, start..end))
}

pub fn unexpected_token<S: AsRef<str>, T>(source: S, token: &Token) -> Result<T> {
    Err(bail(
        source.as_ref(),
        &format!("unexpected token {token}"),
        UNEXPECTED_TOKEN_MSG,
        token.offset(),
    ))
}
