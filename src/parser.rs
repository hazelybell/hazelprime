use crate::proth::Proth;
use nom::{*};
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

pub fn proth(number_s: &str) -> Proth {
    let number_cs = CompleteStr(number_s);
    let number_parsed = parse_proth(number_cs);
    println!("{:?}", number_parsed);
    let n : Proth =  number_parsed.expect("You must provide numbers in the format 943*2^3442990+1").1;
    return n;
}
