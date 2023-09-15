mod eval;
mod timeman;

use std::time::Instant;

use crate::{
    movegen::movegen,
    position::{Move, Position},
    SearchOptions,
};

use self::{eval::eval, timeman::TimeManager};

struct SearchContext {
    timeman: TimeManager,
    nodes: u64,
    repetitions: Vec<u64>,
}

pub fn search(pos: &Position, options: SearchOptions, repetitions: Vec<u64>) -> Move {
    let instant = Instant::now();

    let mut context = SearchContext {
        timeman: TimeManager::new(&options, pos.side),
        nodes: 0,
        repetitions,
    };

    let mut best = Move::NULL;

    for depth in 1..=10 {
        if let Some((m, eval)) = context.negamax(pos, -INF, INF, 0, depth) {
            let elapsed = instant.elapsed();

            let nodes = context.nodes;
            let nps = (nodes as f32 / elapsed.as_secs_f32()) as u64;
            let elapsed = elapsed.as_millis();

            println!(
                "info depth {depth} score cp {eval} nodes {nodes} nps {nps} time {elapsed} pv {m}"
            );

            best = m;
        } else {
            break;
        }
    }

    best
}

const INF: i32 = 1000000;
const CHECKMATE: i32 = 100000;

impl SearchContext {
    fn is_draw(&self, pos: &Position) -> bool {
        if pos.halfmove >= 100 {
            return true;
        }

        if pos.halfmove < 4 || self.repetitions.len() < 4 {
            return false;
        }

        let mut d = self.repetitions.len() - 1;
        let mut n = 0;

        while d > 0 {
            d -= 1;
            if self.repetitions[self.repetitions.len() - 1] == self.repetitions[d] {
                n += 1;
                if n == 2 {
                    return true;
                }
            }
        }

        false
    }

    fn negamax(
        &mut self,
        pos: &Position,
        alpha: i32,
        beta: i32,
        plies: u8,
        depth: u8,
    ) -> Option<(Move, i32)> {
        if depth == 0 {
            return Some((Move::NULL, eval(pos)));
        }

        let in_check = pos.in_check(pos.side);
        let mut n_moves = 0;

        let mut best = (Move::NULL, -INF);

        for m in &movegen(pos) {
            let pos = pos.make_move(*m);

            if !pos.in_check(!pos.side) {
                n_moves += 1;

                self.nodes += 1;
                if self.nodes % 2048 == 0 && self.timeman.stop() && !(plies == 0 && depth == 1) {
                    return None;
                }

                self.repetitions.push(pos.hash());
                let res = if self.is_draw(&pos) {
                    (Move::NULL, 0)
                } else {
                    self.negamax(&pos, -beta, -alpha, plies + 1, depth - 1)?
                };
                self.repetitions.pop();

                if -res.1 > best.1 {
                    best = (*m, -res.1);
                    if res.1 >= beta {
                        return Some((*m, -res.1));
                    }
                }
            }
        }

        if n_moves == 0 {
            if in_check {
                return Some((Move::NULL, -CHECKMATE + plies as i32));
            } else {
                return Some((Move::NULL, 0));
            }
        }

        Some(best)
    }
}
