use knot::lexer::lexer::tokenize;
use knot::lexer::num::Num;
use knot::lexer::token::Token;

fn assert_tokens(source: &str, expected: Vec<Token>) {
    let (tokens, _) = tokenize(source);
    assert_eq!(
        tokens, expected,
        "\nsource: {:?}\nexpected: {:?}\nactual:   {:?}",
        source, expected, tokens,
    );
}

#[test]
fn func_def() {
    assert_tokens(
        "func main() -> I32 { return 42 }",
        vec![
            Token::Keyword("func".into()),
            Token::Identifier("main".into()),
            Token::Operator("(".into()),
            Token::Operator(")".into()),
            Token::Operator("->".into()),
            Token::Identifier("I32".into()),
            Token::Operator("{".into()),
            Token::Keyword("return".into()),
            Token::Number(Num::I32(42)),
            Token::Operator("}".into()),
            Token::Eof,
        ],
    );
}

#[test]
fn hex_literal() {
    assert_tokens(
        "0xFF",
        vec![Token::Number(Num::I32(255)), Token::Eof],
    );
}

#[test]
fn hex_with_suffix() {
    assert_tokens(
        "0xFFi32",
        vec![Token::Number(Num::I32(255)), Token::Eof],
    );
}

#[test]
fn bin_literal() {
    assert_tokens(
        "0b1010",
        vec![Token::Number(Num::I32(10)), Token::Eof],
    );
}

#[test]
fn oct_literal() {
    assert_tokens(
        "0o777",
        vec![Token::Number(Num::I32(511)), Token::Eof],
    );
}

#[test]
fn float_with_suffix() {
    assert_tokens(
        "3.14f32",
        vec![Token::Number(Num::F32(3.14)), Token::Eof],
    );
}

#[test]
fn int_with_suffix() {
    assert_tokens(
        "42i64",
        vec![Token::Number(Num::I64(42)), Token::Eof],
    );
}

#[test]
fn underscore_separator() {
    assert_tokens(
        "1_000_000",
        vec![Token::Number(Num::I32(1000000)), Token::Eof],
    );
}

#[test]
fn string_with_escapes() {
    assert_tokens(
        r#""hello\nworld""#,
        vec![Token::String("hello\nworld".into()), Token::Eof],
    );
}

#[test]
fn operators_multi_char() {
    assert_tokens(
        "== != <= >= && || -> => :: .. ??",
        vec![
            Token::Operator("==".into()),
            Token::Operator("!=".into()),
            Token::Operator("<=".into()),
            Token::Operator(">=".into()),
            Token::Operator("&&".into()),
            Token::Operator("||".into()),
            Token::Operator("->".into()),
            Token::Operator("=>".into()),
            Token::Operator("::".into()),
            Token::Operator("..".into()),
            Token::Operator("??".into()),
            Token::Eof,
        ],
    );
}

#[test]
fn keywords() {
    assert_tokens(
        "func if else while for return class enum mixin static true false null break continue throw try catch match import as operator private abstract args kwargs wrap assert delete new",
        vec![
            Token::Keyword("func".into()),
            Token::Keyword("if".into()),
            Token::Keyword("else".into()),
            Token::Keyword("while".into()),
            Token::Keyword("for".into()),
            Token::Keyword("return".into()),
            Token::Keyword("class".into()),
            Token::Keyword("enum".into()),
            Token::Keyword("mixin".into()),
            Token::Keyword("static".into()),

            Token::Keyword("true".into()),
            Token::Keyword("false".into()),
            Token::Keyword("null".into()),
            Token::Keyword("break".into()),
            Token::Keyword("continue".into()),
            Token::Keyword("throw".into()),
            Token::Keyword("try".into()),
            Token::Keyword("catch".into()),
            Token::Keyword("match".into()),
            Token::Keyword("import".into()),
            Token::Keyword("as".into()),
            Token::Keyword("operator".into()),
            Token::Keyword("private".into()),
            Token::Keyword("abstract".into()),
            Token::Keyword("args".into()),
            Token::Keyword("kwargs".into()),
            Token::Keyword("wrap".into()),
            Token::Keyword("assert".into()),
            Token::Keyword("delete".into()),
            Token::Keyword("new".into()),
            Token::Eof,
        ],
    );
}

#[test]
fn newlines_as_statement_separators() {
    assert_tokens(
        "x\ny\n",
        vec![
            Token::Identifier("x".into()),
            Token::NewLine,
            Token::Identifier("y".into()),
            Token::NewLine,
            Token::Eof,
        ],
    );
}

#[test]
fn invalid_character() {
    let (tokens, _) = tokenize("$");
    assert!(matches!(tokens[0], Token::Error(_)));
}

#[test]
fn unterminated_string() {
    let (tokens, _) = tokenize("\"unclosed");
    assert!(matches!(tokens[0], Token::Error(_)));
}

#[test]
fn unterminated_block_comment() {
    let (tokens, _) = tokenize("/* unclosed block comment");
    assert!(matches!(tokens[0], Token::Error(_)));
}

#[test]
fn comment_skipped() {
    assert_tokens(
        "x // comment\ny",
        vec![
            Token::Identifier("x".into()),
            Token::NewLine,
            Token::Identifier("y".into()),
            Token::Eof,
        ],
    );
}

#[test]
fn block_comment_skipped() {
    assert_tokens(
        "x /* comment */ y",
        vec![
            Token::Identifier("x".into()),
            Token::Identifier("y".into()),
            Token::Eof,
        ],
    );
}

#[test]
fn as_bang_keyword() {
    assert_tokens(
        "as!",
        vec![Token::Keyword("as!".into()), Token::Eof],
    );
}

#[test]
fn integer_types() {
    assert_tokens(
        "42i8 42i16 42i32 42i64",
        vec![
            Token::Number(Num::I8(42)),
            Token::Number(Num::I16(42)),
            Token::Number(Num::I32(42)),
            Token::Number(Num::I64(42)),
            Token::Eof,
        ],
    );
}

#[test]
fn unsigned_types() {
    assert_tokens(
        "42u8 42u16 42u32 42u64",
        vec![
            Token::Number(Num::U8(42)),
            Token::Number(Num::U16(42)),
            Token::Number(Num::U32(42)),
            Token::Number(Num::U64(42)),
            Token::Eof,
        ],
    );
}

#[test]
fn float_types() {
    assert_tokens(
        "3.14f32 3.14f64",
        vec![
            Token::Number(Num::F32(3.14)),
            Token::Number(Num::F64(3.14)),
            Token::Eof,
        ],
    );
}
