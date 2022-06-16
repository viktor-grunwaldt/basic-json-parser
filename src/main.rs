use escape8259::unescape;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::{map_res, value, map, recognize},
    error::{FromExternalError, ParseError},
    number::complete::recognize_float,
    IResult, sequence::{pair, delimited}, character::complete::one_of, multi::many0,
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
    Obj,
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

fn is_normal_char(c:char) -> bool {
    ' ' <= c && '\"' != c && '\\' != c
}

fn p_char<'a, E: ParseError<&'a str>>(i:&'a str) -> IResult<&'a str, &'a str, E> {
    take_while(is_normal_char)(i)
}

fn p_esc_char<'a, E: ParseError<&'a str>>(i:&'a str) -> IResult<&'a str, &'a str, E> {
    recognize(pair(
        tag("\\"), 
        one_of("\"\\/bfnrtu")
    ))(i)
}

fn p_str_body<'a, E: ParseError<&'a str>>(i:&'a str) -> IResult<&'a str, &'a str, E> {
   recognize(many0(alt((p_char, p_esc_char))))(i)
}

// not gonna bother writing unescaper
fn p_str<'a, E>(i:&'a str) -> IResult<&'a str, Jval, E> 
where
    E: ParseError<&'a str> + FromExternalError<&'a str, escape8259::UnescapeError>
{
    let parser = map_res(delimited(
        tag("\""),
        p_str_body,
        tag("\"")
    ), unescape);
    
    map(parser, Jval::Str)(i)
}

fn main() {
    println!("Hello, world!");
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
        assert_eq!(p_float::<()>("69"),       Ok(("", Jval::Float(69.0))));
        assert_eq!(p_float::<()>("-420e-3"),     Ok(("", Jval::Float(-0.42))));
        assert_eq!(p_float::<()>("01.5"), Ok(("", Jval::Float(1.5))));
        assert_eq!(p_float::<()>("fatrue"),  Err(nom::Err::Error(())));
    }
}