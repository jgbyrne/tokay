use crate::reader::Reader;
use crate::tokay::*;
use crate::token::*;
use crate::value::{RefValue, Value};
use crate::compiler::Compiler;
use crate::{tokay, tokay_item, ccl};


pub struct TokayParser(Program);

macro_rules! emit {
    ( $string:literal ) => {
        Atomic::Push(Value::String($string.to_string()).into_ref()).into_box()
    }
}

impl TokayParser {
    pub fn new() -> Self {
        Self(
            tokay!({
// ----------------------------------------------------------------------------

(_ = {
    [' '],
    ['#', (Atomic::Token(UntilChar::new('\n', None)).into_box())]
}),

(T_EOL = {
    [
        (Atomic::Token(Char::new(ccl!['\n'..='\n'])).into_box()),
        _,
        (Atomic::Accept(None).into_box())
    ]
}),

// Basic Tokens (might probably be replaced by something native, pluggable one)

/*
(Identifier = {
    [
        (Atomic::Token(Char::new(
            ccl!['A'..='Z', 'a'..='z', '_'..='_'])).into_box()
        ),
        (Atomic::Token(Chars::new(
            ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
        ).into_box())
    ]
}),
*/

(T_Variable = {
    [
        (Atomic::Token(Char::new(ccl!['a'..='z'])).into_box()),
        (Repeat::new(
            (Atomic::Token(Chars::new(
                ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
            ).into_box()),
            0, 0, false
        ).into_box())
    ]
}),

(T_Constant = {
    [
        (Atomic::Token(Char::new(ccl!['A'..='Z', '_'..='_'])).into_box()),
        (Repeat::new(
            (Atomic::Token(Chars::new(
                ccl!['A'..='Z', 'a'..='z', '0'..='9', '_'..='_'])
            ).into_box()),
            0, 0, false
        ).into_box())
    ]
}),

(T_HeavyString = {
    [
        '"', (Atomic::Token(UntilChar::new('"', Some('\\'))).into_box()), '"'
    ]
}),

(T_LightString = {
    [
        (Atomic::Token(Char::new(ccl!['\''..='\''])).into_box()),
        (Atomic::Token(UntilChar::new('\'', Some('\\'))).into_box()),
        (Atomic::Token(Char::new(ccl!['\''..='\''])).into_box())
    ]
}),

// Structure

(S_Parselet = {
    ['@', _, S_Block]
}),

(S_Block = {
    ['{', _, S_Sequences, '}', _]
}),

(S_Sequences = {
    [S_Sequences, T_EOL, S_Sequence],
    [S_Sequence],
    [T_EOL]
}),

(S_Sequence = {
    (S_Sequence1 = {
        [S_Sequence1, S_Atomic],
        [S_Atomic]
    }),

    [T_Constant, _, '=', _, S_Parselet],
    [S_Sequence1]
}),

(S_Atomic = {
    [T_HeavyString, _],
    [T_LightString, _],
    [T_Constant, _]
}),

[S_Sequence]
// ----------------------------------------------------------------------------
            })
        )
    }

    pub fn parse(&self, code: &'static str) -> Result<Value, String> {
        //self.0.dump();

        let mut reader = Reader::new(
            Box::new(std::io::Cursor::new(code))
        );

        let mut runtime = Runtime::new(&self.0, &mut reader);

        if let Ok(accept) = self.0.run(&mut runtime) {
            println!("{:#?}", accept);
            if let Accept::Push(Capture::Value(value, _)) = accept {
                return Ok(RefValue::into_value(value).unwrap());
            }
        }

        return Err("Parse error?".to_string())
    }
}