mod proth_gmp;
use proth_gmp::{Proth};

extern crate clap; 
use clap::{Arg, App};

#[macro_use]
extern crate nom;
use nom::digit;
use nom::types::CompleteStr;
use std::str::FromStr;
use std::num::ParseIntError;

fn u32_from_cstr(input: CompleteStr) -> Result<u32, ParseIntError> {
    u32::from_str(input.as_ref())
}

named!(uint32<CompleteStr, u32>,
    map_res!(digit, u32_from_cstr)
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
        t: uint32 >>
        times >>
        two >>
        to_the >>
        e: uint32 >>
        plus >>
        one >>
        (Proth { t, e })
    )
);

fn main() {
    let matches = App::new("Hazel's Primality Tester")
        .version("0.1.0")
        .author("Hazel Victoria Campbell")
        .about("Tests Proth numbers for primality")
        .arg(Arg::with_name("number")
            .index(1)
            .required(true)
            .help("A proth number of the format 943*2^3442990+1")
        )
        .arg(Arg::with_name("method")
            .short("m")
            .long("method")
            .value_name("METHOD")
            .help("Select primality testing algorithm")
            .takes_value(true)
            .default_value("gmp2")
        )
        .get_matches();
    assert!(matches.is_present("number"));
    let number_s : &str = matches.value_of("number").expect("What");
    let number_cs = CompleteStr(number_s);
    let number_parsed = parse_proth(number_cs);
    println!("{:?}", number_parsed);
    let n : Proth =  number_parsed.expect("You must provide numbers in the format 943*2^3442990+1").1;
    println!("{:?}", n);
    let method : &str = matches.value_of("method").expect("What");
    match method {
        "gmp_simple" => proth_gmp::simple(n),
        "gmp" => proth_gmp::medium(n),
        "gmp2" => proth_gmp::low(n),
        "gmp_barrett" => proth_gmp::barrett(n),
        _ => panic!("You must select a valid method: gmp_simple, gmp, gmp2, gmp_barrett")
    };
    println!("exit");
}

// tests

#[cfg(test)]
mod tests {
    use crate::{Proth, proth_gmp, proth_gmp_simple};
    
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn test_proth_gmp() {
        let five = Proth { t: 1, e: 2 };
        assert_eq!((proth_gmp(five)).1, -1);
    }
    #[test]
    fn test_proth_gmp_2() {
        let five_26607 = Proth { t: 5, e: 26607 };
        assert_eq!((proth_gmp(five_26607)).1, -1);
    }
    #[test]
    fn test_proth_gmp_3() {
        let five_26606 = Proth { t: 5, e: 26606 };
        let r = proth_gmp(five_26606);
        let r_simple = proth_gmp_simple(five_26606);
        assert_ne!(r.1, -1);
        assert_eq!(r.0, r_simple.0);
        assert_eq!(r.1, r_simple.1);
    }
}
