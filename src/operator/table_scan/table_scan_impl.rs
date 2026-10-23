use std::{cell::RefCell, rc::Rc};

use crate::operator::{BufferPool, Operator, Record, Table, table_scan};

/// A TableScan that returns all records of all pages of the respective table.
/// Has access to a BufferPool to get data pages.
pub struct TableScan {
    table: Rc<dyn Table>,
    buffer: Rc<RefCell<dyn BufferPool>>,
    index_page: usize,
    index_row: usize,
    open_var: bool,
}

impl TableScan {
    pub fn new(table: Rc<dyn Table>, buffer: Rc<RefCell<dyn BufferPool>>) -> Self {
        Self {table, buffer, index_page: 0, index_row: 0, open_var: false}
    }
}

impl Operator for TableScan {
    fn open(&mut self) {
        self.open_var = true;
    }

    /// Emit the next record in this table, taken from the tables' pages.
    /// When all records have been emitted, emit None.
    fn next(&mut self) -> Option<Record> {
        if self.open_var == false {
            panic!()
        }

        let page_id = self.table.page_list()
                                        .get(self.index_page)
                                        .unwrap();
        
        let page = self.buffer.borrow_mut().pin(*page_id);
        
        if page.get_records().len() > self.index_row {
            let return_value = page.get_records().get(self.index_row).unwrap().clone();
            self.index_row += 1;
            self.buffer.borrow_mut().unpin(page);
            return Some(return_value)
        }

        self.buffer.borrow_mut().unpin(page);
        self.index_row = 1;
        self.index_page += 1;

        if self.table.page_list().len() == self.index_page {
            return None
        }
        
        let page_id = self.table.page_list()
                .get(self.index_page)
                .unwrap();
        let page = self.buffer.borrow_mut().pin(*page_id);
        
        let return_value = page.get_records().get(0).unwrap().clone();
        self.buffer.borrow_mut().unpin(page);

        Some(return_value)
    }

    fn close(&mut self) {
        self.open_var = false;
    }
}