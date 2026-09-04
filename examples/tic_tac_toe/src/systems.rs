use soroban_sdk::Vec;

/// All 8 winning lines on a 3x3 board (3 rows, 3 columns, 2 diagonals),
/// indexed 0-8 left-to-right, top-to-bottom.
const LINES: [[u32; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8], // rows
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8], // columns
    [0, 4, 8],
    [2, 4, 6], // diagonals
];

/// Pure win/draw detection over the current board and move count.
///
/// Returns `0` if the game is still in progress, `1` if X has won, `2` if O
/// has won, or `3` if the board is full with no winner (draw). Does not
/// touch storage - callers are responsible for persisting the result.
pub fn detect_winner(cells: &Vec<u32>, move_count: u32) -> u32 {
    for line in LINES.iter() {
        let a = cells.get(line[0]).unwrap_or(0);
        let b = cells.get(line[1]).unwrap_or(0);
        let c = cells.get(line[2]).unwrap_or(0);
        if a != 0 && a == b && b == c {
            return a; // 1 = X wins, 2 = O wins
        }
    }
    if move_count >= 9 {
        3
    } else {
        0
    }
}
