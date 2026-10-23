use std::collections::HashMap;

use crate::Value;
use crate::operator::{Operator, Record};

/// An equi-hash-join for equality joins of two tables.
/// The left child is guaranteed to only contain unique join keys, and the hash-table
/// should always be created using this side (left = build side).
pub struct EquiHashJoin {
    left_child: Box<dyn Operator>,
    /// The column index of the join key in the left child output.
    left_join_key: usize,
    right_child: Box<dyn Operator>,
    /// The column index of the join key in the right child output.
    right_join_key: usize,
    // Add additional fields here if needed.
    open_var: bool,
    hash_table: HashMap<Value, Record>,
}

impl EquiHashJoin {
    pub fn new(
        left_child: Box<dyn Operator>,
        right_child: Box<dyn Operator>,
        left_join_key: usize,
        right_join_key: usize,
    ) -> Self {
        Self {left_child, right_child, left_join_key, right_join_key, 
                open_var: false, hash_table: HashMap::new()}
    }
}

impl Operator for EquiHashJoin {
    /// Open the child operators and build the hash table.
    fn open(&mut self) {
        self.open_var = true;
        self.left_child.open();
        self.right_child.open();

        let mut iter = self.left_child.next();
        while iter != None {
            let rec = iter.unwrap();
            self.hash_table.insert(rec.get(self.left_join_key).unwrap().clone(), rec);

            iter = self.left_child.next();
        }
    }

    /// Emit the next joined tuple or None if there are no more join partners.
    /// A joined tuple is a record of the concatenated values of the two joined rows (left, then right values).
    fn next(&mut self) -> Option<Record> {
        if self.open_var == false {
            panic!()
        }

        let mut rec_right_child;
        let mut rec_left_child_op;

        loop {
            let iter = self.right_child.next();

            if iter == None {
                return None;
            }

            rec_right_child = iter.unwrap();
            let key = rec_right_child.get(self.right_join_key).unwrap().clone();
            rec_left_child_op = self.hash_table.get(&key);

            if rec_left_child_op != None {
                break;
            }
        }

        let mut rec_left_child = rec_left_child_op.unwrap().clone();

        rec_left_child.record.append(&mut rec_right_child.record);

        Some(rec_left_child)  
    }

    fn close(&mut self) {
        self.left_child.close();
        self.right_child.close();
        self.hash_table.clear();
    }
}
