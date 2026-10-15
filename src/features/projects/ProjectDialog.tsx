import { useState } from 'react';
import { Dialog } from '../../components/ui/dialog';
import { Button } from '../../components/ui/button';
import { Layers, Code2, Rocket, Box, Globe, Sparkles } from 'lucide-react';
const icons = {
  layers: Layers,
  code: Code2,
  rocket: Rocket,
  box: Box,
  globe: Globe,
  sparkles: Sparkles,
};
import { useAppStore } from '../../app/store';
import { api } from '../../lib/ipc';
import type { Project, ProjectInput, ProjectIcon, Status } from '../../lib/ipc';
const colors = [
  '#8b5cf6',
  '#3b82f6',
  '#10b981',
  '#f59e0b',
  '#ef4444',
  '#ec4899',
];
export function ProjectDialog({
  project,
  workspaceId,
  onClose,
}: {
  project?: Project;
  workspaceId: string;
  onClose: () => void;
}) {
  const [input, setInput] = useState<ProjectInput>(
    project
      ? {
          workspaceId,
          name: project.name,
          description: project.description,
          icon: project.icon,
          color: project.color,
          status: project.status,
        }
      : {
          workspaceId,
          name: '',
          description: '',
          icon: 'layers',
          color: colors[0],
          status: 'planned',
        },
  );
  const { busy, error, mutate } = useAppStore();
  const update = <K extends keyof ProjectInput>(
    key: K,
    value: ProjectInput[K],
  ) => setInput((old) => ({ ...old, [key]: value }));
  return (
    <Dialog
      title={project ? 'Edit project' : 'New project'}
      description="Give your next idea a place to take shape."
      onClose={onClose}
      busy={busy}
    >
      <form
        onSubmit={async (event) => {
          event.preventDefault();
          if (await mutate(() => api.saveProject(project?.id ?? null, input)))
            onClose();
        }}
      >
        <fieldset disabled={busy} className="form-fields">
          <label htmlFor="project-name">Project name</label>
          <input
            id="project-name"
            required
            maxLength={100}
            autoFocus
            value={input.name}
            onChange={(e) => update('name', e.target.value)}
            placeholder="e.g. Platform redesign"
          />
          <label htmlFor="project-description">
            Description <span className="muted">(optional)</span>
          </label>
          <textarea
            id="project-description"
            maxLength={4000}
            rows={3}
            value={input.description}
            onChange={(e) => update('description', e.target.value)}
            placeholder="What are you building?"
          />
          <div className="form-columns">
            <div>
              <label htmlFor="project-status">Status</label>
              <select
                id="project-status"
                value={input.status}
                onChange={(e) => update('status', e.target.value as Status)}
              >
                <option value="planned">Planned</option>
                <option value="active">Active</option>
                <option value="paused">Paused</option>
                <option value="completed">Completed</option>
              </select>
            </div>
            <div>
              <label htmlFor="project-color">Color</label>
              <div className="color-options">
                {colors.map((color) => (
                  <button
                    type="button"
                    key={color}
                    style={{ background: color }}
                    aria-label={color}
                    aria-pressed={input.color === color}
                    className="color-swatch"
                    onClick={() => update('color', color)}
                  />
                ))}
                <input
                  id="project-color"
                  type="color"
                  aria-label="Custom color"
                  value={input.color}
                  onChange={(e) => update('color', e.target.value)}
                />
              </div>
            </div>
          </div>
          <label>Project icon</label>
          <div role="group" aria-label="Project icon" className="icon-options">
            {Object.entries(icons).map(([name, Icon]) => (
              <button
                key={name}
                type="button"
                aria-label={name}
                aria-pressed={input.icon === name}
                className="icon-choice"
                onClick={() => update('icon', name as ProjectIcon)}
              >
                <Icon size={20} />
              </button>
            ))}
          </div>
        </fieldset>
        {error && (
          <p role="alert" className="form-error">
            {error}
          </p>
        )}
        <div className="dialog-actions">
          <Button
            type="button"
            variant="secondary"
            disabled={busy}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button disabled={busy || !input.name.trim()}>
            {busy ? 'Saving…' : project ? 'Save changes' : 'Create project'}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
