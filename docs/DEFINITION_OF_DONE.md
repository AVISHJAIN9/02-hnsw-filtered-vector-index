# Definition of Done (DoD)

To maintain strict delivery quality and rigorous engineering standards, every feature in this project must adhere to a strict definition of done.

## Strict Rules for Feature Completion

A feature is considered **DONE** if and only if **all** of the following five conditions are met:

1. **Every acceptance-criteria sentence has a passing test**:
   - Each individual clause or sentence within the feature's acceptance criteria must be covered by unit, integration, or property-based tests.
   - Tests must actively verify the required behavior under normal and edge conditions.
2. **The KPI is measured and the number recorded**:
   - The key performance indicator (KPI) specified in `docs/PROGRESS.md` must be empirically measured.
   - The concrete measurement (e.g. latency, recall number, throughput, 0 leak verification, etc.) must be recorded alongside the feature delivery evidence.
3. **No TODO / stub / unimplemented in its code**:
   - Code implementing the feature must be complete and production-grade.
   - No placeholders, mock shortcuts, `todo!()`, `unimplemented!()`, or lingering TODO comments are permitted in the delivered feature code.
4. **The full test suite passes**:
   - All tests across the entire workspace must pass cleanly without regressions.
   - Code must compile with zero errors and pass lints.
5. **Real command output is pasted as evidence**:
   - Verifiable terminal output from executing `cargo test`, benchmark runs, or validation commands must be captured and documented as reproducible proof.

---

## Allowed Status Values

Only the following three status values are permitted in `docs/PROGRESS.md`:

- `NOT_STARTED` — No work has begun on this feature.
- `IN_PROGRESS` — The feature is actively being designed, implemented, or tested.
- `DONE` — All five criteria above have been strictly fulfilled and verified.

> **Important**: Intermediate, ambiguous, or qualifying statuses (such as "mostly", "partial", "almost done", "blocked", or "90%") are strictly forbidden. A feature is either `NOT_STARTED`, `IN_PROGRESS`, or `DONE`.
