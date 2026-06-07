import { CircleStop, Play, RotateCcw } from "lucide-react";
import type { Recipe, RunState } from "../types";

interface RunControlsProps {
  recipes: Recipe[];
  selectedRecipe: string;
  onRecipeChange: (recipeId: string) => void;
  runState: RunState | null;
  busy: boolean;
  onStart: () => void;
  onAbort: () => void;
}

export default function RunControls({
  recipes,
  selectedRecipe,
  onRecipeChange,
  runState,
  busy,
  onStart,
  onAbort,
}: RunControlsProps) {
  const active = runState?.status === "running";

  return (
    <section className="panel run-controls">
      <div className="panel-heading">
        <div>
          <span className="eyebrow">Run Controls</span>
          <h2>Layer sequence</h2>
        </div>
        <span className={`status-pill status-${runState?.status ?? "idle"}`}>
          <i />
          {runState?.status ?? "idle"}
        </span>
      </div>

      <label className="recipe-select">
        <span>Monitoring recipe</span>
        <select
          value={selectedRecipe}
          onChange={(event) => onRecipeChange(event.target.value)}
          disabled={active}
        >
          {recipes.map((recipe) => (
            <option key={recipe.id} value={recipe.id}>
              {recipe.name}
            </option>
          ))}
        </select>
      </label>

      <div className="run-control-actions">
        <button
          className="button button-primary"
          onClick={onStart}
          disabled={active || busy || !selectedRecipe}
        >
          {busy ? <RotateCcw className="spin" size={16} /> : <Play size={16} />}
          Start run
        </button>
        <button
          className="button button-danger"
          onClick={onAbort}
          disabled={!active || busy}
        >
          <CircleStop size={16} />
          Abort
        </button>
      </div>

      <div className="progress-block">
        <div>
          <span>Current stage</span>
          <strong>{runState?.stage ?? "Awaiting run"}</strong>
          <small>
            {Math.round((runState?.stage_progress ?? 0) * 100)}% stage complete
          </small>
        </div>
        <div
          className="progress-ring"
          style={{
            "--progress": `${(runState?.overall_progress ?? 0) * 360}deg`,
          } as React.CSSProperties}
        >
          <span>{Math.round((runState?.overall_progress ?? 0) * 100)}%</span>
        </div>
      </div>
    </section>
  );
}
