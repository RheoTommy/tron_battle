fn main() {
    let h = 10;
    let w = 10;
    let me = (4, 4);
    let opp = (6, 6);

    let mut board = Board::new(h, w, me, opp, 0);
    println!("{}", board);

    while !board.enumerate().is_empty() {
        let op = if board.n == 0 {
            decide_by_input()
        } else {
            board.min_max(12)
        };

        board.operate(op).unwrap();

        println!("{}", board);
    }

    println!("lose: {}", board.n);
}

fn decide_by_input() -> Operation {
    print!("input: ");
    stdout().flush().unwrap();

    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    println!();
    match s.chars().next().unwrap() {
        'w' => Up,
        's' => Down,
        'd' => Right,
        'a' => Left,
        _ => unreachable!(),
    }
}

use std::{
    collections::VecDeque,
    fmt::Display,
    io::{stdin, stdout, Write},
    thread,
    time::Duration,
};

use Operation::*;
use Piece::*;
use rayon::prelude::*;

const INF: usize = 1001001001001001001;

type Position = (usize, usize);

fn next_pos(pos: Position, h: usize, w: usize) -> Vec<Position> {
    let (i, j) = (pos.0 as isize, pos.1 as isize);
    vec![(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)]
        .into_iter()
        .filter(|&(x, y)| 0 <= x && x < h as isize && 0 <= y && y < w as isize)
        .map(|(x, y)| (x as usize, y as usize))
        .collect()
}

fn next_op(pos: Position, op: Operation, h: usize, w: usize) -> Option<Position> {
    let (mut i, mut j) = (pos.0 as isize, pos.1 as isize);

    match op {
        Up => i -= 1,
        Down => i += 1,
        Right => j += 1,
        Left => j -= 1,
    }

    if 0 <= i && i < h as isize && 0 <= j && j < w as isize {
        Some((i as usize, j as usize))
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Empty,
    Player(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Up,
    Down,
    Right,
    Left,
}

#[derive(Clone)]
struct Board {
    h: usize,
    w: usize,
    v: Vec<Vec<Piece>>,
    me: Position,
    opp: Position,
    n: usize,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for i in 0..self.h {
            for j in 0..self.w {
                if i == self.me.0 && j == self.me.1 {
                    s.push('*');
                } else if i == self.opp.0 && j == self.opp.1 {
                    s.push('+');
                } else {
                    match self.v[i][j] {
                        Empty => s.push('.'),
                        Player(k) => s.push_str(&k.to_string()),
                    }
                }
            }
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

impl Board {
    fn new(h: usize, w: usize, me: Position, opp: Position, n: usize) -> Self {
        let mut v = vec![vec![Empty; w]; h];
        v[me.0][me.1] = Player(n);
        v[opp.0][opp.1] = Player(n ^ 1);
        Self {
            h,
            w,
            v,
            me,
            opp,
            n,
        }
    }

    fn enumerate(&self) -> Vec<Operation> {
        vec![Up, Down, Right, Left]
            .into_iter()
            .filter(|&op| {
                let p = next_op(self.me, op, self.h, self.w);
                if p.is_none() {
                    return false;
                }
                let (i, j) = p.unwrap();
                self.v[i][j] == Empty
            })
            .collect()
    }

    fn operate(&mut self, op: Operation) -> Result<(), &str> {
        let opp = next_op(self.me, op, self.h, self.w);
        if self.v[opp.0][opp.1] != Empty || opp.is_none() {
            return Err("Not Empty!");
        }
        let opp = opp.unwrap();
        let me = self.opp;
        self.v[opp.0][opp.1] = Player(self.n);
        self.n ^= 1;
        self.opp = opp;
        self.me = me;
        Ok(())
    }

    fn calc_score(&self) -> isize {
        if self.enumerate().is_empty() {
            return -(INF as isize);
        }

        let mut bfs = vec![vec![INF; self.w]; self.h];
        bfs[self.me.0][self.me.1] = self.n;
        bfs[self.opp.0][self.opp.1] = self.n ^ 1;

        let mut queue = VecDeque::new();
        queue.push_back((self.me, self.n));
        queue.push_back((self.opp, self.n ^ 1));

        let mut me_cnt = 1;
        let mut opp_cnt = 1;

        while let Some((now, p)) = queue.pop_front() {
            for (i, j) in next_pos(now, self.h, self.w) {
                if bfs[i][j] != INF || self.v[i][j] != Empty {
                    continue;
                }

                if p == self.n {
                    me_cnt += 1;
                } else {
                    opp_cnt += 1;
                }

                bfs[i][j] = p;
                queue.push_back(((i, j), p));
            }
        }

        me_cnt - opp_cnt
    }

    fn min_max_sub(&self, depth: usize, mut alpha: isize, beta: isize) -> isize {
        let v = self.enumerate();

        if depth == 0 || v.is_empty() {
            return self.calc_score();
        }

        for op in v {
            let mut state = self.clone();
            state.operate(op).unwrap();
            let score = -state.min_max_sub(depth - 1, -beta, -alpha);
            alpha = alpha.max(score);
            if alpha > beta {
                return alpha;
            }
        }

        alpha
    }

    fn min_max(&self, depth: usize) -> Operation {
        self.enumerate()
            .into_par_iter()
            .max_by_key(|op| {
                let mut state = self.clone();
                state.operate(*op).unwrap();
                -state.min_max_sub(depth - 1, -(INF as isize), INF as isize)
            })
            .unwrap()
    }
}