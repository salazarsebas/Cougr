use soroban_sdk::{contracterror, contracttype, Address, Vec};

// Error type
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CheckersError {
    /// Game has already been initialised.
    AlreadyInitialised = 1,
    /// Game has not been initialised yet.
    NotInitialised = 2,
    /// The caller is not a registered player.
    NotAPlayer = 3,
    /// It is not the caller's turn.
    WrongTurn = 4,
    /// The source cell is empty or owned by the opponent.
    NotYourPiece = 5,
    /// The destination cell is occupied.
    DestinationOccupied = 6,
    /// The move is not a legal diagonal step or jump.
    IllegalMove = 7,
    /// A capture is available and must be taken (forced-capture rule).
    MustCapture = 8,
    /// The game is already over.
    GameOver = 9,
    /// Row or column index is out of the 0–7 range.
    OutOfBounds = 10,
    /// During a chain capture the piece must continue from the landing square.
    ChainCapturePieceMismatch = 11,
    /// The destination must be a dark square (row + col odd).
    NotDarkSquare = 12,
}

// Component types
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BoardComponent {
    pub cells: Vec<i32>,
}

/// **TurnComponent** — whose turn it is and how many moves have been played.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TurnComponent {
    /// `1` = Player One, `2` = Player Two.
    pub current_player: u32,
    pub move_number: u32,
}

/// **GameStatusComponent** — overall game lifecycle.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct GameStatusComponent {
    pub status: GameStatus,
    /// `0` = no winner yet, `1` = Player One won, `2` = Player Two won.
    pub winner: u32,
}

/// Lifecycle state of the game.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GameStatus {
    Active,
    Finished,
}
// Public view types returned by the contract API
/// Full game state snapshot returned by `get_state`.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub board: BoardComponent,
    pub turn: TurnComponent,
    pub status: GameStatusComponent,
    pub player_one: Address,
    pub player_two: Address,
}

/// Board snapshot returned by `get_board`.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BoardState {
    pub cells: Vec<i32>,
}
// Internal: chain-capture tracking stored in persistent state

#[contracttype]
#[derive(Clone)]
pub(crate) struct ChainCapture {
    pub row: u32,
    pub col: u32,
}

// Internal: SmallVec4 — tiny fixed-capacity stack-allocated collection
#[derive(Copy, Clone)]
pub(crate) struct SmallVec4<T: Copy + Default> {
    data: [T; 4],
    len: usize,
}

impl<T: Copy + Default> SmallVec4<T> {
    pub(crate) fn new() -> Self {
        Self {
            data: [T::default(); 4],
            len: 0,
        }
    }

    pub(crate) fn push(&mut self, v: T) {
        if self.len < 4 {
            self.data[self.len] = v;
            self.len += 1;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }
}
