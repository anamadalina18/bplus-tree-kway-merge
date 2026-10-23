# B+ Tree, k-way Merge Sort and Query Operators

Disk-oriented B+ tree index and the physical operators around it, in Rust.

**Context.** This is my implementation for **Lab 1** of **Scalable Data Management Systems**
(TU Darmstadt, FG Data Management, 2025/26, Erasmus+ year). The course provided the project
skeleton: page layouts, the buffer pool, the operator traits, the test suite and the CI pipeline.
This repository contains **only the files I implemented**, so it does not build standalone.

## Architecture

### Page storage

The tree does not own a file: it operates on a `Vec<Page>` supplied by the caller, and a `PageID` is
simply the index into that vector. Each `Page` is a tagged union of `InnerPage` and `LeafPage`, both
generic over record type, header type and capacity. Inner pages hold `(key, child PageID)` records;
leaf pages hold `(key, RecordID)` records plus a `BTreeHeader` with `left`/`right` leaf pointers.

### Insert

`insert_helper` is recursive and returns `Option<BTreeRecord<PageID>>` — the separator record that
must be pushed into the parent when the current page splits:

- **Inner page:** binary search over `records[1..length]` to find the child, recurse, then insert the
  returned separator. On overflow, the page splits at its midpoint: the second half moves into a new
  page, the median key becomes the separator returned upward, and the recursed record is inserted
  into whichever half it belongs to.
- **Leaf page:** a match overwrites the existing `RecordID` in place (the tree is a *unique* index).
  On insert into a full leaf, the upper half moves to a new leaf, the new leaf is linked into the
  leaf chain (`left`/`right`), and the first key of the new leaf becomes the separator.
- **Root split:** if the recursion returns a separator at the root, `insert` allocates a new inner
  root with two children and the tree grows by one level.

### Lookup and range scan

`search_page` descends once with binary search and stops at the first leaf. `range_lookup` starts at
the leaf found for the range's lower bound, binary-searches the position inside it, and then walks
the `right` pointers until a key exceeds the upper bound. Because leaves are linked, the scan costs
one descent plus a sequential walk.

### k-way merge

`sort(k, ...)` merges groups of already-sorted page lists; `merge` performs the actual k-way merge by
selecting the minimum over the current slot of each list, with first-occurrence tie-breaking for
equal keys. When the output page fills up, it is swapped for the next one and the slot state of all
`k` inputs is recorded as a `RecordID` vector, which is how the caller can resume a merge across
page boundaries.

### Operators

`TableScan` walks a table's page list through the buffer pool, pinning and unpinning around every
access and enforcing the `open`/`next`/`close` lifecycle. `EquiHashJoin` builds on the left
(unique-key) side and probes with the right, concatenating the two records. `Sort` materializes its
input and sorts it with a supplied comparator.

## Limitations

- **No deletes.** There is no `remove` operation and no merging of underfull pages, so the tree only
  grows; the lab scope ends at insert/lookup/range.
- **Unique keys only.** Inserting an existing key overwrites its `RecordID` instead of storing
  duplicates.
- **Everything resident.** Pages live in a plain `Vec<Page>` with no eviction, so the "buffer pool"
  is the whole tree in memory and the disk-oriented behaviour of the exercise is only structural.
- **No concurrency.** No latches, no lock coupling, no optimistic validation.
- **Single-threaded merge.** The k-way merge is a sequential minimum scan over `k` candidates; it is
  the pedagogical version, not a tournament tree or a SIMD merge.
- **Blocking operators.** `Sort` and `EquiHashJoin` materialize their input and do not spill, so
  their memory grows with the input size.
- **Range scan assumes well-formed leaves.** It relies on `header.right` being maintained by every
  split, which is why the leaf-linking code runs before the separator is returned.

## Performance

Measured locally on an **Intel Core i5-12450H (8 cores / 12 threads), 16 GB RAM, Windows 11**,
`cargo --release`, single run, synthetic data. Indicative numbers, not a controlled suite.

| Workload | Result |
|---|---|
| Insert 5 M random `i32` keys (production capacities 508/336) | **1.44 M ops/s** |
| Point lookup, same keys (cache-cold order) | **2.81 M ops/s** |
| Range scan, width 10⁶ (~1,164 rows/query) | **5.16 µs/query** ≈ 225 M rows/s |

The tree occupied 21,106 pages (≈ 86 MB at 4 KB/page) for 5 M keys, i.e. ~70% leaf occupancy
after random inserts.

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
