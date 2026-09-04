import type { Puzzle, PuzzleSummary, Suspect, Clue } from '../types';

// ─── Suspect Palette ──────────────────────────────────────────────────────────
export const SUSPECT_COLORS: Record<number, string> = {
  1: '#6b3a2a', // Crimson Mahogany
  2: '#1e3a5f', // Midnight Navy
  3: '#2a4a2a', // Forest Shadow
  4: '#4a2a5f', // Deep Violet
  5: '#5f4a1e', // Aged Brass
};

// ─── Mock Suspects ─────────────────────────────────────────────────────────────
const suspects4x4: Suspect[] = [
  { id: 1, name: 'Col. Hargrove', color: '#6b3a2a', initials: 'CH' },
  { id: 2, name: 'Lady Voss',     color: '#1e3a5f', initials: 'LV' },
  { id: 3, name: 'Dr. Fenwick',   color: '#2a4a2a', initials: 'DF' },
  { id: 4, name: 'Miss Crane',    color: '#4a2a5f', initials: 'MC' },
];

const suspects5x5: Suspect[] = [
  ...suspects4x4,
  { id: 5, name: 'The Butler',    color: '#5f4a1e', initials: 'TB' },
];

// ─── Mock Clues ───────────────────────────────────────────────────────────────
const clues4x4: Clue[] = [
  {
    id: 'c1', type: 'not_same_row', suspectAId: 1, suspectBId: 3,
    description: 'Col. Hargrove was not seen in the same wing as Dr. Fenwick.',
  },
  {
    id: 'c2', type: 'not_same_col', suspectAId: 2, suspectBId: 4,
    description: 'Lady Voss and Miss Crane were never in the same corridor.',
  },
  {
    id: 'c3', type: 'adjacent', suspectAId: 1, suspectBId: 2,
    description: 'Col. Hargrove was found in the room next to Lady Voss.',
  },
  {
    id: 'c4', type: 'not_adjacent', suspectAId: 3, suspectBId: 4,
    description: 'Dr. Fenwick was careful never to be adjacent to Miss Crane.',
  },
];

const clues5x5: Clue[] = [
  ...clues4x4,
  {
    id: 'c5', type: 'same_row', suspectAId: 5, suspectBId: 1,
    description: 'The Butler and Col. Hargrove were spotted in the same hall.',
  },
  {
    id: 'c6', type: 'not_same_col', suspectAId: 5, suspectBId: 2,
    description: 'The Butler avoided Lady Voss\'s corridor all evening.',
  },
];

// ─── Mock Puzzles ─────────────────────────────────────────────────────────────
export const MOCK_PUZZLES: Puzzle[] = [
  {
    id: 'puzzle-001',
    title: 'The Blackwood Manor Affair',
    description:
      'A stormy night. A locked room. Four suspects, four rooms - only the grid holds the truth.',
    gridSize: 4,
    difficulty: 'Easy',
    suspects: suspects4x4,
    clues: clues4x4,
    solution: [1, 2, 3, 4, 2, 3, 4, 1, 3, 4, 1, 2, 4, 1, 2, 3],
    creatorAddress: 'GBETG...K7QW',
    createdAt: '2026-05-01T00:00:00Z',
  },
  {
    id: 'puzzle-002',
    title: 'The Velvet Club Incident',
    description:
      'Five members. Five private rooms. The ledger was stolen at midnight - but who moved where?',
    gridSize: 5,
    difficulty: 'Medium',
    suspects: suspects5x5,
    clues: clues5x5,
    solution: [
      1, 2, 3, 4, 5,
      2, 3, 4, 5, 1,
      3, 4, 5, 1, 2,
      4, 5, 1, 2, 3,
      5, 1, 2, 3, 4,
    ],
    creatorAddress: 'GCX77...P2MN',
    createdAt: '2026-05-10T00:00:00Z',
  },
  {
    id: 'puzzle-003',
    title: 'Death Aboard the Meridian Express',
    description:
      'The train never stopped. The suspects never left. Every clue points somewhere - and nowhere.',
    gridSize: 5,
    difficulty: 'Expert',
    suspects: suspects5x5,
    clues: [
      ...clues5x5,
      {
        id: 'c7', type: 'not_same_row', suspectAId: 1, suspectBId: 5,
        description: 'The Colonel avoided the same carriage as The Butler.',
      },
    ],
    solution: [
      5, 1, 2, 3, 4,
      4, 5, 1, 2, 3,
      3, 4, 5, 1, 2,
      2, 3, 4, 5, 1,
      1, 2, 3, 4, 5,
    ],
    creatorAddress: 'GDMTZ...R9SV',
    createdAt: '2026-05-18T00:00:00Z',
  },
];

export const MOCK_SUMMARIES: PuzzleSummary[] = MOCK_PUZZLES.map((p) => ({
  id: p.id,
  title: p.title,
  gridSize: p.gridSize,
  difficulty: p.difficulty,
  clueCount: p.clues.length,
  creatorAddress: p.creatorAddress,
}));
