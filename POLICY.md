# FDS Code Governance Policy

**Effective 2026.08.19**

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

### Review assignment

| PR owner | Assigned reviewer |
| --- | --- |
| Contributor, Reviewer, or Steward | Responsible Component Maintainer |
| Component Maintainer | Responsible Family Maintainer |
| Family Maintainer | Project Maintainer |
| Project Maintainer | A different Family Maintainer who did not author the change and understands the affected work |

“Assigned reviewer” in this policy is a functional designation for one PR. It
does not appoint the person to the formal Reviewer or Steward role.
When the PR owner holds more than one role, apply the table using the highest
role that is applicable to any affected scope of the PR.

For project-level work without a Component assignment, the Project Maintainer
is the authority and assigned reviewer; the final row applies when they own the
PR.
If the standard assignee authored any reviewed work or otherwise cannot act
independently, a qualified non-author Maintainer who understands the affected
work must replace them. If the standard assignee is the Project Maintainer,
apply the final row of the table and assign a different Family Maintainer;
otherwise, the next higher eligible authority designates the replacement. Only
a Maintainer may be the assigned reviewer. When several Maintainers qualify,
their lowest common higher authority designates the reviewer. For Live QA, the
assigned reviewer is the questioner and others may co-question. This does not
replace affected-family approval or grant project authority to a Family
Maintainer reviewing the Project Maintainer.

With the assigned reviewer's consent, a Maintainer or project Contributor may
join as a co-reviewer to learn the QA process and prepare to conduct QA
independently in the future. A co-reviewer need not yet be familiar with the
repository. The assigned reviewer continues to lead the QA and remains
responsible for the review and record.

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

| Source and target | Update Report | QA review record |
| --- | --- | --- |
| Personal working branch to `theme/xxx` | Required | Required |
| Personal working branch to `master` | Required | Required |
| Nested theme branch to its parent theme | Required | Required |
| `theme/xxx` directly to `master` | Not required; may be submitted voluntarily | Required |
| `master` to `theme/xxx` for the permitted final sync | Required | Required |

Each required QA review record uses exactly one review type: Live QA, or
Trivial Update direct approval when the entire PR is eligible under this
chapter. Trivial Update waives only the live session and supplementary question
sheets. It does not waive an Update Report required by the table, independent
review, applicable CI, affected-family approval, the completed and signed fixed
QA record pages, or record retention.

Personal-to-personal merges are outside this procedure. These routes never
permit self-approval or waive affected-family approval.
Using [GOVERNANCE.md](./GOVERNANCE.md) and Chapter 2, the PR owner identifies
every scope, AI tier, reviewer, and approval. A Contributor may transfer work
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
ink without AI assistance before giving it to the assigned reviewer. Ordinary
and nested-theme entries cover the smallest independent design decision; units
without one may be grouped. Voluntary final-theme reports and required final
`master` sync reports cover integration decisions and risks without repeating
previously reviewed entries. The
[form specification](./governance/forms/README.md) defines units, grouping,
fields, notation, and completion.

For a PR using Trivial Update direct approval, every Update Report required by
the applicability table remains required. The report may group the entire PR as
one unit and record only the essential material needed to identify the changed
scope, classify the trivial work, explain why semantics and contracts are
not altered, and state the relevant validation. It may not omit information
needed to determine eligibility or understand the change.

### Independent review and QA review types

The assigned reviewer must independently review the PR, its validation, and
every required or voluntarily submitted report. The reviewer must understand
the work covered by the QA record and is responsible for the accuracy and
technical judgment recorded in it. For a final
`theme/xxx`-to-`master` PR, the review must also establish all of the following:

1. **Record completeness:** the theme history contains no unaccounted-for
   direct commit, and every merge commit is traceable to its PR, required
   records, and author.
2. **Merge residuals:** every change introduced by human conflict resolution
   has been identified and reviewed.
3. **Integration seams:** interactions among staged changes have been reviewed,
   and the final theme state has passed full CI.

The assigned reviewer leads the review and keeps every QA record. For Live QA,
the reviewer prepares questions privately, acts as the questioner, and may be
joined by affected maintainers co-questioning within their areas under the
designation rule in Chapter 2.

#### Live QA

QA must occur live on a real-time meeting platform. The PR owner answers
without AI assistance and must not complete the QA record. The questioner
completes the record by hand in ink and without AI assistance. QA is
question-oriented and risk-based rather than report-unit-oriented: one
question may cover several related units, and one unit may receive several
questions. The questioner may probe any area, including understanding beyond
doubts recorded by the PR owner, and signs the completed record.

Each Live QA round uses a separate QA record containing both fixed pages and at
least one supplementary question sheet. The questioner selects exactly one
final decision:

1. **Approve:** the Live QA requirement is satisfied for the reviewed PR state.
   This does not waive any other merge condition.
2. **Further QA required:** merge remains blocked. The PR owner resolves the
   required changes or clarifications, and a later Live QA round uses a new
   record. This is the non-final rejection outcome.
3. **Do not approve; close PR:** the PR may not merge and must be closed by its
   owner or the responsible authority. A later proposal requires a new PR and
   new records.

#### Trivial Update direct approval

The assigned reviewer may use Trivial Update direct approval only when the
entire PR consists exclusively of one or more of the following:

1. Typo or editorial fixes that do not change meaning, public or stable
   identifiers, interfaces, or observable behavior.
2. Test-only additions or corrections, including local fixtures or helpers,
   that verify existing behavior without changing production-compiled code,
   shared test infrastructure, production build behavior, CI configuration,
   dependencies, or any documented contract.
3. Corrections to non-normative documentation that do not change an API,
   behavior guarantee, requirement, security statement, release rule, or
   governance rule.

The PR must not alter program semantics, any existing public or internal
contract, production build configuration, dependency resolution, security
properties, release processes, governance requirements, or CI workflow
configuration.
Mixed trivial and non-trivial changes are ineligible. Trivial Update direct
approval is unavailable for a final
`theme/xxx`-to-`master` PR, the permitted final `master`-to-theme sync, or work
subject to the final-theme review requirements above. Uncertainty requires Live
QA.

Trivial Update direct approval uses both fixed QA record pages without a live
session or supplementary question sheets. The reviewer checks the entire diff
and applicable validation, records the basis for classification, signs the
record, and thereby directly approves the QA review. If the final PR state is
not entirely eligible, it must use Live QA.

Any change to the reviewed PR state after an approving Live QA or Trivial
Update record invalidates that approval. This includes adding, removing,
replacing, or rewriting commits, changing the target branch, or otherwise
changing the reviewed diff. The PR owner updates or replaces affected Update
Report coverage, reruns applicable validation and CI, and returns the PR to the
assigned reviewer for a new independent review and applicable QA record.

The assigned reviewer uploads the completed QA record through the pull
request's review function. It must not be posted as an ordinary comment.

The form specification defines the QA fields, question-entry semantics, and
completion method. It may not limit the reviewer's scope, waive independent
review, or expand Trivial Update eligibility beyond this policy.

### Findings, approval, and merge

The PR owner must resolve every required finding and update any affected
report; reviewers verify the result. Review, CI, or Live QA repeats as directed,
and an unresolved finding blocks merge. Before merge, the responsible
authorities confirm required reports, CI, independent and cross-family
approval, and a completed QA record whose review type and outcome permit merge.

Theme branches accept only authorship-preserving merge commits from personal
or nested-theme PRs and one final PR-based merge commit from `master`; squash
merges are forbidden. Merge follows completed review and an approving QA
record.

### Record retention

The PR owner keeps each original Update Report, and the assigned reviewer keeps
each original QA record, including superseded and further-QA records, for four
months after merge or closure. During the first three months, the Project
Maintainer may request delivery by providing the address and service; the
project shall pay all fees incurred by delivery. Otherwise delivery is
unnecessary.
Failure to retain or produce a requested original may cause it to be treated as
unverified or potentially false and referred for governance review.

### Hotfix variation

A change that must reach `master` outside the theme cycle may bypass staging.
The Project Maintainer coordinates with the responsible Component Maintainer;
the direct-to-`master` records remain required, but review may be expedited.
Urgency does not qualify a change for Trivial Update direct approval. A hotfix
may use that review type only when its entire final state independently meets
the eligibility rules above. Otherwise Live QA covers every affected module,
and its outcome controls merge. It need not contain a separate question for
each module.

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
the applicable QA review type, technical review, and required CI to assess the
design rationale, semantics, risks, compatibility, and the responsible human's
command of the code. This careful process is how FDS makes human accountability
and its engineering philosophy visible to other contributors.

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
