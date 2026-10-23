use crate::kwaymerge::KwayPage;
use crate::kwaymerge::basic_tests_kwmerge::is_sorted;
use crate::{PageID, RecordID, SlotID};
use std::iter::zip;
use std::mem::swap;

/// Sorts the input pages leveraging a k-way merge algorithm.
///
/// It is expected that this function makes use of the 'merge' function.
/// `input_pages` and `output_pages` will always have the same length and will never be empty.
/// The single pages within `input_pages` will also never be empty, but will not be necessarily
/// filled completely (page.len() != KwayPage::CAPACITY).
///
/// Note: `input_pages` and `output_pages` are empty if their length is 0 (will never happen).
/// A KwayPage, instead, is considered empty if its `length` attribute is 0, regardless of its
/// actual content.
///
///
/// # Arguments
///
/// * `k`: Arity of the merge operation (k buffers hold the pages to merge, and one extra buffer holds
/// the output of the merge process).
/// * `input_pages`: Slice containing the pages to sort.
/// * `output_pages`: Slice containing the sorted pages.
pub fn sort(k: usize, input_pages: &[KwayPage], output_pages: &mut [KwayPage]) {
    // To help you better understand how this function should behave, we will use the following
    // running example:
    // k = 3
    // input_pages = [5 4 1 7 15 4 9 12 8 2 6 5]
    // output_pages = [0 0 0 0 0 0 0 0 0 0 0 0]
    //
    // This example simplify things by using integers instead of pages. In reality,
    // `input_pages` and `output_pages` will contain elements of type KwayPage.

    // Base case, trivially sort a single page.
    if input_pages.len() == 1 {
        let mut rec = input_pages[0].records.to_vec();
        rec.resize(input_pages[0].length, 0);
        rec.sort();
        output_pages[0].id = input_pages[0].id;
        output_pages[0].length = input_pages[0].length;
        output_pages[0].records[..input_pages[0].length].clone_from_slice(&rec[..]);
        return;
    }

    // `stepsize` represents the distance in the `input_pages` slice between the k
    // lists that need to be merged.
    // Running example:
    // 3-way merge where `input_pages` contains 12 pages.
    // The 3 lists to merge start respectively at index 0, 4 and 8.
    // Input_pages: [5 4 1 7 15 4 9 12 8 2 6 5]
    // Seperation of lists to merge: [(5 4 1 7) (15 4 9 12) (8 2 6 5)]
    // Hint: you cannot call the 'merge' process on these sublists yet, as they are not sorted!
    // However, recursion helps a lot in these situations.
    // If `stepsize` is not the result of a perfect divison the "extra" elements will
    // fall in the last of the k lists.
    // Example:
    // input_pages.len() = 14
    // k = 3
    // stepsize = 14/3 = 4
    // separation of lists to merge: [(x x x x) (x x x x) (x x x x x x)]
    let mut k = k;
    if input_pages.len() < k {
        k = input_pages.len();
    }
    let stepsize = input_pages.len() / k;

    // A vector of size `k` representing the indices of the current pages in the `input_pages`
    // slice that are being merged together (one entry for each list).
    // They change during the merge process when a page is exhausted and the next page
    // from the same list needs to be loaded.
    // Running example:
    // pages_to_merge = [0 4 8], which are the indices of the first page of each list being merged.
    // These indexes increment during the merging process (see `merge` function).
    // When a list is exhausted (all of its pages have been merged), its
    // corresponding entry in `pages_to_merge` will be None.
    let mut pages_to_merge: Vec<Option<usize>> = vec![];

    // You are now expected to:
    // 1) Split the `input_pages` slice in k lists using `stepsize` and `pages_to_merge`.
    //    Note: the "splitting" process does not need any support vector, it can be done by
    //    keeping track of the indices using `pages_to_merge`.
    // 2.1) If the k lists are sorted, merge them using the `merge` function.
    // 2.2) If the k lists are not sorted, recursively sort them with a mix of `sort` and `merge`.
    // 3) Write the final, sorted list in the `output_pages` slice.
    //
    // Hint: It is not relevant the order in which you merge the lists together (left-to-right or
    // right-to-left), as long as `output_pages` contains the sorted result. However, we recommend
    // a left-to-right approach.

    for i in 0..k
    {
        let mut end = (i + 1) * stepsize;
        let start = i * stepsize;

        if i == k - 1
        {
            end = input_pages.len();
        }

        sort(k, &input_pages[start .. end], &mut output_pages[start .. end]);

        if input_pages[i].length == 0 {
            pages_to_merge.push(None);
        } else {
            pages_to_merge.push(Some(start));
        }
    }

    let new_input_pages = output_pages.to_vec();
    
    for i in 0..output_pages.len() {
        output_pages[i].length = 0;
    }

    merge(&mut pages_to_merge, stepsize, &new_input_pages, output_pages);
}

/// Merges k sorted lists of arbitrary size (i.e., arbitrary number of pages in e
/// ach list).
///
/// `input_pages` and `output_pages` will always have the same length and will never be empty.
///
/// # Arguments
///
/// * `pages_to_merge`: A vector of size 'k' representing the indices of the current pages that are
///   being merged (one entry for each list). See `sort` function for a more detailed description.
/// * `stepsize`: the distance in the `input_pages` slice between the k lists that need to be merged.
///   See `sort` function for a more detailed description.
/// * `input_pages`: A vector of pages representing the k lists to merge.
/// * `output_pages`: A vector with the same number of pages as 'input_pages' that will contain the sorted
///   output pages.
///
/// returns: Current RecordIDs (PageID + SlotID) for each of the k pages that are being merged when
///   an output page becomes full and needs to be substituted by a new one.
///
/// # Example
///
/// 3-way merge of the following `input_pages` vector:
/// input_pages = [PageID-0 PageID-1 PageID-2 PageID-3 PageID-4 PageID-5 PageID-6 PageID-7 PageID-8]
/// input_pages.len() = 9
/// pages_to_merge = [0, 3, 6] (which are the indices of PageID-0, PageID-3 and PageID-6). Do not
/// confuse the indices with the IDs of the pages: input_pages[0] now contains PageID-0, but it
/// could've also contained pageID-100!
/// Each page contains page.len() number of records.
///
/// Note: The pages of each list are sorted! Meaning that if we take PageID-0, PageID-1 and PageID-2
/// and we read them sequentially we will find all records sorted not only intra-page,
/// but also across all pages!
///
/// 1) PageID 0: [2 7 8 9]
///                 ^
/// 2) PageID 3: [0 6 7 10]
///                 ^
/// 3) PageID 6: [1 5 8 12]
///                   ^
/// The current states are:
/// (PageID 0, SlotID 1)
/// (PageID 3, SlotID 1)
/// (PageID 6, SlotID 2)
///
/// Assume that adding the value 6 from PageID-3 to the output page resulted in
/// outpage.len() == KwayPage::CAPACITY. Thus, the output page needs to be swapped for a new one.
/// The state to be inserted into the output vector in this case is the one reported above. After
/// that, the SlotID of PageID 3 can be increased by 1.
///
/// # Hints
/// 1) If during the selection of the mimimum record when merging there are duplicates, choose the first
///    occurrence of the value (e.g., merging pages 0 1 and 2 -> pages 0 and 1 have the same record X
///    as a candidate minimum -> pick the value from page 0).
/// 2) After writing a record to the output page, check its length. If it became full (and thus
///    needs to be swapped), use the current slots from each of the k page to populate the
///    resulting Vec<RecordID> and THEN increase the slot of the page that contains the minimum
///    record.
pub fn merge(
    pages_to_merge: &mut Vec<Option<usize>>,
    mut stepsize: usize,
    input_pages: &[KwayPage],
    output_pages: &mut [KwayPage],
) -> Vec<RecordID> {
    //Output vector containing the state of k pages when a given output page was finalized.
    let mut states_at_finalizing: Vec<RecordID> = vec![];

    // The number of lists to merge together ('k-way' merge).
    let k = pages_to_merge.len();

    // A vector of size 'k' representing the current slots for each page in 'pages_to_marge'.
    // curr_slots[0] will contain the current slot (the one you need to check during the merge process)
    // of pages_to_merge[0], which is the current active page of the first of the 'k' lists.
    // You should increase it after a value in a page is selected as the minimum during the merge process.
    let mut curr_slots = vec![0; k];

    // Variable to keep track of the current page in `output_pages` being written.
    let mut outpage_idx: usize = 0;

    // You are now expected to:
    // 1) Iterate over the current active page for each list (see 'pages_to_merge').
    // 2) Choose the minimum value among them (see 'curr_slots').
    // 3) Update the output_pages vector with the minimum.
    // 4) When necessary, update the 'state_at_finalizing' vector.
    
    loop {
        let mut finish = true;
        let mut minim = 0;
        let mut page_idx = 0;
        let mut slot_idx = 0;
        let mut min = false;
        let mut rec_id:RecordID = RecordID { page_id: PageID(0), slot_id: SlotID(0) };

        for i in 0..k
        {
            let page_op = pages_to_merge[i];
            match page_op {
                Some(page) => {
                    finish = false;
                    let rec = input_pages[page].records[curr_slots[i]];

                    if !min {
                        min = true;
                        minim = rec;
                        page_idx = page;
                        slot_idx = i;
                    }

                    if rec < minim {
                        minim = rec;
                        page_idx = page;
                        slot_idx = i;
                    }

                    if output_pages[outpage_idx].length + 1 == KwayPage::CAPACITY {
                        rec_id.page_id = input_pages[page].id;
                        rec_id.slot_id = SlotID(curr_slots[i] as u16);

                        states_at_finalizing.push(rec_id);
                    }
                },
                None => {}
            }
        }

        if finish {
            break;
        }

        output_pages[outpage_idx].records[output_pages[outpage_idx].length] = minim;
        output_pages[outpage_idx].length += 1;
        curr_slots[slot_idx] += 1;

        if curr_slots[slot_idx] == input_pages[page_idx].length {
            curr_slots[slot_idx] = 0;

            let mut end = (slot_idx + 1) * stepsize;

            if slot_idx == k - 1 {
                end = input_pages.len();
            }

            if page_idx + 1 == end || input_pages[page_idx + 1].length == 0 {
                pages_to_merge[slot_idx] = None;
            } else {
                pages_to_merge[slot_idx] = Some(page_idx + 1);
            }
        }

        if output_pages[outpage_idx].length == KwayPage::CAPACITY {
            outpage_idx += 1;

            if outpage_idx == output_pages.len() {
                break;
            }
        }
    }

    return states_at_finalizing
}