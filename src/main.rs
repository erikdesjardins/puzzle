mod u128;

use std::mem::size_of;

use crate::u128::Ext;

fn valid(state: u128) -> bool {
    // d4 c4 b4 a4 d3 c3 b3 a3 d2 c2 b2 a2 d1 c1 b1 a1
    let preceeding_rows = state;
    //             d4 c4 b4 a4 d3 c3 b3 a3 d2 c2 b2 a2
    let succeeding_rows = state >> (4 * 8);
    // compare each row with the following row, ignoring wraparound comparisons
    let rows_match = ((preceeding_rows & succeeding_rows.swap_nibbles()) | 0xffffffff000000000000000000000000).all_bytes_nonzero();
    if !rows_match { return false }

    // d4 c4 b4 a4 d3 c3 b3 a3 d2 c2 b2 a2 d1 c1 b1 a1
    let preceeding_cols = state;
    //    d4 c4 b4 xx d3 c3 b3 xx d2 c2 b2 xx d1 c1 b1
    let succeeding_cols = state >> (1 * 8);
    // compare each col with the following col, ignoring cross-row comparisons
    let cols_match = ((preceeding_cols & succeeding_cols.swap_nibbles()) | 0xff000000ff000000ff000000ff000000).all_bytes_nonzero();
    if !cols_match { return false }

    true
}

// todo perhaps use a "naive" search, recursing 16 times and picking the only pieces that fit
//  to opt, use typelevel numbers to terminate recursion (if necessary) and force inline
fn all_permutations_of(original_state: u128, mut output: impl FnMut(u128)) {
    const N: usize = size_of::<u128>();
    let is_even = |x| x % 2 == 0;

    let mut state = original_state;
    let mut c = [0; N];
    let mut i = 0;

    output(state);

    while i < N {
        if c[i] < i {
            if is_even(i) {
                state = state.swap(0, i);
            } else {
                state = state.swap(c[i], i);
            }
            output(state);
            c[i] += 1;
            i = 0;
        } else {
            c[i] = 0;
            i += 1;
        }
    }
}

// normal
#[cfg(all())]
fn check_all_permutations_of(state: u128) {
    all_permutations_of(state, |state| {
        if valid(state) {
            print(state);
        }
    })
}

// for vtune
#[cfg(any())]
fn check_all_permutations_of(state: u128) {
    // include calls to this to avoid dce
    #[cold]
    #[inline(never)]
    fn sink(x: u128) {
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
    all_permutations_of(state, |state| {
        if valid(state) {
            sink(state);
        }
    });
}

// for time profiling
#[cfg(any())]
fn check_all_permutations_of(state: u128) {
    use std::time::Instant;
    let mut i = 0;
    let mut count = 0;
    let start = Instant::now();
    #[cold]
    #[inline(never)]
    fn end(start: Instant, valid: u32, all: u32) {
        let elapsed = start.elapsed();
        let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64) / (1_000_000_000 as f64);
        let nanos = (elapsed.as_secs() as u128 * 1_000_000_000) + elapsed.subsec_nanos() as u128;
        println!("{:0.1} s, {} ns/1, {:0.0} M/s", secs, nanos / all as u128, all as f64 / secs / 1_000_000.);
        println!("{:0.1} %", valid as f64 / all as f64 * 100.);
        panic!();
    }
    all_permutations_of(state, |state| { i += 1;
        if i == 1_000_000_000 {
            end(start, count, i);
        }
        if valid(state) { count += 1; }
    })
}


#[cold]
#[inline(never)]
fn print(mut state: u128) {
    let mut pull_2_bits = move || {
        let low = state as u8;
        state >>= 2;
        low & 0b11
    };

    for _row in 0..4 {
        for _col in 0..4 {
            for _side in 0..4 {
                print!("{}", pull_2_bits());
            }
            print!(" ");
        }
        println!();
    }
    println!();
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Hole {
    HO /* ctagon */ = 0,
    HC /* ross */ = 1,
    HI /* n arrow */ = 2,
    HA /* rrow */ = 3,
}
use self::Hole::*;

#[derive(Copy, Clone)]
#[repr(u8)]
// note In/Out swapped, since from the perspective of each tile the opposite ones fit together
enum Prod {
    PO /* ctagon */ = 0,
    PC /* ross */ = 1,
    PI /* n arrow */ = 3,
    PA /* rrow */ = 2,
}
use self::Prod::*;

fn to_state(tiles: [[(Hole, Hole, Prod, Prod); 4]; 4]) -> u128 {
    let mut state = 0;
    for (i, row) in tiles.iter().enumerate() {
        for (j, &(h1, h2, p1, p2 )) in row.iter().enumerate() {
            let byte =
                ((h1 as u8) << (0 * 2)) |
                ((h2 as u8) << (1 * 2)) |
                ((p1 as u8) << (2 * 2)) |
                ((p2 as u8) << (3 * 2));
            state |= (byte as u128) << ((i * 4 + j) * 8);
        }
    }
    state
}

fn main() {
    // the ultimate puzzle 4x4
    // https://c1.staticflickr.com/1/67/184473307_8e2cf41093_b.jpg
    let tiles = [
        [(HI, HC, PI, PI), (HI, HO, PC, PA), (HO, HI, PA, PO), (HO, HC, PO, PI)],
        [(HO, HA, PA, PA), (HA, HI, PO, PI), (HI, HI, PO, PC), (HC, HO, PO, PC)],
        [(HO, HI, PO, PO), (HI, HO, PI, PA), (HI, HA, PI, PC), (HC, HA, PO, PI)],
        [(HA, HO, PO, PC), (HO, HC, PA, PO), (HO, HC, PI, PI), (HC, HA, PC, PA)],
    ];
    let state = to_state(tiles);
    check_all_permutations_of(state);
}
