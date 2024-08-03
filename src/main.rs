use std::{str::FromStr, time::Instant};

use mess::chess::{Board, Move, MoveFlag, Square};

fn main() {
    let mut board =
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    board.make_move(Move::new(Square::F2, Square::F3, MoveFlag::Normal));
    board.make_move(Move::new(Square::E7, Square::E5, MoveFlag::Normal));
    board.make_move(Move::new(Square::G2, Square::G4, MoveFlag::Normal));
    board.make_move(Move::new(Square::D8, Square::H4, MoveFlag::Normal));

    println!("\n{board}");

    // let start = Instant::now();
    // let nodes = perft::<true, true>(&mut board, 6);
    // let duration = start.elapsed().as_secs_f64();
    // println!(
    //     "\nnodes {} nps {} mnps",
    //     nodes,
    //     (nodes as f64 / duration) as u64 / 1_000_000
    // );
}

fn perft<const BULK_COUNT: bool, const SPLIT_MOVES: bool>(board: &mut Board, depth: i32) -> usize {
    // Return 1 for current node at depth 0.
    if depth <= 0 {
        return 1;
    }

    // When bulk counting is enabled, return the length of
    // the legal move-list when depth is one. This saves a
    // lot of time cause it saves make moves and recursion.
    if BULK_COUNT && depth == 1 {
        return board.generate_legal_moves().len();
    }

    // Generate legal move-list.
    let moves = board.generate_legal_moves();

    // Variable to cumulate node count in.
    let mut nodes: usize = 0;

    // Recursively call perft for child nodes.
    for chessmove in moves {
        board.make_move(chessmove);
        let new_nodes = perft::<BULK_COUNT, false>(board, depth - 1);
        board.undo_move();

        nodes += new_nodes;

        // If split moves is enabled, display each child move's
        // contribution to the node count separately.
        if SPLIT_MOVES {
            println!("{chessmove}: {new_nodes}");
        }
    }

    // Return cumulative node count.
    nodes
}
