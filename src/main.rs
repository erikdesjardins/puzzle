use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

use self::Edge::*;

#[derive(Copy, Clone)]
#[repr(u8)]
enum Edge {
    // Holes
    HO /* ctagon */ = 0 & 0b1111,
    HC /* ross */ = 1 & 0b1111,
    HI /* n arrow */ = 2 & 0b1111,
    HA /* rrow */ = 3 & 0b1111,
    // Protrusions
    // Each protrusion is the complement of the matching hole
    // Note: In/Out swapped, since from the perspective of each tile the opposite ones fit together
    PO /* ctagon */ = !0 & 0b1111,
    PC /* ross */ = !1 & 0b1111,
    PI /* n arrow */ = !3 & 0b1111,
    PA /* rrow */ = !2 & 0b1111,
}

fn nibble_to_edge(x: u8) -> Edge {
    let x = x & 0b1111;
    match () {
        _ if x == HO as u8 => HO,
        _ if x == HC as u8 => HC,
        _ if x == HI as u8 => HI,
        _ if x == HA as u8 => HA,
        _ if x == PO as u8 => PO,
        _ if x == PC as u8 => PC,
        _ if x == PI as u8 => PI,
        _ if x == PA as u8 => PA,
        _ => unreachable!(),
    }
}

/// Elements in row-major order:
///  00 01 02 03
///  04 05 06 07
///  08 09 10 11
///  12 13 14 15
///
/// Sides clockwise from the top:
///  (a, b, c, d)
/// OR
///  0b dddd cccc bbbb aaaa
///  ┌ a ┐
///  d   b
///  └ c ┘
type Tiles = [[(Edge, Edge, Edge, Edge); 4]; 4];
type State = [u16; 16];
/// Per-side right shifts
const TOP: u8 = 0 * 4;
const RIGHT: u8 = 1 * 4;
const BOTTOM: u8 = 2 * 4;
const LEFT: u8 = 3 * 4;

fn tiles_to_state(tiles: Tiles) -> State {
    let mut state = [0; 16];
    for (i, row) in tiles.iter().enumerate() {
        for (j, &(a, b, c, d)) in row.iter().enumerate() {
            state[i * 4 + j] |= (a as u16) << TOP;
            state[i * 4 + j] |= (b as u16) << RIGHT;
            state[i * 4 + j] |= (c as u16) << BOTTOM;
            state[i * 4 + j] |= (d as u16) << LEFT;
        }
    }
    state
}

fn state_to_tiles(state: State) -> Tiles {
    let mut tiles = [[(HO, HO, HO, HO); 4]; 4];
    for (i, row) in tiles.iter_mut().enumerate() {
        for (j, (a, b, c, d)) in row.iter_mut().enumerate() {
            *a = nibble_to_edge((state[i * 4 + j] >> TOP) as u8);
            *b = nibble_to_edge((state[i * 4 + j] >> RIGHT) as u8);
            *c = nibble_to_edge((state[i * 4 + j] >> BOTTOM) as u8);
            *d = nibble_to_edge((state[i * 4 + j] >> LEFT) as u8);
        }
    }
    tiles
}

fn print(tiles: Tiles) {
    if !PRINT_SOLUTIONS { return; }

    fn edge_to_str(edge: Edge) -> &'static str {
        match edge {
            HO => "(o)",
            HC => "(†)",
            HI => "(A)",
            HA => "(I)",
            PO => " o ",
            PC => " † ",
            PI => " I ",
            PA => " A ",
        }
    }
    for row in tiles.iter() {
        for &(a, _, _, _) in row { print!(" ┌{}┐ ", edge_to_str(a)); }
        println!();
        for &(_, b, _, d) in row { print!("{} {}", edge_to_str(d), edge_to_str(b)); }
        println!();
        for &(_, _, c, _) in row { print!(" └{}┘ ", edge_to_str(c)); }
        println!();
    }
    println!();
}

const ZERO: AtomicUsize = AtomicUsize::new(0);

static VALID: AtomicUsize = ZERO;
static ATTEMPTS: AtomicUsize = ZERO;
static NO_MORE_PIECES_FIT: [AtomicUsize; 16] = [ZERO; 16];
static SUCCESS_IMPOSSIBLE: [AtomicUsize; 16] = [ZERO; 16];
static SUCCESS_POSSIBLE: [AtomicUsize; 16] = [ZERO; 16];

fn apply_transform(tile: u16, tfm: u32) -> u16 {
    let tile = tile.rotate_right((tfm % 4) * 4);
    if tfm < 4 {
        tile
    } else {
        // flip tile by swapping opposite sides:
        //  0b dddd cccc bbbb aaaa
        //  ┌ a ┐      ┌ c ┐
        //  d   b  ->  d   b
        //  └ c ┘      └ a ┘
        //  0xdcba -> 0xdabc
        let side_b_and_d = tile & 0xf0f0;
        let side_a = tile & 0x000f;
        let side_c = tile & 0x0f00;
        side_b_and_d | (side_a << (2 * 4)) | (side_c >> (2 * 4))
    }
}

#[inline(never)]
fn find_solutions(state: State) {
    // recursion with typelevel bounds to ensure all recursive calls can be inlined
    struct Z;
    struct S<T>(PhantomData<T>);
    trait Recur {
        const INDEX: usize;
        fn run(state: State) -> bool /* found some valid state */;
    }
    impl<T: Recur> Recur for S<T> {
        const INDEX: usize = T::INDEX - 1;
        fn run(mut state: State) -> bool {
            let i = Self::INDEX;
            // track whether we recursed and whether a recursive call was successful
            let mut attempts = 0;
            let mut any_recursed = false;
            let mut found_solution = false;
            // try swapping with all future indices,
            // and the current index (i.e. keeping it in place)
            for j in i..16 {
                for tfm in 0..if ALLOW_FLIPPING { 8 } else { 4 } {
                    attempts += 1;
                    // piece to be swapped into the current index
                    let j_piece = state[j];
                    let new_j_piece = apply_transform(j_piece, tfm);
                    // check index immediately before...
                    let is_first_col = (i % 4) == 0;
                    let before_valid = is_first_col || {
                        let before = i - 1;
                        (state[before] >> RIGHT) & 0b1111 == !(new_j_piece >> LEFT) & 0b1111
                    };
                    if !before_valid {
                        continue;
                    }
                    // ...and above
                    let is_first_row = i < 4;
                    let above_valid = is_first_row || {
                        let above = i - 4;
                        (state[above] >> BOTTOM) & 0b1111 == !(new_j_piece >> TOP) & 0b1111
                    };
                    if !above_valid {
                        continue;
                    }
                    // swap pieces and recurse
                    state[j] = state[i];
                    state[i] = new_j_piece;
                    any_recursed = true;
                    found_solution |= T::run(state);
                    state[i] = state[j];
                    state[j] = j_piece;
                }
            }
            ATTEMPTS.fetch_add(attempts, Relaxed);
            if !any_recursed {
                NO_MORE_PIECES_FIT[Self::INDEX].fetch_add(1, Relaxed);
            }
            if !found_solution {
                SUCCESS_IMPOSSIBLE[Self::INDEX].fetch_add(1, Relaxed);
            } else {
                SUCCESS_POSSIBLE[Self::INDEX].fetch_add(1, Relaxed);
            }
            found_solution
        }
    }
    impl Recur for Z {
        const INDEX: usize = 16;
        #[cold]
        fn run(state: State) -> bool {
            // this point is only reached if all indices are valid
            print(state_to_tiles(state));
            VALID.fetch_add(1, Relaxed);
            true
        }
    }

    type S16 = S<S<S<S<S<S<S<S<S<S<S<S<S<S<S<S<Z>>>>>>>>>>>>>>>>;
    assert_eq!(S16::INDEX, 0);
    S16::run(state);
}

const PRINT_SOLUTIONS: bool = true;
const ALLOW_FLIPPING: bool = false;

fn main() {
    // the ultimate puzzle 4x4
    // https://c1.staticflickr.com/1/67/184473307_8e2cf41093_b.jpg
    let tiles = [
        [(HI, HC, PI, PI), (HI, HO, PC, PA), (HO, HI, PA, PO), (HO, HC, PO, PI)],
        [(HO, HA, PA, PA), (HA, HI, PO, PI), (HI, HI, PO, PC), (HC, HO, PO, PC)],
        [(HO, HI, PO, PO), (HI, HO, PI, PA), (HI, HA, PI, PC), (HC, HA, PO, PI)],
        [(HA, HO, PO, PC), (HO, HC, PA, PO), (HO, HC, PI, PI), (HC, HA, PC, PA)],
    ];
    let start_time = Instant::now();
    find_solutions(tiles_to_state(tiles));
    let elapsed_time = start_time.elapsed();

    println!(
        "Valid states: {} ({} flipping) (in {}.{:0>3}s)",
        VALID.load(Relaxed),
        if ALLOW_FLIPPING { "with" } else { "without" },
        elapsed_time.as_secs(),
        elapsed_time.subsec_millis(),
    );
    println!("Attempted states: {}", ATTEMPTS.load(Relaxed));
    let fmt = |arr: &[AtomicUsize; 16]| arr.iter().map(|a| format!("{:>6}", a.load(Relaxed))).collect::<Vec<_>>().join(", ");
    println!("States (by # pieces):  {}", (0..16).map(|i| format!("{:>6}", i)).collect::<Vec<_>>().join("  "));
    println!("- no more pieces fit [{}]", fmt(&NO_MORE_PIECES_FIT));
    println!("- success impossible [{}]", fmt(&SUCCESS_IMPOSSIBLE));
    println!("- success possible   [{}]", fmt(&SUCCESS_POSSIBLE));
}
