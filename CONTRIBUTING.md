# Contributing to Hotaru

Thank you for your interest in contributing to Hotaru. We're excited to build this framework together with the community.

## Development Status

Hotaru is currently in active development (0.8.x). The following areas are still being built:

### In Development

- **Homepage (fds.rs)** - Our official website is under construction
- **Tutorial & Documentation** - Comprehensive guides and examples are being written
- **API Documentation** - Detailed docs for all public APIs
- **Example Projects** - Real-world application examples
- **Backend crate documentation** - Tokio runtime/IO backends now live outside `hotaru_core`, and the feature model needs clear user-facing examples
- **Embedded / no_std support** - Core is being prepared for no_std targets; embedded-io adapters are experimental and real Embassy wiring is deferred

## How You Can Help

We welcome contributions in the following areas:

### Documentation
- Write tutorials for common use cases
- Improve README files and code examples
- Create getting-started guides
- Document best practices and patterns
- Translate documentation to other languages

### Examples
- Build example applications demonstrating Hotaru features
- Create templates for common project types
- Share integration examples with other libraries

### Code
- Fix bugs and improve error messages
- Add tests for uncovered functionality
- Optimize performance
- Implement new features (see our roadmap)

### Community
- Help answer questions in discussions
- Write blog posts or tutorials
- Share your Hotaru projects
- Provide feedback on the API design

## Get Involved

- **GitHub Issues**: https://github.com/Field-of-Dream-Studio/hotaru/issues
- **Discussions**: https://github.com/Field-of-Dream-Studio/hotaru/discussions
- **Email**: redstone@fds.moe
- **Discord Group**: https://discord.gg/Y6b9KRUCux
- **QQ Group**: 860691370
- **Join FDS**: https://forms.office.com/Pages/ResponsePage.aspx?id=DQSIkWdsW0yxEjajBLZtrQAAAAAAAAAAAAMAAC6BwJ5UQ0lQUzdMTjhGR1g3SElLTFdHQUlJV0hFMS4u

## Areas Needing Help

### High Priority
1. **Tutorial Documentation** - Step-by-step guides for:
   - Basic HTTP server setup
   - Middleware creation and usage
   - Session management
   - Custom protocol implementation
   - Custom `TransportSpec` / `RuntimeSpec` implementations
   - Feature selection (`tokio`, `io_futures`, `io_embedded`, `spawn_send`, `spawn_local`)

2. **Homepage Development** - Help build fds.rs:
   - Landing page design
   - Documentation hosting
   - Interactive examples
   - API reference browser

3. **Example Applications**:
   - Blog/CMS system
   - REST API backend
   - Real-time chat application
   - File sharing service
   - Authentication & authorization examples

### Medium Priority
4. **Performance Benchmarks**
   - Compare with other Rust frameworks
   - Identify optimization opportunities
   - Create benchmark suite

5. **Testing**
   - URL routing edge cases
   - Middleware chain testing
   - Integration tests
   - Feature-matrix checks for default Tokio, no-default facade builds, and core-only builds

## Pull request workflow

This is the contributor-facing workflow derived from
[POLICY.md Chapter 3](./POLICY.md#3-pull-request-governance).
`POLICY.md` remains authoritative for applicability, approvals, merge
eligibility, retention, and enforcement. `GOVERNANCE.md` records Hotaru's
eligibility, appointments, ownership, reviewers, and AI tiers. The
[form specification](./governance/forms/README.md) defines how to complete the
required records. This workflow adds no requirement or exception of its own.

### 1. Choose the route and identify the affected scope

#### Personal-branch contribution route

Anyone may submit a pull request to the Maintainer-owned
[`bug-fix`](https://github.com/Field-of-Dream-Studio/hotaru/tree/bug-fix)
branch or, with the branch owner's agreement, to another Contributor's or
Maintainer's personal working branch. These personal-to-personal pull requests
are outside the governed procedure in `POLICY.md`: no Update Report or QA
review record is required for that pull request.

Merging work into a personal branch is not acceptance into Hotaru's canonical
codebase. If the owner of the target branch later carries the work forward to
`theme/xxx` or `master`, they become the owner of the governed integration PR
and take responsibility for understanding the contributed work and for its
validation, required records, review, QA, and approvals. The original author's
authorship remains credited.

This route lowers the cost of contributing or handing off work; it does not
bypass the quality gates for `theme/xxx` or `master`.

#### Governed integration route

To submit work directly for canonical or theme integration, create a personal
working branch and target either an open `theme/xxx` branch or `master`. If you
own a nested theme, target its parent theme. Final theme and permitted
`master`-sync PRs use their existing branches. Look up every affected Family,
Component, maintainer, and AI tier in [GOVERNANCE.md](./GOVERNANCE.md). Identify
the assigned reviewer from the review order in
[POLICY.md Chapter 2](./POLICY.md#2-roles-and-ownership) and note every required
cross-family approval.

These records apply:

| Source and target | Update Report | QA review record |
| --- | --- | --- |
| Personal working branch to `theme/xxx` | Required | Required |
| Personal working branch to `master` | Required | Required |
| Nested theme branch to its parent theme | Required | Required |
| `theme/xxx` directly to `master` | Optional | Required |
| `master` to `theme/xxx` for the permitted final sync | Required | Required |

Each QA review record uses Live QA or, when the entire PR qualifies under
Policy, Trivial Update direct approval. Trivial Update waives only the live
session and supplementary question sheets.

If you are a Contributor and cannot own the required report or, when Live QA is
required, answer it, ask another Contributor or Maintainer to carry the work
from their personal working branch. They become the PR owner and take
responsibility for every included change and record; your original authorship
remains credited.

### 2. Prepare a reviewable change

Follow the applicable AI tier and the [code standards](#code-standards). Keep
mechanical work such as renames, moves, and formatting in commits separate
from semantic changes. Aim for approximately ten Update Report units in a
personal-branch PR. Split independent changes whenever each resulting branch
can still pass its checks.

Write clear commit messages, add tests for new behavior, and update
documentation when an API or contract changes. Do not commit directly to
`theme/xxx` or `master`.

### 3. Validate the proposed branch state

Run every check applicable to the change, including formatting, build, tests,
lint, feature combinations, target builds, and integration checks. Typical
workspace checks include:

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
```

Record a check as `N/A` only when it cannot apply to the changed scope, and
include the reason in the PR. A final theme branch must pass full CI before it
can proceed to `master`.

Use formatting on the files you touch rather than a workspace-wide rewrite
that introduces unrelated noise. When testing example crates, prefer the
`hotaru build` and `hotaru run` CLI commands so templates and static assets are
copied correctly.

### 4. Open the pull request

Open the PR against the selected `theme/xxx` branch or `master`. Include its
scope, affected components, split rationale, and validation results. You may
open it as a draft while preparing the records; opening it first provides the
PR number needed on them.

A final `theme/xxx`-to-`master` PR must link every staged PR and identify its
authors. The theme owner remains responsible for reviewing, understanding,
testing, explaining, modifying, and debugging every integrated change.

### 5. Complete and submit the Update Report when applicable

Use [`report.pdf`](./governance/forms/report.pdf) with at least one copy of
[`report_sup.pdf`](./governance/forms/report_sup.pdf), adding more copies as
needed. Follow the
[Update Report specification](./governance/forms/README.md#update-report).

For a personal-branch or nested-theme PR, create an entry for each report unit
and list every unit in a permitted group. For a voluntary consolidated
theme report or the permitted final `master`-to-theme sync, record integration
decisions and risks instead of repeating definition-level entries already
reviewed in constituent PRs or on `master`.

Complete every page by hand in ink and without AI assistance. Sign the cover
only after the detail pages are complete, then make the complete report
available to the assigned reviewer before review. A final
`theme/xxx`-to-`master` PR may omit its optional consolidated report entirely.

For an eligible Trivial Update, a required report remains required but may
group the entire PR as one unit. Record only the changed scope, classification,
why existing semantics and contracts are not altered, and relevant validation
needed to establish eligibility and understand the change.

### 6. Complete independent review and select the review type

The assigned reviewer independently reviews the complete diff, validation, and
every required or voluntary Update Report. The reviewer selects Trivial Update
direct approval only when the entire PR qualifies under Policy; otherwise the
PR uses Live QA. For Live QA, the reviewer prepares questions privately.

For a final theme PR, which cannot use Trivial Update direct approval, the
reviewer also:

1. checks the theme's first-parent history from its branch point
   and traces every merge to its PR, records, and author;
2. inspects the remerge diff of every merge commit for human conflict
   resolution; and
3. reviews integration seams and confirms full CI on the final branch state.

Useful commands are:

```sh
git log --first-parent --no-merges <theme-branch-point>..HEAD
git show --remerge-diff <merge-commit>
```

The first command must produce no direct theme commits. For a cross-family
or multi-component PR, follow the designation rule in
[POLICY.md Chapter 2](./POLICY.md#2-roles-and-ownership): the assigned
reviewer leads and keeps the record. For Live QA, other affected maintainers
may co-question within their areas.
With the assigned reviewer's consent, a Maintainer or project Contributor may
join as a co-reviewer to learn the QA process, even before becoming familiar
with the repository. The assigned reviewer continues to lead the QA.

### 7. Complete the QA review

Use [`qa.pdf`](./governance/forms/qa.pdf) and follow the
[QA review record specification](./governance/forms/README.md#qa-review-record).

For Live QA, meet on a real-time platform and attach at least one copy of
[`qa_sup.pdf`](./governance/forms/qa_sup.pdf), with additional copies as needed.
The PR owner answers without AI assistance and does not complete the QA record.
The questioner completes the record by hand in ink and without AI assistance.
Questions should follow risk and understanding rather than mechanically
matching one question to each report entry. Select one final decision. Further
QA requires a new round and record; a decision to close prevents merge and ends
that PR.

For Trivial Update direct approval, do not hold a live session or attach
question sheets. The reviewer completes both fixed pages, records why the
complete PR qualifies, and signs the cover sheet. A final state that no longer
qualifies must use Live QA.

For either review type, the assigned reviewer uploads the completed QA record
through the pull request's review function, not as an ordinary comment.

Any change to the reviewed PR state invalidates an earlier approving QA record.
Update or replace affected Update Report coverage, rerun applicable validation
and CI, and repeat the independent review and applicable QA record for the new
PR state.

The permitted final `master`-to-theme sync also cannot use Trivial Update direct
approval.

### 8. Resolve findings

Make every code, test, documentation, and report correction required by review
or QA. Update an affected report entry whenever its recorded design changes.
Repeat review, CI, or Live QA to the extent directed by the responsible
authority. Unresolved required changes block the merge.

### 9. Obtain approval and merge

Confirm required reports, CI, independent approval, an approving QA record, and
all cross-family approvals. A maintainer cannot be the sole approver of their
own change.

Merge personal-branch and nested-theme PRs into `theme/xxx` with merge commits
that preserve authorship; do not squash them. A theme may sync from `master`
through one PR-based merge commit when preparing for final integration.

### 10. Retain the records

After merge or closure, the PR owner keeps every Update Report and the assigned
reviewer keeps every QA record, including superseded and further-QA records.
Keep each original for four months and follow the delivery rules in
[POLICY.md](./POLICY.md#record-retention).

For an urgent direct-to-`master` change, follow the
[hotfix variation](./POLICY.md#hotfix-variation); the records remain
required even when review is expedited. Urgency alone does not make a hotfix a
Trivial Update.

Internal Hotaru crate dependencies should use exact version pins such as
`version = "=0.8.5"` during release-prep updates. Third-party dependencies
should keep normal semver requirements unless there is a specific reason to
pin them.

## Code standards

**Function size.** Keep functions small and single-purpose. If a function
performs several independently meaningful operations, split it. Unusually long
functions are a review focus area and, when used, a Live QA focus area.

**Coupling.** Eliminate coupling between independent changes wherever possible.
If two changes must ship together because splitting would break the build, be
prepared to explain in QA exactly why the coupling is unavoidable. Introducing
a dependency between two changes to avoid splitting them is a code standard
violation.

**Commit separation.** Mechanical changes (renames, moves, formatting) must not
be mixed with semantic changes in the same commit. Keep them in separate
commits so reviewers can read semantic changes without noise.

**Module layout.** Prefer one Rust module per file. If a module or `impl` block
becomes too large to review comfortably or combines distinct responsibilities,
split it into focused files or submodules and separate `impl` blocks. When a
module uses a directory, keep its `mod.rs` focused on module declarations and
public re-exports.

**Test layout.** Tests totaling 50 lines or fewer for a module may remain beside
the implementation. If that module's test code exceeds 50 lines, move it to
`test.rs` in the module's directory and load it with
`#[cfg(test)] mod test;`.

## Code style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting (on changed files only, not `--all`)
- Run `cargo clippy` and fix warnings
- Add doc comments (`///`) for public APIs
- Write descriptive variable and function names

For framework code style and formatting requirements, see
[CONTRIBUTOR_STYLE.md](./CONTRIBUTOR_STYLE.md).

## Project Roadmap

### 0.8.5 (Current)
- Core/backend split: Tokio runtime and IO backends live in sibling crates (`hotaru_rt_tokio`, `hotaru_io_tokio`, `hotaru_io_futures`, `hotaru_io_embedded`)
- `hotaru_core` keeps only platform/sync (`std` / `embedded`) and task-mobility (`spawn_send` / `spawn_local`) feature axes
- no_std / embedded groundwork (experimental; real Embassy wiring deferred)
- HTTP/TLS hardening and documentation

### 0.9.0
- UDP support
- Performance optimization

### 1.0.0
- API stability guarantee
- Complete documentation
- Production deployment guides

## License

By contributing to Hotaru, you agree that your contributions will be licensed under the MIT License.

## Thank You

Your contributions make Hotaru better for everyone. Whether you fix a typo, write documentation, or implement a major feature, every contribution is valuable.

Let's build something great together.
