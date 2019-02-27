use rug::Integer;
use rug::integer::ParseIntegerError;

extern crate clap; 
use clap::{Arg, App};

#[macro_use]
extern crate nom;
use nom::digit;
use nom::types::CompleteStr;
use std::str::FromStr;

fn integer_from_cstr(input: CompleteStr) -> Result<Integer, ParseIntegerError> {
    Integer::from_str(input.as_ref())
}

#[derive(Debug)]
pub struct Proth {
    pub t: Integer,
    pub e: Integer, 
}

named!(parse_integer<CompleteStr, Integer>,
    map_res!(digit, integer_from_cstr)
);

named!(times<CompleteStr, CompleteStr>,
    alt!(tag!("*") | tag!("x") | tag!("."))
);

named!(two<CompleteStr, CompleteStr>,
    tag!("2")
);

named!(to_the<CompleteStr, CompleteStr>,
    alt!( tag!("^") | tag!("e") )
);

named!(plus<CompleteStr, CompleteStr>,
    tag!("+")
);

named!(one<CompleteStr, CompleteStr>,
    tag!("1")
);


named!(parse_proth<CompleteStr, Proth>,
    do_parse!(
        t: parse_integer >>
        times >>
        two >>
        to_the >>
        e: parse_integer >>
        plus >>
        one >>
        (Proth { t, e })
    )
);
    

fn main() {
    // 7 ^ 5 = 16807
    let n = Integer::from(7);
    let e = Integer::from(5);
    let m = Integer::from(1000);
    let power = match n.pow_mod(&e, &m) {
        Ok(power) => power,
        Err(_) => unreachable!(),
    };
    assert_eq!(power, 807);
    println!("{}", power);
    let matches = App::new("Hazel's Primality Tester")
        .version("0.0")
        .author("Hazel Victoria Campbell")
        .about("Tests Proth numbers for primality")
        .arg(Arg::with_name("number")
            .index(1)
            .required(true)
        ).get_matches();
    assert!(matches.is_present("number"));
    let number_s : &str = matches.value_of("number").expect("What");
    let number_cs = CompleteStr(number_s);
    let number_parsed = parse_proth(number_cs);
    println!("{:?}", number_parsed);
    let n : Proth =  number_parsed.expect("You must provide numbers in the format 943*2^3442990+1").1;
    println!("{:?}", n);
}
