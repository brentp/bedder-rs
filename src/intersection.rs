use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::position::{Positioned, PositionedIterator};

pub struct IntersectionIterator<'c, I: PositionedIterator, P: Positioned> {
    base_iterator: I,
    other_iterators: Vec<I>,
    min_heap: BinaryHeap<ReverseOrderPosition<'c, P>>,
    chromosome_order: &'c HashMap<String, usize>,
}

pub struct Intersection<P: Positioned> {
    base_interval: P,
    overlapping_positions: Vec<P>,
}

struct ReverseOrderPosition<'c, P: Positioned> {
    position: P,
    chromosome_order: &'c HashMap<String, usize>,
    file_index: usize,
}

impl<'c, P: Positioned> PartialEq for ReverseOrderPosition<'c, P> {
    fn eq(&self, other: &Self) -> bool {
        self.position.chromosome() == other.position.chromosome()
            && self.position.start() == other.position.start()
            && self.position.stop() == other.position.stop()
    }
}

impl<'c, P: Positioned> Eq for ReverseOrderPosition<'c, P> {}

impl<'c, P: Positioned> PartialOrd for ReverseOrderPosition<'c, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'c, P: Positioned> Ord for ReverseOrderPosition<'c, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        let order = self
            .chromosome_order
            .get(self.position.chromosome())
            .unwrap()
            .cmp(
                self.chromosome_order
                    .get(other.position.chromosome())
                    .unwrap(),
            );

        match order {
            Ordering::Equal => match self.position.start().cmp(&other.position.start()).reverse() {
                Ordering::Equal => self.position.stop().cmp(&other.position.stop()).reverse(),
                _ => order,
            },
            _ => order,
        }
    }
}

//pub struct IntersectionIterator<'a, 'b, I: PositionedIterator<'b>> {

impl<'c, I: PositionedIterator<Item = P>, P: Positioned> IntersectionIterator<'c, I, P> {
    pub fn new(
        mut base_iterator: I,
        mut other_iterators: Vec<I>,
        chromosome_order: &'c HashMap<String, usize>,
    ) -> Self {
        let mut min_heap = BinaryHeap::new();
        if let Some(positioned) = base_iterator.next() {
            min_heap.push(ReverseOrderPosition {
                position: positioned,
                chromosome_order: chromosome_order,
                file_index: 0,
            });
        }

        for (i, iter) in other_iterators.iter_mut().enumerate() {
            if let Some(positioned) = iter.next() {
                min_heap.push(ReverseOrderPosition {
                    position: positioned,
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

impl<'c, I: PositionedIterator<Item = P>, P: Positioned> Iterator
    for IntersectionIterator<'c, I, P>
{
    type Item = Intersection<P>;

    fn next(&mut self) -> Option<Self::Item> {
        let base_interval = self.base_iterator.next()?;

        let mut overlapping_positions = Vec::new();
        let other_iterators = self.other_iterators.as_mut_slice();
        while let Some(ReverseOrderPosition {
            position,
            file_index,
            ..
        }) = &self.min_heap.peek()
        {
            if position.chromosome() == base_interval.chromosome()
                && position.start() <= base_interval.stop()
            {
                let file_index = *file_index;
                let ReverseOrderPosition {
                    position: overlap, ..
                } = self.min_heap.pop().unwrap();
                let f = other_iterators
                    .get_mut(file_index)
                    .expect("expected interval iterator at file index");
                if let Some(n) = f.next() {
                    self.min_heap.push(ReverseOrderPosition {
                        position: n,
                        chromosome_order: self.chromosome_order,
                        file_index: file_index,
                    });
                }

                if overlap.stop() >= base_interval.start() {
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
