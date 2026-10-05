//! `std::sort` exactly as libstdc++ (GCC, the MinGW build of the original) implements it:
//! introsort with median-of-three pivots, a heap-sort fallback and a final insertion sort with a
//! threshold of 16. `std::sort` is not stable, so the order of elements that compare equal
//! depends on the algorithm; reproducing it keeps vendor item lists and other sorted data in the
//! same order as the original.
//!
//! Transcribed from libstdc++'s `bits/stl_algo.h` / `bits/stl_heap.h`. No C++ compiler is
//! available here, so the order of equal elements has not been checked against a compiled
//! reference yet (listed in NOTES.md).

/// `_S_threshold`
const THRESHOLD: usize = 16;

/// `std::sort(first, last, comp)` with `comp(a, b)` meaning "a goes before b".
pub fn sort_by<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], mut comp: F) {
    let n = v.len();
    if n != 0 {
        introsort_loop(v, 0, n, lg(n) * 2, &mut comp);
        final_insertion_sort(v, 0, n, &mut comp);
    }
}

/// `std::sort(first, last)` (`operator<`).
pub fn sort<T: Clone + PartialOrd>(v: &mut [T]) {
    sort_by(v, |a, b| a < b);
}

/// `std::__lg`
fn lg(n: usize) -> usize {
    (usize::BITS - 1 - n.leading_zeros()) as usize
}

fn introsort_loop<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, mut last: usize, mut depth_limit: usize, comp: &mut F) {
    while last - first > THRESHOLD {
        if depth_limit == 0 {
            partial_sort(v, first, last, last, comp);
            return;
        }
        depth_limit -= 1;
        let cut = unguarded_partition_pivot(v, first, last, comp);
        introsort_loop(v, cut, last, depth_limit, comp);
        last = cut;
    }
}

fn move_median_to_first<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], result: usize, a: usize, b: usize, c: usize, comp: &mut F) {
    if comp(&v[a], &v[b]) {
        if comp(&v[b], &v[c]) {
            v.swap(result, b);
        } else if comp(&v[a], &v[c]) {
            v.swap(result, c);
        } else {
            v.swap(result, a);
        }
    } else if comp(&v[a], &v[c]) {
        v.swap(result, a);
    } else if comp(&v[b], &v[c]) {
        v.swap(result, c);
    } else {
        v.swap(result, b);
    }
}

fn unguarded_partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], mut first: usize, mut last: usize, pivot: usize, comp: &mut F) -> usize {
    loop {
        while comp(&v[first], &v[pivot]) {
            first += 1;
        }
        last -= 1;
        while comp(&v[pivot], &v[last]) {
            last -= 1;
        }
        if first >= last {
            return first;
        }
        v.swap(first, last);
        first += 1;
    }
}

fn unguarded_partition_pivot<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, comp: &mut F) -> usize {
    let mid = first + (last - first) / 2;
    move_median_to_first(v, first, first + 1, mid, last - 1, comp);
    unguarded_partition(v, first + 1, last, first, comp)
}

fn unguarded_linear_insert<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], mut last: usize, comp: &mut F) {
    let val = v[last].clone();
    let mut next = last - 1;
    while comp(&val, &v[next]) {
        v[last] = v[next].clone();
        last = next;
        next -= 1;
    }
    v[last] = val;
}

fn insertion_sort<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, comp: &mut F) {
    if first == last {
        return;
    }
    for i in first + 1..last {
        if comp(&v[i], &v[first]) {
            // `std::move_backward(first, i, i + 1)` then `*first = val`
            v[first..=i].rotate_right(1);
        } else {
            unguarded_linear_insert(v, i, comp);
        }
    }
}

fn unguarded_insertion_sort<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, comp: &mut F) {
    for i in first..last {
        unguarded_linear_insert(v, i, comp);
    }
}

fn final_insertion_sort<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, comp: &mut F) {
    if last - first > THRESHOLD {
        insertion_sort(v, first, first + THRESHOLD, comp);
        unguarded_insertion_sort(v, first + THRESHOLD, last, comp);
    } else {
        insertion_sort(v, first, last, comp);
    }
}

fn push_heap<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, mut hole_index: usize, top_index: usize, value: T, comp: &mut F) {
    let mut parent = hole_index.wrapping_sub(1) / 2;
    while hole_index > top_index && comp(&v[first + parent], &value) {
        v[first + hole_index] = v[first + parent].clone();
        hole_index = parent;
        parent = hole_index.wrapping_sub(1) / 2;
    }
    v[first + hole_index] = value;
}

fn adjust_heap<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, mut hole_index: usize, len: usize, value: T, comp: &mut F) {
    let top_index = hole_index;
    let mut second_child = hole_index;
    // `(len - 1) / 2` with signed `ptrdiff_t`: for len == 0 this is 0.
    let half = if len == 0 { 0 } else { (len - 1) / 2 };
    while second_child < half {
        second_child = 2 * (second_child + 1);
        if comp(&v[first + second_child], &v[first + (second_child - 1)]) {
            second_child -= 1;
        }
        v[first + hole_index] = v[first + second_child].clone();
        hole_index = second_child;
    }
    if (len & 1) == 0 && len >= 2 && second_child == (len - 2) / 2 {
        second_child = 2 * (second_child + 1);
        v[first + hole_index] = v[first + (second_child - 1)].clone();
        hole_index = second_child - 1;
    }
    push_heap(v, first, hole_index, top_index, value, comp);
}

fn make_heap<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, comp: &mut F) {
    if last - first < 2 {
        return;
    }
    let len = last - first;
    let mut parent = (len - 2) / 2;
    loop {
        let value = v[first + parent].clone();
        adjust_heap(v, first, parent, len, value, comp);
        if parent == 0 {
            return;
        }
        parent -= 1;
    }
}

fn pop_heap<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, result: usize, comp: &mut F) {
    let value = v[result].clone();
    v[result] = v[first].clone();
    adjust_heap(v, first, 0, last - first, value, comp);
}

fn heap_select<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, middle: usize, last: usize, comp: &mut F) {
    make_heap(v, first, middle, comp);
    for i in middle..last {
        if comp(&v[i], &v[first]) {
            pop_heap(v, first, middle, i, comp);
        }
    }
}

fn sort_heap<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, mut last: usize, comp: &mut F) {
    while last - first > 1 {
        last -= 1;
        pop_heap(v, first, last, last, comp);
    }
}

fn partial_sort<T: Clone, F: FnMut(&T, &T) -> bool>(v: &mut [T], first: usize, middle: usize, last: usize, comp: &mut F) {
    heap_select(v, first, middle, last, comp);
    sort_heap(v, first, middle, comp);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_like_a_sort() {
        let mut seed = 12345u32;
        for n in [0usize, 1, 2, 5, 16, 17, 33, 100, 1000] {
            let mut v: Vec<u32> = (0..n)
                .map(|_| {
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    (seed >> 16) % 50
                })
                .collect();
            let mut expected = v.clone();
            expected.sort();
            sort(&mut v);
            assert_eq!(v, expected);
        }
    }

    #[test]
    fn heap_fallback_sorts() {
        let mut v: Vec<i32> = (0..200).rev().collect();
        let n = v.len();
        partial_sort(&mut v, 0, n, n, &mut |a: &i32, b: &i32| a < b);
        assert_eq!(v, (0..200).collect::<Vec<_>>());
    }
}
