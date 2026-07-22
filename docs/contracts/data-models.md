# Data model sketch

Core Course Planning objects:

- `ProgramPlan`
- `RequirementGroup`
- `RequirementRule`
- `CourseIdentity`
- `CourseAlias`
- `CourseOffering`
- `InstructorIdentity`
- `ReviewReference`
- `UserAcademicSnapshot`
- `UserPreference`
- `PlanCandidate`
- `PlanRationale`
- `SourceRevision`
- `FactProvenance`
- `ConflictRecord`

`CourseIdentity` uses normalized course code as the primary anchor but must support same-name/different-code, old-code/new-code, same-code/different-version, and unresolved cross-source aliases.
