import type { OnboardingStatus } from '../../lib/ipc';
export type SetupStep =
  | 'provider'
  | 'installation'
  | 'authentication'
  | 'repository'
  | 'permissions'
  | 'ready';
export const setupSteps: SetupStep[] = [
  'provider',
  'installation',
  'authentication',
  'repository',
  'permissions',
  'ready',
];
export function nextRequirement(
  status: OnboardingStatus,
  repository: boolean,
): SetupStep {
  if (!status.authorized || !status.compatible) return 'installation';
  if (status.authentication !== 'authenticated') return 'authentication';
  if (!repository) return 'repository';
  if (!status.ready) return 'permissions';
  return 'ready';
}
export function agentExplanation(details: string): string {
  const value = details.toLowerCase();
  if (/changed|identity/.test(value))
    return 'The executable or repository changed. Rescan and authorize the trusted installation or reconnect the folder, then preview again.';
  if (
    /not installed|on path|choose a provider|select and authorize/.test(value)
  )
    return 'Set up your coding provider: install it, rescan, and authorize the trusted executable.';
  if (/security options|capability|identify as/.test(value))
    return 'This CLI is incompatible with Forma’s required security controls. Update from the official provider guide and authorize it again.';
  if (/auth|logged in/.test(value))
    return 'Sign in through the provider’s official login flow, then check authentication again.';
  if (/local changes|dirty/.test(value))
    return 'Your repository has unsaved Git changes. Commit or preserve them yourself, then retry. Forma will not discard them.';
  if (/repository|reconnect/.test(value))
    return 'Connect your project’s Git folder. Use a clean committed branch without submodules or tracked symlinks, then retry.';
  if (/runtime|timeout|timed out/.test(value))
    return 'The process reached its time limit. Check the provider connection and review any retained changes before retrying.';
  if (/policy|permission|isolation|approval|risk/.test(value))
    return 'Review the permission settings and required consent. Forma cannot provide whole-provider OS isolation and will keep that requirement blocked.';
  if (/unsuccessfully|could not start|failed|failure/.test(value))
    return 'The provider could not complete this operation. Check its installation and sign-in, then inspect available logs before retrying.';
  return details;
}
