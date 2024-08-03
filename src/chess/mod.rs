// Namespaced modules.
pub mod castling;
pub mod moves;
pub mod zobrist;

// Non-namespaced modules.
mod bitboard;
mod board;
mod color;
mod fen;
mod mailbox;
mod r#move;
mod piece;
mod square;

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bitboard::*;
pub use self::board::*;
pub use self::color::*;
pub use self::fen::*;
pub use self::mailbox::*;
pub use self::piece::*;
pub use self::r#move::*;
pub use self::square::*;
