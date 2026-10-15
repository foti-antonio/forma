# Private beta exit checklist

All gates are **unchecked** at M07.2 preparation. The owner reviews exact source
and final package hashes; no inferred passes, silent waivers or substituted CI
results. This checklist defines readiness for public distribution, not permission
to create a tag, publish a release or merge main.

- [ ] Coordinator has recorded 5–10 voluntary external testers, consent and
      platform assignments privately. Private channel, test window and retention
      deadline were communicated; withdrawal requests were honored.
- [ ] Acceptance sheets cover all six installers: Linux x86_64 AppImage/DEB,
      Windows x64 NSIS/MSI on Windows 10/11, Intel and Apple Silicon macOS 15+.
      Exact OS builds and the advertised minimum/intended versions are recorded;
      unverified support claims are resolved by testing or explicit support review.
- [ ] Each assigned sheet records hash, source, OS/CPU, installation/first launch,
      workspace/project, issues/Kanban/keyboard, disposable Git linking, sample
      backup/restore/recovery, restart, real upgrade and uninstall/data retention.
      Agent approval/cancellation/results, explicit queue dispatch, independent
      approval of tests and safe worktree cleanup have designated consenting
      coverage on each platform; skipped rows have no implicit pass.
- [ ] Windows installed/missing WebView2 and unavailable-network bootstrap cases
      have recorded outcomes; native picker/save/restore/error/confirmation flows,
      accessibility focus, appearance and minimum-window states are verified.
- [ ] Linux distribution-matched real GPU/Wayland close/restart and durability
      evidence resolves RB06; software-rendered CI alone is insufficient.
- [ ] All S0/S1 defects and mandatory evidence gaps are resolved with exact
      candidate retests. Every S2/S3 has an owner disposition, rationale and any
      safe workaround in release notes. No required test is left Blocked/Not run.
- [ ] Regression checks from README pass for final code; upgrade/restart checks
      preserve append-only migrations, SQLite integrity and user data. No expanded
      capabilities, automatic execution, remote persistence or security bypass.
- [ ] Owner has completed and freshly audited environment/reviewer/tag protection
      and lawful Windows/Apple signing prerequisites in
      [RELEASING](RELEASING.md#m072-owner-action-record-before-any-signing-run).
- [ ] After separate exact-source/version **tag authorization**, the protected
      workflow has rebuilt all six packages, executed all gates and produced only
      an unpublished draft. Final source/manifests/checksums are verified. Signed
      bytes receive fresh acceptance; unsigned hashes do not certify them.
- [ ] Final Windows app/EXE/MSI have trusted expected signatures/timestamps;
      both macOS architectures have Developer ID/notarization/stapling evidence.
      Browser-downloaded packages pass ordinary SmartScreen/Gatekeeper checks on
      clean desktops, with quarantine and security policies intact.
- [ ] Owner has reviewed sanitized cohort findings, remaining documented host AI
      execution risks, unencrypted backup/retention warnings, privacy choices,
      known issues and final installation/release notes. Credentials/private
      repositories/raw backups/sensitive logs have not been collected.
- [ ] Owner has reviewed the final signed artifacts and explicitly authorized
      public publication in a separate recorded decision. Main merge needs its
      own explicit authorization. Neither is performed by this milestone.

Record review UTC date, responsible owner, candidate/source/hashes, links to
sanitized evidence and decision in the release record. Keep contact and consent
records private. Apply the agreed deletion deadline after beta closure; retain
only sanitized technical evidence. If a gate fails, reopen its blocker and repeat
the affected checks. Do not relabel preparation as completed acceptance.
