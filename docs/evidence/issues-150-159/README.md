# D2 frozen batches #150–#159

`core-159-v1` admits **496 new groups / 1,984 fields** at frozen rows 3451–3950, bringing the core to **3,927 groups / 15,708 fields**. The denominator remains 4,087 groups / 3,773 headwords. 160 groups remain unaccepted; 3,924 everyday groups are complete and 163 remain incomplete. The same three specialist senses are counted separately. The core retains 23 zero-field annotations / 92 missing fields.

The four new owner-approved annotations are row 3783 `ya/NOM` ([#156 decision](../issue-156/owner-approval.json)), row 3885 `cm/NOM` ([#158 decision](../issue-158/owner-approval.json)), and rows 3922 `haver/VER` and 3944 `da/ONO` ([#159 decision](../issue-159/owner-approval.json)). They retain exact frozen identities, frequencies and ordering, with four missing fields each; their proposals remain preserved.

| Batch | Frozen rows | Admitted groups / fields | Annotations | Specialist groups |
| --- | --- | --- | --- | --- |
| [#150](../issue-150/README.md) | 3451–3500 | 50 / 200 | 0 | 0 |
| [#151](../issue-151/README.md) | 3501–3550 | 50 / 200 | 0 | 0 |
| [#152](../issue-152/README.md) | 3551–3600 | 50 / 200 | 0 | 0 |
| [#153](../issue-153/README.md) | 3601–3650 | 50 / 200 | 0 | 0 |
| [#154](../issue-154/README.md) | 3651–3700 | 50 / 200 | 0 | 0 |
| [#155](../issue-155/README.md) | 3701–3750 | 50 / 200 | 0 | 0 |
| [#156](../issue-156/README.md) | 3751–3800 | 49 / 196 | 1 | 0 |
| [#157](../issue-157/README.md) | 3801–3850 | 50 / 200 | 0 | 0 |
| [#158](../issue-158/README.md) | 3851–3900 | 49 / 196 | 1 | 0 |
| [#159](../issue-159/README.md) | 3901–3950 | 48 / 192 | 2 | 0 |

All admitted fields passed separate independent quality, construction/spelling and selected-expression rights reviews. Each batch retains fixed source versions, selected spans, contributor records, applicable rights, notices and the authored proposals alongside exact accepted decisions.

The [canonical resource](../../../resources/core-pilot-79/README.md), [notices](../../../resources/core-pilot-79/NOTICES.md), [CC BY-SA 4.0 terms](../../../resources/core-pilot-79/COPYING) and [LICENSE-FranceTerme.pdf](../../../resources/core-pilot-79/LICENSE-FranceTerme.pdf) accompany the matching executable. Conversion uses fixed local inputs. After post-review fixes on `66b12b514ed21cd1da4393b8c6465c57c7819e53`, the Decision 15 measurements are `180,797,580` bytes raw inputs, `18,268,160` bytes read-only SQLite (within the `20,000,000`-byte native bound, SHA-256 `1240f403db18f130943000d37782857ca6695a92f60c0776466789a3ddbed2fc`), `180,896,776` bytes bundled materials, `199,164,936` bytes installed core excluding `report.json`, `199,768,852` bytes uncompressed `dictionary/core-pilot-79/` (`core_installed_bytes`, including the `603,916`-byte `report.json`), `269,347,109` bytes total uncompressed portable installation (`installed_bytes`), and `49,926,877` bytes compressed portable ZIP (SHA-256 `ed30cd6feac68c9085c479edc7d236a32659a6d67cf3afbdecfbb884a7295e6b`, `187` verified files).

All prior **3,431 groups / 13,724 fields**, source arrays, reviews, contexts, spelling relations, frozen membership and compiled payloads remain unchanged. Both the `5fcba9932f74e298d4a798e6eb195faad5f51507` baseline and the post-review `66b12b514ed21cd1da4393b8c6465c57c7819e53` resource-upgrade run preserve previously saved references, digests, notes, examples, FSRS and audio selections from `core-149-v1` after reopening with the new resource.

[Verification](verification-final.json) and [commands/results](commands-results.md) record both the `5fcba9932f74e298d4a798e6eb195faad5f51507` baseline and the post-review Windows x64 rebuild (`Lexique.exe` SHA-256 `3ffc6cc987cdd752f0d74ee1e018495ab1477501918366b8ad067c3f3f999d54`), portable ZIP, Rust/frontend test runs, ten native batch runs and resource-upgrade evidence. Each native run preserves the hard 60-second limit, observes zero external WebView requests after routing begins and finishes with no owned hosts, workers or helpers left running.

[Final review and reuse verification](final-review.md) records both review axes, the shared-helper fixes and native execution with the revised harness.

Learner acceptance, whole-machine offline behavior, other-platform native execution, full D2 / CEFR and formal release acceptance remain unverified. Parent #78 remains open. Resource upgrade and rollback use matching executable/core pairs and preserve the learning data directory and backups. [PR #219](https://github.com/zardonc/lexique/pull/219) contains these ten batches.
