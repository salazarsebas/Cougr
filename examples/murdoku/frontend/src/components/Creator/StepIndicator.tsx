import React from "react";

interface Step { label: string; }
interface StepIndicatorProps {
  steps: Step[];
  currentStep: number;
  onStepClick: (index: number) => void;
}

const StepIndicator: React.FC<StepIndicatorProps> = ({ steps, currentStep, onStepClick }) => (
  <div style={{ display:"flex", alignItems:"center", justifyContent:"center", marginBottom:32, flexWrap:"wrap" }}>
    {steps.map((step, i) => {
      const isCompleted = i < currentStep;
      const isActive = i === currentStep;
      return (
        <React.Fragment key={i}>
          <button
            onClick={() => isCompleted && onStepClick(i)}
            disabled={!isCompleted}
            style={{ display:"flex", flexDirection:"column", alignItems:"center", gap:4, background:"none", border:"none",
              color: isActive ? "#fff" : isCompleted ? "#6a4c93" : "#888",
              fontSize:12, padding:"4px 8px", cursor: isCompleted ? "pointer" : "default" }}
          >
            <span style={{ width:32, height:32, borderRadius:"50%", border:"2px solid currentColor",
              display:"flex", alignItems:"center", justifyContent:"center", fontWeight:700, fontSize:14 }}>
              {isCompleted ? "✓" : i + 1}
            </span>
            <span style={{ whiteSpace:"nowrap", fontSize:11 }}>{step.label}</span>
          </button>
          {i < steps.length - 1 && (
            <div style={{ width:32, height:2, marginBottom:18, backgroundColor: isCompleted ? "#6a4c93" : "#333" }} />
          )}
        </React.Fragment>
      );
    })}
  </div>
);

export default StepIndicator;
