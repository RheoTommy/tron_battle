use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

fn main() {
    let mut v = vec![-1; 100];
    v[4 * 10 + 4] = 0;
    v[6 * 10 + 6] = 1;
    let req = Request {
        size: Point { x: 10, y: 10 },
        player_pos: Point { x: 4, y: 4 },
        ai_pos: Point { x: 6, y: 6 },
        board: v,
    };
    let s = serde_json::to_string(&req).unwrap();
    println!("{}", s);
    println!("{:?}", s.as_bytes());

    let listener = TcpListener::bind("localhost:6583").unwrap();
    println!("Listening on localhost:6583");

    for stream in listener.incoming() {
        match stream {
            Err(e) => eprintln!("{:?}", e),
            Ok(stream) => {
                thread::spawn(move || handler(stream).unwrap_or_else(|e| eprintln!("{:?}", e)));
                stderr().flush().unwrap();
            }
        }
    }
}

fn handler(mut stream: TcpStream) -> Result<(), String> {
    println!("Accessed!");
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|e| e.to_string())?;
    let mut buf = [0u8; 4096];
    stream.read(&mut buf).map_err(|e| e.to_string())?;
    let s = String::from_utf8(buf.to_vec().into_iter().take_while(|k| *k != 0).collect())
        .map_err(|e| e.to_string())?;
    let s = s.trim().lines().nth(5).unwrap().trim();
    let req: Request = serde_json::from_str(&s).unwrap();

    let state = Board::from_request(req);
    let op = state.min_max(10);
    let res = serde_json::to_string(&op)
        .map_err(|e| e.to_string())
        .unwrap();
    stream.write(res.as_bytes()).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

#[derive(Clone, Serialize, Deserialize)]
struct Request {
    size: Point,
    player_pos: Point,
    ai_pos: Point,
    board: Vec<isize>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Point {
    x: usize,
    y: usize,
}

fn cui() {
    let mut rng = ThreadRng::default();
    let h = rng.gen_range(7..=15);
    let w = rng.gen_range(7..=15);
    let me = (rng.gen_range(1..h - 1), rng.gen_range(1..w - 1));
    let opp = (rng.gen_range(1..h - 1), rng.gen_range(1..w - 1));

    if me == opp {
        println!("やばい！");
        return;
    }

    let mut board = Board::new(h, w, me, opp, 0);

    println!("{}", board);

    while !board.enumerate().is_empty() {
        let player_turn = board.n == 0;
        let op = if player_turn {
            decide_by_input()
        } else {
            board.min_max(15)
        };

        let res = board.operate(op);
        if res.is_err() && player_turn {
            println!("Invalid Input!!!");
            continue;
        }

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
        _ => {
            println!("Invalid Input!");
            decide_by_input()
        }
    }
}

use std::io::{stderr, BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{
    collections::VecDeque,
    fmt::Display,
    io::{stdin, stdout, Write},
    thread,
};

use rayon::prelude::*;
use Operation::*;
use Piece::*;

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

fn prev_op(pos: Position, op: Operation, h: usize, w: usize) -> Option<Position> {
    let op = match op {
        Up => Down,
        Down => Up,
        Right => Left,
        Left => Right,
    };
    next_op(pos, op, h, w)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Empty,
    Player(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    log: Vec<Operation>,
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
                    s.push('M');
                } else if i == self.opp.0 && j == self.opp.1 {
                    s.push('E');
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
            log: vec![],
            me,
            opp,
            n,
        }
    }

    fn from_request(req: Request) -> Self {
        let Request {
            size: Point { x: h, y: w },
            player_pos: Point { x: i, y: j },
            ai_pos: Point { x, y },
            board,
        } = req;
        let mut v = vec![vec![Empty; w]; h];
        for i in 0..h {
            for j in 0..w {
                let piece = match board[i * w + j] {
                    -1 => Empty,
                    0 => Player(0),
                    1 => Player(1),
                    _ => unreachable!(),
                };
                v[i][j] = piece;
            }
        }
        Self {
            h,
            w,
            v,
            log: vec![],
            me: (x, y),
            opp: (i, j),
            n: 1,
        }
    }

    fn enumerate(&self) -> Vec<Operation> {
        let mut v = vec![Up, Down, Right, Left];
        if self.log.len() >= 2 {
            for i in 0..4 {
                if v[i] == self.log[self.log.len() - 2] {
                    v.swap(0, i);
                }
            }
        }
        v.into_iter()
            .filter(|&op| {
                next_op(self.me, op, self.h, self.w)
                    .map(|(i, j)| self.v[i][j] == Empty)
                    .unwrap_or(false)
            })
            .collect()
    }

    fn operate(&mut self, op: Operation) -> Result<(), &str> {
        next_op(self.me, op, self.h, self.w)
            .ok_or("Don't sink into the wall!")
            .and_then(|(i, j)| {
                if self.v[i][j] != Empty {
                    Err("Not Empty!")
                } else {
                    let me = self.opp;
                    self.v[i][j] = Player(self.n);
                    self.n ^= 1;
                    self.opp = (i, j);
                    self.me = me;
                    self.log.push(op);
                    Ok(())
                }
            })
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

    fn rollback(&mut self) -> Result<(), &str> {
        self.log
            .pop()
            .ok_or("No log!")
            .and_then(|op| {
                prev_op(self.opp, op, self.h, self.w).ok_or("Invalid operation in prev_op!")
            })
            .and_then(|(i, j)| {
                let (x, y) = self.opp;
                self.v[x][y] = Empty;
                let opp = self.me;
                let me = (i, j);
                self.opp = opp;
                self.me = me;
                self.n ^= 1;
                Ok(())
            })
    }

    fn min_max_sub(&mut self, depth: usize, mut alpha: isize, beta: isize) -> isize {
        let v = self.enumerate();

        if depth == 0 || v.is_empty() {
            return self.calc_score();
        }

        for op in v {
            self.operate(op).unwrap();
            let score = -self.min_max_sub(depth - 1, -beta, -alpha);
            self.rollback().unwrap();
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
