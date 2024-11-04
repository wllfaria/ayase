pub type Result<T> = std::result::Result<T, miette::Error>;

pub static ADDRESS_HELP: &str = "valid addresses takes the form of &FFFF";
pub static ADDRESS_MSG: &str = "[SYNTAX_ERROR]: expected address";

pub static HEX_LIT_HELP: &str = "valid hex literals takes the form of $FFFF";
pub static HEX_LIT_MSG: &str = "[SYNTAX_ERROR]: expected hex literal";

pub static VAR_MSG: &str = "[SYNTAX_ERROR]: variable name must be a valid identifier";
pub static VAR_HELP: &str = "variables must start with a ! [BANG] followed by a valid identifier";

pub static REGISTER_MSG: &str = "[SYNTAX_ERROR]: invalid register name";
pub static REGISTER_HELP: &str = "register name must be in the set of valid registers";

pub static BRACKETED_EXPR_HELP: &str = "invalid bracketed expression";
pub static BRACKETED_EXPR_MSG: &str = "[SYNTAX_ERROR]: invalid bracketed expression";

pub static UNTERMINATED_STRING_HELP: &str = "did you forget a closing \"";
pub static UNTERMINATED_STRING_MSG: &str = "unterminated string";

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
