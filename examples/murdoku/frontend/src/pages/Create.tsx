import React, { useReducer, useState } from "react";
import StepIndicator from "../components/Creator/StepIndicator";
import ClueBuilder from "../components/Creator/ClueBuilder";
import {
  type CreatorState, type GridSize, type Difficulty, type Suspect, type Clue,
  SUSPECT_COLORS, CLUE_TYPE_LABELS, CLUE_NEEDS_SECONDARY,
} from "../types";
import { emptyGrid, isConflict, isCompleteSolution } from "../lib/latinSquare";
import { submitPuzzle } from "../lib/contractClient";

type Action =
  | { type: "SET_GRID_SIZE"; payload: GridSize }
  | { type: "SET_NAME"; payload: string }
  | { type: "SET_DIFFICULTY"; payload: Difficulty }
  | { type: "SET_SUSPECTS"; payload: Suspect[] }
  | { type: "SET_SOLUTION_CELL"; row: number; col: number; suspectId: string }
  | { type: "ADD_CLUE"; payload: Clue }
  | { type: "REMOVE_CLUE"; id: string };

function buildSuspects(size: GridSize, existing: Suspect[]): Suspect[] {
  return Array.from({ length: size }, (_, i) => ({
    id: existing[i]?.id ?? crypto.randomUUID(),
    name: existing[i]?.name ?? `Suspect ${i + 1}`,
    color: SUSPECT_COLORS[i % SUSPECT_COLORS.length],
  }));
}

function reducer(state: CreatorState, action: Action): CreatorState {
  switch (action.type) {
    case "SET_GRID_SIZE": {
      const size = action.payload;
      return { ...state, gridSize: size, suspects: buildSuspects(size, state.suspects), solution: emptyGrid(size), clues: [] };
    }
    case "SET_NAME": return { ...state, puzzleName: action.payload };
    case "SET_DIFFICULTY": return { ...state, difficulty: action.payload };
    case "SET_SUSPECTS": return { ...state, suspects: action.payload };
    case "SET_SOLUTION_CELL": {
      const sol = state.solution.map((r) => [...r]);
      sol[action.row][action.col] = action.suspectId;
      return { ...state, solution: sol };
    }
    case "ADD_CLUE": return { ...state, clues: [...state.clues, action.payload] };
    case "REMOVE_CLUE": return { ...state, clues: state.clues.filter((c) => c.id !== action.id) };
    default: return state;
  }
}

function initialState(): CreatorState {
  const gridSize: GridSize = 4;
  return { gridSize, puzzleName: "", difficulty: "Medium", suspects: buildSuspects(gridSize, []), solution: emptyGrid(gridSize), clues: [] };
}

const STEPS = [{ label:"Grid" },{ label:"Suspects" },{ label:"Solution" },{ label:"Clues" },{ label:"Preview" }];

const S: Record<string, React.CSSProperties> = {
  page: { minHeight:"100vh", background:"#0f0f23", color:"#fff", padding:"32px 16px 64px", fontFamily:"'Segoe UI',sans-serif", maxWidth:640, margin:"0 auto" },
  pageTitle: { textAlign:"center", fontSize:28, fontWeight:800, color:"#6a4c93", marginBottom:24 },
  card: { background:"#16213e", borderRadius:12, padding:24, border:"1px solid #2a2a4a" },
  stepTitle: { fontSize:20, fontWeight:700, margin:"0 0 20px", color:"#e0e0ff" },
  label: { fontSize:13, color:"#aaa", marginBottom:4, display:"block" },
  input: { width:"100%", boxSizing:"border-box" as const, background:"#0f0f23", color:"#fff", border:"1px solid #444", borderRadius:6, padding:"10px 12px", fontSize:14, marginBottom:16, outline:"none" },
  hint: { fontSize:12, color:"#666", margin:"0 0 16px" },
  error: { color:"#e63946", fontSize:13, margin:"0 0 8px" },
  row: { display:"flex", gap:8, flexWrap:"wrap" as const, marginBottom:20 },
  toggleBtn: { background:"#0f0f23", color:"#888", border:"1px solid #444", borderRadius:6, padding:"8px 20px", cursor:"pointer", fontSize:14 },
  toggleBtnActive: { background:"#6a4c93", color:"#fff", border:"1px solid #6a4c93" },
  nextBtn: { background:"#6a4c93", color:"#fff", border:"none", borderRadius:8, padding:"12px 28px", cursor:"pointer", fontSize:15, fontWeight:700 },
  backBtn: { background:"none", color:"#888", border:"1px solid #444", borderRadius:8, padding:"12px 20px", cursor:"pointer", fontSize:14 },
  navRow: { display:"flex", justifyContent:"space-between", alignItems:"center", marginTop:24 },
  suspectGrid: { display:"grid", gridTemplateColumns:"1fr 1fr", gap:12, marginBottom:20 },
  suspectCard: { background:"#0f0f23", border:"1px solid #333", borderRadius:8, padding:12, display:"flex", flexDirection:"column" as const, gap:6 },
  colorDot: { width:16, height:16, borderRadius:"50%", marginBottom:4 },
  solutionGrid: { display:"grid", gap:6, margin:"16px 0" },
  cell: { width:"100%", height:"100%", minHeight:52, display:"flex", alignItems:"center", justifyContent:"center", border:"1px solid #333", borderRadius:6, cursor:"pointer", fontSize:12, fontWeight:600 },
  emptyCell: { width:"100%", height:"100%", minHeight:52, border:"1px dashed #333", borderRadius:6, display:"flex", flexWrap:"wrap" as const, alignItems:"center", justifyContent:"center", gap:3, padding:4, boxSizing:"border-box" as const },
  miniBtn: { width:14, height:14, borderRadius:"50%", border:"none", cursor:"pointer", padding:0 },
  palette: { borderRadius:6, padding:"4px 10px", fontSize:12, fontWeight:600, color:"#fff", whiteSpace:"nowrap" as const },
  clueList: { display:"flex", flexDirection:"column" as const, gap:6 },
  clueItem: { background:"#0f0f23", border:"1px solid #2a2a4a", borderRadius:6, padding:"10px 12px", display:"flex", alignItems:"center", gap:8, fontSize:13 },
  removeBtn: { background:"none", color:"#e63946", border:"none", cursor:"pointer", fontSize:14, padding:4, lineHeight:1 },
  previewSummary: { background:"#0f0f23", borderRadius:8, padding:16, marginBottom:16, display:"flex", flexDirection:"column" as const, gap:8 },
  summaryRow: { display:"flex", justifyContent:"space-between", fontSize:13 },
  successBox: { textAlign:"center" as const, padding:"32px 16px", display:"flex", flexDirection:"column" as const, alignItems:"center", gap:8 },
  errorBox: { background:"#2a0f14", border:"1px solid #e63946", borderRadius:6, padding:"12px 16px", color:"#e63946", fontSize:13, marginBottom:16 },
  playLink: { display:"inline-block", marginTop:12, background:"#06d6a0", color:"#000", fontWeight:700, borderRadius:8, padding:"10px 24px", textDecoration:"none", fontSize:15 },
};

const Create: React.FC = () => {
  const [state, dispatch] = useReducer(reducer, undefined, initialState);
  const [step, setStep] = useState(0);
  const [suspectErrors, setSuspectErrors] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);
  const [submitResult, setSubmitResult] = useState<{ puzzleId: string } | null>(null);
  const [submitError, setSubmitError] = useState("");

  function renderStep0() {
    return (
      <div style={S.card}>
        <h2 style={S.stepTitle}>Step 1 - Grid Configuration</h2>
        <label style={S.label}>Puzzle Name <span style={{ color:"#e63946" }}>*</span></label>
        <input style={S.input} maxLength={64} placeholder="My Murdoku Puzzle" value={state.puzzleName}
          onChange={(e) => dispatch({ type:"SET_NAME", payload:e.target.value })} />
        <p style={S.hint}>{state.puzzleName.length}/64</p>
        <label style={S.label}>Grid Size</label>
        <div style={S.row}>
          {([4,5] as GridSize[]).map((s) => (
            <button key={s} style={{ ...S.toggleBtn, ...(state.gridSize===s ? S.toggleBtnActive : {}) }}
              onClick={() => dispatch({ type:"SET_GRID_SIZE", payload:s })}>{s}×{s}</button>
          ))}
        </div>
        <label style={S.label}>Difficulty</label>
        <div style={S.row}>
          {(["Easy","Medium","Expert"] as Difficulty[]).map((d) => (
            <button key={d} style={{ ...S.toggleBtn, ...(state.difficulty===d ? S.toggleBtnActive : {}) }}
              onClick={() => dispatch({ type:"SET_DIFFICULTY", payload:d })}>{d}</button>
          ))}
        </div>
        <button style={S.nextBtn} disabled={!state.puzzleName.trim()} onClick={() => setStep(1)}>Next →</button>
      </div>
    );
  }

  function handleSuspectName(index: number, name: string) {
    const updated = state.suspects.map((s, i) => i===index ? { ...s, name } : s);
    const errors: Record<string, string> = {};
    updated.forEach((s, i) => {
      if (!s.name.trim()) errors[s.id] = "Name is required.";
      else if (updated.some((o,j) => j!==i && o.name.trim().toLowerCase()===s.name.trim().toLowerCase()))
        errors[s.id] = "Duplicate name.";
    });
    setSuspectErrors(errors);
    dispatch({ type:"SET_SUSPECTS", payload:updated });
  }

  function renderStep1() {
    const hasErrors = Object.keys(suspectErrors).length > 0;
    const allFilled = state.suspects.every((s) => s.name.trim());
    return (
      <div style={S.card}>
        <h2 style={S.stepTitle}>Step 2 - Suspects</h2>
        <p style={S.hint}>Define {state.gridSize} suspects. Each name must be unique (max 32 chars).</p>
        <div style={S.suspectGrid}>
          {state.suspects.map((suspect, i) => (
            <div key={suspect.id} style={S.suspectCard}>
              <div style={{ ...S.colorDot, backgroundColor:suspect.color }} />
              <input style={{ ...S.input, borderColor:suspectErrors[suspect.id]?"#e63946":"#444", marginBottom:0 }}
                maxLength={32} value={suspect.name} placeholder={`Suspect ${i+1}`}
                onChange={(e) => handleSuspectName(i, e.target.value)} />
              {suspectErrors[suspect.id] && <p style={S.error}>{suspectErrors[suspect.id]}</p>}
            </div>
          ))}
        </div>
        <div style={S.navRow}>
          <button style={S.backBtn} onClick={() => setStep(0)}>← Back</button>
          <button style={S.nextBtn} disabled={hasErrors||!allFilled} onClick={() => setStep(2)}>Next →</button>
        </div>
      </div>
    );
  }

  function renderStep2() {
    const complete = isCompleteSolution(state.solution, state.gridSize);
    return (
      <div style={S.card}>
        <h2 style={S.stepTitle}>Step 3 - Solution</h2>
        <p style={S.hint}>Place each suspect once per row and column. Click a filled cell to clear it.</p>
        <div style={S.row}>
          {state.suspects.map((s) => (
            <div key={s.id} style={{ ...S.palette, backgroundColor:s.color }}>{s.name.slice(0,8)}</div>
          ))}
        </div>
        <div style={{ ...S.solutionGrid, gridTemplateColumns:`repeat(${state.gridSize},1fr)` }}>
          {Array.from({ length:state.gridSize }, (_,row) =>
            Array.from({ length:state.gridSize }, (_,col) => {
              const val = state.solution[row][col];
              const suspect = state.suspects.find((s) => s.id===val);
              return (
                <div key={`${row}-${col}`} style={{ aspectRatio:"1" }}>
                  {val ? (
                    <button style={{ ...S.cell, backgroundColor:suspect?.color??"#333", color:"#fff" }}
                      onClick={() => dispatch({ type:"SET_SOLUTION_CELL", row, col, suspectId:"" })}
                      title="Click to clear">
                      {suspect?.name.slice(0,6)}
                    </button>
                  ) : (
                    <div style={S.emptyCell}>
                      {state.suspects.map((s) => {
                        const blocked = isConflict(state.solution, row, col, s.id, state.gridSize);
                        return (
                          <button key={s.id} disabled={blocked}
                            onClick={() => !blocked && dispatch({ type:"SET_SOLUTION_CELL", row, col, suspectId:s.id })}
                            style={{ ...S.miniBtn, backgroundColor:blocked?"#222":s.color, opacity:blocked?0.3:1 }}
                            title={blocked?`${s.name} conflicts`:s.name} />
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
        {!complete && <p style={{ color:"#e9c46a", fontSize:13, textAlign:"center" }}>Fill all {state.gridSize*state.gridSize} cells to continue.</p>}
        <div style={S.navRow}>
          <button style={S.backBtn} onClick={() => setStep(1)}>← Back</button>
          <button style={S.nextBtn} disabled={!complete} onClick={() => setStep(3)}>Next →</button>
        </div>
      </div>
    );
  }

  function renderStep3() {
    return (
      <div style={S.card}>
        <h2 style={S.stepTitle}>Step 4 - Clues</h2>
        <p style={S.hint}>Add at least one clue to help players solve the puzzle.</p>
        <ClueBuilder suspects={state.suspects} gridSize={state.gridSize}
          onAddClue={(clue) => dispatch({ type:"ADD_CLUE", payload:clue })} />
        {state.clues.length > 0 && (
          <div style={S.clueList}>
            <h4 style={{ color:"#aaa", margin:"16px 0 8px", fontSize:13 }}>Added Clues ({state.clues.length})</h4>
            {state.clues.map((clue) => {
              const primary = state.suspects.find((s) => s.id===clue.primarySuspectId);
              const secondary = state.suspects.find((s) => s.id===clue.secondarySuspectId);
              return (
                <div key={clue.id} style={S.clueItem}>
                  <div style={{ flex:1 }}>
                    <span style={{ color:"#6a4c93", fontWeight:700 }}>{CLUE_TYPE_LABELS[clue.type]}</span>
                    {" - "}<span style={{ color:primary?.color }}>{primary?.name}</span>
                    {CLUE_NEEDS_SECONDARY.includes(clue.type) && secondary && <> & <span style={{ color:secondary.color }}>{secondary.name}</span></>}
                    {clue.row!==undefined && <span style={{ color:"#aaa" }}> at row {clue.row+1}, col {clue.col!+1}</span>}
                  </div>
                  <button style={S.removeBtn} onClick={() => dispatch({ type:"REMOVE_CLUE", id:clue.id })}>✕</button>
                </div>
              );
            })}
          </div>
        )}
        <div style={S.navRow}>
          <button style={S.backBtn} onClick={() => setStep(2)}>← Back</button>
          <button style={S.nextBtn} disabled={state.clues.length===0} onClick={() => setStep(4)}>Next →</button>
        </div>
      </div>
    );
  }

  async function handleSubmit() {
    setSubmitting(true); setSubmitError("");
    try {
      const result = await submitPuzzle(state);
      setSubmitResult(result);
    } catch (err: unknown) {
      setSubmitError(err instanceof Error ? err.message : "Contract rejected the transaction. Please review and try again.");
    } finally {
      setSubmitting(false);
    }
  }

  function renderStep4() {
    if (submitResult) {
      return (
        <div style={S.card}>
          <div style={S.successBox}>
            <div style={{ fontSize:48 }}>🎉</div>
            <h2 style={{ color:"#06d6a0", margin:"8px 0 4px" }}>Puzzle Submitted!</h2>
            <p style={{ color:"#aaa", margin:"0 0 16px" }}>Your puzzle has been recorded on-chain.</p>
            <p style={{ color:"#fff" }}>Puzzle ID: <strong style={{ color:"#6a4c93" }}>{submitResult.puzzleId}</strong></p>
            <a href={`/play/${submitResult.puzzleId}`} style={S.playLink}>▶ Play this puzzle</a>
          </div>
        </div>
      );
    }
    return (
      <div style={S.card}>
        <h2 style={S.stepTitle}>Step 5 - Preview & Submit</h2>
        <div style={S.previewSummary}>
          {[["Puzzle Name",state.puzzleName],["Grid Size",`${state.gridSize}×${state.gridSize}`],["Difficulty",state.difficulty],
            ["Suspects",state.suspects.map((s)=>s.name).join(", ")],["Clues",state.clues.length]].map(([k,v]) => (
            <div key={String(k)} style={S.summaryRow}>
              <span style={{ color:"#888" }}>{k}</span>
              <span style={{ color:"#fff", fontWeight:600 }}>{v}</span>
            </div>
          ))}
        </div>
        <h3 style={{ color:"#aaa", fontSize:14, margin:"20px 0 8px" }}>Player View (solution hidden)</h3>
        <div style={{ ...S.solutionGrid, gridTemplateColumns:`repeat(${state.gridSize},1fr)`, marginBottom:16 }}>
          {Array.from({ length:state.gridSize }, (_,row) =>
            Array.from({ length:state.gridSize }, (_,col) => (
              <div key={`${row}-${col}`} style={{ ...S.cell, backgroundColor:"#111", color:"#555" }}>?</div>
            ))
          )}
        </div>
        <h3 style={{ color:"#aaa", fontSize:14, margin:"0 0 8px" }}>Suspects</h3>
        <div style={S.row}>
          {state.suspects.map((s) => <div key={s.id} style={{ ...S.palette, backgroundColor:s.color }}>{s.name}</div>)}
        </div>
        <h3 style={{ color:"#aaa", fontSize:14, margin:"16px 0 8px" }}>Clues</h3>
        <div style={S.clueList}>
          {state.clues.map((clue, i) => {
            const primary = state.suspects.find((s) => s.id===clue.primarySuspectId);
            const secondary = state.suspects.find((s) => s.id===clue.secondarySuspectId);
            return (
              <div key={clue.id} style={S.clueItem}>
                <span style={{ color:"#888", marginRight:8 }}>{i+1}.</span>
                <span style={{ color:"#6a4c93", fontWeight:700 }}>{CLUE_TYPE_LABELS[clue.type]}</span>
                {" - "}<span style={{ color:primary?.color }}>{primary?.name}</span>
                {CLUE_NEEDS_SECONDARY.includes(clue.type) && secondary && <> & <span style={{ color:secondary.color }}>{secondary.name}</span></>}
              </div>
            );
          })}
        </div>
        {submitError && <div style={S.errorBox}><strong>Submission failed:</strong> {submitError}</div>}
        <div style={S.navRow}>
          <button style={S.backBtn} onClick={() => setStep(3)}>← Back</button>
          <button style={{ ...S.nextBtn, opacity:submitting?0.6:1 }} disabled={submitting} onClick={handleSubmit}>
            {submitting ? "Submitting…" : "Submit to Chain ⛓"}
          </button>
        </div>
      </div>
    );
  }

  const stepContent = [renderStep0, renderStep1, renderStep2, renderStep3, renderStep4];

  return (
    <div style={S.page}>
      <h1 style={S.pageTitle}>Create Puzzle</h1>
      <StepIndicator steps={STEPS} currentStep={step} onStepClick={setStep} />
      {stepContent[step]()}
    </div>
  );
};

export default Create;
