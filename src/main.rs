use std::mem::size_of;

fn all_bytes_nonzero(x: u128) -> bool {
    let discriminant = (x - 0x01010101010101010101010101010101) & !x & 0x80808080808080808080808080808080;
    discriminant == 0
}

fn swap_nibbles(x: u128) -> u128 {
    let high_to_low = (x >> 4) & 0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f;
    let low_to_high = (x << 4) & 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0;
    high_to_low | low_to_high
}

fn swap_bytes(x: u128, i: usize, j: usize) -> u128 {
    let i_byte = (x >> (i * 8)) & 0xff;
    let j_byte = (x >> (j * 8)) & 0xff;

    let j_to_i = (x      & !(0xff << (i * 8))) | (j_byte << (i * 8));
    let i_to_j = (j_to_i & !(0xff << (j * 8))) | (i_byte << (j * 8));

    i_to_j
}

fn valid(state: u128) -> bool {
    // d4 c4 b4 a4 d3 c3 b3 a3 d2 c2 b2 a2 d1 c1 b1 a1
    let preceeding_rows = state;
    //             d4 c4 b4 a4 d3 c3 b3 a3 d2 c2 b2 a2
    let succeeding_rows = state >> (4 * 8);
    // compare each row with the following row, ignoring wraparound comparisons
    let rows_match = all_bytes_nonzero((preceeding_rows & swap_nibbles(succeeding_rows)) | 0xffffffff000000000000000000000000);
    if !rows_match { return false }

    // d4 c4 b4 a4 d3 c3 b3 a3 d2 c2 b2 a2 d1 c1 b1 a1
    let preceeding_cols = state;
    //    d4 c4 b4 xx d3 c3 b3 xx d2 c2 b2 xx d1 c1 b1
    let succeeding_cols = state >> (1 * 8);
    // compare each col with the following col, ignoring cross-row comparisons
    let cols_match = all_bytes_nonzero((preceeding_cols & swap_nibbles(succeeding_cols)) | 0xff000000ff000000ff000000ff000000);
    if !cols_match { return false }

    true
}

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
                state = swap_bytes(state, 0, i);
            } else {
                state = swap_bytes(state, c[i], i);
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

fn check_all_permutations_of(state: u128) {
    all_permutations_of(state, |state| {
        if valid(state) {
            print(state);
        }
    })
}

#[cold]
#[inline(never)]
fn print(mut state: u128) {
    let mut pull_low_byte = move || {
        let low = state as u8;
        state >>= 8;
        low
    };

    for _ in 0..4 {
        for _ in 0..4 {
            print!("{:0>2x} ", pull_low_byte());
        }
        println!();
    }
    println!();
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Hole {
    HO /* ctagon */ = 1 << 7,
    HC /* ross */ = 1 << 6,
    HI /* n arrow */ = 1 << 5,
    HA /* rrow */ = 1 << 4,
}
use self::Hole::*;

#[derive(Copy, Clone)]
#[repr(u8)]
// note In/Out swapped, since from the perspective of each tile the opposite ones fit together
enum Prod {
    PO /* ctagon */ = 1 << 3,
    PC /* ross */ = 1 << 2,
    PI /* n arrow */ = 1 << 0,
    PA /* rrow */ = 1 << 1,
}
use self::Prod::*;

fn to_state(tiles: [[(Hole, Hole, Prod, Prod); 4]; 4]) -> u128 {
    let mut state = 0;
    for (i, row) in tiles.iter().enumerate() {
        for (j, &(h1, h2, p1, p2 )) in row.iter().enumerate() {
            let byte = h1 as u8 | h2 as u8 | p1 as u8 | p2 as u8;
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
