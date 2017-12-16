use std::str;
use nom::{IResult, alpha, multispace};
use nom::{InputIter, InputLength, Slice, AsChar, Needed, ErrorKind};
use std::ops::{Range, RangeFrom, RangeTo};

pub fn identifier<T>(input:T) -> IResult<T,T> where
    T: Slice<Range<usize>>+Slice<RangeFrom<usize>>+Slice<RangeTo<usize>>,
    T: InputIter+InputLength, <T as InputIter>::Item: AsChar {
    use self::IResult::{Done, Error, Incomplete};
    let input_length = input.input_len();
    if input_length == 0 {
        return Incomplete(Needed::Unknown);
    }

    for (idx, item) in input.iter_indices() {
        let ch = item.as_char();
        if !(ch.is_alphanum() || ch == '-' || ch == '_' || ch == '.') {
               if idx == 0 {
                   return Error(error_position!(ErrorKind::AlphaNumeric, input))
               } else {
                   return Done(input.slice(idx..), input.slice(0..idx))
               }
           }
    }
    Done(input.slice(input_length..), input)
}

use super::{Symbol, Value, Assertion, AssertMode};

named!(symbol<&[u8], Symbol>, do_parse!(
    peek!(alpha)             >>
    b: identifier            >>
    (Symbol(str::from_utf8(b).unwrap()))
));

named!(standard_string<&[u8], &str>, do_parse!(
    str: delimited!(char!('"'),
         escaped!(is_not!("\"\\"), '\\', one_of!("\"n\\")),
                    char!('"'))   >>
    (str::from_utf8(str).unwrap())
));

named!(multiline_string_value<&[u8], Value>, do_parse!(
    str: delimited!(tag!(r#"""""#),
                    take_until!(r#"""""#),
                    tag!(r#"""""#))   >>
    (Value::MultilineString(str::from_utf8(str).unwrap()))
));

named!(raw_string<&[u8], &str>, do_parse!(
    str: delimited!(tag!("<'"), take_until!("'>"), tag!("'>"))   >>
    (str::from_utf8(str).unwrap())
));


named!(attribute<&[u8], (Symbol, Value)>, do_parse!(
           sym: symbol            >>
                tag!(":")         >>
                opt!(multispace)  >>
           val: value             >>
           ((sym, val))
       ));

named!(is_shortcut<&[u8], (Symbol, Value)>, do_parse!(
         tag!("=>")       >>
         opt!(multispace) >>
    val: value            >>
    ((Symbol("is"), val))
));

named!(assertion<&[u8], Assertion>, do_parse!(
          assign: opt!(do_parse!(tag!("=") >> opt!(multispace) >> name: symbol >> multispace >> (name))) >>
          class:  symbol            >>
                  multispace        >>
          attrs:  ws!(many0!(alt_complete!(attribute | is_shortcut))) >>
          (Assertion { mode: AssertMode::Assert(assign), class, attrs }))
);

named!(duplication<&[u8], Assertion>, do_parse!(
                tag!("+")          >>
                opt!(multispace)   >>
          name: opt!(do_parse!(tag!("@") >> opt!(multispace) >> name: symbol >> (name)))  >>
          attrs: ws!(many0!(alt_complete!(attribute | is_shortcut))) >>
          (Assertion { mode: AssertMode::Duplicate, class: name.unwrap_or(Symbol("UCON___")), attrs }))
);


named!(muted_value<&[u8], Value>, do_parse!(
         tag!("|") >>
         opt!(multispace) >>
        val: value >>
        (Value::Muted(Box::new(val)))
));

named!(assignment_value<&[u8], Value>, do_parse!(
          tag!("=") >>
          opt!(multispace) >>
    name: symbol >>
          multispace >>
    val:  value >>
    body: value >>
    (Value::Assignment(name, Box::new(val), Box::new(body)))
));

named!(ref_value<&[u8], Value>, do_parse!(
          tag!("@") >>
          opt!(multispace) >>
    name: symbol >>
    (Value::Ref(name))
));

named!(symbol_value<&[u8], Value>, do_parse!(tag!("'") >> s: symbol >> (Value::Symbol(s))));
named!(string_value<&[u8], Value>, do_parse!(s: alt!(standard_string | raw_string) >> (Value::String(s))));
named!(assertion_value<&[u8], Value>, do_parse!(a: alt!(assertion | duplication) >> (Value::Assertion(Box::new(a)))));
named!(multi_value<&[u8], Value>, do_parse!(
    values: ws!(delimited!(char!('['), many0!(value), char!(']'))) >>
            (Value::Multi(values))
));

named!(pub value<&[u8], Value>, alt_complete!(multi_value | muted_value | assignment_value | ref_value |
                                              assertion_value | symbol_value | multiline_string_value | string_value));

named!(pub document<&[u8], Vec<Assertion>>, ws!(many0!(alt!(assertion | duplication))));

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn valid_symbol() {
        let (_, result) = symbol(b"abc123.-_").unwrap();
        assert_eq!(Symbol("abc123.-_"), result);
    }

    #[test]
    fn value_symbol() {
        let (_, result) = symbol_value(b"'abc123.-_").unwrap();
        assert_eq!(Value::Symbol(Symbol("abc123.-_")), result);
    }

    #[test]
    fn invalid_symbol_leading_char() {
        assert!(symbol(b"0abc").is_err());
        assert!(symbol(b".abc").is_err());
        assert!(symbol(b"-abc").is_err());
        assert!(symbol(b"_abc").is_err());
    }

    #[test]
    fn valid_string() {
        let (_, result) = standard_string(r#""test""#.as_bytes()).unwrap();
        assert_eq!("test", result);
    }

    #[test]
    fn valid_raw_string() {
        let (_, result) = raw_string(r#"<'t"e"st'>"#.as_bytes()).unwrap();
        assert_eq!("t\"e\"st", result);
    }


    #[test]
    fn valid_newline_string() {
        let (_, result) = standard_string(r#""test
world""#.as_bytes()).unwrap();
        assert_eq!("test\nworld", result);
    }

    #[test]
    fn value_string() {
        let (_, result) = string_value(r#""test""#.as_bytes()).unwrap();
        assert_eq!(Value::String("test"), result);
    }

    #[test]
    fn value_multiline_string() {
        let s = r#""""test
                      this
                   """"#;
        let (_, result) = multiline_string_value(s.as_bytes()).unwrap();
        assert_eq!(Value::MultilineString(&s.replace("\"", "")), result);
    }


    #[test]
    fn valid_multi_value() {
        let (_, result) = multi_value(r#"[]"#.as_bytes()).unwrap();
        assert_eq!(Value::Multi(vec![]), result);
        let (_, result) = multi_value(r#"["test" "value" ]"#.as_bytes()).unwrap();
        assert_eq!(Value::Multi(vec![Value::String("test"), Value::String("value")]), result);
    }

    #[test]
    fn valid_attribute() {
        let (_, result) = attribute(b"test: 'value").unwrap();
        assert_eq!((Symbol("test"), Value::Symbol(Symbol("value"))), result);
    }

    #[test]
    fn valid_is_shortcut() {
        let (_, result) = is_shortcut(b"=> 'value").unwrap();
        assert_eq!((Symbol("is"), Value::Symbol(Symbol("value"))), result);
    }

    #[test]
    fn valid_assertion_with_multi() {
        let (_, result) = assertion(b"construct test: [ 'test ] something: 'good").unwrap();
        assert_eq!(Assertion { mode: AssertMode::Assert(None), class: Symbol("construct"), attrs: vec![(Symbol("test"), Value::Multi(vec![Value::Symbol(Symbol("test"))])),
                                                                       (Symbol("something"), Value::Symbol(Symbol("good")))] }, result);
    }

    #[test]
    fn valid_assertion_with_assignment() {
        let (_, result) = assertion(b"=test construct something: 'good").unwrap();
        assert_eq!(Assertion { mode: AssertMode::Assert(Some(Symbol("test"))), class: Symbol("construct"), attrs: vec![
                                                                       (Symbol("something"), Value::Symbol(Symbol("good")))] }, result);
    }


    #[test]
    fn valid_assertion() {
        let (_, result) = assertion(b"construct test: 'value something: 'good").unwrap();
        assert_eq!(Assertion { mode: AssertMode::Assert(None), class: Symbol("construct"), attrs: vec![(Symbol("test"), Value::Symbol(Symbol("value"))), (Symbol("something"), Value::Symbol(Symbol("good")))] }, result);
    }

    #[test]
    fn valid_assertion_with_is_shortcut() {
        let (_, result) = assertion(b"construct test: 'value => 'good").unwrap();
        assert_eq!(Assertion { mode: AssertMode::Assert(None), class: Symbol("construct"), attrs: vec![(Symbol("test"), Value::Symbol(Symbol("value"))), (Symbol("is"), Value::Symbol(Symbol("good")))] }, result);
    }

    #[test]
    fn valid_duplication() {
        let (_, result) = duplication(b"+ test: 'value something: 'good").unwrap();
        assert_eq!(Assertion { mode: AssertMode::Duplicate, class: Symbol("UCON___"), attrs: vec![(Symbol("test"), Value::Symbol(Symbol("value"))), (Symbol("something"), Value::Symbol(Symbol("good")))] }, result);
    }

    #[test]
    fn valid_ref_duplication() {
         let (_, result) = duplication(b"+ @test test: 'value something: 'good").unwrap();
        assert_eq!(Assertion { mode: AssertMode::Duplicate, class: Symbol("test"), attrs: vec![(Symbol("test"), Value::Symbol(Symbol("value"))), (Symbol("something"), Value::Symbol(Symbol("good")))] }, result);
    }

    #[test]
    fn muted_value() {
        let (_, result) = value(b"|'a").unwrap();
        assert_eq!(Value::Muted(Box::new(Value::Symbol(Symbol("a")))), result);
    }

    #[test]
    fn assignment_value() {
        let (_, result) = value(b"=a 'a [ @a ]").unwrap();
        assert_eq!(Value::Assignment(Symbol("a"), Box::new(Value::Symbol(Symbol("a"))), Box::new(Value::Multi(vec![Value::Ref(Symbol("a"))]))), result);
    }

    #[test]
    fn ref_value() {
        let (_, result) = value(b"@a").unwrap();
        assert_eq!(Value::Ref(Symbol("a")), result);
    }

}
