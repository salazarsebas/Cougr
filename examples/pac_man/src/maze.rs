use crate::types::CellType;
use soroban_sdk::{Env, Vec};

pub fn create_maze(env: &Env) -> Vec<CellType> {
    let mut maze: Vec<CellType> = Vec::new(env);

    // Define maze layout as a string for clarity
    // # = Wall, . = Pellet, P = Power Pellet, ' ' = Empty
    let layout: [&str; 10] = [
        "##########",
        "#P......P#",
        "#.##.##..#",
        "#.#...#..#",
        "#...#....#",
        "#.#.#.##.#",
        "#.#......#",
        "#.##.###.#",
        "#P......P#",
        "##########",
    ];

    for row in layout.iter() {
        for ch in row.chars() {
            let cell = match ch {
                '#' => CellType::Wall,
                '.' => CellType::Pellet,
                'P' => CellType::PowerPellet,
                _ => CellType::Empty,
            };
            maze.push_back(cell);
        }
    }

    maze
}
