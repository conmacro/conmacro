extern crate clap;
extern crate unindent;
#[macro_use]
extern crate nom;

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol<'a>(pub &'a str);

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Symbol(Symbol<'a>),
    String(&'a str),
    MultilineString(&'a str),
    Assertion(Box<Assertion<'a>>),
    Multi(Vec<Value<'a>>),
    Ref(Symbol<'a>),
    Assignment(Symbol<'a>, Box<Value<'a>>, Box<Value<'a>>),
    Muted(Box<Value<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssertMode<'a> {
    Assert(Option<Symbol<'a>>), Duplicate,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assertion<'a> {
    pub(crate) mode: AssertMode<'a>,
    pub(crate) class: Symbol<'a>,
    pub(crate) attrs: Vec<(Symbol<'a>, Value<'a>)>,
}

pub mod parser;

use std::io::{self, Write};

pub trait ClipsWriter<W: Write> {
    fn write(&self, w: &mut W) -> io::Result<()>;
}

impl<'a, W: Write> ClipsWriter<W> for Symbol<'a> {
    fn write(&self, w: &mut W) -> io::Result<()> {
        w.write(self.0.as_bytes())?;
        Ok(())
    }
}

impl<'a, W: Write> ClipsWriter<W> for Value<'a> {
    fn write(&self, w: &mut W) -> io::Result<()> {
        match self {
            &Value::Symbol(ref s) => s.write(w),
            &Value::String(s) => {
                w.write(b"\"")?;
                for c in s.chars() {
                    if c == '"' {
                        w.write(b"\\\"")?;
                    } else {
                        w.write(&[c as u8])?;
                    }
                }
                w.write(b"\"")?;
                Ok(())
            },
            &Value::MultilineString(s) => {
                let s = unindent::unindent(s);
                Value::String(&s).write(w)
            },
            &Value::Assertion(ref a) => a.write(w),
            &Value::Multi(ref vals) => {
                for val in vals {
                    w.write(b" ")?;
                    val.write(w)?;
                    w.write(b"\n")?;
                }
                Ok(())
            },
            &Value::Ref(ref s) => {
                w.write(b"?")?;
                s.write(w)
            },
            &Value::Assignment(ref name, ref value, ref scope) => {
                w.write(b"(bind ?")?;
                name.write(w)?;
                w.write(b" ")?;
                value.write(w)?;
                w.write(b") ")?;
                scope.write(w)?;
                Ok(())
            },
            &Value::Muted(ref value) => {
                w.write(b"(progn ")?;
                value.write(w)?;
                w.write(b" nil)")?;
                Ok(())
            },
        }
    }
}

impl<'a, W: Write> ClipsWriter<W> for Assertion<'a> {
    fn write(&self, w: &mut W) -> io::Result<()> {
        if let AssertMode::Assert(ref opt_sym) = self.mode {
            if let &Some(ref sym) = opt_sym {
                w.write(b"(bind ?")?;
                sym.write(w)?;
                w.write(b" ")?;
            }
            w.write(b"(bind ?UCON___ (assert (")?;
        } else {
            w.write(b"(bind ?UCON__ (duplicate ?")?;
        }
        self.class.write(w)?;
        for &(ref k, ref v) in self.attrs.iter() {
            w.write(b" (")?;
            k.write(w)?;
            w.write(b" ")?;
            v.write(w)?;
            w.write(b")")?;
        }
        if let AssertMode::Assert(ref opt_sym) = self.mode {
            w.write(b")")?;
            if let &Some(_) = opt_sym {
                w.write(b")")?;
            }
        }
        w.write(b"))")?;
        Ok(())
    }
}

use clap::{Arg, App};
use std::path::Path;
use std::fs::File;
use std::io::Read;

fn main() {
    let matches = App::new("uconsyn")
                          .version("1.0")
                          .about("Unobtrusive construct syntax for conmacro")
                          .arg(Arg::with_name("INPUT")
                               .help("Sets the input file to use (example: file.ucon)")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("OUTPUT")
                               .help("Sets the output file to use, defaults to INPUT with a .con")
                               .index(2))
                          .get_matches();
    let input = Path::new(matches.value_of("INPUT").unwrap());
    let suggested_output = input.with_extension("con");
    let output = Path::new(matches.value_of("OUTPUT").unwrap_or(suggested_output.to_str().unwrap()));

    let mut input_file = File::open(input).expect("can't open input file");
    let mut input_data = vec![];
    input_file.read_to_end(&mut input_data).expect("can't read input file");

    let (rest, assertions) = parser::document(&input_data).unwrap();

    if rest.len() > 0 {
        use std::str;
        println!("Superfluous input: {}", str::from_utf8(rest).unwrap());
    }

    let mut output_file = File::create(output).expect("can't open output file");

    output_file.write(b";; Generated by uconsyn\n(reset)\n").expect("error writing output file");

    for assertion in assertions {
        assertion.write(&mut output_file).expect("error writing output file");
        output_file.write(b"\n").expect("error writing output file");
    }

    output_file.write(b"\n").expect("error writing output file");
}
