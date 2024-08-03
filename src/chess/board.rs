// Copyright © 2023 Rak Laptudirm <rak@laptudirm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt, str::FromStr};

use crate::chess::{zobrist, BitBoard, Color, ColoredPiece, File, Move, MoveFlag, Piece, Square};

use super::{castling, moves, Mailbox, Rank, FEN};

use colored::Colorize;

pub struct Board {
    // 8x8 mailbox board representation for
    // fast piece square lookup.
    mailbox: Mailbox,

    // BitBoard board representation.
    color_bbs: [BitBoard; Color::N],
    piece_bbs: [BitBoard; Piece::N],
    friends: BitBoard,
    enemies: BitBoard,
    occupied: BitBoard,

    // Checker info.
    pub checkers: BitBoard,
    pub check_nm: u32,

    // Position metadata.
    side_to_mv: Color,
    pub plys_count: u16,
    draw_clock: u8,
    enp_target: Square,

    // Game metadata.
    is_fischer_random: bool,
    castling_square_info: castling::Info,

    hash: zobrist::Hash,

    pub history: [BoardState; 1024],

    // Move generation specific info.
    pub check_mask: BitBoard,
    pin_mask_l: BitBoard,
    pub pin_mask_d: BitBoard,
    targets: BitBoard,
    threats: BitBoard,
    move_list: Vec<Move>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const THEME: (&str, &str, &str) = (
            "bright magenta", // White squares.
            "magenta",        // Black squares.
            "bright green",   // Move markers.
        );

        let board = self;
        let mut string_rep = String::from(" ");

        let last_move = if board.plys_count >= 1 {
            board.history[board.plys_count as usize - 1].played_move
        } else {
            Move::NULL
        };

        for (square, piece) in board.mailbox.0.into_iter().enumerate() {
            let square = Square::from(square);

            let square_rep = match piece.piece() {
                Piece::Pawn => "P ",
                Piece::Knight => "N ",
                Piece::Bishop => "B ",
                Piece::Rook => "R ",
                Piece::Queen => "Q ",
                Piece::King => "K ",

                Piece::None => "  ",
            }
            .to_string();

            let piece_color = match piece.color() {
                Color::White => "bright white",
                Color::Black => "black",
                _ => "white",
            };

            let mut square_color = match square.color() {
                Color::White => THEME.0,
                Color::Black => THEME.1,
                _ => panic!("display board: illegal state"),
            };

            if !board.checkers.is_empty()
                && piece == ColoredPiece::new(Piece::King, board.side_to_mv)
            {
                square_color = "red";
            } else if last_move != Move::NULL
                && (last_move.source() == square || last_move.target() == square)
            {
                square_color = THEME.2;
            }

            string_rep += &format!("{}", square_rep.color(piece_color).on_color(square_color));

            if square.file() == File::H {
                string_rep += &format!(" {} \n ", square.rank());
            }
        }

        string_rep += " a  b  c  d  e  f  g  h\n";

        let mut checkers = "".to_string();
        for checker in board.checkers {
            checkers += &format!("{} ", checker);
        }

        write!(
            f,
            "{}\nfen: {}\nkey: {}\ncheckers: {}\n",
            string_rep,
            FEN::from(board),
            board.hash,
            checkers
        )
    }
}

#[derive(Clone, Copy, Default)]
pub struct BoardState {
    pub played_move: Move,
    captured_piece: ColoredPiece,

    castling_r: castling::Rights,
    enp_target: Square,
    draw_clock: u8,

    hash: zobrist::Hash,
}

impl FromStr for Board {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match FEN::from_str(s) {
            Ok(fen) => Ok(Board::from(fen)),
            Err(_) => Err(()),
        }
    }
}

impl From<FEN> for Board {
    fn from(fen: FEN) -> Self {
        let mut board = Board {
            mailbox: fen.position,

            piece_bbs: [BitBoard::EMPTY; Piece::N],
            color_bbs: [BitBoard::EMPTY; Color::N],
            friends: BitBoard::EMPTY,
            enemies: BitBoard::EMPTY,
            occupied: BitBoard::EMPTY,

            checkers: BitBoard::EMPTY,
            check_nm: 0,

            side_to_mv: fen.side_to_move,
            plys_count: (fen.full_move_count - 1) * 2 + fen.side_to_move as u16,
            draw_clock: fen.half_move_clock,
            enp_target: fen.en_pass_square,

            is_fischer_random: false,
            hash: zobrist::castling_rights_key(fen.castling_rights),
            castling_square_info: castling::Info::from_squares(
                Square::E1,
                File::H,
                File::A,
                Square::E8,
                File::H,
                File::A,
            ),

            history: [BoardState::default(); 1024],

            check_mask: BitBoard::EMPTY,
            pin_mask_l: BitBoard::EMPTY,
            pin_mask_d: BitBoard::EMPTY,

            targets: BitBoard::EMPTY,
            threats: BitBoard::EMPTY,

            move_list: Vec::new(),
        };

        for (square, piece) in board.mailbox.0.iter().enumerate() {
            let piece = *piece;

            if piece == ColoredPiece::None {
                continue;
            }

            let square = Square::from(square);

            board.piece_bbs[piece.piece() as usize].insert(square);
            board.color_bbs[piece.color() as usize].insert(square);

            board.hash ^= zobrist::piece_square_key(piece, square);
        }

        if board.side_to_mv == Color::Black {
            board.hash ^= zobrist::side_to_move_key();
        }

        if board.enp_target != Square::None {
            board.hash ^= zobrist::en_passant_key(board.enp_target);
        }

        board.friends = board.color_bb(board.side_to_mv);
        board.enemies = board.color_bb(!board.side_to_mv);
        board.occupied = board.friends | board.enemies;

        board.generate_check_masks();

        board
    }
}

impl Board {
    pub fn mailbox(&self) -> Mailbox {
        self.mailbox
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_mv
    }

    pub fn en_passant_target(&self) -> Square {
        self.enp_target
    }

    pub fn plys(&self) -> u16 {
        self.plys_count
    }

    pub fn draw_clock(&self) -> u8 {
        self.draw_clock
    }

    #[inline(always)]
    pub fn colored_piece_bb(&self, piece: ColoredPiece) -> BitBoard {
        self.piece_color_bb(piece.piece(), piece.color())
    }

    #[inline(always)]
    pub fn piece_color_bb(&self, piece: Piece, color: Color) -> BitBoard {
        self.piece_bb(piece) & self.color_bb(color)
    }

    #[inline(always)]
    pub fn piece_bb(&self, piece: Piece) -> BitBoard {
        self.piece_bbs[piece as usize]
    }

    #[inline(always)]
    pub fn color_bb(&self, color: Color) -> BitBoard {
        self.color_bbs[color as usize]
    }

    pub fn const_color_bb<const color: Color>(&self) -> BitBoard {
        self.color_bbs[color as usize]
    }

    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.occupied
    }

    #[inline(always)]
    pub fn is_fischer_random(&self) -> bool {
        self.is_fischer_random
    }
}

impl Board {
    #[inline(always)]
    pub fn piece_at(&self, at: Square) -> ColoredPiece {
        self.mailbox.0[at as usize]
    }

    #[inline(always)]
    pub fn insert_piece(&mut self, square: Square, piece: ColoredPiece) {
        self.mailbox.0[square as usize] = piece;

        self.piece_bbs[piece.piece() as usize].insert(square);
        self.color_bbs[piece.color() as usize].insert(square);

        self.hash ^= zobrist::piece_square_key(piece, square);
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, square: Square) {
        let piece: ColoredPiece = self.mailbox.0[square as usize];
        self.mailbox.0[square as usize] = ColoredPiece::None;

        self.piece_bbs[piece.piece() as usize].remove(square);
        self.color_bbs[piece.color() as usize].remove(square);

        self.hash ^= zobrist::piece_square_key(piece, square);
    }

    #[inline(always)]
    pub fn is_check(&self) -> bool {
        !self.checkers.is_empty()
    }
}

/// Functions for various different terminal checks.
impl Board {
    #[inline(always)]
    pub fn is_mated(&mut self) -> bool {
        self.is_check() && self.generate_legal_moves().is_empty()
    }

    #[inline(always)]
    pub fn is_draw(&mut self) -> bool {
        self.is_50_move_draw()
    }

    #[inline(always)]
    pub fn is_50_move_draw(&mut self) -> bool {
        self.draw_clock >= 100
            && (self.checkers.is_empty() || !self.generate_legal_moves().is_empty())
    }
}

impl Board {
    pub fn make_move(&mut self, chessmove: Move) {
        let board = self;

        let source = chessmove.source();
        let target = chessmove.target();

        let flag = chessmove.flags();

        let source_piece = board.piece_at(source);
        let target_piece = board.piece_at(target);

        let is_capture = target_piece != ColoredPiece::None;

        if board.history[board.plys_count as usize].hash != board.hash {
            board.history[board.plys_count as usize] = BoardState {
                played_move: chessmove,
                captured_piece: target_piece,

                castling_r: board.castling_square_info.rights,
                enp_target: board.enp_target,
                draw_clock: board.draw_clock,
                hash: board.hash,
            };
        } else {
            board.history[board.plys_count as usize].played_move = chessmove;
            board.history[board.plys_count as usize].captured_piece = target_piece;
        }

        board.remove_piece(source); // Remove the moving piece.

        // Update draw clock. Reset it on an irreversible move.
        board.draw_clock = if is_capture || source_piece.is(Piece::Pawn) {
            0
        } else {
            board.draw_clock + 1
        };

        // Reset en passant square, if any.
        if board.enp_target != Square::None {
            board.hash ^= zobrist::en_passant_key(board.enp_target);
            board.enp_target = Square::None;
        }

        // Do castling rights updates, if any.
        board.castling_square_info.rights =
            board.castling_square_info.rights - board.castling_square_info.get_updates(source);
        board.castling_square_info.rights =
            board.castling_square_info.rights - board.castling_square_info.get_updates(target);

        // Remove the captured piece, if any.
        if is_capture {
            board.remove_piece(target);
        }

        match flag {
            MoveFlag::Promotion => {
                let promotion = ColoredPiece::new(chessmove.promot(), board.side_to_mv);
                board.insert_piece(target, promotion);
            }

            MoveFlag::Castle => {
                let (king_target, rook_target) =
                    castling::SideColor::from_sqs(source, target).get_targets();

                // The king has already been removed from it's source square by the
                // general move source square clearing done before, and the rook has
                // already been removed by the captured piece clearing done before.

                board.insert_piece(king_target, source_piece); // Insert King.
                board.insert_piece(rook_target, target_piece); // Insert Rook.
            }

            MoveFlag::EnPassant => {
                // Make the en passant capture.
                board.remove_piece(target.down(board.side_to_mv));
                board.insert_piece(target, source_piece);
            }

            MoveFlag::Normal => {
                // Move the piece to the target square.
                board.insert_piece(target, source_piece);

                // Update en passant target on a double pawn push.
                if source_piece.is(Piece::Pawn) {
                    // Calculate the en passant capture square.
                    let ep_target = target.down(board.side_to_mv);

                    if target.distance(source) == 2
                    // Only set the en passant square if the pawn can be captured
                    // by en passant. This increases the number of tt hits we get.
                    && !moves::pawn_attacks(ep_target, board.side_to_mv)
                    .is_disjoint(board.piece_color_bb(Piece::Pawn, !board.side_to_mv))
                    {
                        // The en passant target square is below
                        // the pawn's square after the double push.
                        board.enp_target = ep_target;
                        board.hash ^= zobrist::en_passant_key(board.enp_target);
                    }
                }
            }
        }

        board.plys_count += 1;
        board.side_to_mv = !board.side_to_mv;
        board.hash ^= zobrist::side_to_move_key();

        board.friends = board.color_bb(board.side_to_mv);
        board.enemies = board.color_bb(!board.side_to_mv);
        board.occupied = board.friends | board.enemies;

        board.generate_check_masks();
    }

    pub fn undo_move(&mut self) {
        let board = self;

        let previous_state = board.history[(board.plys_count - 1) as usize];

        let chessmove = previous_state.played_move;

        let source = chessmove.source();
        let target = chessmove.target();

        let flag = chessmove.flags();

        let target_piece = board.piece_at(target);
        let mut source_piece = target_piece;

        // Switch side.
        board.plys_count -= 1;
        board.side_to_mv = !board.side_to_mv;

        match flag {
            MoveFlag::Castle => {
                let (king_target, rook_target) =
                    castling::SideColor::from_sqs(source, target).get_targets();

                source_piece = board.piece_at(king_target);

                board.remove_piece(king_target);
                board.remove_piece(rook_target);
            }

            MoveFlag::EnPassant => {
                board.remove_piece(target);

                // Put back the pawn captured by en passant.
                board.insert_piece(
                    target.down(board.side_to_mv),
                    ColoredPiece::new(Piece::Pawn, !board.side_to_mv),
                )
            }

            MoveFlag::Promotion => {
                source_piece = ColoredPiece::new(Piece::Pawn, target_piece.color());
                board.remove_piece(target);
            }

            MoveFlag::Normal => board.remove_piece(target),
        }

        // Replace any captured piece.
        if previous_state.captured_piece != ColoredPiece::None {
            board.insert_piece(target, previous_state.captured_piece);
        }

        // Undo the piece's move.
        board.insert_piece(source, source_piece);

        // Reset irreversible info from previous state.
        board.enp_target = previous_state.enp_target;
        board.castling_square_info.rights = previous_state.castling_r;
        board.draw_clock = previous_state.draw_clock;

        // Zobrist hash is reversible, but it is easier to reset.
        board.hash = previous_state.hash;

        board.friends = board.color_bb(board.side_to_mv);
        board.enemies = board.color_bb(!board.side_to_mv);
        board.occupied = board.friends | board.enemies;

        board.generate_check_masks();
    }
}

impl Board {
    fn generate_check_masks(&mut self) {
        let board = self;

        // Get our king's bitboard.
        let king = (board.piece_bb(Piece::King) & board.friends).lsb();

        // Exclude king from blocker masks to allow x-raying.
        let blockers = board.occupied() & !BitBoard::from(king);

        // Get opponent's piece bitboards.
        let p = board.piece_bb(Piece::Pawn) & board.enemies;
        let n = board.piece_bb(Piece::Knight) & board.enemies;
        let b = board.piece_bb(Piece::Bishop) & board.enemies;
        let r = board.piece_bb(Piece::Rook) & board.enemies;
        let q = board.piece_bb(Piece::Queen) & board.enemies;

        // Get opponent's checking pieces.
        let checking_p = p & moves::pawn_attacks(king, board.side_to_mv);
        let checking_n = n & moves::knight(king);
        let checking_b = (b | q) & moves::bishop(king, blockers);
        let checking_r = (r | q) & moves::rook(king, blockers);

        board.checkers = checking_p | checking_n | checking_b | checking_r;
        board.check_nm = board.checkers.popcnt();

        match board.check_nm {
            2 => board.check_mask = BitBoard::EMPTY,
            0 => board.check_mask = BitBoard::UNIVERSE,
            _ => {
                // Include non-sliding checkers in the mask.
                board.check_mask = checking_p | checking_n;
                // For sliding pieces, also include the attack ray in the mask.
                board.check_mask |= checking_b | BitBoard::between(king, checking_b.lsb());
                board.check_mask |= checking_r | BitBoard::between(king, checking_r.lsb());
            }
        }
    }

    fn generate_pin_masks(&mut self) {
        let board = self;

        // Get our king's bitboard.
        let king = (board.piece_bb(Piece::King) & board.friends).lsb();

        // Get opponent's sliding pieces bitboards.
        let b = board.piece_bb(Piece::Bishop) & board.enemies;
        let r = board.piece_bb(Piece::Rook) & board.enemies;
        let q = board.piece_bb(Piece::Queen) & board.enemies;

        // Get possible pinning sliding pieces.
        let pinning_l = (r | q) & moves::rook(king, board.enemies);
        let pinning_d = (b | q) & moves::bishop(king, board.enemies);

        board.pin_mask_l = BitBoard::EMPTY;
        for rook in pinning_l {
            // Possible pinning path.
            let possible_pin = BitBoard::between(king, rook);

            // The rook is pinning if there is only one friendly
            // piece between it and the king.
            if (board.friends & possible_pin).popcnt() == 1 {
                // Add the rook and the ray to the mask.
                board.pin_mask_l |= possible_pin | BitBoard::from(rook);
            }
        }

        board.pin_mask_d = BitBoard::EMPTY;
        for bishop in pinning_d {
            // Possible pinning path.
            let possible_pin = BitBoard::between(king, bishop);

            // The bishop is pinning if there is only one friendly
            // piece between it and the king.
            if (board.friends & possible_pin).popcnt() == 1 {
                // Add the bishop and the ray to the mask.
                board.pin_mask_d |= possible_pin | BitBoard::from(bishop);
            }
        }
    }

    fn generate_threats(&mut self) {
        let board = self;
        let xtm = !board.side_to_mv;

        board.threats = BitBoard::EMPTY;

        let pawns = board.piece_color_bb(Piece::Pawn, xtm);
        for pawn in pawns {
            board.threats |= moves::pawn_attacks(pawn, xtm);
        }

        let knights = board.piece_color_bb(Piece::Knight, xtm);
        for knight in knights {
            board.threats |= moves::knight(knight);
        }

        // Exclude king from blocker masks to allow x-raying.
        let blockers = board.occupied() ^ (board.piece_bb(Piece::King) & board.friends);

        let bishops = board.piece_color_bb(Piece::Bishop, xtm);
        for bishop in bishops {
            board.threats |= moves::bishop(bishop, blockers);
        }

        let rooks = board.piece_color_bb(Piece::Rook, xtm);
        for rook in rooks {
            board.threats |= moves::rook(rook, blockers);
        }

        let queens = board.piece_color_bb(Piece::Queen, xtm);
        for queen in queens {
            board.threats |= moves::queen(queen, blockers);
        }

        board.threats |= moves::king(board.piece_color_bb(Piece::King, xtm).lsb())
    }
}

// Implementation of the Board's legal move generation.
impl Board {
    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        self.generate_moves::<true, true>()
    }

    pub fn generate_quiet_moves(&mut self) -> Vec<Move> {
        self.generate_moves::<true, false>()
    }

    pub fn generate_noisy_moves(&mut self) -> Vec<Move> {
        self.generate_moves::<false, true>()
    }

    #[inline(always)]
    fn generate_moves<const GEN_QUIET: bool, const GEN_NOISY: bool>(&mut self) -> Vec<Move> {
        let board = self;

        // Clear the move-list, but reuse it's memory.
        board.move_list.truncate(0);

        // Generate move generation bitboards.
        board.generate_threats();
        board.generate_pin_masks();

        board.targets = BitBoard::EMPTY;
        if GEN_QUIET {
            board.targets = !board.occupied
        }
        if GEN_NOISY {
            board.targets |= board.enemies
        }

        // King moves can always be legal.
        board.generate_king_moves();

        // If the king is in double check, only
        // king moves can possibly be legal.
        if board.check_nm < 2 {
            board.generate_pawn_moves::<GEN_QUIET, GEN_NOISY>();

            board.generate_knight_moves();
            board.generate_bishop_moves();
            board.generate_rook_moves();

            if GEN_QUIET {
                board.generate_castling_moves()
            }
        }

        board.move_list.clone()
    }
}

impl Board {
    #[inline(always)]
    fn generate_pawn_moves<const GEN_QUIET: bool, const GEN_NOISY: bool>(&mut self) {
        let pawns = self.piece_color_bb(Piece::Pawn, self.side_to_mv) - self.pin_mask_d;

        let pinned = pawns & self.pin_mask_l;
        let unpinned = pawns ^ pinned;

        let pinned_pushed = pinned.up(self.side_to_mv) & self.pin_mask_l;
        let unpinned_pushed = unpinned.up(self.side_to_mv);

        self.serialize_pawn_push::<GEN_QUIET, GEN_NOISY>(pinned_pushed + unpinned_pushed);
    }

    #[inline(always)]
    fn generate_knight_moves(&mut self) {
        let knights = self.piece_color_bb(Piece::Knight, self.side_to_mv)
            - (self.pin_mask_l | self.pin_mask_d);

        for knight in knights {
            self.serialize_moves(knight, moves::knight(knight));
        }
    }

    #[inline(always)]
    fn generate_bishop_moves(&mut self) {
        let bishops = (self.piece_color_bb(Piece::Bishop, self.side_to_mv)
            | self.piece_color_bb(Piece::Queen, self.side_to_mv))
            - self.pin_mask_l;

        let pinned = bishops & self.pin_mask_d;
        let unpinned = bishops ^ pinned;

        for bishop in pinned {
            self.serialize_moves(
                bishop,
                moves::bishop(bishop, self.occupied()) & self.pin_mask_d,
            );
        }

        for bishop in unpinned {
            self.serialize_moves(bishop, moves::bishop(bishop, self.occupied()));
        }
    }

    #[inline(always)]
    fn generate_rook_moves(&mut self) {
        let rooks = (self.piece_color_bb(Piece::Rook, self.side_to_mv)
            | self.piece_color_bb(Piece::Queen, self.side_to_mv))
            - self.pin_mask_d;

        let pinned = rooks & self.pin_mask_l;
        let unpinned = rooks ^ pinned;

        for rook in pinned {
            self.serialize_moves(rook, moves::rook(rook, self.occupied()) & self.pin_mask_l);
        }

        for rook in unpinned {
            self.serialize_moves(rook, moves::rook(rook, self.occupied()));
        }
    }

    #[inline(always)]
    fn generate_king_moves(&mut self) {
        let king = self.piece_color_bb(Piece::King, self.side_to_mv).lsb();
        self.serialize_king_moves(king, moves::king(king));
    }

    #[inline(always)]
    fn generate_castling_moves(&mut self) {
        let board = self;

        // Other pieces in the castling path or attacking the
        // castling path block the king's ability to castle.
        let castling_blockers = board.occupied + board.threats;

        let king = board.piece_color_bb(Piece::King, board.side_to_mv).lsb();

        let castling_info = &board.castling_square_info;

        let a_side = castling::SideColor(board.side_to_mv, castling::Side::A);
        if board.castling_square_info.rights.has(a_side)
            && castling_info.path(a_side).is_disjoint(castling_blockers)
        {
            board.move_list.push(Move::new(
                king,
                castling_info.rook(a_side),
                MoveFlag::Castle,
            ));
        }

        let h_side = castling::SideColor(board.side_to_mv, castling::Side::H);
        if board.castling_square_info.rights.has(h_side)
            && castling_info.path(h_side).is_disjoint(castling_blockers)
        {
            board.move_list.push(Move::new(
                king,
                castling_info.rook(h_side),
                MoveFlag::Castle,
            ));
        }
    }
}

impl Board {
    #[inline(always)]
    fn serialize_moves(&mut self, source: Square, targets: BitBoard) {
        let targets = targets & self.targets & self.check_mask;

        for target in targets {
            self.move_list
                .push(Move::new(source, target, MoveFlag::Normal));
        }
    }

    #[inline(always)]
    fn serialize_pawn_push<const GEN_QUIET: bool, const GEN_NOISY: bool>(
        &mut self,
        targets: BitBoard,
    ) {
        let pushes = (targets & self.check_mask) - self.occupied;

        let promos = pushes & BitBoard::rank(Rank::Eighth.relative(self.side_to_mv));
        let pushes = pushes - promos;

        for pawn in promos {
            // Queen Promotions are noisy moves.
            if GEN_NOISY {
                self.move_list.push(Move::new_with_promotion(
                    pawn.down(self.side_to_mv),
                    pawn,
                    Piece::Queen,
                ));
            }

            // Knight, Bishop, and Rook promotions are quiet moves.
            if GEN_QUIET {
                self.move_list.push(Move::new_with_promotion(
                    pawn.down(self.side_to_mv),
                    pawn,
                    Piece::Knight,
                ));
                self.move_list.push(Move::new_with_promotion(
                    pawn.down(self.side_to_mv),
                    pawn,
                    Piece::Rook,
                ));
                self.move_list.push(Move::new_with_promotion(
                    pawn.down(self.side_to_mv),
                    pawn,
                    Piece::Bishop,
                ));
            }
        }

        if GEN_QUIET {
            for pawn in pushes {
                self.move_list.push(Move::new(
                    pawn.down(self.side_to_mv),
                    pawn,
                    MoveFlag::Normal,
                ));
            }

            let double = targets & BitBoard::rank(Rank::Third.relative(self.side_to_mv));
            let double = (double.up(self.side_to_mv) & self.check_mask) - self.occupied;

            for pawn in double {
                self.move_list.push(Move::new(
                    pawn.down(self.side_to_mv).down(self.side_to_mv),
                    pawn,
                    MoveFlag::Normal,
                ));
            }
        }
    }

    #[inline(always)]
    fn serialize_king_moves(&mut self, source: Square, targets: BitBoard) {
        let targets = (targets & self.targets) - self.threats;

        for target in targets {
            self.move_list
                .push(Move::new(source, target, MoveFlag::Normal));
        }
    }
}
