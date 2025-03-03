use std::iter;

use super::{BoardValue, Move};

struct BoardPattern<const LEN: usize> {
    targets: [bool; LEN],
    def_forced: [bool; LEN],
    atk_forced: [bool; LEN],
}
impl<const LEN: usize> BoardPattern<LEN> {
    fn iter_forced(&self) -> impl Iterator<Item = (&bool, &bool)> {
        iter::zip(self.atk_forced.iter(), self.def_forced.iter())
    }
}

macro_rules! parse_bool_token {
    (T) => {
        true
    };
    (F) => {
        false
    };
}
macro_rules! bool_arr {
    ($($tt:tt)*) => {
        [$(parse_bool_token!($tt)),*]
    };
}

const THREE_PATTERN: BoardPattern<8> = BoardPattern {
    targets: bool_arr!(F F T T T F F F),
    def_forced: bool_arr!(T T F F F T T F),
    atk_forced: bool_arr!(F F F F F T F F),
};
const FOUR_PATTERN1: BoardPattern<6> = BoardPattern {
    targets: bool_arr!(F F T T T T),
    def_forced: bool_arr!(T T F F F F),
    atk_forced: bool_arr!(F T F F F F),
};
const FOUR_PATTERN2: BoardPattern<6> = BoardPattern {
    targets: bool_arr!(F T F T T T),
    def_forced: bool_arr!(T F T F F F),
    atk_forced: bool_arr!(F F T F F F),
};
const FOUR_PATTERN3: BoardPattern<6> = BoardPattern {
    targets: bool_arr!(F T T F T T),
    def_forced: bool_arr!(T F F T F F),
    atk_forced: bool_arr!(F F F T F F),
};
const FOUR_PATTERN4: BoardPattern<6> = BoardPattern {
    targets: bool_arr!(F T T T F T),
    def_forced: bool_arr!(T F F F T F),
    atk_forced: bool_arr!(F F F F T F),
};
const FOUR_PATTERN5: BoardPattern<6> = BoardPattern {
    targets: bool_arr!(F T T T T F),
    def_forced: bool_arr!(T F F F F T),
    atk_forced: bool_arr!(T F F F F T),
};

pub fn get_forced(
    area: &Vec<(BoardValue, usize, usize)>,
    target_value: &BoardValue,
) -> (Vec<Move>, Vec<Move>) {
    let is_target = |(v, _x, _y): &(BoardValue, usize, usize)| -> bool { v == target_value };
    let opposite = target_value.opposite().unwrap();
    let isnt_opposite = |&(v, _x, _y): &(BoardValue, usize, usize)| -> bool { v != opposite };

    let mut atk_forced = Vec::new();
    let mut def_forced = Vec::new();

    let target_iter: Vec<_> = area.iter().map(is_target).collect();
    macro_rules! check_pat {
        ($pat:ident, $i:ident) => {
            let len = $pat.targets.len();
            let area_window = &area[$i..usize::min($i + len, area.len())];
            if area_window.iter().all(isnt_opposite) {
                let window = &target_iter[$i..usize::min($i + len, area.len())];

                if window.iter().eq($pat.targets.iter()) {
                    for (&(_v, x, y), (&atk, &def)) in
                        iter::zip(area_window.iter(), $pat.iter_forced())
                    {
                        if atk {
                            atk_forced.push([x, y]);
                        }
                        if def {
                            def_forced.push([x, y]);
                        }
                    }
                    return (atk_forced, def_forced);
                }
                if window.iter().rev().eq($pat.targets.iter()) {
                    for (&(_v, x, y), (&atk, &def)) in
                        iter::zip(area_window.iter().rev(), $pat.iter_forced())
                    {
                        if atk {
                            atk_forced.push([x, y]);
                        }
                        if def {
                            def_forced.push([x, y]);
                        }
                    }
                    return (atk_forced, def_forced);
                }
            }
        };
    }
    for i in 0..area.len() {
        if fastrand::bool() {
            check_pat!(THREE_PATTERN, i);
        }
        check_pat!(FOUR_PATTERN1, i);
        // check_pat!(FOUR_PATTERN2, i);
        // check_pat!(FOUR_PATTERN3, i);
        // check_pat!(FOUR_PATTERN4, i);
        check_pat!(FOUR_PATTERN5, i);
    }
    (Vec::new(), Vec::new())
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    #[test]
    fn test_forced() {
        let mut board = Board::new(19, 19);
        board.place([0, 3]).unwrap();
        board.place([1, 3]).unwrap();
        board.place([0, 4]).unwrap();
        board.place([2, 4]).unwrap();
        board.place([0, 5]).unwrap();
        println!("{board}");
        println!("{:?}", board.calculate_forced());
    }
}
