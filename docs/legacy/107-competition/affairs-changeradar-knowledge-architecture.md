# Affairs Navigator 与 ChangeRadar Knowledge Architecture

## Metadata

- `Layer`: `Campus Knowledge / First-party Product`
- `Status`: **Develata-confirmed architecture candidate；待团队确认与 implementation evidence**
- `Version`: `0.1.0`
- `Last Review`: `2026-07-21`
- `Authority Owns`: tree/procedure ontology、lookup ladder、supersession、typed materialization、ChangeRadar maintainer/feed contract
- `Authority Defers To`: [`source-registry.md`](source-registry.md), [`agent-market-architecture.md`](agent-market-architecture.md), [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)

## 1. Product split

三个 first-party products 回答不同问题：

```text
Affairs Navigator：我现在该怎么办？
ChangeRadar：什么变了，是否影响我？
Opportunity Graph：什么适合我，下一步选什么？
```

本章只冻结前两者的共享 knowledge architecture。Opportunity Graph 的 profile/memory/consent/graph contract 另行设计，不得由本章猜测。

## 2. Confirmed decisions

| Decision ID | Decision | Status |
| --- | --- | --- |
| `KNOW-AUTH-001` | Git Markdown+YAML 保存 reviewed canonical tree/policy/procedure；PostgreSQL 保存 operational state/search projection；object storage 保存 immutable source evidence | Develata confirmed |
| `KNOW-LOOKUP-001` | lookup 依次使用 exact addressing、structured local search、approved-source targeted refresh；bounded RAG 仅作后续 recall fallback | Develata confirmed |
| `KNOW-STATE-001` | durable schema 使用 typed lifecycle；不使用可冲突的 `new/old + t/f + nullable path` 组合 | Develata confirmed |
| `KNOW-SUP-001` | supersession 保存 direct typed edges；不复制 transitive `old-web` 链；URL 不是 revision identity | Develata confirmed |
| `KNOW-PUB-001` | Agent 只生成 typed candidate；Rust validator 确定性 render Markdown；Demo 只有管理员可 publish canonical Git | Develata confirmed |
| `KNOW-RAG-001` | RAG 只能检索 approved official-source snapshots，不能覆盖 reviewed current procedure 或无依据补造流程 | Develata confirmed |
| `RADAR-MAINT-001` | 多个长期 Agent 是 board-scoped candidate maintainers，共用 source/change ledger，无 canonical publish authority | Develata confirmed |
| `RADAR-FEED-001` | RSS/Atom 只发布 approved semantic changes，订阅绑定 stable node ID，不发布 raw crawl noise | Develata confirmed |

“Develata confirmed”不等于团队已接受候选，也不表示 Source Registry 中已有经审核的 USTC crawl permission。

## 3. Lookup ladder

Affairs Navigator 不以 full-corpus RAG 为第一层。查找顺序固定为：

```text
L0 Exact addressing
   stable node ID / procedure ID / normalized URL / artifact reference

L1 Curated structured retrieval
   tree navigation + PostgreSQL FTS/pg_trgm + reviewed procedure artifacts

L2 Targeted source refresh/materialization
   approved source -> immutable snapshot -> parse -> typed draft
   -> validation -> deterministic Markdown -> admin approval

L3 Bounded corpus retrieval（later）
   semantic retrieval only inside approved official-source snapshots
```

约束：

- L0/L1/L2 不依赖 embedding；它们属于 deterministic/structured retrieval。
- L1 返回的 current reviewed procedure 优先于 L2/L3 candidate。
- L2/L3 只能产生 evidence-bearing candidate；未审核材料必须显示 `unreviewed`。
- 没有足够 authoritative evidence 时返回 `cannot_verify`，不能让模型补齐看似完整的流程。
- query-time refresh 不是默认路径；默认立即返回最后一个 validated/published procedure，并显示 freshness。

## 4. Authority and storage

### 4.1 Git canonical

Git 保存需要公开审计、review 和 rollback 的内容：

```text
stable wiki node definitions
navigation/tree projection
board policy manifests
reviewed procedure artifacts
procedure schemas and renderer templates
approved source registry declarations
```

推荐逻辑布局：

```text
knowledge/
├── tree.yaml
├── policies/
│   └── <policy-id>.yaml
├── procedures/
│   └── <procedure-id>/
│       ├── index.yaml
│       └── <artifact-id>.md
└── schemas/
```

`node_id`、`procedure_id`、`artifact_id` 是 identity；filesystem path 是可移动 projection。移动栏目不得改变 identity。

已发布 artifact 使用 immutable ID/digest；旧 artifact 不删除。`index.yaml` 指向 current artifact，并保留 archived references。Git history 不是唯一的 history query surface。

### 4.2 PostgreSQL operational authority

PostgreSQL 保存：

```text
crawl cursor/lease/retry
URL alias/redirect/revision index
fetch/parse/generation state
candidate changes and approval queue
FTS/pg_trgm search projection
RSS subscriptions/delivery state
maintainer jobs, budgets and receipts
```

PostgreSQL search/catalog projection 必须可从 pinned Git canonical revision 和 immutable evidence 重建。直接改 row 不能成为 canonical publish。

### 4.3 Object storage

保存 immutable raw/normalized evidence：

```text
HTML/PDF snapshots
normalized document snapshots
large semantic diff artifacts
optional rendered screenshots
```

Raw snapshot 可能受版权/隐私/访问条款限制，不因 canonical procedure 公开而自动公开。

## 5. Tree and policy model

### 5.1 Stable node

```text
WikiNode
- node_id
- parent_id?
- slug
- localized title
- policy_id
- status
```

用户按树浏览；Agent/search 按 stable ID 操作。订阅、policy、procedure relationship 均绑定 `node_id`，不绑定 path。

### 5.2 Board policy

每个 node 引用 versioned `policy.yaml`。Policy 至少声明：

```yaml
policy_id: affairs.it-services.v1
replacement_key:
  - procedure_key
  - audience
  - service_scope
authority_order:
  - university-regulation
  - department-notice
  - official-faq
required_sections:
  - applies_to
  - prerequisites
  - steps
  - entry_points
  - sources
max_staleness: P30D
require_admin_publish: true
```

Skill 负责推理/起草；policy 与 Rust validator 负责 deterministic constraints。关键覆盖规则不得只藏在 prompt 中。

## 6. Typed records

### 6.1 SourceRevision

Source identity/retrieval 细节由 [`source-registry.md`](source-registry.md) 拥有。Knowledge layer 只引用 immutable `source_revision_id`。

### 6.2 ProcedureArtifact

```text
ProcedureArtifact
- artifact_id
- procedure_id / procedure_key
- node_id
- state
- artifact_ref?
- digest?
- generated_from_revision_ids[]
- validator_version
- published_commit?
- last_verified_at?
```

状态机：

```text
Discovered -> Generated -> Validated -> Published
                    \-> Failed
Published -> Archived
```

`materialized=true` 不单独持久化；它由 `state == Published && artifact_ref != null` 派生，避免 `true + null` 等非法组合。

内部状态使用 `Current/Archived`。CLI/UI 可以显示短标签 `new/old`，但 `new` 不得同时表示“最近发现”与“当前有效”。

### 6.3 Required procedure content

Agent 先输出 `ProcedureDraft`，至少包含：

```text
procedure_key
title
applies_to
prerequisites[]
steps[]
deadlines[]
entry_points[]
contacts[]
source_revision_ids[]
conflicts[]
uncertainties[]
last_verified_at
```

Canonical Markdown 至少显示：适用对象、前置条件、步骤、截止/时效、入口、联系人、来源、冲突/不确定性、last verified。

## 7. Supersession and archive

### 7.1 Direct edges only

```text
SupersessionEdge
- old_revision_or_artifact_id
- new_revision_or_artifact_id
- coverage: Full | Partial | Clarification | Duplicate
- scope
- reason
- evidence_revision_ids[]
- proposed_by
- decided_by
- decided_at
```

只保存 direct edge：

```text
A -> B
B -> C
```

B 不复制 A，C 不复制 `[A, B]`。完整历史和“当前版本覆盖了哪些旧网页”由图查询派生。

### 7.2 Full coverage gate

只有以下条件同时满足，Agent 才可提出 `Full`：

1. 相同 `procedure_key`；
2. 新 authority 不低于旧 authority；
3. audience/jurisdiction/service scope 不变窄；
4. 旧 prerequisite、step、deadline、entry/contact 全部被保留、替换或明确废止；
5. effective interval 不留下未解释空洞；
6. 官方 evidence 支持替代关系，或管理员接受逐字段 coverage matrix；
7. parser/generator/validator 成功且 snapshot evidence 可重放。

部分覆盖使用 `Partial`；必要时拆分 scoped procedure。不得因相似度高就整体 archive 旧流程。

Archive 是状态转移，不是删除。Hard delete 只用于合法/隐私删除或尚未发布的重复 candidate，并必须审计。

## 8. Typed materialization and validation

Canonical production path：

```text
approved SourceRevision
-> Agent + reviewed Skill
-> ProcedureDraft JSON
-> Rust schema validator
-> cross-field / policy validator
-> citation/source validator
-> deterministic Markdown renderer
-> candidate artifact
-> administrator review/approve
-> atomic Git publish
-> PostgreSQL projection refresh
```

Hook 可以调用统一 validator，并修复纯格式问题，例如空白、heading order、newline。以下问题必须 fail，不得静默补造：

- deadline/entry/contact/适用范围缺失；
- source revision 不存在或不 approved；
- citation 不支持结论；
- authority/conflict 未解决；
- path/ID/digest 不一致；
- hidden/bidi/code/Markdown safety violation。

同一个 Rust validator 必须由 CLI、server、Git hook 与 CI 调用。Local hook 可绕过，因此不是 enforcement boundary。

Agent/maintainer 没有 canonical Git write capability。管理员 publisher 使用独立系统身份、typed publish operation、plan hash、idempotency key 与 audit receipt。

## 9. Search and answer path

### 9.1 Search

- `rg`：管理员本地调试、Git emergency fallback、CI consistency scan；
- PostgreSQL FTS + `pg_trgm`：server primary search；
- 暂不引入 Elasticsearch/Meilisearch 等独立搜索集群。

Search 返回 stable node/procedure/source revision IDs，不只返回 URL 字符串。

### 9.2 Freshness

默认回答显示：

```text
last verified
source observed time
source effective time
stale/refresh-pending warning
conflict/uncertainty
```

只有 source 超过 policy staleness、ChangeRadar 检测到 revision、管理员显式 refresh，或用户明确要求核对最新原文时，才触发 targeted refresh。

### 9.3 RAG boundary

Later bounded RAG 可以：

- 定位长 PDF/附件 evidence spans；
- 找到 keyword search 漏掉的同义表达；
- 提议 source/procedure candidate；
- 辅助起草 typed draft。

它不能覆盖 reviewed current、混入非官方来源、自动 publish/archive，或在冲突未解决时给出确定结论。

## 10. ChangeRadar

### 10.1 Shared change ledger

Affairs Navigator 与 ChangeRadar 共用：

```text
Source Registry
crawl scheduler
immutable snapshots
parser registry
semantic diff engine
candidate/approval state machine
change event ledger
RSS/Atom publisher
```

不得各自维护一套 crawler、source truth 或 baseline。

### 10.2 Board-scoped maintainers

每个长期 maintainer Agent 只拥有：

```text
node_id scope
source allowlist
policy ID/version
lease/cursor
model/tool budget
candidate proposal permission
```

它没有 canonical publish permission。并发最低要求：

- `(node_id, source_id)` lease；
- idempotency key `(source_id, normalized_digest, policy_version)`；
- 同一 revision 只生成一个 candidate；
- fetch/parser 可在 durable boundary 安全重试；
- publish 使用 deterministic event ID 和 durable operation receipt 防重。

### 10.3 Semantic change event

```text
ChangeEvent
- event_id
- node_id
- old_revision_id
- new_revision_id
- semantic_diff_ref
- affected_scope
- state: Proposed | Approved | Published | Rejected
```

HTML 版式变化、重复抓取、parser failure、unreviewed model inference 不构成 published semantic change。

### 10.4 RSS/Atom

首版提供公开 per-board RSS/Atom。Feed item 至少包含：

```text
stable GUID = change_event_id
stable node ID
change title
published_at / effective_at
before/after summary
affected scope
current procedure URL
diff/evidence URL
provenance
```

订阅绑定 stable `node_id`。个性化 private feed 涉及 profile/token/privacy，推迟到 Opportunity Graph 设计。

## 11. Initial CLI projection

CLI 名称最终由 registry slice 冻结；最低语义为：

```bash
ustc-agentctl knowledge node list
ustc-agentctl knowledge search --query <text>
ustc-agentctl source lookup --url <url> [--at <time> | --digest <digest>]
ustc-agentctl source history <revision-id>
ustc-agentctl procedure generate --source-revision <id> --candidate-out <path>
ustc-agentctl procedure validate --candidate <path>
ustc-agentctl procedure render --candidate <path> --out <path>
ustc-agentctl procedure publish-plan --candidate <path>
ustc-agentctl procedure publish-apply --plan <path> --plan-digest <digest>
ustc-agentctl change list --node <node-id>
ustc-agentctl feed build --node <node-id>
```

CLI 是 shared Rust domain contract 的 projection；server 不通过 subprocess 调 CLI 完成正常业务路径。

## 12. MVP slices

```text
A. Tree + typed records + URL/history CLI
B. 单一 board、管理员手工维护、full/partial supersession
C. approved USTC source incremental crawl + immutable snapshots
D. Agent typed draft + validator + admin approval
E. ChangeRadar semantic ledger + per-board RSS/Atom
F. bounded RAG
G. crowd maintenance / personalized feed
```

第一条 engineering slice 是 ChangeRadar 所需的 source/revision/diff core；第一条用户入口是 Affairs Navigator。首个具体 board/source、implementation repo 与 owner 仍待团队冻结。

## 13. Failure semantics

必须显式覆盖：

- URL 同址多 revision 与 redirect ambiguity；
- source stale/conflict；
- fetch succeeded but parse/generate failed；
- candidate validated but publish/audit failed；
- partial replacement误判为 full；
- maintainer duplicate/concurrent run；
- crash during non-idempotent publish；
- Git canonical 与 PostgreSQL projection drift；
- feed publish partial failure；
- RAG 返回未批准来源。

失败默认保留最后一个 reviewed current view、显示 stale/uncertainty，并阻止错误 baseline/publish advancement。

## 14. Acceptance authority

对应 suites：

- `SRC-*`：source/revision/fetch/baseline/evidence；
- `PROC-*`：tree/procedure/supersession/materialization；
- `FP-*`：Affairs/ChangeRadar 用户价值与 feed；
- `EVAL-*`：事实、引用、时效、diff、拒答；
- `AGENT-*`：hook/gate/recovery/idempotency。

完整 case contract 见 [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)。
