use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
extern crate resort;
use crate::resort::bedder_bed::BedderBed;
use crate::resort::intersection::IntersectionIterator;
use crate::resort::position::Positioned;
use crate::resort::string::String;

#[derive(Parser, Debug)]
struct Args {
    a: PathBuf,
    b: PathBuf,

    fai: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let a = BufReader::new(std::fs::File::open(args.a)?);
    let b = BufReader::new(std::fs::File::open(args.b)?);

    let ai = BedderBed::new(a);
    let bi = BedderBed::new(b);

    let fh = BufReader::new(std::fs::File::open(args.fai)?);
    let mut h = HashMap::new();
    fh.lines().enumerate().for_each(|(i, line)| {
        let line = line.expect("error reading line from fai");
        let chrom = line.split("\t").next().expect("error getting line");
        h.insert(String::from(chrom), i);
    });

    let it = IntersectionIterator::new(ai, vec![bi], &h)?;

    for intersection in it {
        let intersection = intersection?;
        println!(
            "{}\t{}\t{}\t{}",
            intersection.base_interval.chrom(),
            intersection.base_interval.start(),
            intersection.base_interval.stop(),
            intersection.overlapping.len()
        );
    }

    Ok(())
}