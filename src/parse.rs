use std::collections::HashMap;

use crate::{
    tokenize::{tokenize, Token, TokenizeError},
    Value,
};

type ParseResult = Result<Value, TokenParseError>;

fn parse(input: String) -> Result<Value, ParseError> {
    let tokens = tokenize(input)?;
    let value = parse_tokens(&tokens, &mut 0)?;
    Ok(value)
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    ParseError(TokenParseError),
}

impl From<TokenizeError> for ParseError {
    fn from(value: TokenizeError) -> Self {
        Self::TokenizeError(value)
    }
}

impl From<TokenParseError> for ParseError {
    fn from(value: TokenParseError) -> Self {
        Self::ParseError(value)
    }
}

fn parse_tokens(tokens: &[Token], index: &mut usize) -> ParseResult {
    let token = &tokens[*index];
    if matches!(
        token,
        Token::Null | Token::False | Token::True | Token::Number(_) | Token::String(_)
    ) {
        *index += 1;
    }
    match token {
        Token::Null => Ok(Value::Null),
        Token::False => Ok(Value::Boolean(false)),
        Token::True => Ok(Value::Boolean(true)),
        Token::Number(num) => Ok(Value::Number(*num)),
        Token::String(s) => parse_string(s),
        Token::LeftBracket => parse_array(tokens, index),
        Token::LeftBrace => parse_object(tokens, index),
        _ => todo!(),
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenParseError {
    /// An escape sequence was started without 4 hexadecimaldigits afterwards
    UnfinishedEscape,

    /// A character in an escape sequence was not valid hexadecimal
    InvalidHexValue,

    /// Invalid unicode value
    InvalidCodePointValue,

    ExpectedComma,
    ExpectedProperty,
    ExpectedColon,
}

fn parse_string(input: &str) -> ParseResult {
    let mut output = String::new();

    let mut is_escaping = false;
    let mut chars = input.chars();
    while let Some(next_char) = chars.next() {
        if is_escaping {
            match next_char {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'b' => output.push('\u{8}'),
                'f' => output.push('\u{12}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    let mut sum = 0;
                    for i in 0..4 {
                        let next_char = chars.next().ok_or(TokenParseError::UnfinishedEscape)?;
                        let digit = next_char
                            .to_digit(16)
                            .ok_or(TokenParseError::InvalidHexValue)?;
                        sum += (16u32).pow(3 - i) * digit;
                    }
                    let unescaped_char =
                        char::from_u32(sum).ok_or(TokenParseError::InvalidCodePointValue)?;
                    output.push(unescaped_char);
                }
                _ => output.push(next_char),
            }
            is_escaping = false;
        } else if next_char == '\\' {
            is_escaping = true;
        } else {
            output.push(next_char);
        }
    }
    Ok(Value::String(output))
}

// [null, [null]]
fn parse_array(tokens: &[Token], index: &mut usize) -> ParseResult {
    let mut array = Vec::new();
    println!("token= {:?}, index= {index}", tokens[*index]);

    loop {
        // if *index == tokens.len() {
        //     break;
        // }
        *index += 1;
        if tokens[*index] == Token::RightBracket {
            break;
        }
        // println!("token= {:?}, index= {index}", tokens[*index]);

        let value = parse_tokens(tokens, index)?;
        array.push(value);

        // *index += 1;
        let token = &tokens[*index];

        match token {
            // ','就继续解析下一个token
            Token::Comma => {}
            // ']'表示结束
            Token::RightBracket => break,
            _ => return Err(TokenParseError::ExpectedComma),
        }
    }
    *index += 1;

    Ok(Value::Array(array))
}

fn parse_object(tokens: &[Token], index: &mut usize) -> ParseResult {
    // OK cases
    // LeftBrace -> RightBrace
    // LeftBrace -> String -> Colon -> Value -> RightBrace
    // LeftBrace -> [String -> Colon -> Value] -> Comma -> (repeat [*]) -> RightBrace
    let mut object = HashMap::new();

    loop {
        *index += 1;

        if tokens[*index] == Token::RightBrace {
            break;
        }
        // { string1 : value1, string2 : value2, string3 : value3 }
        if let Token::String(s) = &tokens[*index] {
            *index += 1;
            if let Token::Colon = tokens[*index] {
                *index += 1;
                let key = s.clone();
                let vlaue = parse_tokens(tokens, index)?;
                object.insert(key, vlaue);

                match &tokens[*index] {
                    Token::Comma => {}
                    Token::RightBrace => break,
                    _ => return Err(TokenParseError::ExpectedComma),
                }
            } else {
                return Err(TokenParseError::ExpectedColon);
            }
        } else {
            return Err(TokenParseError::ExpectedProperty);
        }
    }

    *index += 1;
    Ok(Value::Object(object))
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::parse_tokens;
    use crate::tokenize::Token;
    use crate::Value;

    fn check(input: &[Token], expected: Value) {
        let actual = parse_tokens(input, &mut 0).unwrap();
        assert_eq!(actual, expected);
    }
    #[test]
    fn parses_null() {
        let input = [Token::Null];
        let expected = Value::Null;

        check(&input, expected);
    }

    #[test]
    fn parses_string_no_escapes() {
        let input = [Token::String("hello world".into())];
        let expected = Value::String("hello world".into());

        check(&input, expected);
    }

    #[test]
    fn parses_string_non_ascii() {
        let input = [Token::String("olá_こんにちは_नमस्ते_привіт".into())];
        let expected = Value::String("olá_こんにちは_नमस्ते_привіт".into());

        check(&input, expected);
    }

    #[test]
    fn parses_string_with_emoji() {
        let input = [Token::String("hello 💩 world".into())];
        let expected = Value::String("hello 💩 world".into());

        check(&input, expected);
    }

    #[test]
    fn parses_string_unescape_backslash() {
        let input = [Token::String(r#"hello\\world"#.into())];
        let expected = Value::String(r#"hello\\world"#.into());

        check(&input, expected);
    }

    #[test]
    fn parses_array_one_element() {
        let input = [Token::LeftBracket, Token::True, Token::RightBracket];
        let expected = Value::Array(vec![Value::Boolean(true)]);

        check(&input, expected);
    }

    #[test]
    fn parses_array_two_elements() {
        let input = [
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::Number(16.0),
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Number(16.0)]);

        check(&input, expected);
    }

    #[test]
    fn parses_empty_array() {
        // []
        let input = [Token::LeftBracket, Token::RightBracket];
        let expected = Value::Array(vec![]);

        check(&input, expected);
    }

    #[test]
    fn parses_nested_array() {
        // [null, [null]]
        let input = [
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::LeftBracket,
            Token::Null,
            Token::RightBracket,
            Token::RightBracket,
        ];

        let expected = Value::Array(vec![Value::Null, Value::Array(vec![Value::Null])]);
        check(&input, expected);
    }

    #[test]
    fn parse_empty_object() {
        let input = [Token::LeftBrace, Token::RightBrace];
        let expected = Value::Object(HashMap::new());

        check(&input, expected);
    }

    #[test]
    fn parse_object_one_string_value() {
        let input = [
            Token::LeftBrace,
            Token::String("lyf".to_string()),
            Token::Colon,
            Token::String("QAQ".to_string()),
            Token::RightBrace,
        ];
        let expected = Value::Object(HashMap::from([(
            "lyf".to_string(),
            Value::String("QAQ".to_string()),
        )]));

        check(&input, expected);
    }

    #[test]
    fn parses_object_escaped_key() {
        let input = [
            Token::LeftBrace,
            Token::String(r#"\u540D\u524D"#.to_string()),
            Token::Colon,
            Token::String("davimiku".to_string()),
            Token::RightBrace,
        ];
        let expected = Value::Object(
            [(
                "\\u540D\\u524D".to_string(),
                Value::String("davimiku".to_string()),
            )]
            .into(),
        );

        check(&input, expected);
    }
}
