# Private beta manual acceptance sheet

Copy one sheet per OS/architecture/package/upgrade pair into the agreed private
channel. Read [BETA](BETA.md) first. **All rows below start Not run**; an empty
checkbox is not a pass. Record Pass / Fail / Blocked / Not run for every ID,
UTC execution time, sanitized observation/evidence and bug/blocker ID. A skipped
optional agent test stays Not run; another assigned tester must cover it for exit.
Never infer manual acceptance from CI or a previous milestone.

```text
Tester alias (no real name):
Consent date / session UTC date:
OS edition, full version/build, CPU architecture:
Linux desktop/display backend/GPU and WebKitGTK version (if applicable):
Windows WebView2 version / initially installed or missing (if applicable):
macOS exact point version (15+), native Intel or Apple Silicon:
Forma version / candidate source SHA / coordinator handoff ID:
Installer file / format / SHA-256 / hash comparison result:
Install scope / sanitized path / clean account or prior test install:
Upgrade from version + installer SHA-256, if assigned:
Per row: ID | status | observed vs expected | UTC time | sanitized evidence/bug ID
```

Use sample workspace `Beta Lab`, project `Acceptance` and two sample issues. For
Git, create a new local folder using your Git client, initialize it without a
remote, add a harmless README and make an initial commit with a test identity.
Create a second local branch/commit. Do not clone a private repository, include
credentials or link a folder containing personal files. Keep this repository for
the session; backups do not contain its files. Agent execution remains optional
for each individual and requires separate informed agreement.

| ID  | Action and required observation                                                                                                                                                                                                                                                     | Initial status |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| I01 | Compare full SHA-256 to reviewed manifest; select correct CPU/package. Mismatch stops the session.                                                                                                                                                                                  | Not run        |
| I02 | Install in clean test account using guide; record scope/path, runtime provisioning and every trust prompt. A blocked prompt stops launch; leave remaining rows Blocked or Not run with reason.                                                                                      | Not run        |
| I03 | First launch renders an empty state, usable focus and no crash; create no data until this is recorded.                                                                                                                                                                              | Not run        |
| W01 | Create `Beta Lab` and `Acceptance`; switch workspace, rename workspace, edit project; changes persist. Invalid blank names show errors and retain edits.                                                                                                                            | Not run        |
| W02 | Archive/restore project. In separate throwaway records, cancel deletion and verify retention, then confirm deletion and verify removal/cascade notice. Never delete the main sample before restart/backup tests.                                                                    | Not run        |
| Q01 | Create two issues, edit title/Markdown/status/priority/due date; add label and subtask. Verify identifier, preview, activity and combined search/filters, then clear filters.                                                                                                       | Not run        |
| Q02 | Switch to Board; move between columns and reorder with pointer and keyboard (Alt+arrows or Space/arrows/Space). Open with Enter. List and Board agree after refresh.                                                                                                                | Not run        |
| Q03 | Cancel unsaved edits, archive/restore an issue, cancel then confirm deletion of a separate disposable issue; inspect confirmation and retained children.                                                                                                                            | Not run        |
| G01 | Select only the disposable repository in project Git panel; inspect status, branches and commit history; cancel native picker once without changing the connection.                                                                                                                 | Not run        |
| G02 | In a saved issue, review and confirm branch/commit links. Disconnect/reconnect: historical links remain readable. No remote operation or automatic commit occurs.                                                                                                                   | Not run        |
| A01 | With separate agent-session consent, select a trusted supported CLI in AI agents & runs. Record provider/version only. Authenticate externally, never in feedback. Preview exact task/repository/permissions; cancel preview: no process or worktree run starts.                    | Not run        |
| A02 | Preview a harmless README-only task in the disposable repository; explicitly approve. Observe lifecycle, bounded logs and retained worktree. Review diff/results, record accept/reject for manual integration. Main working tree remains untouched; no automatic commit/merge/push. | Not run        |
| A03 | Approve a separate cancellable task; Cancel, observe terminal status and stopped ordinary descendants, then review retained changes. If it finishes too quickly, mark cancellation Not run and arrange a controlled fixture with coordinator.                                       | Not run        |
| A04 | Explicitly approve queued tasks; observe no execution until Start Next / Start Approved Queue. Stop dispatch and cancel active work as needed; distinguish stopping dispatch from cancelling processes.                                                                             | Not run        |
| A05 | Optionally select a trusted independent test runner, review exact arguments/cwd, separately approve and inspect its result separately from provider claims. Never run arbitrary downloaded scripts.                                                                                 | Not run        |
| A06 | For a stopped, reviewed, clean known worktree, cancel cleanup once, then explicitly confirm cleanup; branch/history remain. Dirty/unknown worktree cleanup must not be forced.                                                                                                      | Not run        |
| B01 | After agent rows if performed, export all sample data with native dialog to protected local storage. Cancel export once. Verify success; do not attach the file.                                                                                                                    | Not run        |
| B02 | Change sample issue, select exported backup, cancel replacement: change remains. Repeat and confirm replacement: exported content returns, frontend refreshes and recovery location is shown. Recovery must exist before considering restore passed.                                | Not run        |
| B03 | Restore recovery copy and verify pre-restore change returns. After each restore, historical Git/runs remain readable but repository/executable/worktree authorization is cleared; select folders/executables again for any further use. No queued task resumes.                     | Not run        |
| B04 | Select a harmless non-backup text file: reject with useful error and retain current data. Do not corrupt a real database or simulate power loss.                                                                                                                                    | Not run        |
| R01 | Close normally, reopen: workspace/project/issue/view/order/appearance persist without crash. Repeat close/reopen; inspect shutdown symptoms as well as the UI. Historical runs remain readable, unused approvals invalidate, unfinished execution must not auto-resume.             | Not run        |
| R02 | Switch light/dark/system, test keyboard focus, dialogs, empty/loading/error states and 760×540 scrolling. Record accessibility problems and missing state coverage.                                                                                                                 | Not run        |
| U01 | Only with coordinator-assigned reviewed baseline and new package: export sample backup, close, upgrade in same scope, verify data/IDs/order/history then restart. Record both hashes and actual destination. Same-version reinstall is not upgrade evidence.                        | Not run        |
| U02 | Uninstall normally; verify app removed and sample app-data retained. Optionally reinstall verified same package to confirm retention, then uninstall. Preserve worktree files before deliberate test-data cleanup.                                                                  | Not run        |

Windows package coverage must include separate NSIS and MSI sessions on Windows
10/11; coordinator also assigns clean missing-WebView2 and unavailable-network
bootstrap cases. Record safe failure instead of weakening policy. macOS requires
both CPUs with quarantine intact; Linux requires both formats and real desktop
GPU/Wayland close/restart evidence. Full acceptance on the exact macOS 15.0 floor
and any other intended OS version remains unverified until recorded.

If data loss, unexpected execution, crash or trust failure occurs, stop the
affected scenario, record a [bug](BETA-REPORT.md) and ask the coordinator to triage
before resuming. Keep data local; submit only sanitized observations. The
[blocker register](RELEASE-BLOCKERS.md) tracks unresolved failures, not this blank
sheet. No checklist results have been collected by M07.2.
