use crate::index::*;
use crate::{PageID, RecordID};

impl UniqueBPlusTree {
    /// Creates the initial B+-Tree where the root node is a leaf. You should leave this as it is.
    ///
    /// # Arguments
    ///
    /// * `buffer`: The vector to store the pages in. The expectation here is that the vector is
    ///   empty, and this function adds the first page to it.
    ///
    /// returns: UniqueBPlusTree
    pub fn new(buffer: &mut Vec<Page>) -> Self {
        let tree = Self {
            root_page: PageID(0),
        };
        let root_page = Page::default();
        buffer.push(root_page);
        tree
    }

    /// Searches for the entry with the exact corresponding key and returns its RecordID
    /// or None if the key is not found in the tree.
    ///
    /// # Arguments
    ///
    /// * `buffer`: The buffer manager containing the b-tree's pages.
    /// * `key`: The key to search for.
    ///
    /// returns: `Option<RecordID>` Some(RecordID) if found and None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer: Vec<Page> = Vec::new();
    /// let mut tree = UniqueBPlusTree::new(&mut buffer);
    ///
    /// tree.insert(&mut buffer, 69, RecordID::new(69u32.into(), 0.into()));
    /// let result = tree.lookup(&buffer, 69); // Some(RecordID(69, 0))
    /// ```
    pub fn lookup(&self, buffer: &Vec<Page>, key: i32) -> Option<RecordID> {
        let page_id = self.search_page(buffer, key);
        let Page::LEAF(page) = &buffer[page_id.0 as usize] else {panic!()};

        let var = page.records[0..page.length].binary_search_by_key(&key, |rec| rec.key);

        match var {
            Ok(i) => {
                Some(page.records[i].value)
            },
            Err(_) => {
                None
            },
        }
    }

    /// Searches for all keys in the specified min/max range (both inclusive) and returns a vector
    /// of all corresponding RecordIDs.
    ///
    /// # Arguments
    ///
    /// * `buffer`: The buffer manager containing the b-tree's pages.
    /// * `range`: The range to search for.
    ///
    /// returns: Vec<RecordID, Global>
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer: Vec<Page> = Vec::new();
    /// let mut tree = UniqueBPlusTree::new(&mut buffer);
    ///
    /// tree.insert(&mut buffer, 69, RecordID::new(69u32.into(), 0.into()));
    /// let result = tree.range_lookup(&buffer, (0, 100)); // Vec([RecordID(69, 0)])
    /// ```
    pub fn range_lookup(&self, buffer: &Vec<Page>, range: (i32, i32)) -> Vec<RecordID> {
        let mut page_id = self.search_page(buffer, range.0);
        let mut page = Self::get_leaf_page(page_id, buffer).unwrap();

        let var = page.records[0..page.length].binary_search_by_key(&range.0, |rec| rec.key);
        let mut vec:Vec<RecordID> = Vec::new();

        let mut i= var.unwrap_or_else(|idx| idx);

        if i == page.length {
            i = 0;

            match page.header.right {
                Some(id) => {
                    page_id = id;
                    page = Self::get_leaf_page(page_id, buffer).unwrap();
                },
                None => {
                    return vec
                }
            }
        }

        while page.records[i].key <= range.1 {
            vec.push(page.records[i].value);
            i += 1;

            if i == page.length {
                i = 0;

                match page.header.right {
                    Some(id) => {
                        page_id = id;
                        page = Self::get_leaf_page(page_id, buffer).unwrap();
                    },
                    None => break
                }
            }
        }

        vec
    }

    /// Insert a tuple with RecordID rid and key key into the BPlusTree. Overwrite the existing
    /// RecordID if the key already exists.
    ///
    /// # Arguments
    ///
    /// * `buffer`: The buffer manager containing the b-tree's pages.
    /// * `key`: The key to insert/update.
    /// * `rid`: The RecordID to insert.
    ///
    /// returns: ()
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer: Vec<Page> = Vec::new();
    /// let mut tree = UniqueBPlusTree::new(&mut buffer);
    ///
    /// tree.insert(&mut buffer, 69, RecordID::new(69u32.into(), 0.into()));
    /// ```
    pub fn insert(&mut self, buffer: &mut Vec<Page>, key: i32, rid: RecordID) {
        let ret = self.insert_helper(buffer, key, rid, self.root_page);

        match ret {
            None => {},
            Some(btr) => {
                let mut new_root: InnerPage = InnerPage::default();
                new_root.length = 2;
                new_root.records[0].value = self.root_page;
                new_root.records[1] = btr;
                new_root.id = PageID(buffer.len() as u32);

                self.root_page = new_root.id;
                buffer.push(Page::INNER(new_root));
            }
        }
    }

    pub fn insert_helper(&mut self, buffer: &mut Vec<Page>, key: i32, rid: RecordID, page_id: PageID) -> Option<BTreeRecord<PageID>> {
        let buffer_len = buffer.len();
        let page = &mut buffer[page_id.0 as usize];

        match page {
            Page::INNER(inner) => {
                let var = inner.records[1..inner.length].binary_search_by_key(&key, |rec| rec.key);

                let i = match var {
                    Ok(i) => i + 1,
                    Err(i) => i,
                };

                let id = inner.records[i].value;
                let ret = self.insert_helper(buffer, key, rid, id);
                let buffer_len = buffer.len();
                let page = &mut buffer[page_id.0 as usize];
                let Page::INNER(inner) = page else { panic!() };

                let b_tree_rec = match ret {
                    None => return None,
                    Some(b_tree_rec) => b_tree_rec
                };

                if inner.length < InnerPage::CAPACITY {
                    let pos = inner
                        .records[1..inner.length]
                        .binary_search_by_key(&b_tree_rec.key, |rec| rec.key)
                        .unwrap_or_else(|idx| idx + 1);

                    inner.insert(b_tree_rec, pos).expect("inner insert error");

                    None
                } else {
                    let mid = inner.length / 2;
                    let key_mid = inner.records[mid].key;

                    let mut new_inner_node: InnerPage = InnerPage::default();
                    new_inner_node.id = PageID(buffer_len as u32);

                    for i in mid..inner.length {
                        new_inner_node.records[i - mid] = inner.records[i];
                    }

                    new_inner_node.length = mid;
                    inner.length = mid;

                    if key < new_inner_node.records[1].key {
                        let pos = inner
                            .records[1..inner.length]
                            .binary_search_by_key(&key, |rec| rec.key)
                            .unwrap_or_else(|idx| idx + 1);

                        inner.insert(b_tree_rec, pos).expect("insert inner error");
                    } else {
                        let pos = new_inner_node
                            .records[1..new_inner_node.length]
                            .binary_search_by_key(&key, |rec| rec.key)
                            .unwrap_or_else(|idx| idx + 1);

                        new_inner_node.insert(b_tree_rec, pos).expect("insert inner error");
                    }

                    let btr = BTreeRecord {
                        key: key_mid,
                        value: new_inner_node.id
                    };

                    buffer.push(Page::INNER(new_inner_node));

                    Some(btr)
                }
            },
            Page::LEAF(leaf) => {
                let var = leaf.records[0..leaf.length].binary_search_by_key(&key, |rec| rec.key);

                match var {
                    Ok(i) => {
                        leaf.records[i].value = rid;
                        None
                    },
                    Err(i) => {
                        if leaf.length < LeafPage::CAPACITY {
                            let b_tree_rec = BTreeRecord {
                                key,
                                value: rid,
                            };
                            leaf.insert(b_tree_rec, i).expect("insert leaf error");
                            None
                        } else {
                            let mut new_leaf_node: LeafPage = LeafPage::default();

                            for idx in  leaf.length / 2 .. leaf.length {
                                new_leaf_node.records[new_leaf_node.length] =
                                    leaf.records[idx];

                                new_leaf_node.length += 1;
                            }

                            new_leaf_node.id = PageID(buffer_len as u32);
                            new_leaf_node.header.left = Some(leaf.id);
                            new_leaf_node.header.right = leaf.header.right;
                            leaf.header.right = Some(new_leaf_node.id);

                            leaf.length = leaf.length / 2;

                            if key < new_leaf_node.records[0].key {
                                let pos = leaf
                                    .records[0..leaf.length]
                                    .binary_search_by_key(&key, |rec| rec.key)
                                    .unwrap_or_else(|idx| idx);

                                let b_tree_rec = BTreeRecord {
                                    key,
                                    value: rid,
                                };
                                leaf.insert(b_tree_rec, pos).expect("insert leaf error");
                            } else {
                                let pos = new_leaf_node
                                    .records[0..new_leaf_node.length]
                                    .binary_search_by_key(&key, |rec| rec.key)
                                    .unwrap_or_else(|idx| idx);

                                let b_tree_rec = BTreeRecord {
                                    key,
                                    value: rid,
                                };
                                new_leaf_node.insert(b_tree_rec, pos).expect("insert leaf error");
                            }

                            let btr = BTreeRecord{
                                key: new_leaf_node.records[0].key,
                                value: new_leaf_node.id
                            };

                            buffer.push(Page::LEAF(new_leaf_node));

                            Some(btr)
                        }
                    },
                }
            },
        }
    }

    pub fn search_page(&self, buffer: &Vec<Page>, key: i32) -> PageID {
        let mut page_id = self.root_page;
    
            loop {
                let page = &buffer[page_id.0 as usize];
    
                match page {
                    Page::INNER(inner) => {
                        let var = inner.records[1..inner.length].binary_search_by_key(&key, |rec| rec.key);

                        match var {
                            Ok(i) => {
                                page_id = inner.records[i + 1].value;
                            },
                            Err(i) => {
                                page_id = inner.records[i].value;
                            },
                        }
                    },
                    Page::LEAF(_) => {
                        return page_id
                    },
                }
            }
    }

    pub fn get_leaf_page(page_id: PageID, buffer: &Vec<Page>) -> Option<&LeafPage> {
        match &buffer[page_id.0 as usize] {
            Page::INNER(_) => None,
            Page::LEAF(leaf_page) => Some(&leaf_page)
        }
    }
}
