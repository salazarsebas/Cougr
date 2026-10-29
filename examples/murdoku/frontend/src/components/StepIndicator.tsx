import { Check } from 'lucide-react';
import type { CreateStep } from '../types';

const STEPS: { key: CreateStep; label: string; index: number }[] = [
  { key: 'config',   label: 'Configure',  index: 0 },
  { key: 'solution', label: 'Solution',   index: 1 },
  { key: 'clues',    label: 'Clues',      index: 2 },
  { key: 'review',   label: 'Review',     index: 3 },
];

interface StepIndicatorProps {
  currentStep: CreateStep;
}

export function StepIndicator({ currentStep }: StepIndicatorProps) {
  const currentIndex = STEPS.findIndex((s) => s.key === currentStep);

  return (
    <nav aria-label="Create puzzle steps" style={{ padding: '0 0 1.5rem' }}>
      <ol
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 0,
          listStyle: 'none',
        }}
      >
        {STEPS.map((step, i) => {
          const isCompleted = i < currentIndex;
          const isActive = i === currentIndex;
          const isFuture = i > currentIndex;

          const dotColor = isCompleted
            ? 'var(--accent-gold)'
            : isActive
            ? 'var(--noir-text)'
            : 'var(--noir-border)';

          const labelColor = isCompleted
            ? 'var(--accent-gold)'
            : isActive
            ? 'var(--noir-text)'
            : 'var(--noir-muted)';

          return (
            <li
              key={step.key}
              style={{ display: 'flex', alignItems: 'center', flex: i < STEPS.length - 1 ? 1 : 'initial' }}
            >
              {/* Step dot */}
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.3rem' }}>
                <div
                  aria-current={isActive ? 'step' : undefined}
                  aria-label={`Step ${i + 1}: ${step.label}${isCompleted ? ', completed' : isActive ? ', current' : ''}`}
                  style={{
                    width: 28, height: 28,
                    borderRadius: '50%',
                    border: `2px solid ${dotColor}`,
                    background: isCompleted ? 'var(--accent-gold)' : isActive ? 'rgba(232,224,208,0.08)' : 'transparent',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    transition: 'border-color 200ms, background 200ms',
                    flexShrink: 0,
                  }}
                >
                  {isCompleted ? (
                    <Check size={13} style={{ color: '#0f0f0f' }} aria-hidden="true" />
                  ) : (
                    <span
                      style={{
                        fontFamily: 'var(--font-mono)',
                        fontSize: '0.6875rem',
                        fontWeight: 600,
                        color: isFuture ? 'var(--noir-border)' : dotColor,
                      }}
                    >
                      {i + 1}
                    </span>
                  )}
                </div>

                <span
                  style={{
                    fontFamily: 'var(--font-mono)',
                    fontSize: '0.6875rem',
                    color: labelColor,
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                    whiteSpace: 'nowrap',
                    transition: 'color 200ms',
                  }}
                >
                  {step.label}
                </span>
              </div>

              {/* Connector line */}
              {i < STEPS.length - 1 && (
                <div
                  aria-hidden="true"
                  style={{
                    flex: 1,
                    height: 1,
                    marginBottom: 22,
                    background: isCompleted ? 'var(--accent-gold)' : 'var(--noir-border)',
                    transition: 'background 200ms',
                  }}
                />
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
