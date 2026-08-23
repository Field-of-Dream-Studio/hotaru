# FDS Code Policy Form Specification

This specification defines the structure, field meanings, and completion
semantics of the FDS Code Update Report and Code Quality Assurance Record. It
is incorporated by
[POLICY.md Chapter 3](../../POLICY.md#3-pull-request-governance).

`POLICY.md` controls authority, record applicability, approval, merge
eligibility, retention, and enforcement. This specification explains how to
record the information required by that policy; it does not create a role,
approval, merge gate, or exception. The `.tex` sources are printable layouts,
and the checked-in PDFs are generated copies. They may repeat these
instructions so that printed forms remain usable offline.

## General completion rules

- Complete every required or voluntarily submitted record by hand in ink and
  without AI assistance.
- Write in English, Japanese, Traditional Chinese, or Simplified Chinese.
- Do not silently leave a required field blank. If a field cannot apply, write
  `N/A` and a short reason.
- If no printed choice is accurate, write the accurate value beside the field
  rather than selecting an inaccurate choice.
- Where several values apply, select or write every value. Separate multiple
  written values with commas; if the available line is insufficient, attach a
  clearly labeled continuation page and include it in the page count.
- For project-level work that has no Family or Component assignment in
  `GOVERNANCE.md`, write `N/A — project-level scope` in those two fields rather
  than inventing an assignment. Select **Forbidden** for a documentation-only
  project-level change; other project-level work uses the AI tier recorded in
  `GOVERNANCE.md`.
- `Repository` identifies the repository containing the PR. `PR number` is the
  number assigned by that repository, and `Target branch` is the PR's base
  branch.
- `Supplementary pages` records the number of attached supplementary pages and
  excludes the cover sheet. In each page footer, the total includes the
  cover sheet; number the complete set consecutively beginning with the cover
  as page 1.
- The person who signs a record is responsible for the accuracy and technical
  judgment recorded in it.

## Update Report

Use [`report.pdf`](./report.pdf) as the cover sheet and attach as many copies
of [`report_sup.pdf`](./report_sup.pdf) as needed. The corresponding sources
are [`report.tex`](./report.tex) and [`report_sup.tex`](./report_sup.tex).

Complete one cover sheet for each PR for which an Update Report is required or
voluntarily submitted. The route table in
[POLICY.md](../../POLICY.md#applicability-and-required-records)
determines whether the report is required, optional, or outside the governed
procedure.

### Cover sheet fields

- **Project:** the project under which the PR is reviewed.
- **Repository, PR number, and target branch:** identify the exact PR.
- **PR author:** the PR owner responsible for the complete submitted change,
  even when it preserves work authored by other contributors.
- **Questioner:** the questioner assigned under the live-QA order and any
  applicable designation rule in
  [POLICY.md Chapter 2](../../POLICY.md#2-roles-and-ownership).
- **Families and Components:** every affected short name from
  [GOVERNANCE.md](../../GOVERNANCE.md). Do not collapse a cross-family or
  multi-component PR into one value.
- **Date:** the date on which the PR author completes and signs the report.
- **Supplementary pages:** the number of attached Update Report detail sheets.
- **Role:** the PR author's actual role for this PR. Select one of Contributor,
  Reviewer or Steward, Component Maintainer, Family Maintainer, or Project
  Maintainer.
- **Applicable AI tier(s):** every tier that applies to the affected scopes. A
  documentation-only project-level change uses **Forbidden**.
- **AI assistance used in this PR:** whether AI assisted the PR work. When the
  answer is `Yes`, select every applicable category: Design, Code, and Tests.
  Documentation-only AI assistance is outside AI usage. This disclosure
  concerns the PR work; the report itself must still be completed without AI
  assistance.

Complete the declaration and signature only after every detail page is
complete.

### Report units

For a personal working branch or nested theme PR, each entry covers the
smallest changed, non-trivial design unit at definition granularity. A changed
function, method, struct, enum, trait, type alias, macro, constant, or static
definition carrying an independent design decision is a separate report unit.
Statements and nested blocks within a definition are not separate units. A
module or `impl` block does not replace entries for its definitions; changed
non-trivial methods in one `impl` are recorded separately.

Definitions without an independent design decision may be grouped in one
entry. Examples include plain accessors, direct delegators without added
behavior, mechanical re-exports, and substantially equivalent definitions
implementing the same design. Short or one-line code remains separate when it
introduces independent semantics, risk, invariants, or compatibility
consequences. A grouped entry must list every covered definition.

For work without source definitions, use the smallest changed unit carrying an
independent design decision. Depending on the file, this may be a named
documentation section, configuration key or table, workflow job or step,
template section, or the whole file when it is not meaningfully divisible.
Mechanical or substantially equivalent non-Rust units without independent
design decisions may be grouped, and every grouped path and unit must be
listed.

For a voluntarily submitted consolidated `theme/xxx`-to-`master` report, each
entry instead covers the smallest distinct integration decision or risk. It
must identify the merge residual or integration seam, link the constituent PRs
and reports, explain how integration was performed, and justify the resolution
and its risks. Do not reproduce constituent definition-level entries. The
required report for the permitted final `master`-to-`theme/xxx` sync uses the
same integration-focused entry model and identifies the relevant changes
already reviewed on `master`.

### Detail entry fields

- **Work unit:** for definition-level entries, write the
  source-definition path rather than a re-export path. For Rust, use
  `crate + module/item`: the crate is required, `+` separates the crate from
  the symbol path, and `/` may replace `::` when handwriting. Continue through
  the containing type or trait to the exact definition. Write an inherent
  method as `crate + module/Type/method`. Write a trait implementation as
  `crate + module/Type + Trait/method`; the second `+` means "implementation
  of." Use a full trait path when necessary. For non-Rust work, use the
  repository-relative file path and identify the section, key, table, job,
  step, or other unit when the file contains more than one unit. For a
  consolidated theme report, identify the integration seam or merge residual
  instead.
- **Change type:** select every accurate printed type. `New definition` covers
  a newly added module, function, method, type, trait, macro, constant, or
  static. If none is accurate, write the actual type beside the field.
- **Design approach:** state what design was implemented and how it works.
- **Justification of the approach:** state why the design should be adopted.
  For a redesign, compare it with the previous approach. Include material
  alternatives, risks, compatibility consequences, or unresolved doubts when
  they affect that judgment.

## Live QA Record

Use [`qa.pdf`](./qa.pdf) as the cover sheet and attach as many copies of
[`qa_sup.pdf`](./qa_sup.pdf) as needed. The corresponding sources are
[`qa.tex`](./qa.tex) and [`qa_sup.tex`](./qa_sup.tex).

The assigned questioner completes one cover sheet for every PR requiring live
QA. The questioner leads the session and keeps the record even when affected
maintainers co-question within their areas.

### Cover sheet fields

- **Project, repository, PR number, target branch, and PR author:** identify
  the exact PR and its owner.
- **Questioner:** the independent questioner assigned under the live-QA order
  and any applicable designation rule in
  [POLICY.md Chapter 2](../../POLICY.md#2-roles-and-ownership).
- **Families and Components:** every affected short name from
  [GOVERNANCE.md](../../GOVERNANCE.md).
- **Date:** the date of the live QA session.
- **Supplementary pages:** the number of attached QA question sheets.
- **Review conditions:** mark every statement that is true. Mark the Update
  Report as received only if a required or voluntarily submitted report was
  actually received before QA. Leave that statement unmarked when the optional
  report for a final theme PR was not submitted.
- **Applicable AI tier(s):** every tier that applies to the affected scopes. A
  documentation-only project-level change uses **Forbidden**.

The questioner completes and signs the declaration after the live session.

### Question entry fields

- **Scope examined:** identify the module, work unit, symbol, integration seam,
  or topic examined. A single question may cover several related definitions,
  and a definition may be examined by several questions.
- **Question asked:** record the question asked live.
- **Change required:** record whether the questioner requires a code, test,
  documentation, report, or clarification change under the review outcome.
- **PR author — Fully, Partly, or Unable:** record how completely the PR author
  demonstrated the required understanding in the live answer. This rating
  does not override the `Change required` selection; the questioner must record
  required follow-up explicitly.
- **Required change or supplementary clarification:** when `Change required`
  is `Yes`, describe the required follow-up here. When it is `No`, this field
  is optional and may provide context supporting the question or evaluation.

QA is risk-based and question-oriented, not mechanically matched to Update
Report units. The questioner may examine any relevant area and must not use
this specification to narrow the review required by governance.

## Retention

The PR owner keeps the original Update Report, and the questioner keeps the
original QA record. The project shall pay all fees incurred by delivery.
Retention duration, delivery requests, and the consequences of missing records
are governed by
[POLICY.md](../../POLICY.md#record-retention).
