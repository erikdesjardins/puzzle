use std::marker::PhantomData;

use self::Edge::*;

#[derive(Copy, Clone)]
#[repr(u8)]
enum Edge {
    // Holes
    HO /* ctagon */ = 0 & 0b1111,
    HC /* ross */ = 1 & 0b1111,
    HI /* n arrow */ = 2 & 0b1111,
    HA /* rrow */ = 3 & 0b1111,
    // Prods
    // Each prod is the complement of the matching hole
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
        for (j, &(a, b, c, d )) in row.iter().enumerate() {
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

fn find_solutions(state: State) {
    // recursion with typelevel bounds to ensure all recursive calls can be inlined
    struct Z;
    struct S<T>(PhantomData<T>);
    trait Recur {
        const INDEX: usize;
        fn run(state: State);
    }
    impl<T: Recur> Recur for S<T> {
        const INDEX: usize = T::INDEX - 1;
        fn run(mut state: State) {
            let i = Self::INDEX;
            // try swapping with all future indices,
            // and the current index (i.e. keeping it in place)
            for j in i..16 {
                for rot in 0..4 {
                    // piece to be swapped into the current index
                    let j_piece = state[j].rotate_right(rot * 4);
                    // check index immediately before and above
                    let is_first_col = (i % 4) == 0;
                    let before_valid = is_first_col || {
                        let before = i - 1;
                        (state[before] >> RIGHT) & 0b1111 == !(j_piece >> LEFT) & 0b1111
                    };
                    let is_first_row = i < 4;
                    let above_valid = is_first_row || {
                        let above = i - 4;
                        (state[above] >> BOTTOM) & 0b1111 == !(j_piece >> TOP) & 0b1111
                    };
                    if before_valid && above_valid {
                        state[j] = state[i];
                        state[i] = j_piece;
                        T::run(state);
                    }
                }
            }
        }
    }
    impl Recur for Z {
        const INDEX: usize = 16;
        #[cold]
        #[inline(never)]
        fn run(state: State) {
            // this point is only reached if all indices are valid
            print(state_to_tiles(state));
        }
    }

    type S16 = S<S<S<S<S<S<S<S<S<S<S<S<S<S<S<S<Z>>>>>>>>>>>>>>>>;
    assert_eq!(S16::INDEX, 0);
    S16::run(state);
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
    find_solutions(tiles_to_state(tiles));
}
