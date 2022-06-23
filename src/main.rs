use std::collections::HashMap;

use escape8259::unescape;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1, escaped},
    combinator::{map_res, value, map, recognize},
    error::{FromExternalError, ParseError},
    number::complete::recognize_float,
    sequence::{delimited, terminated, separated_pair, }, 
    character::complete::{one_of, multispace0}, 
    multi::{many0, separated_list0},
    IResult, 
};
// cargo doc --open --package nom

#[derive(Debug, PartialEq, Clone)]
pub enum Jval {
    Null,
    Bool(bool),
    // Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Jval>),
    Obj(HashMap<String, Jval>),
}

fn p_value_unspaced<'a, E>(i: &'a str) -> IResult<&'a str, Jval, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>
                           + FromExternalError<&'a str, escape8259::UnescapeError>
{
    delimited(multispace0, p_value, multispace0)(i)
}

fn p_value<'a, E>(i: &'a str) -> IResult<&'a str, Jval, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>
                           + FromExternalError<&'a str, escape8259::UnescapeError>
{
    alt((
        p_null,
        p_bool,
        p_float,
        p_str,
        p_list,
        p_obj,
    ))(i)
}

fn p_null<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Jval, E> {
    value(Jval::Null, tag("null"))(i)
}

fn p_bool<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Jval, E> {
    alt((
        value(Jval::Bool(true), tag("true")), 
        value(Jval::Bool(false), tag("false")))
    )(i)
}


// fn p_float<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>>(
//     i: &'a str,
// ) -> IResult<&'a str, f64, E> {
//     map_res(
//         recognize(tuple((
//             opt(char('-')),
//             digit1,
//             char('.'),
//             digit1,
//             opt(tuple((
//                 one_of("eE"),
//                 opt(one_of("-+")),
//                 digit1,
//             ))),
//         ))),
//         |float_str: &'a str| float_str.parse(),
//     )(i)
// }

// fuck recognise_float it's too inclusive
fn p_float<'a, E>(i: &'a str,) -> IResult<&'a str, Jval, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>
{
    let parser = map_res(
        recognize_float,
        |float_str: &'a str| float_str.parse(),
    );
    map(parser, Jval::Float)(i)
}
// tuple((opt(one_of("-+")), digit1))

// fn p_int<'a, E>(i: &'a str) -> IResult<&'a str, Jval, E> 
// where
//     E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>
// {
//     let parser = map_res(
//         recognize(tuple((
//             opt(tag("-")), 
//             digit1
//         ))) , 
//         FromStr::from_str
//     );
//     map(parser, Jval::Int)(i)
// }

fn p_char<'a, E: ParseError<&'a str>>(i:&'a str) -> IResult<&'a str, &'a str, E> {
    take_while1(|c| ' ' <= c && '\"' != c && '\\' != c)(i)
}

// fn p_esc_char<'a, E: ParseError<&'a str>>(i:&'a str) -> IResult<&'a str, &'a str, E> {
//     recognize(pair(
//         tag("\\"), 
//         one_of("\"\\/bfnrtu")
//     ))(i)
// }

fn p_escaped<'a, E: ParseError<&'a str>>(i:&'a str) -> IResult<&'a str, &'a str, E> {
    escaped(
        p_char,
        '\\',
        one_of(r#""\/bfnrtu"#)
    )(i)
}

fn p_str_body<'a, E: ParseError<&'a str> >(i:&'a str) -> IResult<&'a str, &'a str, E> {
   recognize(many0(p_escaped))(i)
}

// not gonna bother writing unescaper
// but if I were, nom::bytes::complete::escaped_transform
// is a good place to start
fn p_str_lit<'a, E>(i:&'a str) -> IResult<&'a str, String, E> 
where
    E: ParseError<&'a str> + FromExternalError<&'a str, escape8259::UnescapeError>
{
    map_res(delimited(
        tag("\""),
        p_str_body,
        tag("\"")
    ), unescape)(i)
}

fn p_str<'a, E>(i:&'a str) -> IResult<&'a str, Jval, E> 
where
    E: ParseError<&'a str> + FromExternalError<&'a str, escape8259::UnescapeError>
{
    map(p_str_lit, Jval::Str)(i)
}


fn p_list<'a, E>(i: &'a str) -> IResult<&'a str, Jval, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>
                           + FromExternalError<&'a str, escape8259::UnescapeError>
{
    let parser = delimited(
        terminated(tag("["), multispace0),
        separated_list0(tag(","), p_value_unspaced),
        tag("]")
    );

    map(parser, Jval::List)(i)
}

fn p_obj_entry<'a, E>(i: &'a str) -> IResult<&'a str, (String, Jval), E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>
                           + FromExternalError<&'a str, escape8259::UnescapeError>
{
    separated_pair(delimited(multispace0, p_str_lit, multispace0), tag(":"), p_value_unspaced)(i)
}

fn p_obj<'a, E>(i: &'a str) -> IResult<&'a str, Jval, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>
                           + FromExternalError<&'a str, escape8259::UnescapeError>
{
    let parser = delimited(
        terminated(tag("{"), multispace0),
        separated_list0(tag(","), p_obj_entry),
        tag("}")
    );
    map(parser, |v| Jval::Obj(HashMap::from_iter(v.into_iter())))(i)
}

fn main() {
    let json_inp = r#"{"Person" : {
        "name": "Adam",
        "age": 19,
        "fav_lang": "rust!"
    }}"#;
    if let Ok(json) = p_value::<()>(json_inp){
        dbg!(json);
    }
    else {
        println!("An error occured while parsing json");
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_null() {
        assert_eq!(p_null::<()>("null"),    Ok(("", Jval::Null)));
        assert_eq!(p_null::<()>("nul"),     Err(nom::Err::Error(())));
    }
    
    #[test]
    fn test_bool() {
        assert_eq!(p_bool::<()>("true"),    Ok(("", Jval::Bool(true))));
        assert_eq!(p_bool::<()>("falsedd"), Ok(("dd", Jval::Bool(false))));
        assert_eq!(p_null::<()>("fatrue"),  Err(nom::Err::Error(())));
    }
    
    // #[test]
    // fn test_int() {
    //     assert_eq!(p_int::<()>("69"),       Ok(("", Jval::Int(69))));
    //     assert_eq!(p_int::<()>("-420e10"),     Ok(("e10", Jval::Int(-420))));
    //     assert_eq!(p_int::<()>("01"), Ok(("", Jval::Int(1))));
    //     assert_eq!(p_int::<()>("-420000000000000000000e10"),     Err(nom::Err::Error(())));
    //     assert_eq!(p_int::<()>("fatrue"),  Err(nom::Err::Error(())));
    // }

    #[test]
    fn test_float() {
        assert_eq!(p_float::<()>("69"),         Ok(("", Jval::Float(69.0))));
        assert_eq!(p_float::<()>("-420e-3"),    Ok(("", Jval::Float(-0.42))));
        assert_eq!(p_float::<()>("01.5"),       Ok(("", Jval::Float(1.5))));
        assert_eq!(p_float::<()>("fatrue"),     Err(nom::Err::Error(())));
    }

    #[test]
    fn test_string() {
        // Plain Unicode strings with no escaping
        assert_eq!(p_str::<()>(r#""""#),        Ok(("", Jval::Str("".into()))));
        assert_eq!(p_str::<()>(r#""Hello""#),   Ok(("", Jval::Str("Hello".into()))));
        assert_eq!(p_str::<()>(r#""の""#),       Ok(("", Jval::Str("の".into()))));
        assert_eq!(p_str::<()>(r#""𝄞""#),       Ok(("", Jval::Str("𝄞".into()))));

        // valid 2-character escapes
        assert_eq!(p_str::<()>(r#""  \\  ""#), Ok(("", Jval::Str("  \\  ".into()))));
        assert_eq!(p_str::<()>(r#""  \"  ""#), Ok(("", Jval::Str("  \"  ".into()))));

        // valid 6-character escapes
        assert_eq!(p_str::<()>(r#""\u0000""#),       Ok(("", Jval::Str("\x00".into()))));
        assert_eq!(p_str::<()>(r#""\u00DF""#),       Ok(("", Jval::Str("ß".into()))));
        assert_eq!(p_str::<()>(r#""\uD834\uDD1E""#), Ok(("", Jval::Str("𝄞".into()))));

        // Invalid because surrogate characters must come in pairs
        assert!(p_str::<()>(r#""\ud800""#).is_err());
        // Unknown 2-character escape
        assert!(p_str::<()>(r#""\x""#).is_err());
        // Not enough hex digits
        assert!(p_str::<()>(r#""\u""#).is_err());
        assert!(p_str::<()>(r#""\u001""#).is_err());
        // Naked control character
        assert!(p_str::<()>(r#""\x0a""#).is_err());
        // Not a JSON string because it's not wrapped in quotes
        assert!(p_str::<()>("abc").is_err());
    }

    #[test]
    fn test_array() {
        assert_eq!(p_list::<()>("[ ]"), Ok(("", Jval::List(vec![]))));
        assert_eq!(p_list::<()>("[ 1 ]"), Ok(("", Jval::List(vec![Jval::Float(1.0)]))));
    
        let expected = Jval::List(vec![Jval::Float(1.0), Jval::Str(" x".into())]);
        assert_eq!(p_list::<()>(r#"[ 1 , " x" ]"#), Ok(("", expected)));
    }

    #[test]
    fn test_object() {
        assert_eq!(p_obj::<()>("{ }"), Ok(("", Jval::Obj(HashMap::new()))));
        let dict = HashMap::from([
            ("1".into(), Jval::Str("2".into())),
            ("2".into(), Jval::List(vec![
                Jval::Float(-1.0),
                Jval::Str("b".into())
            ]))
        ]);

        let expected = Jval::Obj(dict);
        assert_eq!(p_obj::<()>(r#"{ "1" : "2", "2": [-1.0, "b"] }"#), Ok(("", expected)));
    }
}