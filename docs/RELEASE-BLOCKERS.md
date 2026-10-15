# Beta and public distribution blocker register

Updated 2026-10-16 for M07.2. **Preparation complete does not mean acceptance
passed.** No external sessions have run in this milestone. M07.1 evidence is
historical; live GitHub configuration/artifact availability was not reverified
because `gh` is unavailable in this environment.

Severity: **S0 Critical** = suspected security boundary failure, unexpected
execution or data loss; stop affected sessions and triage privately. **S1 High** =
trust/install/crash/core-workflow blocker; blocks relevant acceptance and public
exit. **S2 Medium** = bounded defect with safe workaround, requires explicit owner
disposition. **S3 Low** = cosmetic/documentation defect. Missing mandatory evidence
is a release gate, not evidence of a product defect. No S0 defect is asserted here.

Status: Open / Investigating / Awaiting owner / Ready for retest / Resolved.
Only Resolved with exact candidate/platform retest evidence closes a blocker.
A waiver is recorded separately with rationale and scope; S0/S1 and mandatory
trust/data/acceptance gates cannot be waived for public distribution.

| ID / severity | Affected platform and gate                              | Reproduction or evidence gap                                                                                                                                                                                                                      | Resolution status / responsible role                 | Evidence required to close                                                                                                                                                                             |
| ------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| RB01 / S1     | Windows x64; trust and public exit                      | Ordinary CI produces unsigned EXE/MSI. No executed trusted Authenticode evidence.                                                                                                                                                                 | Awaiting owner — signing owner (person unassigned)   | Provision trusted lawful signing integration; verify expected signer, Valid SHA-256/RFC3161 signatures on app/EXE/MSI and downloaded clean Windows 10/11 SmartScreen behavior.                         |
| RB02 / S1     | Intel + Apple Silicon macOS 15+; trust and public exit  | Ordinary CI uses ad-hoc signing; no trusted Developer ID/notarization run. Gatekeeper acceptance remains unverified.                                                                                                                              | Awaiting owner — signing owner (person unassigned)   | Developer ID signed app/DMG, accepted notarization, stapled tickets, codesign/spctl verification and quarantined browser download acceptance on both CPUs.                                             |
| RB03 / S1     | Release infrastructure; signed distribution             | M07.1 found no release-signing environment/secrets/rulesets; M07.2 could not refresh live state. Current workflow requires protected exact tag.                                                                                                   | Awaiting owner — repository owner                    | Fresh read-only configuration audit, independent reviewer, prevent-self-review, no admin bypass, exact approved Tag rule and restricted tag creation/update/deletion; secret names only, never values. |
| RB04 / S1     | Windows 10/11 x64 NSIS/MSI; full manual acceptance      | Hosted Windows Server UI/upgrade smoke is insufficient. Clean Windows desktop full feature/dialog/upgrade/uninstall and installed/missing/offline WebView2 cases lack complete evidence.                                                          | Open — QA coordinator (person unassigned)            | Completed acceptance sheets with OS builds, both package hashes, trust prompts, runtime cases and actual upgrade/restart/data retention; no security overrides.                                        |
| RB05 / S1     | Intel + Apple Silicon macOS 15+; full manual acceptance | Hosted mount/copy/AX/restart smoke is not full Finder/trust/feature/upgrade/uninstall acceptance. Exact 15.0 floor and other intended versions lack results.                                                                                      | Open — QA coordinator (person unassigned)            | Full sheets on both CPUs and intended versions including native dialogs, backup/agent consent flows, real upgrade and trust with quarantine intact.                                                    |
| RB06 / S1     | Linux x86_64 real GPU/Wayland; shutdown stability       | M07.1 native agent --beta functional suite passed but WebKit PID 402253 crashed at shutdown in extracted WebKit/host graphics mix. Distribution-matched Ubuntu Xvfb checks passed; hardware root cause unresolved. Do not rerun on personal data. | Investigating — Linux maintainer (person unassigned) | Matched distribution runtime, real desktop/GPU/Wayland install/close/restart/durability results for DEB/AppImage; characterize/fix remaining crash and retest. See M07.1 Linux stability section.      |
| RB07 / S1     | All six installers; cohort handoff/provenance           | Actions candidates and upgrade baseline expire after 30 days. Existing hashes are pinned but availability not checked in M07.2.                                                                                                                   | Open — coordinator/build owner (people unassigned)   | Verify available exact bytes/manifests/source; preserve reviewed baseline in approved storage or rebuild and review new hashes. Repeat acceptance for changed bytes; never skip upgrade.               |
| RB08 / S1     | All platforms; cohort operations and beta exit          | No participants, consent, assigned private channel/window/retention deadline or external acceptance results. Program intentionally not started.                                                                                                   | Awaiting owner — coordinator (person unassigned)     | Owner assigns coordinator/channel/window/deadline; 5–10 explicit opt-ins, alias-only roster and required per-package coverage; sanitized results and triage, withdrawal honored.                       |

Unsigned/ad-hoc candidates may be evaluated only within a separately authorized
controlled session where ordinary security policy permits. If blocked, stop and
record the result; RB01/RB02 remain open. No owner can turn a security override
into a passing trust test. A Linux session need not wait for a macOS credential,
but every relevant handoff must pass its provenance/privacy/safety checks.

## New blocker / retest record

Copy for each new defect or evidence update; store sensitive evidence privately.
Never include credentials, private repositories, raw backups or full logs.

```text
ID / title / severity / rationale:
Status / responsible owner alias / target review date:
Affected OS versions, architecture and installer formats:
Candidate version / source SHA / installer SHA-256:
Acceptance row IDs / sanitized bug reference:
Reproduction steps with sample data:
Expected / actual / frequency / impact:
Sanitized evidence location:
Resolution or proposed fix / commit:
Retest candidate hash / OS / UTC date / tester alias:
Retest steps, actual result, remaining limits:
Owner disposition / closure date (only after evidence review):
```

Coordinator triages at each agreed session checkpoint. S0 goes privately through
[SECURITY](../SECURITY.md) when applicable; pause affected distribution and tests.
For S1, stop the failing scenario and assign an owner before retrying. Retain the
original failure, fix and retest as separate records. A green CI run does not
close a manual gate. [Exit criteria](BETA-EXIT.md) apply before any public release.
