use crate::operator::{Operator, Record};
use std::cmp::Ordering;

/// An operator that emits its input in sorted order.
/// "cmp" is a compare function that defines the sort order.
pub struct Sort {
    child: Box<dyn Operator>,
    cmp: fn(&Record, &Record) -> Ordering,
    records: Vec<Record>,
    index: i32,
}

impl Sort {
    pub fn new(child: Box<dyn Operator>, cmp: fn(&Record, &Record) -> Ordering) -> Self {
        Self {child, cmp, records: Vec::new(), index: -1}
    }
}

impl Operator for Sort {
    /// Open the child operator and sort its output into an intermediate data structure.
    fn open(&mut self) {
        self.child.open();
        self.index = 0;

        let mut iter = self.child.next();
        while iter != None {
            self.records.push(iter.unwrap());

            iter = self.child.next();
        }

        self.records.sort_by(self.cmp);
    }

    /// Return the child output in sorted order, or None when done.
    fn next(&mut self) -> Option<Record> {
        if self.index == -1 {
            panic!()
        }

        if self.index as usize == self.records.len() {
            return None
        }

        self.index += 1;
        Some(self.records.get(self.index as usize - 1).unwrap().clone())
    }

    fn close(&mut self) {
        self.child.close();
        self.records.clear();
    }
}
