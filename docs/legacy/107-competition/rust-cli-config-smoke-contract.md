# Rust CLI and Configuration Smoke Contract

## Metadata

- `Layer`: `Verification / Configuration / Operator Surface`
- `Status`: **Develata-confirmed architecture requirement；implementation not started**
- `Version`: `0.1.0`
- `Last Review`: `2026-07-21`
- `Authority Owns`: Rust CLI-first verification、typed configuration、config smoke、evidence schema、exit semantics
- `Authority Defers To`: [`agent-market-architecture.md`](agent-market-architecture.md), [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md), [`project-documentation-and-execution-blueprint.md`](project-documentation-and-execution-blueprint.md)

## 1. Governing decisions

| Decision ID | Requirement | Status |
| --- | --- | --- |
| `VER-CLI-001` | 尽可能以 Rust CLI 暴露配置、诊断、验证、catalog/package inspection 与 acceptance entrypoints | Develata confirmed |
| `VER-CONFIG-001` | 所有可交付配置必须通过 typed Rust loader 与 configuration smoke | Develata confirmed |
| `VER-MATRIX-001` | 必须维护完整 plan/feature/case/evidence 验收矩阵，语义参考 Deve-Notebook | Develata confirmed |
| `VER-EVIDENCE-001` | Smoke/acceptance 输出 machine-readable、可追溯、可验证；skip/unavailable 不得冒充 pass | Develata confirmed |

这些 `VER-*` 是 [`agent-market-architecture.md`](agent-market-architecture.md) 中 parent decision `MKT-VER-001` 的分解投影，不是第二套平行决策：

| Parent | Projection | Primary acceptance |
| --- | --- | --- |
| `MKT-VER-001` | `VER-CLI-001` | `DOC-004`, `SEC-*`, `DEP-*` |
| `MKT-VER-001` | `VER-CONFIG-001` | `CFG-001..020` |
| `MKT-VER-001` | `VER-MATRIX-001` | `DOC-001..008` |
| `MKT-VER-001` | `VER-EVIDENCE-001` | `DOC-003`, `DOC-008`, `REL-001`, `REL-008` |

## 2. Rust crate first, CLI projection second

“Rust CLI 化”不表示 production services 彼此通过 shell subprocess 通信。正确分层：

```text
Rust domain/config/verification crates
├── central server startup
├── workers/runtime controller
├── Market/admin API
├── ustc-agentctl CLI
└── tests/CI/deploy/restore gates
```

建议 workspace ownership：

```text
crates/
├── config-contract        # typed config, merge, validation, redaction
├── catalog-contract       # manifest/schema/resolution
├── capability-contract    # capability registry and diff
├── acceptance-contract    # case registry, runner traits, result model
├── evidence               # atomic evidence writer/verifier
└── diagnostics            # doctor/preflight/read-only probes

apps/
├── server
├── worker
└── ustc-agentctl
```

不变量：

- public behavior 先由 plan/registry 定义，再投影为 Rust types；
- CLI、server 与 tests 使用同一 crates，不各写一套 parser/validator；
- production request path 不 shell-out 到 `ustc-agentctl`；
- shell 只做 environment bootstrap 与顺序编排；
- Python 只在协议 conformance、独立 differential oracle 或外部生态 SDK smoke 中使用；
- web/admin UI 只调用 typed API，不拼接 CLI command string；
- mutating CLI 默认为 dry-run，必须显式 `--apply`，且由 domain service 重复权限检查。

## 3. CLI name and global contract

建议单一 operator/developer binary：

```text
ustc-agentctl
```

所有 subcommand 支持：

```text
--format human|json
--profile <name>
--config <path>
--evidence-out <path>
--correlation-id <id>
```

约束：

- `--format json` 时 stdout 只输出一个 versioned JSON document；
- diagnostics 写 stderr，且统一 redaction；
- secret value、authorization header、raw token、private payload 不进入 stdout/stderr/evidence；
- 未指定 `--evidence-out` 时不得静默在当前目录生成文件；
- evidence 写入必须 atomic replace，失败则整次 command 非 pass；
- command/help/schema 进入 CLI registry，并由 drift check 对比实际 clap command tree；
- `--help`、JSON schema 和 docs registry 中的 command/key 名必须一致。

## 4. Public command tree

```text
ustc-agentctl
├── config
│   ├── schema
│   ├── check
│   ├── print-effective
│   ├── diff
│   └── smoke
├── doctor
├── preflight
├── catalog
│   ├── validate
│   ├── import-plan
│   ├── sync
│   └── drift-check
├── plugin
│   ├── validate
│   ├── resolve
│   ├── permission-diff
│   └── installation-inspect
├── capability
│   ├── registry-check
│   └── classify
├── runtime
│   ├── inspect
│   ├── admission-check
│   ├── cold-start-smoke
│   └── drain-smoke
├── acceptance
│   ├── list
│   ├── matrix-check
│   ├── run
│   └── report
└── evidence
    └── verify
```

### 4.1 Mutating command discipline

- `catalog import-plan` 只生成 deterministic plan；
- `catalog sync` 默认 dry-run，`--apply` 才写 projection；
- `runtime cold-start-smoke` 只能操作 dedicated test namespace/tenant/artifact；
- acceptance runner 不得借 smoke 修改 production user/plugin/catalog state；
- 不提供 `shell`、`exec-any`、`sql`、`docker-run`、`set-arbitrary-key`；
- arbitrary passthrough arguments 不得越过 typed command schema。

## 5. Configuration authority

### 5.1 Sources and precedence

推荐确定性层次：

```text
compiled safe defaults
  < base config.toml
  < selected profile overlay
```

Environment 只允许：

- 选择 config path/profile；
- 解析 config 中显式声明的 secret/env reference；
- CI/test 中注入 ephemeral test endpoints。

Environment 不得任意覆盖所有 config keys，否则 effective config 无法审计。Production 不接受任意 `--set key=value`；测试 override 必须属于独立 test profile，并写入 evidence metadata。

### 5.2 Typed schema

每个 key 至少定义：

```text
key
Rust type
required/default
allowed values/range
profile applicability
secret classification
reload class
owner module
validation rule
redaction rule
plan_ref
```

规范：

- unknown key：hard error；
- wrong type/range：hard error；
- duplicate key：hard error；
- missing required key：hard error；
- deprecated key：在明确 migration window 内给出 stable warning/error；首个公开 release 前不保留无权威历史兼容；
- secret value 不得直接出现在 config；只能使用 typed `SecretRef`；
- endpoint/path/identity/capability 不是普通 String，必须使用 dedicated newtype validation；
- startup 必须调用与 `config check/smoke` 相同的 loader/validator。

### 5.3 Config registry projection

未来 implementation repo 必须维护：

```text
docs/registry/config-key-registry.md
docs/registry/cli-command-registry.md
```

语义 authority 在 plan；Rust `ConfigKeySpec` 是 executable schema；registry Markdown/JSON Schema 由 Rust CLI 生成或校验，不允许手工维护第二份独立 key list。

推荐 gate：

```bash
ustc-agentctl config schema --format json > target/config-schema.generated.json
ustc-agentctl acceptance matrix-check --strict
```

CI 比较 generated schema/command tree 与 checked-in registry projection；有 drift 即失败。

## 6. Configuration smoke levels

### 6.1 `static`

```bash
ustc-agentctl config smoke \
  --level static \
  --profile demo \
  --config config.toml \
  --format json
```

只读、无网络、无 DB、无 secret value resolution，检查：

- parse/type/range/unknown/duplicate keys；
- profile merge determinism；
- required key/ref syntax；
- URL/path/domain/locale/digest/capability identifier syntax；
- dev provider 禁止进入 production profile；
- public/admin/runtime listen surface 不能冲突或错误暴露；
- resource limits 与 8C16G profile budget 基本一致；
- declared component/config schema 与 registry revision 一致；
- redaction classification 完整。

`static` 不得创建目录、cache、DB、lock、migration state、telemetry 或 evidence，除非显式给出 `--evidence-out`。

### 6.2 `resolved`

```bash
ustc-agentctl config smoke \
  --level resolved \
  --profile demo \
  --config config.toml \
  --format json
```

在 `static` 基础上只解析存在性与权限，不泄露 value：

- secret/env references 是否存在；
- config/cert/key/artifact/storage paths 是否可安全访问；
- owner/mode 是否满足要求；
- hostnames/DNS names 与 endpoint allowlist 是否可解析为允许类别；
- exact artifact/schema/catalog revisions 是否存在；
- production profile 不引用 development credential/provider。

不得建立业务 session、执行 migration、发邮件、提交表单、启动 MCP、写 catalog projection。

### 6.3 `live-readonly`

```bash
ustc-agentctl config smoke \
  --level live-readonly \
  --profile demo \
  --config config.toml \
  --format json
```

在 `resolved` 基础上执行最小 read-only dependency probes：

- PostgreSQL connect + transaction rollback/read-only query；
- schema migration compatibility check，不自动 migrate；
- Git catalog revision/read surface；
- artifact registry digest HEAD/read metadata；
- reverse proxy/backend readiness；
- IdentityProvider metadata/assertion validation configuration，不创建普通用户；
- configured external endpoint TLS/SSRF classification；
- optional Redis ping；Redis unavailable 不得丢 durable truth，但若 profile 声明 required 则 smoke 失败。

禁止：install、publish、revoke、cold start、grant、credential rotation、user creation、migration apply、schema mutation。

## 7. Config smoke mandatory cases

| Case ID | Assertion | Level |
| --- | --- | --- |
| `CFG-001` | minimal valid config parses deterministically | static |
| `CFG-002` | unknown key fails closed | static |
| `CFG-003` | wrong type/range fails closed | static |
| `CFG-004` | missing required key/ref fails closed | static |
| `CFG-005` | profile precedence is deterministic and recorded | static |
| `CFG-006` | duplicate/conflicting declaration fails closed | static |
| `CFG-007` | secret literal in config is rejected | static |
| `CFG-008` | effective config output is redacted | static |
| `CFG-009` | unsafe URL/path/listen surface is rejected | static |
| `CFG-010` | production profile rejects development identity/provider | static |
| `CFG-011` | config registry and Rust schema have zero drift | static |
| `CFG-012` | all service binaries use the same checked loader | offline Rust integration / PR |
| `CFG-013` | missing secret/env reference is reported without value leakage | resolved |
| `CFG-014` | unsafe owner/mode/path is rejected | resolved |
| `CFG-015` | missing exact artifact/catalog/schema revision fails | resolved |
| `CFG-016` | PostgreSQL connectivity/schema compatibility is read-only | live-readonly |
| `CFG-017` | catalog and artifact read probes resolve exact revision/digest | live-readonly |
| `CFG-018` | IdP validation config is complete and protocol-appropriate | live-readonly |
| `CFG-019` | optional Redis loss cannot change durable truth | live-readonly/integration |
| `CFG-020` | every smoke level leaves durable state unchanged | all |

## 8. Doctor and preflight

`config smoke` 证明 config contract；`doctor` 汇总当前 deployment health，但不能放宽 config failure。

```bash
ustc-agentctl doctor --scope all --profile demo --format json
ustc-agentctl preflight --target demo-host --format json
```

`doctor` suites：

```text
config
catalog
identity
postgres
artifact
market-api
runtime
first-party-services
audit/evidence
```

`preflight` 只读检查 host/runtime prerequisites：OS/arch、CPU/RAM/disk、filesystem、ports、DNS/TLS、container/user namespace、backup target、time sync、SSH、egress、IdP access。System state 检查必须来自 real host evidence，不从 profile 文档猜测。

## 9. Acceptance runner

```bash
ustc-agentctl acceptance list --format json
ustc-agentctl acceptance matrix-check --strict --format json
ustc-agentctl acceptance run --suite config --mode offline --format json
ustc-agentctl acceptance run --suite market --mode integration --format json
ustc-agentctl acceptance run --case PKG-006 --mode integration --format json
ustc-agentctl acceptance run --required-for demo --format json
ustc-agentctl evidence verify --dir evidence/<run-id> --format json
```

Modes：

```text
offline       # parser/schema/unit/fixture; no network
integration   # isolated DB/services/test tenant
browser       # browser automation + screenshots/console/network evidence
real-host     # target deployment read/write tests in explicit test namespace
external      # external protocol/client/source conformance
```

规则：

- case registry 是 Rust typed static/data registry，不扫描 Markdown 猜 case；
- docs matrix 必须与 Rust registry 双向覆盖；
- case dependency 显式声明；
- destructive/real-host case 只能操作 test namespace，并需要 explicit target；
- required case 的 `Unavailable/Skipped/NotRun` 均使 suite non-pass；
- manual case 必须在 current `acceptance-bindings.tsv` / future `docs/acceptance-bindings.tsv` 绑定 type、owner、evidence path、status；`owner=unassigned` 或 `status!=pass` 使 required gate non-pass；
- suite 退出成功要求所有 required cases 为 `Pass`；
- flaky retry 次数与每次结果进入 evidence，不把最终一次成功抹去历史失败。

## 10. Evidence schema

建议 JSON envelope：

```json
{
  "schema_version": "ustc-agent.acceptance/v1",
  "run_id": "generated-id",
  "source_revision": "exact-revision",
  "binary": {
    "name": "ustc-agentctl",
    "version": "0.1.0",
    "digest": "sha256:..."
  },
  "profile": "demo",
  "mode": "integration",
  "config_digest": "sha256:...",
  "started_at": "RFC3339",
  "finished_at": "RFC3339",
  "status": "pass|fail|unavailable|skipped|not-run",
  "cases": [
    {
      "case_id": "CFG-001",
      "status": "pass",
      "duration_ms": 12,
      "evidence": ["relative/path.json"],
      "error_code": null
    }
  ],
  "summary": {
    "required": 1,
    "passed": 1,
    "failed": 0,
    "unavailable": 0,
    "skipped": 0,
    "not_run": 0
  }
}
```

约束：

- source revision、binary digest、config digest、target/profile 必须存在；
- timestamps 使用 UTC/RFC3339；
- evidence paths 相对 run directory，不允许 path traversal；
- JSON schema versioned；
- output 中 secret field 始终 redacted/omitted；
- `evidence verify` 检查 schema、hash、case registry、file existence 与 path safety；
- browser evidence 至少包含 screenshot、console/network error summary 与 viewport/locale；
- real-host evidence 包含 target identity fingerprint，但不包含 credential。
- required case 为 `skipped` 时 suite exit 非零：前置条件缺失使用 exit `4`；runner selection/contract 错误使用 exit `3`。Optional case 可记录 `skipped`，但不得增加 passed count。

## 11. Exit codes

| Code | Meaning |
| ---: | --- |
| `0` | all required assertions passed |
| `2` | invalid CLI usage/input |
| `3` | config/contract validation failed |
| `4` | required prerequisite unavailable；不是 pass |
| `5` | runtime/integration assertion failed |
| `6` | evidence write/verification failed |
| `7` | security/policy gate denied operation |
| `70` | internal software error |

Expected-denial case 若实际被正确拒绝，则 case 为 Pass；这不等于 command 本身把所有 policy denial 当成功。

## 12. CI and deployment gates

### Pull request quick gate

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
ustc-agentctl config smoke --level static --profile ci
ustc-agentctl acceptance matrix-check --strict
ustc-agentctl acceptance run --required-for pr --mode offline
```

### Integration gate

```text
config smoke: resolved + live-readonly
acceptance: integration suites
catalog projection rebuild
isolated test-tenant install/disable/enable/revoke
browser Market journey
```

### Demo deployment gate

```text
preflight real host
config smoke: all three levels
doctor: all required scopes
acceptance run --required-for demo
backup/restore dry-run
browser en-US/zh-CN
real-host hosted MCP test namespace
external read-back/evidence verify
```

### Release gate

Release artifact 必须从 authenticated/public target surface 重新下载，在 clean directory 执行：checksum、extract、`--version`、config static smoke、acceptance registry/matrix check。Local build success 不代替 remote artifact evidence。

## 13. Failure and recovery

必须可诊断：

- config parse/validation error 指向 key 与 stable error code，不打印 value；
- unresolved secret 只打印 ref identity；
- live probe timeout 区分 DNS/TLS/auth/connect/schema；
- evidence write failure 不留下标记为 pass 的 partial report；
- acceptance interrupted 标为 incomplete/not-run；
- matrix drift 阻止 merge/release；
- smoke 不自动修复 production config；另设 explicit plan/apply workflow；
- `--dry-run` 不创建 durable baseline/cache/lock，不掩盖下一次真实运行；
- production startup 复用 checked loader，不能用 hidden fallback 绕过 smoke。

## 14. Explicit non-goals

- Bash 作为配置 authority；
- Python 复制一套业务 validator；
- production services 通过 CLI subprocess 完成正常 domain call；
- generic `exec` / arbitrary SQL / arbitrary Docker arguments；
- “command exited 0”但 required cases skipped；
- 手工 Markdown key list 与 Rust schema 双向漂移；
- smoke 自动修复、迁移或发布；
- 把 local fixture 结果当 real-host/SSO/network evidence。
