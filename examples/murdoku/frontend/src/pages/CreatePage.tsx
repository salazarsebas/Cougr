import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowLeft, ArrowRight, Plus, Trash2, Send } from 'lucide-react';
import { StepIndicator } from '../components/StepIndicator';
import type { CreateStep, Suspect, Clue } from '../types';

const DEFAULT_SUSPECTS: Suspect[] = [
  { id: 1, name: 'Col. Hargrove', color: '#6b3a2a', initials: 'CH' },
  { id: 2, name: 'Lady Voss',     color: '#1e3a5f', initials: 'LV' },
  { id: 3, name: 'Dr. Fenwick',   color: '#2a4a2a', initials: 'DF' },
  { id: 4, name: 'Miss Crane',    color: '#4a2a5f', initials: 'MC' },
];

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--noir-surface)',
  border: '1px solid var(--noir-border)',
  borderRadius: 4,
  padding: '0.5rem 0.75rem',
  color: 'var(--noir-text)',
  fontFamily: 'var(--font-mono)',
  fontSize: '0.875rem',
  outline: 'none',
  transition: 'border-color 150ms',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  fontFamily: 'var(--font-mono)',
  fontSize: '0.6875rem',
  color: 'var(--noir-muted)',
  textTransform: 'uppercase' as const,
  letterSpacing: '0.08em',
  marginBottom: '0.375rem',
};

const sectionStyle: React.CSSProperties = {
  background: 'var(--noir-surface)',
  border: '1px solid var(--noir-border)',
  borderRadius: 6,
  padding: '1.5rem',
  marginBottom: '1.25rem',
};

export function CreatePage() {
  const navigate = useNavigate();
  const [step, setStep] = useState<CreateStep>('config');
  const [gridSize, setGridSize] = useState<4 | 5>(4);
  const [title, setTitle] = useState('');
  const [suspects, setSuspects] = useState<Suspect[]>(DEFAULT_SUSPECTS.slice(0, 4));
  const [solution, setSolution] = useState<number[]>(Array(16).fill(0));
  const [clues, setClues] = useState<Clue[]>([]);
  const [newClueDesc, setNewClueDesc] = useState('');
  const [submitted, setSubmitted] = useState(false);

  function handleGridSizeChange(size: 4 | 5) {
    setGridSize(size);
    setSolution(Array(size * size).fill(0));
    setSuspects(DEFAULT_SUSPECTS.slice(0, size));
  }

  function handleSolutionCell(idx: number, val: string) {
    const num = parseInt(val, 10);
    setSolution(prev => prev.map((v, i) => i === idx ? (isNaN(num) ? 0 : Math.min(num, gridSize)) : v));
  }

  function addClue() {
    if (!newClueDesc.trim()) return;
    const clue: Clue = {
      id: `clue-${Date.now()}`,
      type: 'not_same_row',
      suspectAId: suspects[0]?.id ?? 1,
      suspectBId: suspects[1]?.id ?? 2,
      description: newClueDesc.trim(),
    };
    setClues(prev => [...prev, clue]);
    setNewClueDesc('');
  }

  function removeClue(id: string) {
    setClues(prev => prev.filter(c => c.id !== id));
  }

  function handlePublish() {
    setSubmitted(true);
  }

  const steps: CreateStep[] = ['config', 'solution', 'clues', 'review'];
  const stepIndex = steps.indexOf(step);

  function goNext() { if (stepIndex < steps.length - 1) setStep(steps[stepIndex + 1]); }
  function goPrev() { if (stepIndex > 0) setStep(steps[stepIndex - 1]); }

  if (submitted) {
    return (
      <main id="main-content" style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '2rem' }}>
        <div style={{ textAlign: 'center', maxWidth: 420 }}>
          <div style={{ width: 56, height: 56, borderRadius: '50%', background: 'rgba(201,168,76,0.12)', border: '1px solid var(--accent-gold)', display: 'flex', alignItems: 'center', justifyContent: 'center', margin: '0 auto 1.25rem' }}>
            <Send size={22} style={{ color: 'var(--accent-gold)' }} />
          </div>
          <h1 style={{ fontFamily: 'var(--font-serif)', fontSize: '1.75rem', color: 'var(--accent-gold)', marginBottom: '0.75rem' }}>Case Filed</h1>
          <p style={{ fontFamily: 'var(--font-serif)', fontStyle: 'italic', color: 'var(--noir-muted)', marginBottom: '1.75rem', lineHeight: 1.6 }}>
            Your puzzle has been submitted to the on-chain registry. The detectives await.
          </p>
          <button className="btn-gold" onClick={() => navigate('/')} style={{ margin: '0 auto' }}>
            Back to Case Files
          </button>
        </div>
      </main>
    );
  }

  return (
    <main id="main-content" style={{ flex: 1, padding: '2rem 1.25rem', maxWidth: 720, margin: '0 auto', width: '100%' }}>
      {/* Back */}
      <button id="btn-create-back" className="btn-outline" onClick={() => navigate('/')} style={{ marginBottom: '1.5rem' }}>
        <ArrowLeft size={14} aria-hidden="true" /> Case Files
      </button>

      <h1 style={{ fontFamily: 'var(--font-serif)', fontSize: 'clamp(1.5rem, 4vw, 2rem)', marginBottom: '0.375rem', color: 'var(--noir-text)' }}>
        New Case
      </h1>
      <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--noir-muted)', marginBottom: '2rem' }}>
        Craft a murder mystery puzzle and publish it on-chain.
      </p>

      <StepIndicator currentStep={step} />

      {/* ── Step 1: Config ── */}
      {step === 'config' && (
        <section aria-labelledby="step-config-heading">
          <h2 id="step-config-heading" style={{ fontFamily: 'var(--font-serif)', fontSize: '1.125rem', marginBottom: '1.25rem', color: 'var(--noir-text)' }}>
            Configure the Case
          </h2>

          <div style={sectionStyle}>
            <div style={{ marginBottom: '1rem' }}>
              <label htmlFor="input-title" style={labelStyle}>Case Title</label>
              <input id="input-title" type="text" value={title} onChange={e => setTitle(e.target.value)}
                placeholder="e.g. The Blackwood Manor Affair" style={inputStyle}
                onFocus={e => (e.target.style.borderColor = 'var(--accent-gold)')}
                onBlur={e => (e.target.style.borderColor = 'var(--noir-border)')} />
            </div>

            <div>
              <p style={labelStyle} id="grid-size-label">Grid Size</p>
              <div role="radiogroup" aria-labelledby="grid-size-label" style={{ display: 'flex', gap: '0.75rem' }}>
                {([4, 5] as const).map(size => (
                  <button key={size} id={`btn-grid-${size}`} role="radio" aria-checked={gridSize === size}
                    onClick={() => handleGridSizeChange(size)}
                    style={{ padding: '0.5rem 1.25rem', border: `1px solid ${gridSize === size ? 'var(--accent-gold)' : 'var(--noir-border)'}`, borderRadius: 4, background: gridSize === size ? 'rgba(201,168,76,0.1)' : 'transparent', color: gridSize === size ? 'var(--accent-gold)' : 'var(--noir-muted)', fontFamily: 'var(--font-mono)', fontSize: '0.875rem', cursor: 'pointer', transition: 'border-color 150ms, color 150ms, background 150ms' }}>
                    {size}×{size}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </section>
      )}

      {/* ── Step 2: Solution ── */}
      {step === 'solution' && (
        <section aria-labelledby="step-solution-heading">
          <h2 id="step-solution-heading" style={{ fontFamily: 'var(--font-serif)', fontSize: '1.125rem', marginBottom: '0.5rem', color: 'var(--noir-text)' }}>
            Enter the Solution Grid
          </h2>
          <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--noir-muted)', marginBottom: '1.25rem' }}>
            Fill each cell with a suspect number (1–{gridSize}). Each number must appear exactly once per row and column.
          </p>

          <div style={sectionStyle}>
            <div role="grid" aria-label="Solution grid" style={{ display: 'grid', gridTemplateColumns: `repeat(${gridSize}, 1fr)`, gap: 6, maxWidth: gridSize === 4 ? 260 : 320 }}>
              {solution.map((val, idx) => (
                <input key={idx} id={`solution-cell-${idx}`} type="number" min={1} max={gridSize} value={val || ''}
                  aria-label={`Row ${Math.floor(idx / gridSize) + 1}, Col ${(idx % gridSize) + 1}`}
                  onChange={e => handleSolutionCell(idx, e.target.value)}
                  style={{ ...inputStyle, textAlign: 'center', padding: '0.5rem 0', aspectRatio: '1', width: '100%' }}
                  onFocus={e => (e.target.style.borderColor = 'var(--accent-gold)')}
                  onBlur={e => (e.target.style.borderColor = 'var(--noir-border)')} />
              ))}
            </div>

            {/* Suspect legend */}
            <div style={{ marginTop: '1.25rem', display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
              {suspects.map(s => (
                <span key={s.id} style={{ fontFamily: 'var(--font-mono)', fontSize: '0.6875rem', padding: '0.2rem 0.5rem', borderRadius: 3, background: `${s.color}25`, color: s.color, border: `1px solid ${s.color}50` }}>
                  {s.id} = {s.name}
                </span>
              ))}
            </div>
          </div>
        </section>
      )}

      {/* ── Step 3: Clues ── */}
      {step === 'clues' && (
        <section aria-labelledby="step-clues-heading">
          <h2 id="step-clues-heading" style={{ fontFamily: 'var(--font-serif)', fontSize: '1.125rem', marginBottom: '1.25rem', color: 'var(--noir-text)' }}>
            Add Evidence Clues
          </h2>

          <div style={sectionStyle}>
            <label htmlFor="input-clue" style={labelStyle}>New Clue</label>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <input id="input-clue" type="text" value={newClueDesc} onChange={e => setNewClueDesc(e.target.value)}
                placeholder="e.g. The Colonel was not in the same wing as the Doctor."
                onKeyDown={e => { if (e.key === 'Enter') addClue(); }}
                style={{ ...inputStyle, flex: 1 }}
                onFocus={e => (e.target.style.borderColor = 'var(--accent-gold)')}
                onBlur={e => (e.target.style.borderColor = 'var(--noir-border)')} />
              <button id="btn-add-clue" className="btn-gold" onClick={addClue} aria-label="Add clue" style={{ flexShrink: 0 }}>
                <Plus size={14} aria-hidden="true" /> Add
              </button>
            </div>
          </div>

          {clues.length > 0 && (
            <ol aria-label="Added clues" style={{ listStyle: 'none', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              {clues.map((clue, i) => (
                <li key={clue.id} style={{ display: 'flex', alignItems: 'flex-start', gap: '0.75rem', background: 'var(--noir-surface)', border: '1px solid var(--noir-border)', borderRadius: 4, padding: '0.625rem 0.875rem' }}>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.6875rem', color: 'var(--noir-muted)', flexShrink: 0, paddingTop: 2 }}>
                    {String(i + 1).padStart(2, '0')}.
                  </span>
                  <p style={{ fontFamily: 'var(--font-serif)', fontStyle: 'italic', fontSize: '0.875rem', color: 'var(--noir-muted)', flex: 1 }}>
                    {clue.description}
                  </p>
                  <button onClick={() => removeClue(clue.id)} aria-label={`Remove clue ${i + 1}`}
                    style={{ background: 'none', border: 'none', color: 'var(--noir-muted)', cursor: 'pointer', padding: '0.125rem', flexShrink: 0 }}>
                    <Trash2 size={13} />
                  </button>
                </li>
              ))}
            </ol>
          )}

          {clues.length === 0 && (
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--noir-muted)', textAlign: 'center', padding: '1.5rem' }}>
              No clues added yet. A puzzle without clues is unsolvable.
            </p>
          )}
        </section>
      )}

      {/* ── Step 4: Review ── */}
      {step === 'review' && (
        <section aria-labelledby="step-review-heading">
          <h2 id="step-review-heading" style={{ fontFamily: 'var(--font-serif)', fontSize: '1.125rem', marginBottom: '1.25rem', color: 'var(--noir-text)' }}>
            Review & Publish
          </h2>

          <div style={sectionStyle}>
            <dl style={{ display: 'flex', flexDirection: 'column', gap: '0.875rem' }}>
              {[
                { label: 'Title',       value: title || '(untitled)' },
                { label: 'Grid Size',   value: `${gridSize}×${gridSize}` },
                { label: 'Suspects',    value: suspects.map(s => s.name).join(', ') },
                { label: 'Clues',       value: `${clues.length} clue${clues.length !== 1 ? 's' : ''}` },
                { label: 'Solution',    value: solution.some(v => v === 0) ? '⚠ Incomplete - some cells are empty' : '✓ Complete' },
              ].map(({ label, value }) => (
                <div key={label} style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
                  <dt style={{ ...labelStyle, marginBottom: 0, minWidth: 80, flexShrink: 0 }}>{label}</dt>
                  <dd style={{ fontFamily: 'var(--font-mono)', fontSize: '0.875rem', color: value.startsWith('⚠') ? 'var(--accent-red)' : value.startsWith('✓') ? 'var(--accent-green)' : 'var(--noir-text)' }}>
                    {value}
                  </dd>
                </div>
              ))}
            </dl>
          </div>

          {solution.some(v => v === 0) && (
            <p role="alert" style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8125rem', color: 'var(--accent-red)', marginBottom: '1rem' }}>
              The solution grid has empty cells. Go back and complete it before publishing.
            </p>
          )}
        </section>
      )}

      {/* Navigation buttons */}
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', marginTop: '1.5rem' }}>
        <button id="btn-step-prev" className="btn-outline" onClick={goPrev} disabled={stepIndex === 0}
          style={{ opacity: stepIndex === 0 ? 0.3 : 1, cursor: stepIndex === 0 ? 'not-allowed' : 'pointer' }}
          aria-label="Previous step">
          <ArrowLeft size={14} aria-hidden="true" /> Back
        </button>

        {step !== 'review' ? (
          <button id="btn-step-next" className="btn-gold" onClick={goNext} aria-label="Next step">
            Next <ArrowRight size={14} aria-hidden="true" />
          </button>
        ) : (
          <button id="btn-publish" className="btn-gold" onClick={handlePublish}
            disabled={solution.some(v => v === 0)} aria-label="Publish puzzle to on-chain registry">
            <Send size={14} aria-hidden="true" /> Publish Case
          </button>
        )}
      </div>
    </main>
  );
}
