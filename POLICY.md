# FDS Code Governance Policy

**Effective 2026.09.09**

## 1. Scope and purpose

This policy governs repository changes in FDS-led code projects. A
**change** may affect code, tests, configuration, build or release workflows,
documentation, or integration. Anyone may use a project under its license,
raise an issue, propose a design, submit a pull request, or review work.
Project-wide governance remains with FDS; family and component authority may
be delegated under the project's eligibility rules in `GOVERNANCE.md`.

This policy defines technical authority, escalation, appointments, and
succession. Each project's current assignments are recorded in
[GOVERNANCE.md](./GOVERNANCE.md); community moderation is governed separately by
the [Code of Conduct](./CODE_OF_CONDUCT.md).

## 2. Roles and ownership

| Role | Authority |
| --- | --- |
| **Project Maintainer** | Repository-wide policy, permissions, releases, security, licensing, and cross-family decisions |
| **Family Maintainer** | Senior technical authority for every component in a family |
| **Component Maintainer** | Delegated technical authority and first contact for one component |
| **Reviewer or Steward** | Review and guidance without final governance or merge authority |

The Project Maintainer appoints or removes Family Maintainers. A Family
Maintainer may act directly, approve or block merges, appoint or remove
Component Maintainers, publish public family rules, and delegate ordinary
merges. Component Maintainers follow those rules. Family rules may not conflict
with FDS or project-wide policy, the license, the Code of Conduct, security
rules, or required CI.
No maintainer may solely approve their own change; every affected family must
approve a cross-family change. Questions escalate through Component, Family,
and Project Maintainers.

### Live-QA assignment

| PR owner | Questioner |
| --- | --- |
| Contributor, Reviewer, or Steward | Responsible Component Maintainer |
| Component Maintainer | Responsible Family Maintainer |
| Family Maintainer | Project Maintainer |
| Project Maintainer | A different Family Maintainer who did not author the change and understands the affected work |

For project-level work without a Component assignment, the Project Maintainer
is the authority and questioner; the final row applies when they own the PR.
When several people qualify, their lowest common higher authority designates a
non-author to lead and keep the record; others may co-question. This does not
replace affected-family approval or grant project authority to a Family
Maintainer questioning the Project Maintainer.

### Branch definitions

| Branch | Definition and authority |
| --- | --- |
| `master` | Canonical integration and release branch |
| Personal working branch | A Contributor's or Maintainer's branch; no naming rule; may target `theme/xxx` or `master` |
| `theme/xxx` | Integration branch opened only by the Project Maintainer for cross-family work or a Family Maintainer within their family; may nest to reflect the development hierarchy |

## 3. Pull request governance

This chapter is binding for PRs targeting `theme/xxx` or `master`.
[CONTRIBUTING.md](./CONTRIBUTING.md#pull-request-workflow) provides the derived
workflow and, where linked, incorporated engineering standards; the
[form specification](./governance/forms/README.md) controls record structure,
fields, and completion. This document controls conflicts concerning authority,
applicability, approval, merge eligibility, retention, and enforcement.
Neither subordinate document may create a role, merge gate, approval, or
policy exception.

### Applicability and required records

| Source and target | Update Report | Live QA |
| --- | --- | --- |
| Personal working branch to `theme/xxx` | Required | Required |
| Personal working branch to `master` | Required | Required |
| Nested theme branch to its parent theme | Required | Required |
| `theme/xxx` directly to `master` | Not required; may be submitted voluntarily | Required |
| `master` to `theme/xxx` for the permitted final sync | Required | Required |

Personal-to-personal merges are outside this procedure. These routes never
permit self-approval or waive affected-family approval.
Using [GOVERNANCE.md](./GOVERNANCE.md) and Chapter 2, the PR owner identifies
every scope, AI tier, questioner, and approval. A Contributor may transfer work
to another Contributor's or Maintainer's personal branch; authorship remains,
but PR and record responsibility transfers.

### Change scope, branch history, and validation

- Follow the applicable AI tier and
  [code standards](./CONTRIBUTING.md#code-standards). Commit mechanical and
  semantic changes separately.
- A personal-branch PR should contain approximately ten Update Report units.
  Split independently valid changes; coupling introduced to avoid splitting is
  forbidden.
- Run every applicable check and update tests and documentation when contracts
  change. Record `N/A` with a reason only when a check cannot apply. A final
  theme state must pass full CI.
- Changes enter `theme/xxx` and `master` only through PRs. Direct commits block
  merge. Each PR identifies its scope, affected components, split rationale,
  and validation; a final theme PR also links staged PRs and their authors.
- A theme owner must review, understand, explain, modify, test, and debug every
  integrated change. Theme staging is not project acceptance.

### Update Report policy

The PR owner is responsible for the accuracy and judgment in every required or
voluntary report, must understand its covered work, and completes it by hand in
ink without AI assistance before giving it to the questioner. Ordinary and
nested-theme entries cover the smallest independent design decision; units
without one may be grouped. Voluntary final-theme reports and required final
`master` sync reports cover integration decisions and risks without repeating
previously reviewed entries. The
[form specification](./governance/forms/README.md) defines units, grouping,
fields, notation, and completion.

### Independent review and live QA

The assigned questioner must independently review the PR, its validation, and
every required or voluntarily submitted report, then prepare questions
privately. The questioner must understand the work covered by the QA record and
is responsible for the accuracy and technical judgment recorded in it. For a
final `theme/xxx`-to-`master` PR, the review must also establish all of the
following:

1. **Record completeness:** the theme history contains no unaccounted-for
   direct commit, and every merge commit is traceable to its PR, required
   records, and author.
2. **Merge residuals:** every change introduced by human conflict resolution
   has been identified and reviewed.
3. **Integration seams:** interactions among staged changes have been reviewed,
   and the final theme state has passed full CI.

The assigned questioner leads and keeps the QA record. Other affected
maintainers may co-question within their areas under the designation rule in
Chapter 2.

QA must occur live on a real-time meeting platform. The PR owner answers
without AI assistance and must not complete the QA record. The questioner
completes the record by hand in ink and without AI assistance. QA is
question-oriented and risk-based rather than report-unit-oriented: one
question may cover several related units, and one unit may receive several
questions. The questioner may probe any area, including understanding beyond
doubts recorded by the PR owner, and signs the completed record.

The form specification defines the QA fields, question-entry semantics, and
completion method. It may not limit the questioner's review scope or waive the
independence and live-session requirements above.

### Findings, approval, and merge

The PR owner must resolve every required finding and update any affected
report; reviewers verify the result. Review, CI, or QA repeats as directed, and
an unresolved finding blocks merge. Before merge, the responsible authorities
confirm CI, independent and cross-family approval, and live QA.

Theme branches accept only authorship-preserving merge commits from personal
or nested-theme PRs and one final PR-based merge commit from `master`; squash
merges are forbidden. Merge follows completed review and QA.

### Record retention

The PR owner keeps each original Update Report, and the questioner keeps each
original QA record, for four months after merge. During the first three months,
the Project Maintainer may request delivery by providing the address and
service; the project shall pay all fees incurred by delivery. Otherwise
delivery is unnecessary.
Failure to retain or produce a requested original may cause it to be treated as
unverified or potentially false and referred for governance review.

### Hotfix variation

A change that must reach `master` outside the theme cycle may bypass staging.
The Project Maintainer coordinates with the responsible Component Maintainer;
the direct-to-`master` records remain required, but review and QA may be
expedited. QA covers every affected module, and its outcome controls merge.
It need not contain a separate question for each module.

## 4. Project, release, and RFC governance

Project-level scope, reviewers, component ownership, and AI assignments are
recorded in [GOVERNANCE.md](./GOVERNANCE.md). Examples inherit the ownership and
AI declaration of the components they demonstrate.

Release scope, readiness, and timing are decided at the Wednesday and Sunday
coordination meetings under the
[FDS Administrator Rules](https://doc.fds.moe/policies/admin/). The Project
Maintainer records and carries out the release decision.

Each Family Maintainer defines their family RFC process. Family rules may be
stricter, but must preserve these requirements:

1. A patch-level update that changes only the final version component, such as
   `0.a.b` to `0.a.c`, must not break a stable API.
2. A major or breaking public API change must be proposed to the community and
   discussed at an internal meeting before approval.

## 5. AI declarations

FDS supports AI-assisted and AI-copilot development. The human responsible
for a change **must** understand and be able to explain every part of its
design and implementation, regardless of which tools helped produce it. They
must also be able to modify, test, and debug the work without asking an AI
system to reconstruct it for them.

FDS does not scan code for an "AI rate," estimate the percentage of code
generated by AI, or use such a percentage as a merge criterion. AI tiers
describe the kind of collaboration, not the amount of generated text. Merge
review instead uses every required or voluntarily submitted Update Report,
live QA, technical review, and required CI to assess the design rationale,
semantics, risks, compatibility, and the responsible human's command of the
code. This careful process is how FDS makes human accountability and its
engineering philosophy visible to other contributors.

| Tier | Definition |
| --- | --- |
| **Forbidden** | Code design, proofs, program semantics, and novel implementation logic are human-authored. Documentation-only AI assistance is outside AI usage. |
| **Author-Owned** | AI may assist with drafts or completion; the human owns the design and committed work. |
| **Human-Led** | The human writes the structure and load-bearing logic; AI may assist with helpers and boilerplate. |
| **Co-Authored** | AI may assist with design and implementation; the human must fully internalize the result. |

Each Family Maintainer chooses and updates the declarations for components in
their family. When scopes inside one component use different tiers, the more
specific declaration applies. The per-component tier assignments are recorded
in [GOVERNANCE.md](./GOVERNANCE.md).

A documentation-only project-level change uses the **Forbidden** tier. Other
project-level work uses the assignment recorded in `GOVERNANCE.md`.

## 6. Eligibility and succession

Each project records eligibility for Project Maintainer, Family Maintainer,
Component Maintainer, Reviewer or Steward, and Contributor in `GOVERNANCE.md`.
Eligibility may include project-specific partner organizations or restrict a
role to a narrower group; a project may declare the Reviewer or Steward role
unavailable. Eligibility does not itself confer authority: appointment follows
Chapter 2, and the Project Maintainer is appointed and succeeded under the
[FDS Charter](https://doc.fds.moe/policies/constitution/). Project eligibility
rules must comply with FDS policy.

A maintainer planning to resign or take leave must arrange a successor or
acting candidate for confirmation by the next higher authority. For an
unexpected vacancy, authority temporarily moves upward. Loss or expiration of
the membership required for a role suspends maintainer authority immediately;
every transition must be recorded in `GOVERNANCE.md` and repository permissions.
This applies the succession principle in the
[FDS Administrator Rules](https://doc.fds.moe/policies/admin/).
