#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

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
struct ChainCapture {
    pub row: u32,
    pub col: u32,
}

// Internal: SmallVec4 — tiny fixed-capacity stack-allocated collection
#[derive(Copy, Clone)]
struct SmallVec4<T: Copy + Default> {
    data: [T; 4],
    len: usize,
}

impl<T: Copy + Default> SmallVec4<T> {
    fn new() -> Self {
        Self {
            data: [T::default(); 4],
            len: 0,
        }
    }

    fn push(&mut self, v: T) {
        if self.len < 4 {
            self.data[self.len] = v;
            self.len += 1;
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }
}

// Internal: board cell access
#[inline]
fn board_get(cells: &Vec<i32>, row: u32, col: u32) -> i32 {
    cells.get(row * 8 + col).unwrap_or(0)
}

#[inline]
fn board_set(cells: &mut Vec<i32>, row: u32, col: u32, val: i32) {
    cells.set(row * 8 + col, val);
}
// Internal: piece helpers
/// `true` when `piece` belongs to `player` (1 or 2).
#[inline]
fn owned_by(piece: i32, player: u32) -> bool {
    match player {
        1 => piece == 1 || piece == 2,
        2 => piece == -1 || piece == -2,
        _ => false,
    }
}

/// `true` when `piece` is a king (moves in all four diagonal directions).
#[inline]
fn is_king(piece: i32) -> bool {
    piece == 2 || piece == -2
}

#[inline]
fn forward_delta(player: u32) -> i32 {
    if player == 1 {
        1
    } else {
        -1
    }
}
// System: MoveValidationSystem
fn legal_steps(cells: &Vec<i32>, row: u32, col: u32, player: u32) -> SmallVec4<(u32, u32)> {
    let piece = board_get(cells, row, col);
    let fd = forward_delta(player);

    let mut deltas: [(i32, i32); 4] = [(0, 0); 4];
    deltas[0] = (fd, 1);
    deltas[1] = (fd, -1);
    let count = if is_king(piece) {
        deltas[2] = (-fd, 1);
        deltas[3] = (-fd, -1);
        4
    } else {
        2
    };

    let mut out = SmallVec4::<(u32, u32)>::new();
    for &(dr, dc) in deltas.iter().take(count) {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if (0..8).contains(&nr) && (0..8).contains(&nc) {
            let (nr, nc) = (nr as u32, nc as u32);
            if board_get(cells, nr, nc) == 0 {
                out.push((nr, nc));
            }
        }
    }
    out
}

/// Returns all legal capture destinations from `(row, col)` for `player`.
fn legal_captures(
    cells: &Vec<i32>,
    row: u32,
    col: u32,
    player: u32,
) -> SmallVec4<(u32, u32, u32, u32)> {
    let piece = board_get(cells, row, col);
    let fd = forward_delta(player);

    let mut deltas: [(i32, i32); 4] = [(0, 0); 4];
    deltas[0] = (fd, 1);
    deltas[1] = (fd, -1);
    let count = if is_king(piece) {
        deltas[2] = (-fd, 1);
        deltas[3] = (-fd, -1);
        4
    } else {
        2
    };

    let mut out = SmallVec4::<(u32, u32, u32, u32)>::new();
    for &(dr, dc) in deltas.iter().take(count) {
        let mr = row as i32 + dr;
        let mc = col as i32 + dc;
        let lr = row as i32 + 2 * dr;
        let lc = col as i32 + 2 * dc;

        // Bounds-check both squares.
        if !(0..8).contains(&mr) || !(0..8).contains(&mc) {
            continue;
        }
        if !(0..8).contains(&lr) || !(0..8).contains(&lc) {
            continue;
        }

        let (mr, mc) = (mr as u32, mc as u32);
        let (lr, lc) = (lr as u32, lc as u32);

        let mid = board_get(cells, mr, mc);
        let land = board_get(cells, lr, lc);

        // Middle must be an opponent piece; landing must be empty.
        if owned_by(mid, 3 - player) && land == 0 {
            out.push((lr, lc, mr, mc));
        }
    }
    out
}

/// `true` if `player` has at least one capture available anywhere on the board.
fn any_capture_available(cells: &Vec<i32>, player: u32) -> bool {
    for row in 0u32..8 {
        for col in 0u32..8 {
            if owned_by(board_get(cells, row, col), player)
                && !legal_captures(cells, row, col, player).is_empty()
            {
                return true;
            }
        }
    }
    false
}

/// `true` if `player` has at least one legal move (step or capture) anywhere.
fn any_legal_move(cells: &Vec<i32>, player: u32) -> bool {
    for row in 0u32..8 {
        for col in 0u32..8 {
            if owned_by(board_get(cells, row, col), player) {
                if !legal_captures(cells, row, col, player).is_empty() {
                    return true;
                }
                if !legal_steps(cells, row, col, player).is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

// System: PromotionSystem
fn maybe_promote(cells: &mut Vec<i32>, row: u32, col: u32, player: u32) {
    let promotion_row = if player == 1 { 7u32 } else { 0u32 };
    if row == promotion_row {
        let piece = board_get(cells, row, col);
        if !is_king(piece) {
            let king_val = if player == 1 { 2i32 } else { -2i32 };
            board_set(cells, row, col, king_val);
        }
    }
}
// System: EndConditionSystem
/// Returns the winning player number (1 or 2), or 0 if the game continues.
fn check_winner(cells: &Vec<i32>) -> u32 {
    let mut p1_pieces = 0u32;
    let mut p2_pieces = 0u32;
    for row in 0u32..8 {
        for col in 0u32..8 {
            let v = board_get(cells, row, col);
            if v == 1 || v == 2 {
                p1_pieces += 1;
            } else if v == -1 || v == -2 {
                p2_pieces += 1;
            }
        }
    }

    // A player with no pieces loses immediately.
    if p2_pieces == 0 {
        return 1;
    }
    if p1_pieces == 0 {
        return 2;
    }

    if !any_legal_move(cells, 1) {
        return 2;
    }
    if !any_legal_move(cells, 2) {
        return 1;
    }

    0 // Game still active.
}

// Board initialisation helper
fn initial_board(env: &Env) -> Vec<i32> {
    let mut cells: Vec<i32> = Vec::new(env);
    for row in 0u32..8 {
        for col in 0u32..8 {
            let dark = (row + col) % 2 == 1;
            let val: i32 = if dark && row <= 2 {
                1
            } else if dark && row >= 5 {
                -1
            } else {
                0
            };
            cells.push_back(val);
        }
    }
    cells
}

// Contract
#[contract]
pub struct CheckersContract;

#[contractimpl]
impl CheckersContract {
    // init_game
    pub fn init_game(
        env: Env,
        player_one: Address,
        player_two: Address,
    ) -> Result<(), CheckersError> {
        if env.storage().persistent().has(&symbol_short!("STATUS")) {
            return Err(CheckersError::AlreadyInitialised);
        }

        env.storage().persistent().set(
            &symbol_short!("BOARD"),
            &BoardComponent {
                cells: initial_board(&env),
            },
        );
        env.storage().persistent().set(
            &symbol_short!("TURN"),
            &TurnComponent {
                current_player: 1,
                move_number: 1,
            },
        );
        env.storage().persistent().set(
            &symbol_short!("STATUS"),
            &GameStatusComponent {
                status: GameStatus::Active,
                winner: 0,
            },
        );
        env.storage()
            .persistent()
            .set(&symbol_short!("P1"), &player_one);
        env.storage()
            .persistent()
            .set(&symbol_short!("P2"), &player_two);

        Ok(())
    }

    // submit_move
    pub fn submit_move(
        env: Env,
        player: Address,
        from_row: u32,
        from_col: u32,
        to_row: u32,
        to_col: u32,
    ) -> Result<(), CheckersError> {
        let status: GameStatusComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("STATUS"))
            .ok_or(CheckersError::NotInitialised)?;

        if status.status == GameStatus::Finished {
            return Err(CheckersError::GameOver);
        }

        let p1: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("P1"))
            .unwrap();
        let p2: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("P2"))
            .unwrap();
        let turn: TurnComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("TURN"))
            .unwrap();

        // Identify caller
        let caller_num: u32 = if player == p1 {
            1
        } else if player == p2 {
            2
        } else {
            return Err(CheckersError::NotAPlayer);
        };

        if caller_num != turn.current_player {
            return Err(CheckersError::WrongTurn);
        }

        // Bounds check
        if from_row >= 8 || from_col >= 8 || to_row >= 8 || to_col >= 8 {
            return Err(CheckersError::OutOfBounds);
        }
        if (to_row + to_col).is_multiple_of(2) {
            return Err(CheckersError::NotDarkSquare);
        }

        // Load board
        let mut board: BoardComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("BOARD"))
            .unwrap();

        // Chain-capture continuation
        let chain: Option<ChainCapture> = env.storage().persistent().get(&symbol_short!("CHAIN"));

        if let Some(ref c) = chain {
            if from_row != c.row || from_col != c.col {
                return Err(CheckersError::ChainCapturePieceMismatch);
            }
        }

        // Piece ownership
        let piece = board_get(&board.cells, from_row, from_col);
        if !owned_by(piece, caller_num) {
            return Err(CheckersError::NotYourPiece);
        }

        let row_diff = (to_row as i32 - from_row as i32).abs();
        let col_diff = (to_col as i32 - from_col as i32).abs();

        if row_diff != col_diff || (row_diff != 1 && row_diff != 2) {
            return Err(CheckersError::IllegalMove);
        }

        let is_capture = row_diff == 2;

        // Destination occupancy
        if board_get(&board.cells, to_row, to_col) != 0 {
            return Err(CheckersError::DestinationOccupied);
        }

        // Validate specific move
        let mut cap_row: Option<u32> = None;
        let mut cap_col: Option<u32> = None;

        if is_capture {
            let caps = legal_captures(&board.cells, from_row, from_col, caller_num);
            let mut found = false;
            for &(lr, lc, mr, mc) in caps.as_slice() {
                if lr == to_row && lc == to_col {
                    cap_row = Some(mr);
                    cap_col = Some(mc);
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(CheckersError::IllegalMove);
            }
        } else {
            let steps = legal_steps(&board.cells, from_row, from_col, caller_num);
            let mut found = false;
            for &(nr, nc) in steps.as_slice() {
                if nr == to_row && nc == to_col {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(CheckersError::IllegalMove);
            }
        }

        //  Forced-capture enforcement
        // Outside a chain, a step is illegal whenever any capture is available.
        if chain.is_none() && !is_capture && any_capture_available(&board.cells, caller_num) {
            return Err(CheckersError::MustCapture);
        }

        // Apply move
        let piece_val = board_get(&board.cells, from_row, from_col);
        board_set(&mut board.cells, from_row, from_col, 0);
        board_set(&mut board.cells, to_row, to_col, piece_val);

        if let (Some(cr), Some(cc)) = (cap_row, cap_col) {
            board_set(&mut board.cells, cr, cc, 0);
        }

        // System: PromotionSystem
        maybe_promote(&mut board.cells, to_row, to_col, caller_num);

        // Determine chain continuation
        let mut turn_ends = true;
        let mut next_chain: Option<ChainCapture> = None;

        if is_capture {
            let further = legal_captures(&board.cells, to_row, to_col, caller_num);
            if !further.is_empty() {
                turn_ends = false;
                next_chain = Some(ChainCapture {
                    row: to_row,
                    col: to_col,
                });
            }
        }

        // Persist board
        env.storage()
            .persistent()
            .set(&symbol_short!("BOARD"), &board);

        // System: EndConditionSystem
        let winner = check_winner(&board.cells);
        if winner > 0 {
            env.storage().persistent().set(
                &symbol_short!("STATUS"),
                &GameStatusComponent {
                    status: GameStatus::Finished,
                    winner,
                },
            );
            env.storage().persistent().remove(&symbol_short!("CHAIN"));
            return Ok(());
        }

        // System: TurnSystem
        if turn_ends {
            let next_player = 3 - caller_num; // 1 → 2, 2 → 1
            env.storage().persistent().set(
                &symbol_short!("TURN"),
                &TurnComponent {
                    current_player: next_player,
                    move_number: turn.move_number + 1,
                },
            );
            env.storage().persistent().remove(&symbol_short!("CHAIN"));
        } else {
            // Multi-hop in progress — hold the turn, record the chain square.
            env.storage()
                .persistent()
                .set(&symbol_short!("CHAIN"), next_chain.as_ref().unwrap());
        }

        Ok(())
    }

    // get_state
    /// Return the full game state snapshot.
    pub fn get_state(env: Env) -> Result<GameState, CheckersError> {
        let board: BoardComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("BOARD"))
            .ok_or(CheckersError::NotInitialised)?;

        let turn: TurnComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("TURN"))
            .unwrap();

        let status: GameStatusComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("STATUS"))
            .unwrap();

        let player_one: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("P1"))
            .unwrap();

        let player_two: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("P2"))
            .unwrap();

        Ok(GameState {
            board,
            turn,
            status,
            player_one,
            player_two,
        })
    }

    // get_board
    /// Return the current board cells (64 values, row-major order).
    pub fn get_board(env: Env) -> Result<BoardState, CheckersError> {
        let board: BoardComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("BOARD"))
            .ok_or(CheckersError::NotInitialised)?;

        Ok(BoardState { cells: board.cells })
    }

    // get_current_player
    /// Return the `Address` of the player whose turn it currently is.
    pub fn get_current_player(env: Env) -> Result<Address, CheckersError> {
        let turn: TurnComponent = env
            .storage()
            .persistent()
            .get(&symbol_short!("TURN"))
            .ok_or(CheckersError::NotInitialised)?;

        let key = if turn.current_player == 1 {
            symbol_short!("P1")
        } else {
            symbol_short!("P2")
        };

        let addr: Address = env.storage().persistent().get(&key).unwrap();
        Ok(addr)
    }
}

// Tests
#[cfg(test)]
mod test;
