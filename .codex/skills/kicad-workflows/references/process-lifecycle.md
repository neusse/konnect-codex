# Application and Helper-Process Lifecycle

Use this gate whenever a workflow launches, restarts, or depends on KiCad
editors, Freerouting, Java, a simulator, or another external helper. The goal is
an owned session: the workflow knows what it started, preserves pre-existing
user sessions, and leaves no task-owned window or orphan behind.

## Ownership baseline

Before the first launch or restart:

1. List the relevant application windows and matching processes. Record each
   process ID, parent process, command line, target project or board when
   observable, and whether it existed before the workflow.
2. Classify every instance as `pre-existing`, `task-owned`, or `ownership
   unknown`. Never infer ownership from an executable name alone.
3. Require the domain's live-owner rule before mutation. If an unexpected
   editor, lock, or helper already owns the target, stop and use the project
   recovery workflow instead of starting another instance.
4. Record every application or helper launched after the baseline, including
   child processes created indirectly by Konnect, an ActionPlugin, or a probe.

## During execution

- Reconcile the previous instance before every restart. A crash or failed tool
  call does not authorize stacking another editor, MCP server, Java process, or
  helper on top of an unresolved one.
- Give probes and external commands a bounded timeout. After a timeout, inspect
  the exact child and its descendants; terminate only the task-owned process
  tree and record the timeout as failed evidence.
- Keep the active Konnect MCP/companion process that supplies the current task.
  Multiple same-name processes can belong to different Codex tasks or agents;
  parentage and command line, not count alone, determine ownership.
- When desktop control is available, close visible applications normally and
  re-observe after every action. Use forceful termination only for a verified
  task-owned orphan that remains after normal close and a short bounded grace
  period.

## Completion cleanup gate

After the requested artifact and validation evidence are saved:

1. Confirm the intended project files are saved and no unresolved save,
   import, or settings dialog remains.
2. Close task-owned secondary windows first, then their parent application.
3. Wait a short bounded grace period and inventory the relevant windows and
   processes again.
4. For each survivor, verify its process ID, parent, command line, and baseline
   classification. Terminate an exact task-owned orphan; preserve pre-existing
   and current-session infrastructure.
5. Complete only when no task-owned application window or helper process
   remains. Report what was closed, what was terminated, what was deliberately
   left running, and why.

If ownership is ambiguous, do not kill the process. Return `INCOMPLETE` with
the observed identifiers and the smallest user decision needed. Cleanup does
not authorize discarding unsaved work or closing an application the user had
open before the workflow.
