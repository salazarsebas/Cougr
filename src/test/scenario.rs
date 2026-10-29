//! Declarative multi-player / multi-turn scenario runner.

use super::harness::PlayerSlot;

/// Zero-based turn counter passed to scenario step callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnIndex(pub u32);

/// Multi-player turn driver for contract integration tests.
pub struct Scenario {
    name: &'static str,
    player_count: u32,
    turn_count: u32,
}

impl Scenario {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            player_count: 1,
            turn_count: 1,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn players(mut self, count: u32) -> Self {
        self.player_count = count.max(1);
        self
    }

    pub fn turns(mut self, count: u32) -> Self {
        self.turn_count = count;
        self
    }

    pub fn player_count(&self) -> u32 {
        self.player_count
    }

    pub fn turn_count(&self) -> u32 {
        self.turn_count
    }

    /// Run `step` for each turn, rotating [`PlayerSlot`] across `player_count`.
    pub fn run<H, Step>(&self, harness: &H, mut step: Step)
    where
        Step: FnMut(PlayerSlot, TurnIndex, &H),
    {
        for turn in 0..self.turn_count {
            let player = PlayerSlot(turn % self.player_count);
            step(player, TurnIndex(turn), harness);
        }
    }

    /// Run turns then evaluate `expect` once against the harness.
    pub fn run_and_assert<H, Step, Expect>(&self, harness: &H, step: Step, expect: Expect)
    where
        Step: FnMut(PlayerSlot, TurnIndex, &H),
        Expect: FnOnce(&H),
    {
        self.run(harness, step);
        expect(harness);
    }
}
