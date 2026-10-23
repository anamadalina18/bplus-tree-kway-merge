# B+ Tree, k-way Merge Sort and Query Operators

Disk-oriented B+ tree index and the physical operators around it, in Rust.

**Context.** This is my implementation for **Lab 1** of **Scalable Data Management Systems**
(TU Darmstadt, FG Data Management, 2025/26, Erasmus+ year). The course provided the project
skeleton: page layouts, the buffer pool, the operator traits, the test suite and the CI pipeline.
This repository contains **only the files I implemented**, so it does not build standalone.

## What I implemented

**B+ tree (`src/index/btree.rs`)** — a `UniqueBPlusTree` over a `Vec<Page>` acting as the page
buffer:

- `insert` / `insert_helper`: recursive descent with binary search in inner pages; leaves split on
  overflow and propagate the first key of the new leaf upwards; inner pages split with the median
  key promoted; a root split creates a new inner root. Inserting an existing key overwrites its
  `RecordID` in place.
- `lookup`: descends to the leaf and binary-searches it.
- `range_lookup`: descends once, then walks the leaf chain through the `left`/`right` pointers
  until the range end is exceeded.
- `search_page` / `get_leaf_page`: path helper and typed leaf accessor.
- Leaves are linked (`header.left` / `header.right`), which is what makes range scans linear in the
  number of returned records.

**k-way merge sort (`src/kwaymerge/kwmerge.rs`)** —

- `sort(k, input_pages, output_pages)`: repeatedly merges `k` already-sorted page lists.
- `merge(...)`: classic k-way merge by minimum selection over the current slot of each list, with
  first-occurrence tie-breaking for duplicate keys, output-page swapping when a page fills up, and
  a `RecordID` state vector recorded at every page boundary.

**Operators (`src/operator/`)** —

- `TableScan`: iterator over all pages of a table through the buffer pool (`pin`/`unpin` around
  every access) with a strict `open`/`next`/`close` lifecycle.
- `EquiHashJoin`: build phase hashes the left (unique-key) side into a `HashMap<Value, Record>`,
  probe phase appends matching right rows to the left row and re-iterates on non-matches.
- `Sort`: materializes the child's output and sorts it with a caller-supplied comparator.

## Files

| Path | Contents |
|---|---|
| `src/index/btree.rs` | Insert with splits, lookup, range lookup, page navigation |
| `src/kwaymerge/kwmerge.rs` | k-way merge and the multi-pass sort driver |
| `src/operator/table_scan/table_scan_impl.rs` | Table scan operator |
| `src/operator/join/join_impl.rs` | Hash join (build left, probe right) |
| `src/operator/sort/sort_impl.rs` | Sort operator |

## Notes

- In my working copy I temporarily lowered the page capacities (`InnerPage` 508 → 4,
  `LeafPage` 336 → 4) to force splits and deep trees in the tests; that change is not part of this
  repository, so the files here reflect the tree logic, not the test tweak.
- Correctness was checked against the course's test suite.
