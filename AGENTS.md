# Repository agent guidance

## Research experiments and prototypes

- Preserve reproducible experiments, prototypes, fixtures, and referenced
  measurements in the appropriate dedicated directory under `spikes/`.
- Research documents should link to the checked-in experiment or fixture used
  to support a claim.
- Do not delete an experiment directory merely because its current run is
  complete. Keep the reusable source, harness, inputs, and any result fixture
  cited by documentation.
- Add a narrow `.gitignore` in the experiment area for regenerable local data
  such as interpreter caches, compiler outputs, and scratch work. Do not ignore
  referenced results or evidence needed to reproduce a documented conclusion.
- Temporary operating-system directories are acceptable for isolated execution
  only when the checked-in harness reconstructs them. Cleanup should target
  regenerable run products, never the preserved experiment itself.
