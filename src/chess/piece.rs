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

use crate::chess;

use crate::util::type_macros;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Default, FromPrimitive)]
#[rustfmt::skip]
pub enum ColoredPiece {
    WhitePawn, WhiteKnight, WhiteBishop,
    WhiteRook, WhiteQueen, WhiteKing,
    BlackPawn, BlackKnight, BlackBishop,
    BlackRook, BlackQueen, BlackKing,

    #[default] None,
}

impl ColoredPiece {
    pub const N: usize = 12;

    #[inline(always)]
    pub fn new(piece: Piece, color: chess::Color) -> ColoredPiece {
        ColoredPiece::from(color as usize * 2 + piece as usize)
    }

    #[inline(always)]
    pub fn piece(self) -> Piece {
        if self == ColoredPiece::None {
            Piece::None
        } else {
            Piece::from(self as usize % chess::Piece::N)
        }
    }

    #[inline(always)]
    pub fn color(self) -> chess::Color {
        chess::Color::from(self as usize / Piece::N)
    }

    #[inline(always)]
    pub fn is(self, piece: Piece) -> bool {
        self.piece() == piece
    }
}

type_macros::impl_from_integer_for_enum! {
    for ColoredPiece:

    // unsigned integers
    usize, ColoredPiece::from_usize;
    u8, ColoredPiece::from_u8; u16, ColoredPiece::from_u16;
    u32, ColoredPiece::from_u32; u64, ColoredPiece::from_u64;

    // signed integers
    isize, ColoredPiece::from_isize;
    i8, ColoredPiece::from_i8; i16, ColoredPiece::from_i16;
    i32, ColoredPiece::from_i32; i64, ColoredPiece::from_i64;
}

#[derive(Copy, Clone, PartialEq, Default, FromPrimitive)]
#[rustfmt::skip]
pub enum Piece {
    Pawn, Knight, Bishop,
    Rook, Queen, King,

    #[default]
    None,
}

impl Piece {
    pub const N: usize = 6;
}

type_macros::impl_from_integer_for_enum! {
    for Piece:

    // unsigned integers
    usize, Piece::from_usize;
    u8, Piece::from_u8; u16, Piece::from_u16;
    u32, Piece::from_u32; u64, Piece::from_u64;

    // signed integers
    isize, Piece::from_isize;
    i8, Piece::from_i8; i16, Piece::from_i16;
    i32, Piece::from_i32; i64, Piece::from_i64;
}
