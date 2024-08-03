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

use colored::Colorize;

use super::{Color, ColoredPiece, File, Piece, Rank, Square};

#[derive(Clone, Copy)]
pub struct Mailbox(pub [ColoredPiece; Square::N]);

#[derive(Debug)]
pub enum MailboxParseErr {
    JumpTooLong,
    InvalidPieceIdent,
    FileDataIncomplete,
    TooManyFields,
}

impl FromStr for Mailbox {
    type Err = MailboxParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mailbox = Mailbox([ColoredPiece::None; Square::N]);

        let ranks: Vec<&str> = s.split('/').collect();

        let mut rank = Rank::Eighth;
        let mut file = File::A;
        for rank_data in ranks {
            // Rank pointer ran out, but data carried on.
            if rank == Rank::None {
                return Err(MailboxParseErr::TooManyFields);
            }

            for data in rank_data.chars() {
                let square = Square::new(file, rank) as usize;
                match data {
                    'P' => mailbox.0[square] = ColoredPiece::WhitePawn,
                    'N' => mailbox.0[square] = ColoredPiece::WhiteKnight,
                    'B' => mailbox.0[square] = ColoredPiece::WhiteBishop,
                    'R' => mailbox.0[square] = ColoredPiece::WhiteRook,
                    'Q' => mailbox.0[square] = ColoredPiece::WhiteQueen,
                    'K' => mailbox.0[square] = ColoredPiece::WhiteKing,

                    'p' => mailbox.0[square] = ColoredPiece::BlackPawn,
                    'n' => mailbox.0[square] = ColoredPiece::BlackKnight,
                    'b' => mailbox.0[square] = ColoredPiece::BlackBishop,
                    'r' => mailbox.0[square] = ColoredPiece::BlackRook,
                    'q' => mailbox.0[square] = ColoredPiece::BlackQueen,
                    'k' => mailbox.0[square] = ColoredPiece::BlackKing,

                    '1'..='8' => {
                        file = File::from(file as usize + data as usize - '1' as usize);

                        if file == File::None {
                            return Err(MailboxParseErr::JumpTooLong);
                        }
                    }

                    _ => return Err(MailboxParseErr::InvalidPieceIdent),
                }

                file = File::from(file as usize + 1);
            }

            // After rank data runs out, file pointer should be
            // at the last file, i.e, rank is completely filled.
            if file != File::None {
                return Err(MailboxParseErr::FileDataIncomplete);
            }

            // Switch rank pointer and reset file pointer.
            rank = Rank::from(rank as usize + 1);
            file = File::A;
        }

        Ok(mailbox)
    }
}

impl fmt::Display for Mailbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut empty_counter = 0;
        let mut string_rep = String::from("");

        for (square, piece) in self.0.into_iter().enumerate() {
            let square = Square::from(square);
            let crossed_edge = square.file() == File::A;

            if empty_counter > 0 && (piece != ColoredPiece::None || crossed_edge) {
                string_rep += &empty_counter.to_string();
                empty_counter = 0
            }

            if crossed_edge && square != Square::A8 {
                string_rep += "/";
            }

            match piece {
                ColoredPiece::WhitePawn => string_rep += "P",
                ColoredPiece::WhiteKnight => string_rep += "N",
                ColoredPiece::WhiteBishop => string_rep += "B",
                ColoredPiece::WhiteRook => string_rep += "R",
                ColoredPiece::WhiteQueen => string_rep += "Q",
                ColoredPiece::WhiteKing => string_rep += "K",

                ColoredPiece::BlackPawn => string_rep += "p",
                ColoredPiece::BlackKnight => string_rep += "n",
                ColoredPiece::BlackBishop => string_rep += "b",
                ColoredPiece::BlackRook => string_rep += "r",
                ColoredPiece::BlackQueen => string_rep += "q",
                ColoredPiece::BlackKing => string_rep += "k",

                ColoredPiece::None => empty_counter += 1,
            }
        }

        write!(f, "{string_rep}")
    }
}
