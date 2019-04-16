#![warn(rust_2018_idioms)]

mod proth;
use proth::{Proth};
mod proth_gmp;

mod parser;

use clap::{Arg, App};

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
    let n : Proth =  parser::proth(number_s);
    println!("{:?}", n);
    let method : &str = matches.value_of("method").expect("What");
    match method {
        "gmp_simple" => proth_gmp::simple(n),
        "gmp_medium" => proth_gmp::medium(n),
        "gmp_low" => proth_gmp::low(n),
        "gmp_barrett" => proth_gmp::barrett(n),
        _ => panic!("You must select a valid method: gmp_simple, gmp_medium, gmp_low, gmp_barrett")
    };
    println!("exit");
}

