# Hotaru Governance and Component Ownership

**Effective since 2026.08.01** 

## 1. Project nature and purpose

Hotaru is an FDS-led open-source project. Anyone may use the project, raise an
issue, propose a design, submit a pull request, or review code. Project-wide
governance remains with FDS; family and component authority may be delegated
to eligible FDS or PMINE members.

This document identifies the technical head for each part of Hotaru, defines
the escalation path for decisions, and makes appointments and succession
predictable. Technical maintainership is separate from community moderation
under the [Code of Conduct](./CODE_OF_CONDUCT.md).

## 2. Roles and ownership

- **Project Maintainer** — governs repository-wide policy, permissions,
  releases, security, licensing, and cross-family decisions. Current:
  [@Redstone-D](https://github.com/Redstone-D).
- **Family Maintainer** — the senior technical head of every component in a
  family.
- **Component Maintainer** — the delegated technical head and first contact for
  one component, one rank below the Family Maintainer.
- **Reviewer or Steward** — assists with review and technical guidance without
  final governance or merge authority.

Family and Component Maintainers are both technical heads for a component. The
Family Maintainer may operate directly, approve or block merges, appoint
Component Maintainers, and publish additional family rules. Component
Maintainers must follow those rules. Family rules must be public and may not
conflict with FDS policy, the license, the Code of Conduct, security rules,
required CI, or project-wide governance.

Ordinary component merges may be delegated to Component Maintainers. No
maintainer may be the sole approver of their own change. A cross-family change
requires approval from every affected family. Questions start with the
Component Maintainer, escalate to the Family Maintainer, and finally to the
Project Maintainer.

Live QA follows the role of the final PR owner:

- a Contributor is questioned by the responsible Component Maintainer;
- a Component Maintainer is questioned by the responsible Family Maintainer;
- a Family Maintainer is questioned by the Project Maintainer; and
- the Project Maintainer is questioned by a different Family Maintainer who
  did not author the change and understands the affected code.

A Family Maintainer conducting QA for the Project Maintainer acts only as the
independent questioner for that change and does not acquire project-wide
authority.

### Branch tiers and integration routes

Every change to the canonical repository flows through one of two routes:

| Route | When to use | Records and QA |
| --- | --- | --- |
| Standalone canonical PR | A complete, self-contained change submitted directly to `master` by its Contributor or Maintainer | The PR owner completes the Update Report and answers the live QA; the questioner assigned by the QA order above completes and keeps the QA record. |
| Theme branch integration | Related contributions collected by the Project Maintainer or a Family Maintainer in a theme branch, then submitted as a consolidated canonical PR to `master` | The theme branch owner completes a consolidated Update Report and answers the live QA; the questioner assigned by the QA order above completes and keeps the QA record. |

The person who completes any required form must understand the code covered by
that form and is responsible for the accuracy and technical judgment recorded
in it. These routes do not alter the rules against self-approval or the
approval required for cross-family changes.

**Any merge whose target is a `theme:xxx` branch or `master` requires an
Update Report and live QA.** The questioner is assigned by the QA order above.

#### Personal working branches

A personal working branch is any branch created by a Contributor or Maintainer
for their own work. There are no naming requirements. A personal working branch
may target either a `theme:xxx` branch or `master` directly.

A contributor who cannot complete the Update Report or live QA may ask another
Contributor or Maintainer to carry the work inside their own personal working
branch. The original contributor's authored work is still credited; the person
submitting the PR owns the Update Report and live QA for everything they
include. The code standards that apply to all contributions are defined in the
[Code standards](./CONTRIBUTING.md#code-standards) section of `CONTRIBUTING.md`.

#### Theme branches

Only the Project Maintainer and Family Maintainers may open theme branches.
The Project Maintainer opens theme branches that span multiple families; a
Family Maintainer opens theme branches within their family. Theme branch names
must carry the prefix `theme:` (for example, `theme:0.9-update`). Theme
branches may nest at any depth; nesting should match the actual development
structure.

Theme branches accept only two kinds of commits:

- merge commits from personal working branch or nested theme PRs, with
  authorship preserved — squash merges are forbidden; and
- a single merge commit syncing from `master`, made once when the theme branch
  is ready to merge, not continuously during development.

Direct commits to a `theme:xxx` branch or to `master` are forbidden; all
changes must arrive through a pull request. A direct commit discovered during
record completeness review is a policy violation that blocks the merge until it
is resolved by the theme branch owner.

A merge of a personal working branch into a `theme:xxx` branch is not
acceptance into Hotaru. The theme branch owner must personally review,
understand, explain, modify, test, and debug every change they integrate. The
final consolidated PR must link all staged contributions and identify their
authors.

#### Conducting a theme merge review

The review of a theme merge targets three bounded objects rather than the full
diff:

1. **Record completeness.** `git log --first-parent --no-merges` on the theme
   branch must produce no output. Each merge commit is traced to its PR and
   Update Report. Any unaccounted-for commit blocks the merge until resolved.

2. **Merge residuals.** `git show --remerge-diff <merge-commit>` on each merge
   commit reveals what human conflict resolution added beyond the automatic
   result. These are the only lines not covered by any prior review.

3. **Integration seams.** Full CI on the final theme branch state. The live QA
   session probes the owner on which entries interact and what semantic
   conflicts were resolved during integration.

The Update Report for a theme merge covers integration decisions and risks.
Entry-level content is referenced by linking to constituent PR forms and is
not rewritten.

**Live QA sequence for a theme merge:**

1. The theme branch owner submits the consolidated Update Report in advance.
2. The questioner reads the report and independently reviews the diff.
3. The questioner prepares questions privately.
4. Live session: the questioner asks; the owner answers. The questioner may
   probe any area, including depth of understanding beyond recorded doubts.
5. The questioner keeps the record.

For a theme that spans multiple families, the assigned questioner leads and
keeps the record; affected Family or Component Maintainers co-question within
their areas. Cross-family approval still applies.

#### Hotfix path

A change that must reach `master` outside the normal theme cycle — for example
a critical security fix — may bypass theme staging. The Project Coordinator
contacts the responsible Component Maintainer directly. The Component
Maintainer conducts an expedited live QA using the standard form, assessing
each affected module individually. The merge decision follows the QA outcome.

Printable source files and compiled PDFs for the Update Report and live QA are
kept in [`governance/forms/`](./governance/forms/).

Root workspace files, root documentation, and `.github/**` are governed at the
project level. [@Redstone-D](https://github.com/Redstone-D) and
[@JerrySu5379](https://github.com/JerrySu5379) are the primary reviewers.
Examples inherit the ownership and AI declaration of the components they
demonstrate.

**Release governance.** Release scope, readiness, and timing are decided at
the regular Wednesday and Sunday coordination meetings conducted under the
[FDS Administrator Rules](https://doc.fds.moe/policies/admin/). The Project
Maintainer records and carries out the release decision.

**RFC governance.** Each Family Maintainer defines the RFC process for their
family, subject to these project-wide requirements:

1. A patch-level update that changes only the final version component, such as
   `0.a.b` to `0.a.c`, must not break a stable API.
2. A major or breaking public API change must be proposed to the community and
   discussed at an internal meeting before approval.
3. Family RFC rules may be stricter than these requirements, but not weaker.

### Core framework

Core contracts and the procedural-macro DSL.

**Family Maintainer:** [@Redstone-D](https://github.com/Redstone-D)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Core contracts and semantics | `hotaru_core/**` except the URL paths below | [@Redstone-D](https://github.com/Redstone-D) |
| DSL and procedural macros | `hotaru_trans/**` | [@Redstone-D](https://github.com/Redstone-D) |

### Facade and tooling

Routing, the public facade and feature surface, CLI tooling, templates, and
shared user-facing utilities.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Routing and URL semantics | `hotaru_core/src/url.rs`, `hotaru_core/src/url/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Facade and public feature surface | `hotaru/src/lib.rs`, `hotaru/src/prelude.rs`, `hotaru/src/http.rs`, `hotaru/src/test.rs`, `hotaru/Cargo.toml`, `hotaru/readme.md` | [@Redstone-D](https://github.com/Redstone-D) |
| CLI and project templates | `hotaru/src/main.rs`, `templates/**`, `programfiles/**`, `hotaru_style_guide/**` | [@Redstone-D](https://github.com/Redstone-D) |
| Shared utilities | `hotaru_lib/**` | [@Redstone-D](https://github.com/Redstone-D) |

### Protocol implementations

Wire protocols, protocol-specific security, and standard middleware.

**Family Maintainer:** [@Redstone-D](https://github.com/Redstone-D)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| HTTP, TLS, and web middleware | `hotaru_http/**`, `hotaru_tls/**`, `htmstd/**`, `ahttpm/**` | [@Redstone-D](https://github.com/Redstone-D) |
| MQTT client and broker | [`Field-of-Dream-Studio/hotaru_mqtt`](https://github.com/Field-of-Dream-Studio/hotaru_mqtt) | [@JerrySu5379](https://github.com/JerrySu5379) |
| Experimental protocol integrations | `h2per/**`, `hotaru_grpc/**` | [@Redstone-D](https://github.com/Redstone-D), [@JerrySu5379](https://github.com/JerrySu5379) |

The MQTT repository should maintain its own matching ownership rules.

### Runtime implementations

Runtime scheduling, spawning, and runtime-specific integration.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Tokio runtime | `hotaru_rt_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Embassy runtime | `hotaru_rt_embassy/**` | [@zkmaojack](https://github.com/zkmaojack) |

### I/O implementations

Adapters between Hotaru's transport contracts and concrete I/O ecosystems.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Tokio I/O | `hotaru_io_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Futures I/O | `hotaru_io_futures/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Embedded I/O | `hotaru_io_embedded/**` | [@zkmaojack](https://github.com/zkmaojack) |

## 3. AI declarations

Hotaru supports AI-assisted and AI-copilot development. The human responsible
for a change **must** understand and be able to explain every part of its
design and implementation, regardless of which tools helped produce it. They
must also be able to modify, test, and debug the work without asking an AI
system to reconstruct it for them.

Hotaru does not scan code for an "AI rate," estimate the percentage of code
generated by AI, or use such a percentage as a merge criterion. AI tiers
describe the kind of collaboration, not the amount of generated text. Merge
review instead uses the Update Report, live QA, technical review, and required
CI to assess the design rationale, semantics, risks, compatibility, and the
responsible human's command of the code. This careful process is how Hotaru
makes human accountability and its engineering philosophy visible to other
contributors.

| Tier | Definition |
| --- | --- |
| **Forbidden** | Design, proofs, semantics, and novel logic are human-authored. |
| **Author-Owned** | AI may assist with drafts or completion; the human owns the design and committed work. |
| **Human-Led** | The human writes the structure and load-bearing logic; AI may assist with helpers and boilerplate. |
| **Co-Authored** | AI may assist with design and implementation; the human must fully internalize the result. |

Each Family Maintainer chooses and updates the declarations for components in
their family. When scopes inside one component use different tiers, the more
specific declaration applies.

| Family | Component or scope | Tier |
| --- | --- | --- |
| Core framework | Core `app`, `connection`, `executable`, and `protocol` | **Author-Owned** |
| Core framework | Remaining core contracts and semantics | **Human-Led** |
| Core framework | DSL `endpoint`, `outpoint`, and `middleware` | **Author-Owned** |
| Core framework | Remaining DSL and procedural macros | **Human-Led** |
| Facade and tooling | Routing and URL semantics | **Author-Owned** |
| Facade and tooling | Facade and public feature surface | **Co-Authored** |
| Facade and tooling | CLI and project templates | **Co-Authored** |
| Facade and tooling | Shared utilities | **Human-Led** |
| Protocol implementations | HTTP, CORS, and session middleware | **Human-Led** |
| Protocol implementations | TLS, remaining middleware, and `ahttpm` | **Co-Authored** |
| Protocol implementations | MQTT client and general implementation | **Human-Led** |
| Protocol implementations | MQTT broker and traits | **Co-Authored** |
| Protocol implementations | Experimental protocol integrations | **Co-Authored** |
| Runtime implementations | Tokio and Embassy runtimes | **Co-Authored** |
| I/O implementations | Tokio, Futures, and embedded I/O | **Co-Authored** |

## 4. Eligibility and succession

The Project Maintainer must be an active FDS member. A Family or Component
Maintainer may qualify through either active FDS membership or active PMINE
membership. PMINE membership is independent and does not imply FDS membership.

| Role | Eligibility and appointment |
| --- | --- |
| Project Maintainer | An active FDS member appointed and succeeded under FDS policy |
| Family Maintainer | An active FDS or PMINE member appointed or removed by the Project Maintainer |
| Component Maintainer | An active FDS or PMINE member appointed or removed by the Family Maintainer |
| Reviewer or Steward | Open to trusted contributors; organizational membership is not required |
| Contributor | Open to everyone |

The Project Maintainer follows the
[FDS Charter](https://doc.fds.moe/policies/constitution/). Family and Component
Maintainers are Hotaru technical roles with two independent eligibility paths:
[FDS membership](https://doc.fds.moe/policies/join/) or
[PMINE membership](https://pmine.rs).

A maintainer planning to resign or take leave must arrange a successor or
acting candidate for confirmation by the next higher authority. For an
unexpected vacancy, authority temporarily moves upward. Loss or expiration of
the membership required for a role suspends maintainer authority immediately;
a Family or Component Maintainer remains eligible while actively belonging to
at least one of FDS or PMINE. Every transition must be recorded here and
reflected in code ownership and repository permissions.

This applies the succession principle in the
[FDS Administrator Rules](https://doc.fds.moe/policies/admin/) to Hotaru.
