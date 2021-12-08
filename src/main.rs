//use std::fs;
//use regex::Regex;
//use std::path::PathBuf;
use std::env;


fn main() {
    const RAW_REGEX: &str = r"^(?P<label>.+)\s*;\s*(?P<passes>\d+)\s*;\s*(?P<duration>\d+([.]\d+)?)\s*;\s*(?P<threads>\d+)(;(?P<extra>.+))?$";
    const TARGET: &str = env!("TARGET");
    //let teststr = "mike-barber_bit-extreme-hybrid;253422;5.0002145767;12;algorithm=base,faithful=yes,bits=1";
    //let testregex = Regex::new(rawregex).unwrap();
    //println!("{:?}", testregex.is_match(teststr))
    
    plummerprimes::run_benchmarks(RAW_REGEX, TARGET);
}
