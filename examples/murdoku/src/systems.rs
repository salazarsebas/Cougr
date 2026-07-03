//! Murdoku game systems using cougr-core

use crate::components::{Clues, GridSize, Solution, Suspects};
use crate::PuzzleError;
use cougr_core::component::ComponentTrait;
use cougr_core::simple_world::SimpleWorld;
use cougr_core::SimpleQueryBuilder;
use soroban_sdk::{panic_with_error, Env, Vec};

/// Ephemeral system to validate a submitted puzzle definition.
pub fn puzzle_validation_system(world: &mut SimpleWorld, env: &Env) {
    // Find the single puzzle entity that has been spawned in the ephemeral world
    let entities = SimpleQueryBuilder::new(env)
        .with_component(GridSize::component_type())
        .build()
        .execute(world, env);

    if entities.is_empty() {
        return;
    }
    let entity_id = entities.get(0).unwrap();

    // Retrieve the components
    let grid_size_comp = match world.get_typed::<GridSize>(env, entity_id) {
        Some(c) => c,
        None => return,
    };
    let suspects_comp = match world.get_typed::<Suspects>(env, entity_id) {
        Some(c) => c,
        None => return,
    };
    let clues_comp = match world.get_typed::<Clues>(env, entity_id) {
        Some(c) => c,
        None => return,
    };
    let solution_comp = match world.get_typed::<Solution>(env, entity_id) {
        Some(c) => c,
        None => return,
    };

    let n = grid_size_comp.size;

    // 1. Grid size must be 4 or 5
    if n != 4 && n != 5 {
        panic_with_error!(env, PuzzleError::InvalidGridSize);
    }

    // 2. Suspects length must equal grid_size; each suspect must have a non-empty name
    if suspects_comp.list.len() != n {
        panic_with_error!(env, PuzzleError::InvalidSuspects);
    }
    for i in 0..n {
        let name = suspects_comp.list.get(i).unwrap();
        if name.len() == 0 {
            panic_with_error!(env, PuzzleError::InvalidSuspects);
        }
    }

    // 3. Solution length must equal grid_size * grid_size
    if solution_comp.grid.len() != n * n {
        panic_with_error!(env, PuzzleError::InvalidSolution);
    }

    // 4. Solution must be a valid Latin square:
    // each suspect index (1..=N) appears exactly once per row and exactly once per column
    for r in 0..n {
        let mut seen = Vec::new(env);
        for c in 0..n {
            let val = solution_comp.grid.get(r * n + c).unwrap();
            if val < 1 || val > n {
                panic_with_error!(env, PuzzleError::InvalidSolution);
            }
            let mut found = false;
            for i in 0..seen.len() {
                if seen.get(i).unwrap() == val {
                    found = true;
                    break;
                }
            }
            if found {
                panic_with_error!(env, PuzzleError::InvalidSolution);
            }
            seen.push_back(val);
        }
    }

    for c in 0..n {
        let mut seen = Vec::new(env);
        for r in 0..n {
            let val = solution_comp.grid.get(r * n + c).unwrap();
            if val < 1 || val > n {
                panic_with_error!(env, PuzzleError::InvalidSolution);
            }
            let mut found = false;
            for i in 0..seen.len() {
                if seen.get(i).unwrap() == val {
                    found = true;
                    break;
                }
            }
            if found {
                panic_with_error!(env, PuzzleError::InvalidSolution);
            }
            seen.push_back(val);
        }
    }

    // 5. Clues list must be non-empty
    if clues_comp.list.len() == 0 {
        panic_with_error!(env, PuzzleError::InvalidClues);
    }

    // 6. Every clue must reference only valid suspect indices and valid coordinates for the given grid size
    for i in 0..clues_comp.list.len() {
        let clue = clues_comp.list.get(i).unwrap();
        if clue.row >= n || clue.col >= n {
            panic_with_error!(env, PuzzleError::InvalidClues);
        }
        if clue.suspect_idx < 1 || clue.suspect_idx > n {
            panic_with_error!(env, PuzzleError::InvalidClues);
        }

        // Validate that clue value matches the solution value at that coordinate
        let sol_val = solution_comp.grid.get(clue.row * n + clue.col).unwrap();
        if clue.suspect_idx != sol_val {
            panic_with_error!(env, PuzzleError::InvalidClues);
        }
    }
}
