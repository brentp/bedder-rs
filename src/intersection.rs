use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::position::{Position, Positioned, PositionedIterator};

pub struct IntersectionIterator<'c, I: PositionedIterator<'c>> {
    base_iterator: I,
    other_iterators: Vec<I>,
    min_heap: BinaryHeap<ReverseOrderPosition<'c>>,
    chromosome_order: &'c HashMap<String, usize>,
}

pub struct Intersection<'c> {
    base_interval: Position<'c>,
    overlapping_positions: Vec<Position<'c>>,
}

struct ReverseOrderPosition<'c> {
    position: Position<'c>,
    chromosome_order: &'c HashMap<String, usize>,
    file_index: usize,
}

impl<'c> PartialEq for ReverseOrderPosition<'c> {
    fn eq(&self, other: &Self) -> bool {
        self.position.chromosome == other.position.chromosome
            && self.position.start == other.position.start
            && self.position.stop == other.position.stop
    }
}

impl<'c> Eq for ReverseOrderPosition<'c> {}

impl<'c> PartialOrd for ReverseOrderPosition<'c> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'c> Ord for ReverseOrderPosition<'c> {
    fn cmp(&self, other: &Self) -> Ordering {
        let order = self
            .chromosome_order
            .get(self.position.chromosome)
            .unwrap()
            .cmp(
                self.chromosome_order
                    .get(other.position.chromosome)
                    .unwrap(),
            );

        match order {
            Ordering::Equal => match self.position.start.cmp(&other.position.start).reverse() {
                Ordering::Equal => self.position.stop.cmp(&other.position.stop).reverse(),
                _ => order,
            },
            _ => order,
        }
    }
}

//pub struct IntersectionIterator<'a, 'b, I: PositionedIterator<'b>> {

impl<'c, I: PositionedIterator<'c> + 'c> IntersectionIterator<'c, I> {
    pub fn new(
        mut base_iterator: I,
        mut other_iterators: Vec<I>,
        chromosome_order: &'c HashMap<String, usize>,
    ) -> Self {
        let mut min_heap = BinaryHeap::new();
        if let Some(positioned) = base_iterator.next() {
            min_heap.push(ReverseOrderPosition {
                position: positioned.position(),
                chromosome_order: chromosome_order,
                file_index: 0,
            });
        }

        for (i, iter) in other_iterators.iter_mut().enumerate() {
            if let Some(positioned) = iter.next() {
                min_heap.push(ReverseOrderPosition {
                    position: positioned.position(),
                    chromosome_order: chromosome_order,
                    file_index: i + 1, // Adjust the file_index accordingly
                });
            }
        }
        Self {
            base_iterator,
            other_iterators,
            min_heap,
            chromosome_order,
        }
    }

    // fn init_heap(
    //     base_iterator: &mut I,
    //     other_iterators: &mut [I],
    //     chromosome_order: &'c HashMap<String, usize>,
    // ) -> BinaryHeap<ReverseOrderPosition<'c>> {
    //     min_heap
    // }
}

impl<'c, I: PositionedIterator<'c>> Iterator for IntersectionIterator<'c, I> {
    type Item = Intersection<'c>;

    fn next(&mut self) -> Option<Self::Item> {
        let base_interval = self.base_iterator.next()?.position();

        let mut overlapping_positions: Vec<Position> = Vec::new();
        let other_iterators = self.other_iterators.as_mut_slice();
        while let Some(ReverseOrderPosition {
            position,
            file_index,
            ..
        }) = &self.min_heap.peek()
        {
            if position.chromosome == base_interval.chromosome
                && position.start <= base_interval.stop
            {
                let file_index = *file_index;
                let ReverseOrderPosition {
                    position: overlap, ..
                } = self.min_heap.pop().unwrap();
                let f = other_iterators
                    .get_mut(file_index)
                    .expect("expected interval iterator at file index");
                let n = f.next();
                if n.is_some() {
                    self.min_heap.push(ReverseOrderPosition {
                        position: n.unwrap().position(),
                        chromosome_order: self.chromosome_order,
                        file_index: file_index,
                    });
                }

                if overlap.stop >= base_interval.start {
                    overlapping_positions.push(overlap);
                }
            } else {
                break;
            }
        }

        Some(Intersection {
            base_interval,
            overlapping_positions,
        })
    }
}
