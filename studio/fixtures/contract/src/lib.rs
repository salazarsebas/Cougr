#![no_std]
use soroban_sdk::{Address, Env, Symbol, contract, contracterror, contractimpl, contracttype};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Player {
    X,
    O,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnBasedConfig {
    pub board_width: u32,
    pub board_height: u32,
    pub win_length: u32,
    pub first_player: Player,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidBoardWidth = 1,
    InvalidBoardHeight = 2,
    InvalidWinLength = 3,
}

#[contract]
pub struct FixtureContract;

#[contractimpl]
impl FixtureContract {
    pub fn init(env: Env, admin: Address, config: TurnBasedConfig) -> Result<(), Error> {
        admin.require_auth();
        if config.board_width < 3 || config.board_width > 8 {
            return Err(Error::InvalidBoardWidth);
        }
        if config.board_height < 3 || config.board_height > 8 {
            return Err(Error::InvalidBoardHeight);
        }
        let min_dim = if config.board_width < config.board_height {
            config.board_width
        } else {
            config.board_height
        };
        if config.win_length < 3 || config.win_length > min_dim {
            return Err(Error::InvalidWinLength);
        }

        env.storage()
            .instance()
            .set(&Symbol::new(&env, "config"), &config);
        Ok(())
    }
}
