import React, { useState } from "react";
import { ClueType, CLUE_TYPE_LABELS, CLUE_NEEDS_SECONDARY, CLUE_NEEDS_COORDS, type Clue, type Suspect } from "../../types";

interface ClueBuilderProps {
  suspects: Suspect[];
  gridSize: number;
  onAddClue: (clue: Clue) => void;
}

const ClueBuilder: React.FC<ClueBuilderProps> = ({ suspects, gridSize, onAddClue }) => {
  const [clueType, setClueType] = useState<ClueType>(ClueType.SameRow);
  const [primaryId, setPrimaryId] = useState("");
  const [secondaryId, setSecondaryId] = useState("");
  const [row, setRow] = useState(0);
  const [col, setCol] = useState(0);
  const [error, setError] = useState("");

  const needsSecondary = CLUE_NEEDS_SECONDARY.includes(clueType);
  const needsCoords = CLUE_NEEDS_COORDS.includes(clueType);
  const coords = Array.from({ length: gridSize }, (_, i) => i);

  const s: Record<string, React.CSSProperties> = {
    wrapper: { background:"#1a1a2e", border:"1px solid #333", borderRadius:8, padding:20, display:"flex", flexDirection:"column", gap:8 },
    label: { fontSize:12, color:"#aaa", marginBottom:2 },
    select: { background:"#0f0f23", color:"#fff", border:"1px solid #444", borderRadius:4, padding:"8px 10px", fontSize:14, width:"100%" },
    btn: { marginTop:4, background:"#6a4c93", color:"#fff", border:"none", borderRadius:6, padding:"10px 0", cursor:"pointer", fontSize:14, fontWeight:600 },
  };

  function handleAdd() {
    if (!primaryId) { setError("Select a primary suspect."); return; }
    if (needsSecondary && !secondaryId) { setError("Select a secondary suspect."); return; }
    if (needsSecondary && secondaryId === primaryId) { setError("Suspects must be different."); return; }
    setError("");
    onAddClue({
      id: crypto.randomUUID(),
      type: clueType,
      primarySuspectId: primaryId,
      secondarySuspectId: needsSecondary ? secondaryId : undefined,
      row: needsCoords ? row : undefined,
      col: needsCoords ? col : undefined,
    });
    setPrimaryId(""); setSecondaryId(""); setRow(0); setCol(0);
  }

  return (
    <div style={s.wrapper}>
      <h3 style={{ margin:"0 0 8px", color:"#fff", fontSize:16 }}>Add a Clue</h3>
      <label style={s.label}>Clue Type</label>
      <select style={s.select} value={clueType} onChange={(e) => { setClueType(e.target.value as ClueType); setSecondaryId(""); }}>
        {Object.values(ClueType).map((t) => <option key={t} value={t}>{CLUE_TYPE_LABELS[t]}</option>)}
      </select>
      <label style={s.label}>Primary Suspect</label>
      <select style={s.select} value={primaryId} onChange={(e) => setPrimaryId(e.target.value)}>
        <option value=""> - select - </option>
        {suspects.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
      </select>
      {needsSecondary && <>
        <label style={s.label}>Secondary Suspect</label>
        <select style={s.select} value={secondaryId} onChange={(e) => setSecondaryId(e.target.value)}>
          <option value=""> - select - </option>
          {suspects.filter((s) => s.id !== primaryId).map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
      </>}
      {needsCoords && (
        <div style={{ display:"flex", gap:12 }}>
          <div style={{ flex:1 }}>
            <label style={s.label}>Row</label>
            <select style={s.select} value={row} onChange={(e) => setRow(Number(e.target.value))}>
              {coords.map((i) => <option key={i} value={i}>Row {i+1}</option>)}
            </select>
          </div>
          <div style={{ flex:1 }}>
            <label style={s.label}>Column</label>
            <select style={s.select} value={col} onChange={(e) => setCol(Number(e.target.value))}>
              {coords.map((i) => <option key={i} value={i}>Col {i+1}</option>)}
            </select>
          </div>
        </div>
      )}
      {error && <p style={{ color:"#e63946", fontSize:13, margin:0 }}>{error}</p>}
      <button style={s.btn} onClick={handleAdd}>+ Add Clue</button>
    </div>
  );
};

export default ClueBuilder;
