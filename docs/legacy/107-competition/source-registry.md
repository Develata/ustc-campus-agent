# USTC Campus Source Registry Contract

## Metadata

- `Layer`: `Campus Trust Kernel / Source Authority`
- `Status`: **Develata-confirmed contract candidate；source entries/crawl permissions 尚待逐项审核**
- `Version`: `0.1.0`
- `Last Review`: `2026-07-21`
- `Authority Owns`: source identity、revision、URL normalization、retrieval policy、immutable evidence、baseline advancement
- `Authority Defers To`: [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md), [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)

## 1. Purpose

Source Registry 回答：

- 哪个 official source 被平台批准；
- 谁拥有和审核该 source；
- 如何抓取、限速、解析和验证；
- 当前有哪些 immutable revisions；
- 哪个 baseline 已安全推进；
- provenance 如何回到 exact URL、snapshot、digest 与时间。

Registry 不是通用 web crawler catalog，也不意味着 `*.ustc.edu.cn` 全域默认获准抓取。

## 2. Confirmed decisions

| Decision ID | Decision | Status |
| --- | --- | --- |
| `SRC-AUTH-001` | 首期 source 必须是逐项审核的 USTC official/public source；wildcard domain 只作 egress upper bound，不等于 crawl approval | Develata confirmed |
| `SRC-ID-001` | `source_id` 与 `source_revision_id` 是 stable identity；URL 只是 lookup key，同一 URL 可对应多个 revisions | Develata confirmed |
| `SRC-FETCH-001` | fetch 只接受 registry-approved source/URL policy；redirect/DNS/IP/content-type/size/time 均 fail-closed | Develata confirmed |
| `SRC-EVIDENCE-001` | raw/normalized snapshots 与 content digest immutable；procedure/change 只引用 exact source revision | Develata confirmed |
| `SRC-BASELINE-001` | fetch/parse/normalize/diff/candidate durable commit 全部成功后才能推进 baseline | Develata confirmed |

## 3. Authority split

### Git canonical declaration

Public reviewed Git 保存：

```text
source identity and display metadata
official owner/authority class
approved URL patterns/endpoints
retrieval method and rate policy
parser/policy IDs and versions
crawl permission/review evidence references
expected content types/locales
```

### PostgreSQL operational state

PostgreSQL 保存：

```text
crawl cursor/lease/retry
URL aliases/redirects
observations and revision metadata
fetch/parse/diff/candidate state
current accepted baseline pointer
source health and diagnostics
```

### Object storage

保存 immutable raw and normalized snapshots；object key 必须包含 source/revision/digest identity。Blob existence 不等于 source approved 或 revision accepted。

## 4. SourceDefinition

最小 schema：

```text
SourceDefinition
- source_id
- node_ids[]
- title/locales
- authority_class
- owning_organization
- operator_owner
- approved_url_policy
- retrieval_method
- crawl_permission_evidence_ref
- rate_policy
- parser_id / parser_version
- board_policy_id / board_policy_version
- expected_content_types[]
- status: Proposed | Approved | Suspended | Revoked
- review_revision
```

约束：

- `source_id` 不随 URL/path/title 改变；
- `Approved` 必须有 owner、authority、URL、retrieval、rate 与 permission review；
- missing/unknown authority、owner、parser、policy 或 URL rule 均 fail-closed；
- `Suspended/Revoked` 阻止新 fetch，但保留历史 evidence；
- source declaration 不包含 secret、private endpoint 或 user credential。

首版 public source capabilities 只读。需要 USTC login/cookie/token 的 source 不得混入 public default registry；必须另做 identity/data-class review。

## 5. SourceRevision

```text
SourceRevision
- source_revision_id
- source_id
- canonical_url
- retrieved_url
- url_aliases[]
- title
- authority_class
- published_at?
- observed_at
- effective_from?
- effective_to?
- response_metadata
- raw_digest
- normalized_digest
- raw_snapshot_ref
- normalized_snapshot_ref
- parser_id / parser_version
- status: Observed | Parsed | Accepted | Archived | Rejected
```

必须区分：

- `published_at`：网页声明的发布时间；
- `observed_at`：平台抓取时间；
- `effective_from/to`：规则适用时间；
- `retrieved_url`：实际请求 URL；
- `canonical_url`：规范化 logical URL；
- raw 与 normalized digest。

同一 URL 原地更新时生成新 `source_revision_id`。Redirect 后 URL 改变但内容/identity 连续时保留 alias/redirect evidence，不覆盖旧 revision。

## 6. URL normalization and lookup

URL normalization 至少处理：

- scheme/host case；
- default port；
- fragment removal；
- reviewed query parameter policy；
- redirect chain；
- canonical link 仅作 evidence，不自动信任；
- Unicode/percent-encoding normalization；
- duplicate slash/path normalization where safe。

不能仅凭 URL 唯一定位 revision。CLI：

```bash
ustc-agentctl source lookup --url <url>
ustc-agentctl source lookup --url <url> --at <RFC3339>
ustc-agentctl source lookup --url <url> --digest <sha256>
ustc-agentctl source history <revision-id>
```

无 `--at/--digest` 时若存在多个 revisions，返回 ordered list；不得任意选择一条旧 revision。

## 7. Fetch security boundary

Agent 不能把任意 URL 直接交给 privileged server fetch。URL 必须：

1. 来自 `Approved` SourceDefinition 的 exact/pattern rule；或
2. 先进入 source candidate review，不在当前 user request 中执行。

Fetch contract：

```text
HTTPS by default
exact reviewed host/path policy
DNS resolve and IP revalidation before connect
redirect after every hop re-authorized
loopback/private/link-local/metadata/multicast deny
bounded redirects/body/time/concurrency
content-type allowlist
conditional GET with ETag/Last-Modified when available
no user credential forwarding by default
untrusted page content treated as data, never instructions
```

Wildcard `*.ustc.edu.cn` 只可作为 platform egress ceiling；每个 SourceDefinition 仍需 exact host/path/retrieval review。

HTML/PDF parser 必须隔离 active content；禁止执行 page script、macro、embedded binary 或 arbitrary attachment。

## 8. Incremental retrieval

推荐 pipeline：

```text
scheduler acquires (source_id, node_id) lease
-> conditional fetch
-> immutable raw snapshot
-> deterministic normalization
-> digest compare
-> parser
-> normalized revision
-> semantic diff candidate
-> durable candidate/evidence commit
-> baseline advance
```

没有内容变化时记录 observation/health，但不生成 published change。

### Baseline invariant

不得在以下状态推进 current baseline：

- HTTP success but snapshot write failed；
- raw snapshot exists but parser failed；
- normalized digest unavailable；
- semantic diff/candidate durable write failed；
- audit/evidence receipt failed。

Crash/retry 使用 deterministic revision/event IDs，避免相同内容重复 revision/change。Non-idempotent publish 不属于 crawler fetch transaction。

## 9. Targeted refresh

Affairs query 默认读取 last reviewed procedure，不在每次请求中 live fetch。Targeted refresh 仅在以下情况触发：

- source 超过 board `max_staleness`；
- scheduled crawl/change detection 已发现新 revision；
- 管理员显式 refresh；
- 用户明确要求核对最新原文。

即使 refresh 成功，新的 ProcedureDraft/ChangeEvent 仍为 candidate；不得绕过 admin publish。

## 10. Maintainer agents

每个 source/board maintainer 只获得：

```text
approved source read/fetch
node-scoped candidate write
lease/cursor update
bounded model/tool budget
```

不得获得：

```text
canonical Git publish
arbitrary URL fetch
cross-board source mutation
user profile/memory broad read
platform admin/runtime credential
```

Agent output 必须记录 model/provider、skill/policy/parser versions、source revisions、run ID 与 candidate digest。

## 11. Initial registry scope

首个 concrete board/source 尚未冻结。进入 implementation 前必须为每个 source 建立 reviewed entry，并人工核对：

- 是否确为 USTC official/public authority；
- robots/terms/notice/crawl permission evidence；
- rate limit/refresh cadence；
- stable URL/attachment patterns；
- authority order 与 conflicts；
- parser fixture；
- historical revision availability；
- owner 与 suspension procedure。

赛事官方通知/FAQ 可作为 competition evidence source，不自动成为 Affairs 首个 board 的最佳 source。

## 12. CLI and diagnostics

最低 operator projection：

```bash
ustc-agentctl source registry-check --strict
ustc-agentctl source list [--status approved]
ustc-agentctl source inspect <source-id>
ustc-agentctl source lookup --url <url> [--at <time> | --digest <digest>]
ustc-agentctl source crawl-plan <source-id>
ustc-agentctl source crawl-apply --plan <path> --plan-digest <digest>
ustc-agentctl source history <revision-id>
ustc-agentctl source doctor <source-id> --live-readonly
```

`crawl-plan` 必须 non-mutating；不得提前推进 cursor/baseline、创建 durable snapshot 或 masking cache。

## 13. Failure and recovery

必须区分：

- source unavailable/timeout；
- unauthorized redirect/SSRF denial；
- content type/size violation；
- changed raw content but parser failure；
- normalized duplicate；
- conflicting official sources；
- stale source；
- object store/DB/audit write failure；
- baseline drift；
- source suspended/revoked during in-flight fetch。

失败保留最后 accepted baseline，输出 stale/uncertainty/diagnostic；不得静默使用未提交 revision。

## 14. Acceptance authority

主要 cases：

- `SRC-001..*`：definition、revision、URL、fetch、snapshot、baseline；
- `PROC-*`：source revision 到 procedure candidate/publish；
- `FP-*`：Affairs/ChangeRadar source-to-user journey；
- `EVAL-*`：citation、freshness、diff 与 refusal。

完整 matrix 见 [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)。
