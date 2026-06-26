#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;

use components::{
    BoardComponent, BoardState, ChainCapture, CheckersError, GameState, GameStatus,
    GameStatusComponent, TurnComponent,
};
use systems::{
    any_capture_available, board_get, board_set, check_winner, initial_board, legal_captures,
    legal_steps, maybe_promote, owned_by,
};

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

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
