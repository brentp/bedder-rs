use crate::position::{Field, FieldError, Positioned, Value};
use crate::string::String;
pub use noodles::bed;
use std::io::{self, BufRead};
use std::result;

impl crate::position::Positioned for bed::record::Record<3> {
    #[inline]
    fn chrom(&self) -> &str {
        self.reference_sequence_name()
    }

    #[inline]
    fn start(&self) -> u64 {
        // noodles position is 1-based.
        self.start_position().get() as u64 - 1
    }

    #[inline]
    fn stop(&self) -> u64 {
        self.end_position().get() as u64
    }

    fn value(&self, v: crate::position::Field) -> result::Result<Value, FieldError> {
        match v {
            Field::String(s) => Ok(Value::Strings(vec![s])),
            Field::Int(i) => match i {
                0 => Ok(Value::Strings(vec![String::from(self.chrom())])),
                1 => Ok(Value::Ints(vec![self.start() as i64])),
                2 => Ok(Value::Ints(vec![self.stop() as i64])),
                _ => Err(FieldError::InvalidFieldIndex(i)),
            },
        }
    }
}

struct Last {
    chrom: String,
    start: u64,
    stop: u64,
}

pub struct BedderBed<R>
where
    R: BufRead,
{
    reader: bed::Reader<R>,
    buf: std::string::String,
    last_record: Option<Last>,
    line_number: u64,
}

impl<R> BedderBed<R>
where
    R: BufRead,
{
    pub fn new(r: R) -> BedderBed<R> {
        BedderBed {
            reader: bed::Reader::new(r),
            buf: std::string::String::new(),
            last_record: None,
            line_number: 0,
        }
    }
}

impl<R> crate::position::PositionedIterator for BedderBed<R>
where
    R: BufRead,
{
    // type Item = bed::record::Record<3>;
    type Item = Box<dyn Positioned>;

    fn next_position(
        &mut self,
        _q: Option<&dyn crate::position::Positioned>,
    ) -> Option<std::result::Result<Self::Item, std::io::Error>> {
        self.buf.clear();
        loop {
            self.line_number += 1;
            return match self.reader.read_line(&mut self.buf) {
                Ok(0) => None,
                Ok(_) => {
                    if self.buf.starts_with('#') || self.buf.is_empty() {
                        continue;
                    }
                    let record: bed::record::Record<3> = match self.buf.parse() {
                        Err(e) => {
                            let msg = format!(
                                "line#{:?}:{:?} error: {:?}",
                                self.line_number, &self.buf, e
                            );
                            return Some(Err(io::Error::new(io::ErrorKind::InvalidData, msg)));
                        }
                        Ok(r) => r,
                    };

                    match &mut self.last_record {
                        None => {
                            self.last_record = Some(Last {
                                chrom: String::from(record.chrom()),
                                start: record.start(),
                                stop: record.stop(),
                            })
                        }
                        Some(r) => {
                            if r.chrom != record.chrom() {
                                r.chrom = String::from(record.chrom())
                            }
                            r.start = record.start();
                            r.stop = record.stop();
                        }
                    }

                    Some(Ok(Box::new(record)))
                }
                Err(e) => Some(Err(e)),
            };
        }
    }

    fn name(&self) -> String {
        String::from(format!("bed:{}", self.line_number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intersection::IntersectionIterator;
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn test_bed_read() {
        // write a test for bed from a string using BufRead
        let ar = BedderBed::new(Cursor::new("chr1\t20\t30\nchr1\t21\t33"));
        let br = BedderBed::new(Cursor::new("chr1\t21\t30\nchr1\t22\t33"));

        let chrom_order = HashMap::from([(String::from("chr1"), 0), (String::from("chr2"), 1)]);

        let it = IntersectionIterator::new(Box::new(ar), vec![Box::new(br)], &chrom_order)
            .expect("error creating iterator");

        let mut n = 0;
        it.for_each(|int| {
            let int = int.expect("error getting intersection");
            //dbg!(&int.overlapping);
            assert!(int.overlapping.len() == 2);
            n += 1;
        });
        assert!(n == 2);
    }
}
