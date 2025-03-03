use std::{fmt::Display, iter};

use ndarray::{Array, Array2};
use radix_fmt::radix;

mod patterns;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoardValue {
    X,
    O,
    Empty,
}
impl BoardValue {
    fn opposite(&self) -> Option<BoardValue> {
        match self {
            BoardValue::X => Some(BoardValue::O),
            BoardValue::O => Some(BoardValue::X),
            BoardValue::Empty => None,
        }
    }

    fn player(&self) -> Option<Player> {
        match self {
            BoardValue::X => Some(Player::X),
            BoardValue::O => Some(Player::O),
            BoardValue::Empty => None,
        }
    }
}
impl Display for BoardValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BoardValue::X => "\x1b[91mX\x1b[0m",
            BoardValue::O => "\x1b[94mO\x1b[0m",
            BoardValue::Empty => " ",
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}
impl Player {
    const FIRST: Player = Player::X;
    fn board_value(&self) -> BoardValue {
        match self {
            Player::X => BoardValue::X,
            Player::O => BoardValue::O,
        }
    }
    fn next(&mut self) {
        *self = match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

pub type Move = [usize; 2];
pub type Util = f32;

#[derive(Debug)]
pub enum PlacingError {
    OutOfBounds,
    Occupied,
    FullBoard,
}
impl Display for PlacingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlacingError::OutOfBounds => f.write_str("Move is out of bound!"),
            PlacingError::Occupied => f.write_str("Move is occupied"),
            PlacingError::FullBoard => f.write_str("Entire board is filled"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameResult {
    XWins,
    OWins,
    Draws,
    NotTerminated,
    NotCalculated,
}
impl GameResult {
    fn is_terminate(&self) -> bool {
        match self {
            GameResult::XWins => true,
            GameResult::OWins => true,
            GameResult::Draws => true,
            GameResult::NotTerminated => false,
            GameResult::NotCalculated => false,
        }
    }
    fn win(player: Player) -> GameResult {
        match player {
            Player::X => GameResult::XWins,
            Player::O => GameResult::OWins,
        }
    }
    fn utility(&self, player: Player) -> Option<Util> {
        match self {
            GameResult::XWins => match player {
                Player::X => Some(1.0),
                Player::O => Some(0.0),
            },
            GameResult::OWins => match player {
                Player::X => Some(0.0),
                Player::O => Some(1.0),
            },
            GameResult::Draws => Some(0.5),
            GameResult::NotTerminated => None,
            GameResult::NotCalculated => None,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    grid: Array2<BoardValue>,
    pub player: Player,
    last_placement: Option<Move>,
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
    default_bound: bool,
    game_result: GameResult,
    x_forced: Option<Vec<Move>>,
    o_forced: Option<Vec<Move>>,
    width: usize,
    height: usize,
}
impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            grid: Array::from_elem((width, height), BoardValue::Empty),
            player: Player::FIRST,
            last_placement: None,
            min_x: 0,
            min_y: 0,
            max_x: width - 1,
            max_y: height - 1,
            default_bound: true,
            game_result: GameResult::NotTerminated,
            x_forced: Some(Vec::new()),
            o_forced: Some(Vec::new()),
            width,
            height,
        }
    }

    pub fn place(&mut self, m: Move) -> Result<(), PlacingError> {
        let [x, y] = m;
        if x >= self.width || y >= self.height {
            return Err(PlacingError::OutOfBounds);
        }
        if self.grid[[y, x]] != BoardValue::Empty {
            return Err(PlacingError::Occupied);
        }

        // println!("Moving {} {}", x, y);
        self.grid[[y, x]] = self.player.board_value();
        match (self.player, &self.x_forced, &self.o_forced) {
            (Player::X, Some(fa), _) if fa.is_empty() || fa.contains(&m) => {
                self.x_forced = None;
                self.o_forced = None;
            }
            (Player::O, _, Some(fa)) if fa.is_empty() || fa.contains(&m) => {
                self.x_forced = None;
                self.o_forced = None;
            }
            _ => {}
        }

        self.player.next();
        self.last_placement = Some(m);
        self.update_bounds();

        if !self.game_result.is_terminate() {
            self.game_result = GameResult::NotCalculated;
        }
        Ok(())
    }

    fn update_bounds(&mut self) {
        macro_rules! set_bounds {
            ($min_x:expr, $min_y:expr, $max_x:expr, $max_y:expr) => {
                self.min_x = $min_x;
                self.min_y = $min_y;
                self.max_x = $max_x;
                self.max_y = $max_y;
            };
            ($x:expr, $y:expr) => {
                self.min_x = usize::min(self.min_x, $x);
                self.min_y = usize::min(self.min_y, $y);
                self.max_x = usize::max(self.max_x, $x);
                self.max_y = usize::max(self.max_y, $y);
            };
        }

        if let Some([x, y]) = self.last_placement {
            if self.default_bound {
                set_bounds!(x, y, x, y);
                self.default_bound = false;
                return;
            }
            set_bounds!(x, y);
            return;
        }

        let mut all_empty = true;
        set_bounds!(usize::MAX, usize::MAX, usize::MIN, usize::MIN);
        for x in 0..self.width {
            for y in 0..self.height {
                if self.grid[[y, x]] == BoardValue::Empty {
                    continue;
                }
                all_empty = false;
                set_bounds!(x, y);
            }
        }
        if all_empty {
            set_bounds!(0, 0, self.width - 1, self.height - 1);
        } else {
            self.default_bound = false;
        }
    }

    pub fn utility(&mut self, player: Player) -> Option<Util> {
        self.calculate_game_result().utility(player)
    }

    fn calculate_game_result(&mut self) -> GameResult {
        macro_rules! set_and_return {
            ($res:expr) => {
                self.game_result = $res;
                return $res;
            };
        }

        if self.game_result != GameResult::NotCalculated {
            return self.game_result;
        }

        if self.is_board_full() {
            set_and_return!(GameResult::Draws);
        }
        if let Some([x, y]) = self.last_placement {
            if self.check_all_dir(x, y) {
                set_and_return!(GameResult::win(self.grid[[y, x]].player().unwrap()));
            }
            set_and_return!(GameResult::NotTerminated);
        }

        // may have lots of redundant checks if no last_placement
        for x in 0..self.width {
            for y in 0..self.height {
                if self.grid[[y, x]] == BoardValue::Empty {
                    continue;
                }
                if self.check_all_dir(x, y) {
                    set_and_return!(GameResult::win(self.grid[[y, x]].player().unwrap()));
                }
            }
        }
        set_and_return!(GameResult::NotTerminated);
    }

    pub fn place_random(&mut self) -> Result<(), PlacingError> {
        loop {
            let actions = self.actions();
            if actions.is_empty() {
                return Err(PlacingError::FullBoard);
            }
            let [x, y] = actions[fastrand::usize(..actions.len())];

            let mut count = 0;
            for dx in [-1, 0, 1] {
                for dy in [-1, 0, 1] {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    let sx = x.saturating_add_signed(dx);
                    let sy = y.saturating_add_signed(dy);
                    if x.checked_add_signed(dx).is_none()
                        || y.checked_add_signed(dy).is_none()
                        || sx >= self.width
                        || sy >= self.height
                    {
                        continue;
                    }
                    if self.grid[[sy, sx]] == BoardValue::Empty {
                        count += 1;
                    }
                }
            }
            if count == 8 && fastrand::f32() < 0.9 {
                continue;
            }
            self.place([x, y])?;
            return Ok(());
        }
    }
    fn is_board_full(&self) -> bool {
        self.grid.iter().all(|&v| v != BoardValue::Empty)
    }

    fn calculate_forced(&mut self) -> Vec<Move> {
        fn extend<T>(v: &mut Vec<T>, u: Vec<T>)
        where
            T: PartialEq,
        {
            for e in u {
                if !v.contains(&e) {
                    v.push(e);
                }
            }
        }

        match (self.player, &self.x_forced, &self.o_forced) {
            (Player::X, Some(fa), _) => return fa.clone(),
            (Player::O, _, Some(fa)) => return fa.clone(),
            _ => {}
        }
        let mut x_forced = Vec::new();
        let mut o_forced = Vec::new();

        if let Some([x, y]) = self.last_placement {
            for target_value in [BoardValue::X, BoardValue::O] {
                for area in self.get_areas_from_point(x, y) {
                    let (af, df) = patterns::get_forced(&area, &target_value);
                    // println!("Area {:?} {:?} {:?}", area, af, df);
                    match target_value.player().unwrap() {
                        Player::X => {
                            extend(&mut x_forced, af.clone());
                            extend(&mut o_forced, df.clone());
                        }
                        Player::O => {
                            extend(&mut x_forced, df.clone());
                            extend(&mut o_forced, af.clone());
                        }
                    }
                }
            }
            self.x_forced = Some(x_forced.clone());
            self.o_forced = Some(o_forced.clone());
            return match self.player {
                Player::X => x_forced,
                Player::O => o_forced,
            };
        }

        // lots of redundant checks if no last_placement
        // for ((y, x), target_value) in self.grid.indexed_iter() {
        //     for area in self.get_areas_from_point(x, y) {
        //         if let Some((df, af)) = patterns::get_forced(&area, &target_value) {
        //             if self.player == target_value.player().unwrap() {
        //                 af
        //             } else {
        //                 df
        //             }
        //             self.forced_actions = Some(fa.clone());
        //             return fa;
        //         }
        //     }
        // }
        self.x_forced = Some(Vec::new());
        self.o_forced = Some(Vec::new());
        Vec::new()
    }
    pub fn are_there_threats(&mut self) -> bool {
        // return false;
        self.calculate_forced();
        !self.x_forced.as_ref().unwrap().is_empty() && !self.o_forced.as_ref().unwrap().is_empty()
    }

    fn get_areas_from_point(&self, x: usize, y: usize) -> [Vec<(BoardValue, usize, usize)>; 4] {
        let x_vary = (x.saturating_sub(6))..(usize::min(x + 7, self.width));
        let y_vary = (y.saturating_sub(6))..(usize::min(y + 7, self.height));
        let x_const = iter::repeat(x);
        let y_const = iter::repeat(y);

        let board_value = |(x, y)| -> (BoardValue, usize, usize) { (self.grid[[y, x]], x, y) };

        let horz_area = iter::zip(x_vary.clone(), y_const.clone())
            .map(board_value)
            .collect();
        let vert_area = iter::zip(x_const.clone(), y_vary.clone())
            .map(board_value)
            .collect();
        let mut diag_area1 = Vec::new();
        for t in -5..=5 {
            match (x.checked_add_signed(t), y.checked_add_signed(t)) {
                (Some(rx), Some(ry)) if rx < self.width && ry < self.height => {
                    diag_area1.push(board_value((rx, ry)));
                }
                _ => continue,
            }
        }
        let mut diag_area2 = Vec::new();
        for t in -5..=5 {
            match (x.checked_add_signed(t), y.checked_add_signed(-t)) {
                (Some(rx), Some(ry)) if rx < self.width && ry < self.height => {
                    diag_area2.push(board_value((rx, ry)));
                }
                _ => continue,
            }
        }
        // let diag_area1 = iter::zip(x_vary.clone(), y_vary.clone())
        //     .map(board_value)
        //     .collect();
        // let diag_area2 = iter::zip(x_vary.clone(), y_vary.rev().clone())
        //     .map(board_value)
        //     .collect();

        [horz_area, vert_area, diag_area1, diag_area2]
    }

    pub fn actions(&mut self) -> Vec<Move> {
        let forced_actions = self.calculate_forced();
        if !forced_actions.is_empty() {
            return forced_actions;
        }
        let mut res = Vec::new();
        let left = self.min_x.saturating_sub(1);
        let up = self.min_y.saturating_sub(1);
        let right = usize::min(self.max_x + 2, self.width);
        let down = usize::min(self.max_y + 2, self.height);
        for y in up..down {
            for x in left..right {
                if self.grid[[y, x]] == BoardValue::Empty {
                    res.push([x, y]);
                }
            }
        }
        res
    }

    fn count_ray<I>(
        &self,
        x: usize,
        y: usize,
        ray: I,
        v: BoardValue,
        blocking_v: BoardValue,
    ) -> (usize, bool)
    where
        I: Iterator<Item = (isize, isize)>,
    {
        fn add_with_limit(n: usize, dn: isize, lim: usize) -> Option<usize> {
            n.checked_add_signed(dn)
                .and_then(|sum| if sum < lim { Some(sum) } else { None })
        }

        let mut counter = 0;
        let mut blocked = false;
        for (dx, dy) in ray {
            let xo = add_with_limit(x, dx, self.width);
            let yo = add_with_limit(y, dy, self.height);
            match (xo, yo) {
                (Some(xc), Some(yc)) => {
                    if self.grid[[yc, xc]] != v {
                        blocked = self.grid[[yc, xc]] == blocking_v;
                        break;
                    }
                }
                (_, _) => {
                    blocked = true;
                    break;
                }
            }
            counter += 1;
        }
        (counter, blocked)
    }

    fn check_dir<I1, I2>(
        &self,
        x: usize,
        y: usize,
        first_ray: I1,
        second_ray: I2,
        v: BoardValue,
        blocking_v: BoardValue,
    ) -> bool
    where
        I1: Iterator<Item = (isize, isize)>,
        I2: Iterator<Item = (isize, isize)>,
    {
        let (first_ray_count, first_ray_blocked) = self.count_ray(x, y, first_ray, v, blocking_v);
        let (second_ray_count, second_ray_blocked) =
            self.count_ray(x, y, second_ray, v, blocking_v);
        let count = first_ray_count + second_ray_count + 1;
        let blocked = first_ray_blocked && second_ray_blocked;
        (count == 5 && !blocked) || count > 5
    }

    fn check_all_dir(&self, x: usize, y: usize) -> bool {
        macro_rules! return_if_true {
            ($b:expr) => {
                if $b {
                    return true;
                }
            };
        }

        let v = self.grid[[y, x]];
        let blocking_v = v.opposite().expect("grid at this position is empty");

        let pos_ray = 1..=5;
        let neg_ray = (-5..=-1).rev();

        // horizontal
        let horz_first_range = iter::zip(pos_ray.clone(), iter::repeat(0));
        let horz_second_range = iter::zip(neg_ray.clone(), iter::repeat(0));
        return_if_true!(self.check_dir(x, y, horz_first_range, horz_second_range, v, blocking_v,));

        // vertical
        let vert_first_range = iter::zip(iter::repeat(0), pos_ray.clone());
        let vert_second_range = iter::zip(iter::repeat(0), neg_ray.clone());
        return_if_true!(self.check_dir(x, y, vert_first_range, vert_second_range, v, blocking_v));

        // diagonals
        let diag1_first_range = iter::zip(pos_ray.clone(), pos_ray.clone());
        let diag1_second_range = iter::zip(neg_ray.clone(), neg_ray.clone());
        return_if_true!(self.check_dir(x, y, diag1_first_range, diag1_second_range, v, blocking_v));
        let diag2_first_range = iter::zip(pos_ray.clone(), neg_ray.clone());
        let diag2_second_range = iter::zip(neg_ray.clone(), pos_ray.clone());
        return_if_true!(self.check_dir(x, y, diag2_first_range, diag2_second_range, v, blocking_v));

        false
    }

    pub fn heuristic(&self, player: Player) -> Util {
        const W1: usize = 0;
        const W2: usize = 2;
        const W3: usize = 3;
        const W4: usize = 4;

        let get_w = |s| match s {
            1 => W1,
            2 => W2,
            3 => W3,
            4 => W4,
            _ => 0,
        };

        let mut x_h = 0;
        let mut o_h = 0;
        let mut current_x_straight = 0;
        let mut current_o_straight = 0;

        let mut update = |v| match v {
            BoardValue::X => current_x_straight += 1,
            BoardValue::O => current_o_straight += 1,
            BoardValue::Empty => {
                x_h += get_w(current_x_straight);
                o_h += get_w(current_o_straight);
                current_x_straight = 0;
                current_o_straight = 0;
            }
        };
        // horizontal straights
        for y in 0..self.height {
            for x in 0..self.width {
                update(self.grid[[y, x]]);
            }
        }
        update(BoardValue::Empty);

        // vertical straights
        for x in 0..self.width {
            for y in 0..self.height {
                update(self.grid[[y, x]]);
            }
        }
        update(BoardValue::Empty);

        // diagonal 1
        for k in 0..(self.width + self.height - 1) {
            for x in 0..self.width {
                if let Some(y) = k.checked_sub(x) {
                    if y < self.height {
                        update(self.grid[[y, x]]);
                    }
                }
            }
        }
        update(BoardValue::Empty);

        // diagonal 2
        for k in 0..(self.width + self.height - 1) {
            for x in (0..self.width).rev() {
                if let Some(y) = k.checked_sub(self.width - x - 1) {
                    if y < self.height {
                        update(self.grid[[y, x]]);
                    }
                }
            }
        }
        update(BoardValue::Empty);

        // println!("{} {}", x_h, o_h);
        let x_h = x_h as f32;
        let o_h = o_h as f32;
        match player {
            Player::X => x_h / (x_h + o_h),
            Player::O => o_h / (x_h + o_h)
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // top border
        write!(f, "    ")?;
        for i in 0..self.width {
            write!(f, " {:#}  ", radix(i + 1, self.width as u8 + 1))?;
        }
        writeln!(f)?;

        writeln!(f, "   ┌{}┐", ("───┬").repeat(self.width - 1) + "───")?;

        // rows
        for y in 0..self.height {
            write!(f, " {:#} │", radix(y + 1, self.height as u8 + 1))?;
            for x in 0..self.width {
                match self.last_placement {
                    Some(m) if m == [x, y] => {
                        write!(f, " \x1b[100m{}\x1b[0m │", self.grid[[y, x]])?
                    }
                    _ => write!(f, " {} │", self.grid[[y, x]])?,
                }
            }
            writeln!(f)?;
            // bottom border of this row (top border of next row)
            if y != self.height - 1 {
                writeln!(f, "   ├{}┤", ("───┼").repeat(self.width - 1) + "───")?;
            }
        }

        // bottom border
        writeln!(f, "   └{}┘", ("───┴").repeat(self.width - 1) + "───")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Player;

    use super::Board;

    #[test]
    fn util_test() {
        let mut board = Board::new(19, 19);
        board.place([2, 2]).unwrap();
        board.place([3, 2]).unwrap();
        board.place([2, 3]).unwrap();
        board.place([4, 2]).unwrap();
        board.place([2, 4]).unwrap();
        board.place([4, 3]).unwrap();
        board.place([2, 5]).unwrap();
        board.place([5, 4]).unwrap();
        board.place([2, 6]).unwrap();
        assert_eq!(board.utility(Player::X), Some(1.0));

        let mut board = Board::new(19, 19);
        board.place([7, 2]).unwrap();
        board.place([7, 3]).unwrap();
        board.place([8, 3]).unwrap();
        board.place([8, 4]).unwrap();
        board.place([11, 6]).unwrap();
        board.place([7, 4]).unwrap();
        board.place([10, 5]).unwrap();
        board.place([6, 1]).unwrap();
        board.place([9, 4]).unwrap();
        assert_eq!(board.utility(Player::X), Some(1.0));

        let mut board = Board::new(19, 19);
        board.place([1, 6]).unwrap();
        board.place([0, 7]).unwrap();
        board.place([2, 8]).unwrap();
        board.place([1, 7]).unwrap();
        board.place([2, 6]).unwrap();
        board.place([2, 7]).unwrap();
        board.place([3, 6]).unwrap();
        board.place([3, 7]).unwrap();
        board.place([5, 7]).unwrap();
        board.place([4, 7]).unwrap();
        assert_eq!(board.utility(Player::X), None);

        let mut board = Board::new(19, 19);
        board.place([11, 10]).unwrap();
        board.place([11, 11]).unwrap();
        board.place([12, 11]).unwrap();
        board.place([12, 10]).unwrap();
        board.place([10, 12]).unwrap();
        board.place([13, 9]).unwrap();
        board.place([16, 6]).unwrap();
        board.place([14, 8]).unwrap();
        board.place([15, 8]).unwrap();
        board.place([15, 17]).unwrap();
        assert_eq!(board.utility(Player::X), None);
    }

    // #[test]
    // fn test_area() {
    //     let board = Board::new(19, 19);
    //     println!("{:?}", board.get_areas_from_point(10, 10));
    // }
}
