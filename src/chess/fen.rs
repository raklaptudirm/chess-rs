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

use std::{fmt::Display, num::ParseIntError, str::FromStr};

use super::{
    castling, Board, Color, ColorParseError, Mailbox, MailboxParseErr, Square, SquareParseError,
};

// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
pub struct FEN {
    pub position: Mailbox,
    pub side_to_move: Color,
    pub castling_rights: castling::Rights,
    pub en_pass_square: Square,
    pub half_move_clock: u8,
    pub full_move_count: u16,
}

impl FEN {
    const MAILBOX_OFFSET: usize = 0;
    const SIDE_TM_OFFSET: usize = 1;
    const CASTLINGOFFSET: usize = 2;
    const EN_PASS_OFFSET: usize = 3;
    const HALF_MV_OFFSET: usize = 4;
    const FULL_MV_OFFSET: usize = 5;
}

impl From<&Board> for FEN {
    fn from(board: &Board) -> Self {
        FEN {
            position: board.mailbox(),
            side_to_move: board.side_to_move(),
            castling_rights: castling::Rights::BA,
            en_pass_square: board.en_passant_target(),
            half_move_clock: board.draw_clock(),
            full_move_count: board.plys() / 2 + 1,
        }
    }
}

impl Display for FEN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} cas {} {} {}",
            self.position,
            self.side_to_move,
            self.en_pass_square,
            self.half_move_clock,
            self.full_move_count
        )
    }
}

pub enum FENParseError {
    WrongFieldNumber,
    MailboxParseError(MailboxParseErr),
    SideToMoveParseError(ColorParseError),
    CastlingParseError,
    EnPassantSqParseError(SquareParseError),
    HalfMoveClockParseError(ParseIntError),
    FullMoveClockParseError(ParseIntError),
}

impl FromStr for FEN {
    type Err = FENParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split fen into it's fields along the whitespace.
        let fields: Vec<&str> = s.split_whitespace().collect();

        // Verify the presence of the 6 fen fields.
        if fields.len() != 6 {
            return Err(FENParseError::WrongFieldNumber);
        }

        // Parse mailbox position representation.
        let position = match Mailbox::from_str(fields[FEN::MAILBOX_OFFSET]) {
            Ok(mailbox) => mailbox,
            Err(err) => return Err(FENParseError::MailboxParseError(err)),
        };

        // Parse side to move.
        let side_to_move = match Color::from_str(fields[FEN::SIDE_TM_OFFSET]) {
            Ok(stm) => stm,
            Err(err) => return Err(FENParseError::SideToMoveParseError(err)),
        };

        // Parse en passant target square.
        let en_pass_square = match Square::from_str(fields[FEN::EN_PASS_OFFSET]) {
            Ok(target) => target,
            Err(err) => return Err(FENParseError::EnPassantSqParseError(err)),
        };

        // Parse half move clock.
        let half_move_clock = match str::parse::<u8>(fields[FEN::HALF_MV_OFFSET]) {
            Ok(half_move_clock) => half_move_clock,
            Err(err) => return Err(FENParseError::HalfMoveClockParseError(err)),
        };

        // Parse full move count.
        let full_move_count = match str::parse::<u16>(fields[FEN::FULL_MV_OFFSET]) {
            Ok(full_move_count) => full_move_count,
            Err(err) => return Err(FENParseError::FullMoveClockParseError(err)),
        };

        Ok(FEN {
            position,
            side_to_move,
            castling_rights: castling::Rights::WH
                + castling::Rights::WA
                + castling::Rights::BH
                + castling::Rights::BA,
            en_pass_square,
            half_move_clock,
            full_move_count,
        })
    }
}
