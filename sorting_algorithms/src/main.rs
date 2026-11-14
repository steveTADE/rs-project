use std::thread::sleep;
use std::time::Duration;
use std::sync::Mutex;
use std::thread;

type IntegerType = i16; // change this to i8, i32, i64, u16, etc.

const ARRAY_LENGTH: usize = 110;
const THRESHOLD: usize = 8; // low so the radix passes are visible on small arrays
const TOTAL_BITS: u32 = IntegerType::BITS;
const MIN_RUN: usize = 8; // runs shorter than this are extended with binary insertion
const SOFT_RESET       : &str = "\u{1b}[2J\u{1b}[3J";
const HIDE_CURSOR      : &str = "\u{1b}[?25l";
const SHOW_CURSOR      : &str = "\u{1b}[?25h";
const CURSOR_HOME      : &str = "\u{1b}[H";

/// Maps a value to an unsigned key with the same ordering (MIN -> 0).
const fn key(x: IntegerType) -> u64 {
    (x as i128 - IntegerType::MIN as i128) as u64
}

pub fn hybrid_radix_sort(arr: &mut [IntegerType], delay: Duration) {
    let len = arr.len();
    msd_sort(arr, 0, len, TOTAL_BITS, delay);
}

/// Sorts arr[lo..hi] using the low `bits` bits of each key.
fn msd_sort(arr: &mut [IntegerType], lo: usize, hi: usize, bits: u32, delay: Duration) {
    if hi - lo <= THRESHOLD {
        insertion_sort(arr, lo, hi, delay);
        return;
    }
    if bits == 0 {
        return; // all remaining bits are equal, so the slice is sorted
    }

    let width = bits.min(8);
    let shift = bits - width;
    let mask = (1u64 << width) - 1;
    let buckets = 1usize << width;

    // Count elements per bucket.
    let mut counts = vec![0usize; buckets];
    for &x in &arr[lo..hi] {
        counts[((key(x) >> shift) & mask) as usize] += 1;
    }

    // Starting offset of each bucket, absolute within the full array.
    let mut starts = vec![0usize; buckets];
    let mut total = lo;
    for i in 0..buckets {
        starts[i] = total;
        total += counts[i];
    }

    // In-place distribution, animated on every swap.
    let mut next = starts.clone();
    for b in 0..buckets {
        let end = starts[b] + counts[b];
        while next[b] < end {
            let target = ((key(arr[next[b]]) >> shift) & mask) as usize;
            if target == b {
                next[b] += 1;
            } else {
                arr.swap(next[b], next[target]);
                next[target] += 1;
                let status = format!(
                    "MSD partition: bits {}..{}, slice [{}..{}), swapped into bucket {}",
                    shift + width - 1, shift, lo, hi, target
                );
                display(arr, &status, delay);
            }
        }
    }

    // Recurse into each bucket on the next lower bits.
    for b in 0..buckets {
        let s = starts[b];
        let e = s + counts[b];
        if counts[b] > 1 {
            msd_sort(arr, s, e, shift, delay);
        }
    }
}

/// Insertion sort on arr[lo..hi], animated on every shift.
fn insertion_sort(arr: &mut [IntegerType], lo: usize, hi: usize, delay: Duration) {
    for i in lo + 1..hi {
        let current = arr[i];
        let mut j = i;
        while j > lo && arr[j - 1] > current {
            arr[j] = arr[j - 1];
            j -= 1;
            display(arr, "Insertion sort on a small group", delay);
        }
        arr[j] = current;
        display(arr, "Insertion sort on a small group", delay);
    }
}

/// Merges two sorted slices into a new sorted Vec.
fn merge(a: &[IntegerType], b: &[IntegerType]) -> Vec<IntegerType> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            out.push(a[i]);
            i += 1;
        } else {
            out.push(b[j]);
            j += 1;
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
    out
}

/// Bottom-up parallel merge sort. Each pair of runs at a given width is
/// merged on its own thread. Writes go through a shared Mutex so that
/// `display` sees a consistent array.
pub fn parallel_merge_sort(arr: &mut [IntegerType], delay: Duration) {
    let n = arr.len();
    let shared = Mutex::new(arr.to_vec());
    let mut width = 1;

    while width < n {
        thread::scope(|s| {
            for start in (0..n).step_by(2 * width) {
                let mid = (start + width).min(n);
                let end = (start + 2 * width).min(n);
                if mid >= end {
                    continue; // odd run at the end, already in place
                }
                let shared = &shared;
                s.spawn(move || {
                    // Copy both runs out so the merge itself runs without the lock.
                    let (left, right) = {
                        let guard = shared.lock().unwrap();
                        (guard[start..mid].to_vec(), guard[mid..end].to_vec())
                    };
                    let merged = merge(&left, &right);

                    if delay.is_zero() {
                        // Fast path: one lock, one copy.
                        shared.lock().unwrap()[start..end].copy_from_slice(&merged);
                    } else {
                        // Animated path: write and redraw one element at a time.
                        let status = format!("Merging width {width}: [{start}..{end})");
                        for (k, &v) in merged.iter().enumerate() {
                            let mut guard = shared.lock().unwrap();
                            guard[start + k] = v;
                            display(&guard, &status, delay);
                        }
                    }
                });
            }
        });
        width *= 2;
    }

    arr.copy_from_slice(&shared.into_inner().unwrap());
}


/// Timsort-style sort: find natural runs, extend short runs to MIN_RUN with
/// binary insertion, then merge adjacent runs using a run stack.
pub fn timsort(arr: &mut [IntegerType], delay: Duration) {
    let n = arr.len();
    if n < 2 {
        return;
    }

    let mut runs: Vec<(usize, usize)> = Vec::new(); // (start, length)
    let mut lo = 0;

    while lo < n {
        // 1. Find the natural run starting at lo.
        let mut hi = lo + 1;
        if hi < n && arr[hi] < arr[lo] {
            // Strictly descending run: extend it, then reverse it in place.
            while hi < n && arr[hi] < arr[hi - 1] {
                hi += 1;
            }
            arr[lo..hi].reverse();
            display(arr, "Reversed a descending run", delay);
        } else {
            // Non-decreasing run.
            while hi < n && arr[hi] >= arr[hi - 1] {
                hi += 1;
            }
        }

        // 2. Extend short runs to MIN_RUN with binary insertion sort.
        let target = (lo + MIN_RUN).min(n);
        if hi < target {
            let sorted_end = hi;
            hi = target;
            binary_insertion(arr, lo, sorted_end, hi, delay);
        }

        // 3. Push the run and merge while the stack invariant is violated.
        runs.push((lo, hi - lo));
        while runs.len() >= 2 {
            let len = runs.len();
            if runs[len - 2].1 > runs[len - 1].1 {
                break;
            }
            merge_top_two(arr, &mut runs, delay);
        }

        lo = hi;
    }

    // 4. Merge whatever is left on the stack.
    while runs.len() >= 2 {
        merge_top_two(arr, &mut runs, delay);
    }
}

/// Inserts arr[sorted_end..hi] into the already-sorted arr[lo..sorted_end].
fn binary_insertion(
    arr: &mut [IntegerType],
    lo: usize,
    sorted_end: usize,
    hi: usize,
    delay: Duration,
) {
    let status = "Binary insertion (extending a short run)";
    for i in sorted_end..hi {
        let x = arr[i];

        // Find the first position in arr[lo..i] whose value is greater than x.
        let (mut start, mut end) = (lo, i);
        while start < end {
            let mid = (start + end) / 2;
            if arr[mid] <= x {
                start = mid + 1;
            } else {
                end = mid;
            }
        }

        // Shift the larger elements right by one, animating each shift.
        for j in (start..i).rev() {
            arr[j + 1] = arr[j];
            display(arr, status, delay);
        }
        arr[start] = x;
        display(arr, status, delay);
    }
}

/// Merges the top two runs on the stack, which are adjacent in arr.
fn merge_top_two(arr: &mut [IntegerType], runs: &mut Vec<(usize, usize)>, delay: Duration) {
    let (_, right_len) = runs.pop().unwrap();
    let (left_start, left_len) = runs.pop().unwrap();
    let mid = left_start + left_len;
    let end = mid + right_len;

    merge_runs(arr, left_start, mid, end, delay);
    runs.push((left_start, left_len + right_len));
}

/// Merges sorted arr[start..mid] and arr[mid..end] in place,
/// using a copy of the left run as a buffer. Animates each write.
fn merge_runs(arr: &mut [IntegerType], start: usize, mid: usize, end: usize, delay: Duration) {
    let status = format!("Merging [{start}..{mid}) with [{mid}..{end})");
    let left: Vec<IntegerType> = arr[start..mid].to_vec();

    let (mut i, mut j, mut k) = (0, mid, start);
    while i < left.len() && j < end {
        if arr[j] < left[i] {
            arr[k] = arr[j];
            j += 1;
        } else {
            arr[k] = left[i];
            i += 1;
        }
        k += 1;
        display(arr, &status, delay);
    }
    // Any remaining left elements go into place. Remaining right elements
    // are already in the correct positions.
    while i < left.len() {
        arr[k] = left[i];
        i += 1;
        k += 1;
        display(arr, &status, delay);
    }
}
/// Clears the terminal and draws the array as bars.
/// this is the only part i have to manually intervene cause its flickering
/// rest is slop
fn display(arr: &[IntegerType], status: &str, delay: Duration) {
    if delay.is_zero() {
        return;
    }
    let maximum: &i16 = arr.iter().max().unwrap();
    print!("{}", CURSOR_HOME); // clear screen, cursor to top-left
    println!("{status}\n");
    for &x in arr {
        println!("{}{}", "#".repeat(x.abs() as usize), " ".repeat((maximum - x) as usize));
    }
    sleep(delay);
}

fn main() {
    let tmp : [IntegerType; ARRAY_LENGTH] = std::array::from_fn(|_| fastrand::i16(0..200));
    let mut data: Vec<IntegerType> = tmp.to_vec();

    println!("{}{}{}Before: {:?}", SOFT_RESET, CURSOR_HOME, HIDE_CURSOR, data);
    //hybrid_radix_sort(&mut data, Duration::from_millis(60));
    hybrid_radix_sort(&mut data, Duration::from_millis(10));
    println!("\x1B[2J\x1B[HAfter:  {:?}{}", data, SHOW_CURSOR);
}