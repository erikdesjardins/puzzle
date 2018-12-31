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
    assert!(i < j);
    let i_byte = (x >> (i * 8)) as u8;
    let j_byte = (x >> (j * 8)) as u8;

    let i_replaced = (x          & !(0xff << (i * 8))) | (i_byte << (i * 8)) as u128;
    let j_replaced = (i_replaced & !(0xff << (j * 8))) | (j_byte << (j * 8)) as u128;

    j_replaced
}

fn valid(state: u128) -> bool {
    // a1 b1 c1 d1 a2 b2 c2 d2 a3 b3 c3 d3 a4 b4 c4 d4
    let succeeding_rows = state;
    //             a1 b1 c1 d1 a2 b2 c2 d2 a3 b3 c3 d3
    let preceeding_rows = swap_nibbles(state >> (4 * 8));
    // compare each row with the following row, ignoring wraparound comparisons
    let rows_match = all_bytes_nonzero((preceeding_rows & succeeding_rows) | 0xffffffff000000000000000000000000);

    // a1 b1 c1 d1 a2 b2 c2 d2 a3 b3 c3 d3 a4 b4 c4 d4
    let succeeding_cols = state;
    //    a1 b1 c1 xx a2 b2 c2 xx a3 b3 c3 xx a4 b4 c4
    let preceeding_cols = swap_nibbles(state >> (1 * 8));
    // compare each col with the following col, ignoring cross-row comparisons
    let cols_match = all_bytes_nonzero((preceeding_cols & succeeding_cols) | 0xff000000ff000000ff000000ff000000);

    // TODO try using a branch in between cols/rows and see if it's faster
    // TODO it will probably get threaded into the loop branch anyways
    rows_match && cols_match
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
    let mut pull_high_byte = move || {
        let high = (state >> (128 - 8)) as u8;
        state <<= 8;
        high
    };

    for _ in 0..4 {
        for _ in 0..4 {
            print!("{:0>2x} ", pull_high_byte());
        }
        println!();
    }
}

fn main() {
    let state = 0x7fb8670bc88b6743d115534466e2f4fd;
    check_all_permutations_of(state);
}
