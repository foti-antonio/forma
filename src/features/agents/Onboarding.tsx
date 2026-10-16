import { useEffect, useState } from 'react';
import { Button } from '../../components/ui/button';
import { Dialog } from '../../components/ui/dialog';
import {
  errorMessage,
  type GitRepository,
  type OnboardingStatus,
} from '../../lib/ipc';
import { agents } from './store';
import {
  agentExplanation,
  nextRequirement,
  setupSteps,
  type SetupStep,
} from './onboarding-state';

export function AgentError({ details }: { details: string }) {
  return (
    <div role="alert">
      <p>{agentExplanation(details)}</p>
      <details>
        <summary>Technical diagnostics</summary>
        <pre className="agent-output">{details}</pre>
      </details>
    </div>
  );
}
export function Onboarding({
  projectId,
  initialProvider = 'codex',
  onClose,
  onReady,
}: {
  projectId?: string;
  initialProvider?: string;
  onClose: () => void;
  onReady?: (provider: string) => void | Promise<void>;
}) {
  const [provider, setProvider] = useState(initialProvider);
  const [step, setStep] = useState<SetupStep>('provider');
  const [status, setStatus] = useState<OnboardingStatus | null>(null);
  const [repository, setRepository] = useState<GitRepository | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [login, setLogin] = useState('idle');
  const [notice, setNotice] = useState('');
  const [checkedPermissions, setCheckedPermissions] = useState(false);
  const refresh = async () => {
    const data = await agents.onboarding(provider);
    setStatus(data);
    setLogin(data.login);
    const connected = projectId ? await agents.connection(projectId) : null;
    setRepository(connected);
    return { status: data, repository: connected };
  };
  const perform = async (action: () => Promise<void>) => {
    setBusy(true);
    setError('');
    try {
      await action();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  useEffect(() => {
    if (login !== 'running') return;
    let stale = false;
    const timer = setInterval(() => {
      void agents
        .loginStatus()
        .then((state) => {
          if (!stale) setLogin(state);
        })
        .catch((e) => {
          if (!stale) setError(errorMessage(e));
        });
    }, 1000);
    return () => {
      stale = true;
      clearInterval(timer);
    };
  }, [login]);
  const closeSetup = () => {
    if (login === 'running')
      void perform(async () => {
        await agents.cancelLogin();
        onClose();
      });
    else onClose();
  };
  const name = provider === 'codex' ? 'Codex' : 'Claude Code';
  const guide =
    provider === 'codex'
      ? 'https://developers.openai.com/codex/cli/'
      : 'https://code.claude.com/docs/en/quickstart';
  const requirement = status
    ? nextRequirement(status, !projectId || !!repository?.trusted)
    : 'installation';
  return (
    <Dialog
      title="Set up AI"
      description="Choose a provider, authorize its installation, sign in, and connect your project. Every task still needs your approval."
      onClose={closeSetup}
      onReturnFocus={() =>
        document
          .getElementById(projectId ? 'issue-run-ai' : 'agent-setup')
          ?.focus()
      }
      busy={busy}
      className="issue-dialog"
    >
      <p role="status" aria-live="polite">
        Step {setupSteps.indexOf(step) + 1} of {setupSteps.length}: {step}
      </p>
      {busy && <p role="status">Checking setup…</p>}
      {error && <AgentError details={error} />}
      {notice && <p role="status">{notice}</p>}
      {step === 'provider' && (
        <>
          <label htmlFor="setup-provider">Choose your AI provider</label>
          <select
            id="setup-provider"
            value={provider}
            disabled={busy || login === 'running'}
            onChange={(e) => {
              setProvider(e.target.value);
              setStatus(null);
              setCheckedPermissions(false);
              setError('');
              setNotice('');
            }}
          >
            <option value="codex">Codex</option>
            <option value="claude">Claude Code</option>
          </select>
          <p>
            Provider inference may use the internet and your paid provider
            account. Forma never asks for passwords, API keys or tokens.
          </p>
          <Button
            disabled={busy}
            onClick={() =>
              void perform(async () => {
                await refresh();
                setStep('installation');
              })
            }
          >
            Find installation
          </Button>
        </>
      )}
      {step === 'installation' && status && (
        <>
          <h3>Install and authorize {name}</h3>
          <p>
            {status.platform === 'windows'
              ? 'Windows: use a native .exe installation. PowerShell, .cmd and .bat wrappers are not supported; do not disable Windows security.'
              : status.platform === 'macos'
                ? 'macOS: use the official native installation or Homebrew. Both Apple Silicon and Intel Homebrew locations are checked.'
                : 'Linux: use the official native installation instructions for your distribution and architecture.'}
          </p>
          <p>
            Installation is manual. Forma does not download installers, execute
            installation commands, change PATH or request administrator
            privileges.
          </p>
          <p className="git-target">{guide}</p>
          <Button
            variant="secondary"
            disabled={busy}
            onClick={() =>
              void perform(async () => {
                await agents.setupDocs(provider);
              })
            }
          >
            Open official installation guide
          </Button>
          {provider === 'claude' && status.platform !== 'linux' && (
            <>
              <label htmlFor="install-command">
                Optional package manager command (run yourself)
              </label>
              <input
                id="install-command"
                readOnly
                value={
                  status.platform === 'windows'
                    ? 'winget install Anthropic.ClaudeCode'
                    : 'brew install --cask claude-code'
                }
              />
              <p>
                Requires {status.platform === 'windows' ? 'WinGet' : 'Homebrew'}{' '}
                already installed. Select the command to copy it.
              </p>
            </>
          )}
          <p>
            Prerequisites not found on PATH:{' '}
            {status.prerequisites.join(', ') || 'none'}. Node/npm are optional
            for native installs; Git is needed for project setup. These are file
            checks, not version verification.
          </p>
          <p>
            Discovery:{' '}
            {status.candidates.length
              ? `${status.candidates.length} installation(s) found`
              : 'No supported executable found'}
            . Authorized: {status.authorized ? 'yes' : 'no'}. Compatible:{' '}
            {status.compatible ? 'yes' : 'not verified'}.
          </p>
          <p>
            Authorize only software you trust. Authorization permits
            version/help and authentication status commands; a task requires a
            separate approval. SHA-256 identifies bytes, not the publisher.
          </p>
          {status.candidates.map((c) => (
            <article key={c.token}>
              <p className="git-target">{c.path}</p>
              <p className="git-target">SHA-256: {c.sha256}</p>
              <Button
                disabled={busy}
                onClick={() =>
                  void perform(async () => {
                    await agents.authorizeDiscovered(c.token);
                    await refresh();
                  })
                }
              >
                Authorize this installation: {c.path}
              </Button>
            </article>
          ))}
          <Button
            disabled={busy}
            variant="secondary"
            onClick={() =>
              void perform(async () => {
                await agents.authorize(provider);
                await refresh();
              })
            }
          >
            Choose another executable…
          </Button>
          <Button
            disabled={busy}
            variant="secondary"
            onClick={() =>
              void perform(async () => {
                await refresh();
              })
            }
          >
            Rescan installations
          </Button>
          {status.diagnostics && <AgentError details={status.diagnostics} />}
          <Button
            disabled={busy || !status.authorized || !status.compatible}
            onClick={() => setStep('authentication')}
          >
            Continue to sign-in
          </Button>
        </>
      )}
      {step === 'authentication' && status && (
        <>
          <h3>Sign in to {name}</h3>
          <p>Authentication: {status.authentication}</p>
          <p>
            Use the provider’s browser window to sign in. Credentials stay with
            the provider. Forma runs only{' '}
            {provider === 'codex' ? 'codex login' : 'claude auth login'} after
            your click; it never stores login output. The flow ends after three
            minutes.
          </p>
          <Button
            disabled={busy || login === 'running' || !status.compatible}
            onClick={() =>
              void perform(async () => {
                await agents.login(provider);
                setLogin('running');
              })
            }
          >
            Open {name} login
          </Button>
          <p role="status">Login flow: {login}</p>
          {login === 'running' && (
            <Button
              variant="secondary"
              onClick={() =>
                void perform(async () => {
                  await agents.cancelLogin();
                })
              }
            >
              Cancel login
            </Button>
          )}
          {['failed', 'cancelled'].includes(login) && (
            <p>
              Login did not finish. Retry, or open a terminal and run the login
              command above. Forma cannot display interactive fallback codes.
              Then check authentication.
            </p>
          )}
          <Button
            disabled={busy || login === 'running'}
            variant="secondary"
            onClick={() =>
              void perform(async () => {
                await refresh();
              })
            }
          >
            Check authentication
          </Button>
          <p>
            Unknown or unavailable status means Forma could not confirm a
            session. Update the CLI or use its documented login, then check
            again. A confirmed session does not guarantee available credits or
            network access.
          </p>
          <Button
            disabled={busy || status.authentication !== 'authenticated'}
            onClick={() => setStep(projectId ? 'repository' : 'permissions')}
          >
            Continue
          </Button>
        </>
      )}
      {step === 'repository' && (
        <>
          <h3>Connect your project</h3>
          <p>
            Select the existing Git folder for this project. This authorizes
            Forma to inspect it and prepare a separate working copy only after
            you approve a task. Your current checkout stays in place.
          </p>
          <p>
            {repository?.trusted
              ? `Connected: ${repository.path}`
              : 'Repository not connected or awaiting renewed authorization.'}
          </p>
          <Button
            disabled={busy}
            onClick={() =>
              void perform(async () => {
                if (projectId) {
                  await agents.connect(projectId);
                  setRepository(await agents.connection(projectId));
                }
              })
            }
          >
            Select Git folder…
          </Button>
          <p>
            Use a clean repository with a committed branch. Commit or preserve
            existing changes yourself. Forma never discards them.
          </p>
          <Button
            disabled={busy || !repository?.trusted}
            onClick={() => setStep('permissions')}
          >
            Continue to permissions
          </Button>
        </>
      )}
      {step === 'permissions' && (
        <>
          <h3>Conservative permissions</h3>
          <p>
            Recommended: one process, a ten-minute limit, bounded logs,{' '}
            {provider === 'codex'
              ? 'Codex workspace-write restrictions with no escalation'
              : 'Claude restricted file tools with no shell or MCP tools'}
            . Each task needs explicit execution approval; Claude also needs
            separate consent for host filesystem risk.
          </p>
          <p role="note">
            Git worktrees are not OS security sandboxes. Providers run as your
            OS user and may access host files, credentials and network.
            Requiring whole-provider OS isolation blocks execution in this
            build.
          </p>
          <Button
            disabled={busy}
            variant="secondary"
            onClick={() =>
              void perform(async () => {
                const settings = (await agents.providers()).find(
                  (p) => p.settings.provider === provider,
                )?.settings;
                if (!settings)
                  throw new Error(
                    'Provider settings unavailable. Retry setup.',
                  );
                await agents.saveSettings({
                  ...settings,
                  policy: provider === 'codex' ? 'restricted' : 'claude-tools',
                  maxRuntime: 600,
                  logLimit: 65536,
                });
                await agents.setConcurrency(1);
                await refresh();
                setNotice(
                  'Recommended limits saved. Existing OS isolation requirements are preserved.',
                );
              })
            }
          >
            Apply recommended configuration
          </Button>
          <label>
            <input
              type="checkbox"
              checked={checkedPermissions}
              onChange={(e) => setCheckedPermissions(e.target.checked)}
            />
            I understand these permissions and will review each task before
            approving execution.
          </label>
          {status?.diagnostics && <AgentError details={status.diagnostics} />}
          <Button
            disabled={busy || !checkedPermissions}
            onClick={() =>
              void perform(async () => {
                const current = await refresh();
                const next = nextRequirement(
                  current.status,
                  !projectId || !!current.repository?.trusted,
                );
                setStep(next);
                if (next !== 'ready')
                  setNotice(
                    'Complete the highlighted requirement before continuing.',
                  );
              })
            }
          >
            Check readiness
          </Button>
        </>
      )}
      {step === 'ready' && (
        <>
          <h3>
            {name} is ready
            {projectId ? ' for task preview' : ' to connect to a project'}
          </h3>
          <p>
            {projectId
              ? 'Next, review the exact prompt, repository, executable and permissions. Only your final approval starts execution and prepares the working copy.'
              : 'Open a saved issue and choose Run with AI to connect its Git folder, preview the task and approve execution.'}
          </p>
          <Button
            disabled={busy || requirement !== 'ready'}
            onClick={() =>
              void perform(async () => {
                const current = await refresh();
                if (
                  nextRequirement(
                    current.status,
                    !projectId || !!current.repository?.trusted,
                  ) !== 'ready'
                ) {
                  setStep(
                    nextRequirement(
                      current.status,
                      !projectId || !!current.repository?.trusted,
                    ),
                  );
                  return;
                }
                await agents.savePreferred(provider);
                if (onReady) await onReady(provider);
                else onClose();
              })
            }
          >
            {projectId ? 'Preview first task' : 'Finish setup'}
          </Button>
        </>
      )}
      <div className="dialog-actions">
        <Button
          variant="secondary"
          disabled={busy || step === 'provider' || login === 'running'}
          onClick={() => {
            setStep(
              !projectId && step === 'permissions'
                ? 'authentication'
                : setupSteps[Math.max(0, setupSteps.indexOf(step) - 1)],
            );
            setNotice('');
          }}
        >
          Back
        </Button>
        <Button variant="secondary" disabled={busy} onClick={closeSetup}>
          Close setup
        </Button>
      </div>
    </Dialog>
  );
}
