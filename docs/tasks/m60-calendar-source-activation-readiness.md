# M60 calendar source activation readiness proposal

## Mutable state

- `Stage`: `PROPOSAL_ONLY_R48_REVIEWED_RECEIPT_BOUND`
- `Disposition`: `READY_FOR_NON_AUTHORITATIVE_DIRECTION_DECISION_NOT_ACCEPTED_CONTRACT`
- `Bound source commit`: `54d758fbf2f1c08df2e1993919287569b501b115`
- `Bound source tree`: `973b999d14feb91f5ebe84b1712006e18e21baeb`
- `Source candidate`: `ustc-teach-calendar-fall-2026`
- `Exact URL`: `https://www.teach.ustc.edu.cn/calendar/20135.html`
- `Permission/rate/retention posture`: selected by Develata on 2026-09-02 (Asia/Shanghai)
- `Current SourceStatus`: `Proposed candidate only; no durable source row exists`
- `Packet digest`: `sha256:9a7583db7104b3b8f500a87ff9eb5f7260157bab6fe776fc5fb7a0c8c77368b5` over `153508` bytes beginning immediately after the packet `BEGIN` marker newline and ending immediately before the `END` marker token, including the final packet newline
- `Candidate generation`: `R48`
- `Remote posture`: no push, Draft PR, CI, merge, source-status mutation, live retrieval, retained Rust implementation, deployment, release or publication is authorized by this packet

## Authority receipt

Develata selected the following source-specific posture on 2026-09-02 (Asia/Shanghai):

> 接受公开发布 + robots 未禁止作为竞赛原型的 bounded approval 依据；批准 exact HTML URL 每 6 小时最多一次，仅内部保留 raw evidence、产品只展示 normalized facts + source links

This settles Develata's internal bounded-use risk posture, rate and retention policy for this one competition-prototype source candidate. It is not source-owner licence or republication permission, does not approve another URL or host, and does not waive typed source review, parser-fixture, M00/M10 admission, M60 authority, M90 transport or durable-evidence gates.

The source remains only a `Proposed` candidate until one exact definition is durably inserted as `Proposed`, all four review-evidence axes close on one exact generation and an accepted retrieval protocol represents the exact request bytes. Public accessibility and robots posture are technical observations considered in Develata's internal decision; neither is generalized into a copyright licence or wholesale-republication claim.

<!-- M60_CALENDAR_SOURCE_ACTIVATION_PACKET:BEGIN -->
## 1. Objective, authority and strongest claim

This proposal freezes the next source-activation contract boundary:

```text
one exact reviewed SourceDefinitionV2
→ one exact identified request protocol
→ one M00-admitted source.approve command
→ M60 durable source authority + admission/start/journal/snapshot semantics
→ M90 non-authority transport and physical persistence adapters
→ deterministic source-specific parser fixture
→ later SourceRevision / baseline / product work
```

The strongest current claim is:

```text
The exact 2026 autumn teaching-calendar HTML source has a Develata-selected
competition-prototype bounded-use/rate/retention posture and an exact source
review candidate. It is eligible for a semantic contract decision, but it is
not approved or activated: retained probes suggest User-Agent-dependent admission,
exact-v0 request-byte causality is unverified, the production User-Agent has not
yet been exercised by an approved attempt, no retained
parser fixture exists, and no durable approval/B3/M90 path exists.
```

This file is proposal authority only. It does not amend a live contract, create or approve a source row, fetch a source, retain raw bytes in Git, implement Rust, promote acceptance, or make M60 complete.

## 2. Exact source and evidence custody

The candidate page identifies itself as `2026年秋季学期`, is published under the 中国科学技术大学教务处 site, and presents the teaching-calendar table, explanatory dates and class-period table.[1] The official calendar index links the same exact page as the 2026 autumn entry.[2] The department-responsibility page says 教务处编印《中国科学技术大学教学日历》, supporting `SourceAuthority::ReviewedOfficialSource` for this exact document rather than for the whole domain.[4]

The current `robots.txt` does not disallow `/calendar/20135.html` or `/calendar/`; it disallows search and selected unrelated paths and separately denies named high-volume bots.[3] This is not source-owner licence. Develata accepted the residual risk only for the exact competition-prototype use above.

Read-only evidence collected outside Git is bound by manifest `ustc-teach-calendar-fall-2026-source-review-20260902-r1`, sha256 `6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1` over `4628` bytes. It verifies `11/11` named artifacts:

```text
headers.txt                         143  546d88812180015d9b03ac05740b87614fab7691f2c3529004d480d11297a1ad
transport-summary.txt               108  ab74f048c5b0ccd613512d68afc95787d5514f0e93ad46d6e58daf8e82c05a54
headers-with-ua.txt                 295  75718f4c7b48d67b6ab1c305867cafb1de52f4fd2ac599bb41c0c4d25052d4c6
transport-with-ua-summary.txt       127  def6b686c4ef568d700562e973f380169f5a7d9f0c3f02f570ffcc00e0e3e7fa
calendar-20135-with-ua.html       44815  fa704b60205058d3c73326de41fba7acb97e98336fff3107fe08c7b2e87059cc
article-post-20135.html             9620  e426cffd8e2a39d330e2b5c4428f832c63e087b703785a8d4310e046da7526ad
proposed-normalized-oracle.json      812  07ac00567dcfd7bd7b832c3120b7649205c3dadb5b2b8999df69f9eff6223c75
citations.json                      2097  40f6cee289da154388f91937eec602995c01f888410a572015ad3ef0827b370c
calendar-index-extract.txt           283  0a93a253b201924de511bd15fe796203085769677da00cea979d732aa272d6bc
robots-extract.txt                   210  ce3ee8053e1ea8f3ef902d695d4bb007f46158b9297f32505ef67c003ae21670
responsibility-extract.txt           403  ba8c67116b557f6c0612c434540b91fb91d4e85ec98392bc2b59e7dcff1eeab3
```

Reconnaissance custody also records one third, out-of-scope pre-policy request at `2026-09-01T17:08:22Z` to the public Google Calendar ICS URL embedded by the page: `https://calendar.google.com/calendar/ical/7m03dafoaj5coevc3ai95ocs5c@group.calendar.google.com/public/basic.ics`. It returned `302`, redirects were not followed and the retained body was empty. Response headers contained a sensitive third-party `Set-Cookie`, whose value is intentionally absent from this repository/proposal and all summaries. This request is not one of the two exact-source probes, is not source-review/permission/rate/parser evidence, does not widen the approved exact USTC URL/host and must never be replayed by the product.

Those excluded ICS headers/summary/empty body/checksum plus one redundant exact-source body checksum were moved out of the source-review custody directory into a mode-`0700` quarantine with mode-`0600` files. The exclusion receipt `uca-source-evidence-20260902-unbound-r1` is sha256 `da34d8908f4ceca1921585234333063489bb3c72d146a934b8062504b6ea156f` over `2010` bytes. It binds five files without projecting the sensitive header value. The canonical source-review directory now contains exactly its manifest plus the manifest's `11` entries; the manifest is the self-identity carrier and is not recursively listed as its own artifact.

`citations.json` uses the `Asia/Shanghai` calendar date for `accessed`; capture timestamps and exclusion receipts use UTC RFC 3339. This timezone distinction is presentation metadata only and does not change any captured byte digest.

```text
without User-Agent:
  HTTP 403
  Content-Type: text/html
  Content-Length: 162

with User-Agent: USTC-Campus-Agent-Source-Review/0.1:
  HTTP 200
  Content-Type: text/html; charset=UTF-8
  Content-Length: 44815
  redirect: none
  content-encoding / ETag / Last-Modified: absent
  body sha256: fa704b60205058d3c73326de41fba7acb97e98336fff3107fe08c7b2e87059cc

article#post-20135 exact slice:
  bytes: 9620
  sha256: e426cffd8e2a39d330e2b5c4428f832c63e087b703785a8d4310e046da7526ad
  script/form/iframe/object/embed count: 0
  tables: 2
```

The complete HTML, headers and article bytes remain internal and outside Git. Repository/product surfaces may retain only digests, the exact source link, parser identity, normalized oracle and typed facts. The retained summaries do not include immutable pre-TLS sent-request bytes or request digests, so they do not prove that `User-Agent` was the sole causal wire delta; the negative observation is calibration evidence only. The two exact-source probes, 40 seconds apart, and the separately disclosed excluded ICS request occurred before Develata selected the `21600`-second policy; none is rate-compliance evidence. Every future attempt, including production-User-Agent conformance, is subject to the selected interval and a new immutable manifest.

## 3. Source-specific policy target

```text
SourceId: ustc-teach-calendar-fall-2026
SourceOwner: 中国科学技术大学教务处 / www.teach.ustc.edu.cn
SourceUrl: https://www.teach.ustc.edu.cn/calendar/20135.html
SourceAuthority: ReviewedOfficialSource
minimum_interval_seconds: 21600
maximum_response_bytes: 131072
maximum_elapsed_seconds: 20
expected_media_type: text/html
public_ip_policy_version: V0Ipv4Only20260809
protocol_version: future accepted identified-request version; NOT current v0
source_use_policy: SourceUsePolicyV1(Denied,
                   InternalRawEvidenceOnly,
                   NormalizedFactsAndExactLinksOnly)
```

Restrictions:

1. only the exact lowercase HTTPS URL is in scope;
2. no query, fragment, alternate path, index traversal, attachment, PDF, Word, Google Calendar URL or redirect is admitted;
3. `21600` seconds is the hard minimum between actual attempt starts; retry, failure and no-change do not bypass it;
4. `SourceOperatorPolicyPort::authorize_start` returns transaction-current `SourceStartAuthorization.capabilities = AttemptOnly`; `AttemptAndRateOverride` fails before reservation/start and mints no effect-ready carrier;
5. response body is capped at `131072` bytes and elapsed time at `20` seconds; all stricter generic wire/head/framing limits remain;
6. credentials, cookies, proxy, compression, referer, client certificate and environment-derived headers are forbidden;
7. M60 owns `InternalRawEvidenceOnly + NormalizedFactsAndExactLinksOnly`; M90 persists only through M60-owned ports and cannot broaden retention/publication;
8. user-visible output carries normalized facts, qualifications and the exact source link, never wholesale HTML;
9. the project remains a student competition prototype, not an official USTC service.

The incompatible successor is a distinct `source-import/v2` contract whose
`SourceDefinitionV2` embeds one closed, versioned M60 authority value rather
than extending accepted `source-import/v1` in place:

```text
SourceUsePolicyV1 {
  rate_override: Denied,
  raw_retention: InternalRawEvidenceOnly,
  public_projection: NormalizedFactsAndExactLinksOnly,
}

SourceRetrievalProtocolVersionV2 =
  V0StrictHttpsIpv4Http11_20260809 |
  V1StrictHttpsIpv4Http11Identified_20260902

SourceDefinitionSchemaVersionV2 = V2

SourceDefinitionVersionTag = V1 | V2

SourceRetrievalPolicyV2 {
  minimum_interval_seconds: u32,
  maximum_response_bytes: u32,
  maximum_elapsed_seconds: u32,
  expected_media_type: SourceMediaType,
  protocol_version: SourceRetrievalProtocolVersionV2,
  public_ip_policy_version: PublicIpPolicyVersion,
  use_policy: SourceUsePolicyV1,
}

SourceDefinitionV2 {
  schema_version: SourceDefinitionSchemaVersionV2,
  source_id: SourceId,
  owner: SourceOwner,
  url: SourceUrl,
  authority: SourceAuthority,
  retrieval_policy: SourceRetrievalPolicyV2,
  authority_revision: SourceAuthorityRevision,
  status: SourceStatus,
}

RetrievalSubjectV2 {
  source_id, owner, url, authority, retrieval_policy,
  authority_revision, approval_receipt
}
```

`source-import/v1`, its exact six-field `SourceRetrievalPolicy`, its one-variant
`SourceRetrievalProtocolVersion` and all persisted/public v1 values remain byte-
and behavior-compatible. R1 must introduce the explicitly version-discriminated
v2 types above and a v2 repository row; it may reuse existing nominal identity,
status and authority types but may not deserialize/reinterpret a v1 definition as
v2. `propose_exact_v2` rejects a same-`SourceId` v1 row as
`LegacyVersionOccupied`; migration, if ever required, is a separately accepted
operation. This candidate has no durable source row, so R5 creates a fresh v2 row
rather than migrating v1 state.

The three use-policy fields are part of the complete v2 definition, its authority
revision and canonical definition digest. `propose_exact_v2`, approval/evidence
anchors, admission/replay identity, transaction-current start authorization,
raw-snapshot acceptance and every later publication gate therefore bind the same
value. Unknown enum values fail decode; no default/fallback exists. For this
source, any override-bearing command or authorization is a `PolicyDenied`
admission rejection before `Admitted`, slot acquisition or override consumption;
only its required durable dual-index rejection tombstone is written, and no
start/effect carrier can exist. Raw evidence can enter only the M60-owned
immutable evidence port, while any product projection must separately prove
`NormalizedFactsAndExactLinksOnly` against this same v2 definition
revision/digest.

## 4. Retrieval protocol version closure

`source-retrieval/v0` serializes exactly `Host`, `Accept`, `Accept-Encoding: identity` and `Connection: close` and forbids `User-Agent`. A retained no-User-Agent observation returned `403`, but absent sent-request bytes it is not exact-v0 causality proof. Independently of that causal uncertainty, current v0 cannot represent the identified request shape used by the successful review probe, and M90 must not inject a header outside a versioned protocol.

The successful probe used `USTC-Campus-Agent-Source-Review/0.1`. The proposed production identifier is unverified. A future contract may add exactly:

```text
V1StrictHttpsIpv4Http11Identified_20260902
```

The proposed protocol delta relative to accepted v0 is exactly:

```text
User-Agent: USTC-Campus-Agent/0.1\r\n
```

Exact header order:

```text
GET <exact path> HTTP/1.1\r\n
Host: <exact host>\r\n
Accept: text/html\r\n
Accept-Encoding: identity\r\n
User-Agent: USTC-Campus-Agent/0.1\r\n
Connection: close\r\n
\r\n
```

The value is fixed platform identification, not caller/configuration input. Every v0 SSRF, DNS/public-IP, TLS, redirect, status, response-head, media-type, framing, body, wire and deadline invariant remains unchanged. V0 remains byte-exact; v1 definitions do not migrate; one v2 definition chooses one protocol; unsupported/version-mismatched values fail closed. `source-retrieval/v1` and `SourceRetrievalProtocolVersionV2` in `source-import/v2` must be introduced synchronously with serializer/checker/mutation tests; accepted `source-import/v1` remains unchanged.

V1 byte representability will be proved in R2 by exact serializer tests. Remote acceptance of the production User-Agent and the exact v1 header order remains `NEEDS_VERIFICATION` until the first approved, B3-journaled attempt; the successful review probe used a different identifier and header order and is not byte-order evidence. No post-policy network probe may occur outside that journal merely to make approval look safer: endpoint liveness is not source-review authority, and a first approved attempt may fail without creating a revision or baseline. No Rust mutation precedes explicit semantic acceptance of the exact R1 owning-contract patchset.

## 5. Parser-fixture contract

The parser is a deterministic M60 peer. It consumes bounded inert bytes after retrieval validation and emits a typed candidate; it owns no approval, baseline or publication authority.

Fixture layers:

1. internal exact-source body/article identified by §2 hashes, never committed to Git;
2. repository-safe synthetic minimal HTML exercising article/table/list structure without wholesale copying;
3. repository-safe normalized oracle containing permitted typed facts and exact source URL.

The canonical proposed oracle uses schema `ustc-teach-calendar-fall/v0-proposed-oracle` and sha256 `07ac00567dcfd7bd7b832c3120b7649205c3dadb5b2b8999df69f9eff6223c75` over `812` UTF-8 bytes. Canonicalization is sorted keys, `,`/`:` separators, `ensure_ascii=false`, no other whitespace and one trailing LF. Exact bytes before LF:

```json
{"classes_start_date":"2026-08-31","freshman_classes_start_date":"2026-09-07","freshman_exam_date":"2026-08-22","freshman_registration_date":"2026-08-21","freshman_training_end_date":"2026-09-06","freshman_training_start_date":"2026-08-23","holiday_adjustment_qualification":"具体放假及调课安排以学校通知为准","next_classes_start_date":"2027-02-22","next_registration_date":"2027-02-21","page_published_at_local":"2026-05-21T15:00:00+08:00","registration_date":"2026-08-30","schema":"ustc-teach-calendar-fall/v0-proposed-oracle","semester":"2026-fall","semester_end_date":"2027-01-15","source_id":"ustc-teach-calendar-fall-2026","source_url":"https://www.teach.ustc.edu.cn/calendar/20135.html","teaching_week_count":20,"winter_break_end_date":"2027-02-20","winter_break_start_date":"2027-01-16"}
```

The parser consumes the bounded full-page HTML, requires one `article#post-20135`, and derives `page_published_at_local` only from exactly one `li.meta-date[title="发布时间"]`; the print-summary timestamp and `li.meta-last-modified[title="上次修改时间"]` are explicitly ignored and cannot satisfy or conflict with that field. Missing/duplicate `meta-date`, substituting the `15:04` last-modified value or using the calendar index rejects in focused fixtures; the index is corroboration, not the fact source. Inside the article, `a` and `img` elements are inert structure: their `href`/`src`/other URI attributes are ignored and never parsed, dereferenced, fetched or emitted. `script`, `style`, `iframe`, `form`, `object`, `embed`, `base`, refresh-capable `meta`, event-handler attributes and `javascript:`/equivalent executable URI schemes reject. The parser selects semantic structures by labels rather than table ordinal alone, preserves the holiday qualification, infers no absent fact and rejects missing/duplicate root, duplicate/contradictory field, invalid date/week count or oracle overflow. Parser identity/version and raw/normalized digests are immutable. Exact-source conformance replays the already retained body from §2 and proves the same oracle without another network attempt; CI uses only the synthetic fixture/oracle. Until both layers pass review, `parser_fixture` evidence is incomplete.

## 6. Exact source approval operation

The R1 owning patchset must define `platform-request-context/v1` as the M00 successor and add the following exact closed enum variants; they are not display strings and do not alter accepted v0 decoding:

```text
PermissionClass::PlatformOperatorWrite  // serde: "platform_operator_write"
EffectClass::PlatformAuthorityMutation  // serde: "platform_authority_mutation"
PermissionClass::PlatformOperatorEffect // serde: "platform_operator_effect"
EffectClass::PrivilegedExternalEffect   // serde: "privileged_external_effect"
```

The only coherent new pairs are `PlatformOperatorWrite -> PlatformAuthorityMutation` and `PlatformOperatorEffect -> PrivilegedExternalEffect`. No existing permission maps to either new effect, neither new permission maps to any other effect, and the two new pairs cannot cross. Both permissions are private platform-operator permissions: an anonymous actor or an incoherent descriptor is `MalformedCommand { operation_id: Some(current_operation_id) }`; policy/session/capability/infrastructure denials retain the exact fourteen-class v0 rejection/projection variants and order, including `PolicyDenied | PolicyExpired`, `SessionNotFound | SessionIdMismatch | SessionNotAdmitted`, `CapabilityMissing | CapabilityDisabled | CapabilityRevoked`, `InfrastructurePortUnavailable` and `IdempotencyStoreUnavailable`. Those denials are terminal under the existing idempotency fence; correcting a descriptor/grant/policy requires a fresh idempotency key or unkeyed attempt, while an infrastructure-incomplete reservation returns the existing retry-not-before carrier. No new retryable domain projection is invented. The approval operation descriptor is:

```text
OperationId: source.approve
SchemaIdentity: schema:source-approve:v1
SchemaDigest: c76081e0601533596259bfa34d5c992dd09c399ee4dc9ff1b322a15e1b3d167e
PermissionClass: PlatformOperatorWrite
EffectClass: PlatformAuthorityMutation
DecoderIdentity: decoder:source-approve:v1
DispatcherIdentity: dispatcher:m60-source-approval:v1
Actor: Authenticated only
AdapterAllowlist: [adapter:platform-operator-application:v1]
Initial projections: none; no public HTTP/Web/CLI/inbound-MCP projection
```

The schema digest is over exactly `1371` UTF-8 bytes: lexicographically sorted JSON object keys, `,`/`:` separators, no other whitespace and one trailing LF. JSON array order is semantic and fixed exactly as displayed; canonicalization never sorts array elements, including every `required` list. The exact bytes before that LF are:

```json
{"$id":"schema:source-approve:v1","additionalProperties":false,"properties":{"expected_authority_revision":{"maximum":18446744073709551615,"minimum":1,"type":"integer"},"expected_definition_digest":{"maxLength":64,"minLength":64,"pattern":"^[0-9a-f]{64}$","type":"string"},"expected_evidence_bundle_digest":{"maxLength":64,"minLength":64,"pattern":"^[0-9a-f]{64}$","type":"string"},"review_receipt":{"additionalProperties":false,"properties":{"parser_fixture":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"permission":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"rate":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"review":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"reviewer":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"}},"required":["reviewer","review","permission","rate","parser_fixture"],"type":"object"},"source_id":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"}},"required":["source_id","expected_authority_revision","expected_definition_digest","expected_evidence_bundle_digest","review_receipt"],"type":"object"}
```

The descriptor snapshot binds every field above and its nonzero snapshot version. The new permission/effect values are deliberate M00 contract changes; mapping source approval to tenant-private mutation is forbidden. M00 adds exact capability `platform.source.approve` with `PlatformOperator` scope and `auto_grant = Never`. Its current grant snapshot binds authenticated `UserId`, operation ID, schema identity/digest, capability ID, grant ID/revision, policy snapshot ID, `not_before`, `not_after` and `Active` state. Missing, disabled, revoked, expired, schema-stale or actor-mismatched grant denies before M10 dispatch.

R1 must synchronously freeze the retrieval-effect descriptor already named by accepted `source-retrieval/v0`; it is not inferred from the approval descriptor:

```text
OperationId: source.retrieval.attempt/v0
SchemaIdentity: schema:source-retrieval-attempt:v0
SchemaDigest: 5320abe84fb9c475126effb4d54832782ef1436ad5876263f8661d3cf9e59552
SchemaProjection: exact source-retrieval/v0 §3 RetrievalAttemptCommand fields in declaration order
PermissionClass: PlatformOperatorEffect
EffectClass: PrivilegedExternalEffect
DecoderIdentity: decoder:source-retrieval-attempt:v0
DispatcherIdentity: dispatcher:m60-source-retrieval:v0
AdapterAllowlist: [adapter:operator-application:v1]
CapabilityId: platform.source.retrieve
CapabilityScope: PlatformOperator
AutoGrant: Never
```

The retrieval schema digest is over exactly `1028` UTF-8 bytes: lexicographically sorted JSON object keys, semantic arrays in the displayed order, compact `,`/`:` separators, no other whitespace and one trailing LF. The exact pre-LF bytes are:

```json
{"$id":"schema:source-retrieval-attempt:v0","additionalProperties":false,"properties":{"attempt_id":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"command_id":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"expected_authority_revision":{"maximum":18446744073709551615,"minimum":1,"type":"integer"},"override_request":{"anyOf":[{"type":"null"},{"additionalProperties":false,"properties":{"evidence_id":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"},"override_id":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"}},"required":["override_id","evidence_id"],"type":"object"}]},"source_id":{"maxLength":128,"minLength":1,"pattern":"^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$","type":"string"}},"required":["command_id","attempt_id","source_id","expected_authority_revision","override_request"],"type":"object"}
```

The literal digest above is bound by the descriptor snapshot, platform-operator grant, `RequestAdmitted` event validation, decoder and dispatcher checks. R1 reconstructs it independently and rejects any different byte count, key/array order or digest.

`PlatformOperatorEffect -> PrivilegedExternalEffect` is the only coherent pair for the retrieval operation; every other pair on that descriptor rejects before dispatch. The operation/schema version is the accepted attempt-command API version, not the outbound wire protocol: `RetrievalAttemptCommand` carries no protocol field, and M60 transaction-currently loads the `SourceDefinitionV2` whose closed `SourceRetrievalProtocolVersionV2` selects v0 or v1 serialization. Therefore the exact `/v0` command bytes and decoder remain unchanged while an approved definition may select `V1StrictHttpsIpv4Http11Identified_20260902`; request candidate/definition digest and serializer checks prevent cross-version substitution. R1 adopts and independently recomputes the literal schema bytes/digest above. M10 accepts only authenticated current grant/context, appends and byte-equal reads back this operation's exact `RequestAdmitted` event, and mints one owner-private `VerifiedRequestAdmissionEvidence`; then it calls `SourceRetrievalUnitOfWorkPort::decide_admission(&PlatformRequestContext, VerifiedRequestAdmissionEvidence, RetrievalAttemptCommand)`. M60 validates all sealed context/witness bindings before any command/attempt ledger lookup or source read, derives the accepted owner-private `RetrievalAuthorityEvidence` from that exact context, and persists both it and the full admission witness in `RetrievalAdmissionEnvelopeV1`; mismatch returns `SourceAdmissionErrorV1::RepositoryCorrupt` with zero M60 row/index/slot mutation. M60 domain denial remains the closed admission/start algebra below; M00 denial remains its request-context rejection algebra. `SourceStartAuthorization` can be minted only from this descriptor/capability/context/admission lineage. No approval/evidence-record context, generic dispatcher, public route or caller-authored effect witness can start retrieval. This descriptor and executable consumer are in R1/R3/R4 before R5's first retrieval.

The inherited first stage is exactly `platform-request-context/v0` §§5–6, in this order:

1. Compute the unchanged `platform-request-context/v0` envelope hash from its accepted preimage; schema identity/digest are deliberately absent from that preimage and are validated from the request-scoped descriptor after reservation.
2. Reserve or retrieve the four-way idempotency state.
3. Classify prior or in-flight state before descriptor lookup.

After those three numbered operations complete in ascending order, M00 retains prior/in-flight state for the same validated values only as an opaque non-projecting handle, while retaining a distinct private owned-reservation token for `New`/`Reclaimed`; then it validates the exact descriptor snapshot and permission/effect coherence; reads one trusted clock; validates platform-policy currentness, authenticated-session currentness and exact platform-operator capability/grant currentness; finalizes the sealed context; and only then unseals/returns the matching prior or in-flight projection. The lookup always follows input validation, including denial paths, and its public shape exposes no state discriminant or payload before M00's private post-context unseal. If a post-reservation check rejects and M00 owns `New`/`Reclaimed`, the coordinator calls the existing owner-private `AdmissionPorts::finalize_idempotency(&token, &FinalAdmissionDisposition::Rejected(rejection))`: `Committed` returns `M00AdmissionResult::Rejected(rejection)` and the winning finalizer releases capacity; equal `AlreadySame(prior)` requires the persisted prior to be the same rejection, returns `promote_persisted_prior(prior) = M00AdmissionResult::PriorRejected(rejection)`, performs no second release and leaves no owned reservation because the winning finalizer already terminalized it; `LostReservation` returns `M00AdmissionResult::Incomplete` and no context; port error returns the typed fail-closed infrastructure result and leaves recovery/reclamation, not the caller, responsible for any uncertain reservation. Every non-authoritative branch calls no M10/M60 port, and no branch silently drops an owned token as success. If the opaque handle names prior/in-flight state, denial mutates nothing and projects none of that state. Anonymous, session-stale, grant-missing/revoked/expired, actor-mismatched or schema-stale calls receive the current typed denial and cannot distinguish prior absence/content through any typed result, field or error; no M10/M60 port runs. The contract does not claim timing-side-channel equivalence. A replay is returned only when its persisted actor, operation, schema, grant and policy bindings exactly match the newly sealed current context. The sealed `PlatformRequestContext` successor retains private-field `PlatformOperatorAdmission { admitted_user_id, capability_id, grant_id, grant_revision, operation_id, schema_digest, policy_snapshot_id, observed_at }`; only M00 constructs it, all fields are read-only, and it has no Serde, `Default` or public constructor.

The typed command body is exactly:

```text
SourceApproveCommandV1 {
  source_id: SourceId,
  expected_authority_revision: SourceAuthorityRevision,
  expected_definition_digest: Sha256Digest,
  expected_evidence_bundle_digest: Sha256Digest,
  review_receipt: SourceReviewReceipt,
}
```

`PlatformRequestContext.command_id()` is the command identity. M10 exposes only `M60SourceApprovalPort::approve(&PlatformRequestContext, VerifiedRequestAdmissionEvidence, SourceApproveCommandV1) -> Result<SourceApproveResultV1, SourceApprovalApplicationError>` through `dispatcher:m60-source-approval:v1`; no generic domain dispatcher, repository handle or raw registry method is admitted. M10 requires the exact operation/schema/permission/effect/adapter tuple and checked `PlatformOperatorAdmission`. Before that one M60 call, M10 derives exact `PlatformControlEvent::from_admitted_request(&context)`, calls accepted `ControlEvidenceAppendPort::append_once`, admits only `Appended | AlreadySame`, reloads `ControlEvidenceKey::Request { command_id }` through `ControlEvidenceReadPort`, and requires byte-equal event read-back. Conflict, unavailable/corrupt/limit/internal error, missing read-back or unequal event fails closed and M60 is not invoked. Only successful read-back mints owner-private, non-Serde `VerifiedRequestAdmissionEvidence { key, event_digest, command_id, correlation_id, descriptor_snapshot_id, policy_snapshot_id, admission_observed_at, admitted_user_id, capability_id, grant_id, grant_revision, operation_id, schema_digest, permission_class, effect_class, adapter_identity }`; the event/key fields are checked against the byte-equal `RequestAdmitted` read-back, while `admission_observed_at = context.observed_at()` and all remaining operator/descriptor bindings are copied from the same sealed `PlatformRequestContext`. M60 requires exact context/command/witness equality for every field, and persists the complete witness identity in the approval/evidence-record command ledger and receipt transaction. The context actor must be `Authenticated`. Public actor, absent/wrong operator admission or unrepresentable identity is rejected by M00/M10 before M60 and creates no M60 ledger row. After that admission, M60 derives `SourceReviewerId` from `context.actor().identities().user_id().as_str()` and records any caller-supplied `review_receipt.reviewer` mismatch as a pending post-load validation fact; it does not construct or persist a reviewer decision before the immutable bundle outcome is known. Only after `Found(bundle)` passes the earlier completeness/digest/anchor/evidence-ID/parser checks does that fact become `SourceApprovalEvidenceDecisionV1::Reject { reason: ReviewerMismatch, evidence_observation_digest }` and enter `transact_approval`, which persists the terminal rejection while forbidding the source CAS. Every entry context/witness mismatch instead is the `RepositoryCorrupt` application error below with zero ledger lookup/mutation. Equal application replay may reproduce `AlreadySame` plus equal read-back, but no approval may commit without this durable M00 causation witness.

### 6.1 Canonical definition digest

`expected_definition_digest` is SHA-256 of the generic v2 encoder below. Every encoded value is read from the `SourceDefinitionV2` supplied to `propose_exact_v2`; no source-specific literal may substitute for a field:

```text
ASCII domain: "ustc-source-definition-approval/v2\0"
then, in order:
  enc(definition.source_id)
  enc(definition.owner)
  enc(definition.url)
  enc(wire_tag(definition.authority))
  u64be(definition.retrieval_policy.minimum_interval_seconds)
  u64be(definition.retrieval_policy.maximum_response_bytes)
  u64be(definition.retrieval_policy.maximum_elapsed_seconds)
  enc(wire_tag(definition.retrieval_policy.expected_media_type))
  enc(wire_tag(definition.retrieval_policy.protocol_version))
  enc(wire_tag(definition.retrieval_policy.public_ip_policy_version))
  enc(wire_tag(definition.retrieval_policy.use_policy.rate_override))
  enc(wire_tag(definition.retrieval_policy.use_policy.raw_retention))
  enc(wire_tag(definition.retrieval_policy.use_policy.public_projection))
where enc(s) = u64be(UTF-8 byte length) || UTF-8 bytes
```

`SourceDefinitionSchemaVersionV2` must be exactly `V2`; it is bound once by the `.../v2\0` domain separator and is not redundantly encoded as a field token. Mutable `authority_revision`, `status` and approval receipt are excluded. All nominal text values use their exact validated scalar/canonical URL; `wire_tag` returns the exact explicit string declared by the owning enum contract with no automatic case conversion. Snake_case is used only by enums whose owning contract declares snake_case; in particular the protocol/public-IP version tags remain exactly `V1StrictHttpsIpv4Http11Identified_20260902` and `V0Ipv4Only20260809`, while authority/use-policy tags use the displayed snake_case strings. There is no BOM, trimming, Unicode normalization, locale transformation, JSON or platform newline. Two v2 definitions that differ in any encoded field above must therefore differ in preimage and cannot equal-replay merely because source ID or URL matches.

The calendar source is an independent golden vector for that generic encoder, not the encoder definition. Substitute exactly:

```text
source_id = "ustc-teach-calendar-fall-2026"
owner = "中国科学技术大学教务处 / www.teach.ustc.edu.cn"
url = "https://www.teach.ustc.edu.cn/calendar/20135.html"
authority = "reviewed_official_source"
minimum_interval_seconds = 21600
maximum_response_bytes = 131072
maximum_elapsed_seconds = 20
expected_media_type = "text/html"
protocol_version = "V1StrictHttpsIpv4Http11Identified_20260902"
public_ip_policy_version = "V0Ipv4Only20260809"
rate_override = "denied"
raw_retention = "internal_raw_evidence_only"
public_projection = "normalized_facts_and_exact_links_only"
```

The owner input is exactly the displayed UTF-8 scalar sequence above. `reviewed_official_source` is the exact wire spelling of Rust `SourceAuthority::ReviewedOfficialSource`; the protocol/public-IP/use-policy tags are the exact spellings shown in §§3–4. The controller generated this golden through a standalone Python standard-library `hashlib` script, not the future production encoder: the exact candidate preimage remains `436` bytes and hashes to `101c2c8956b75586edd38347a1fa90e0da391a12723f52110b1780194b1d0228`. R1 must test both this literal golden and one-axis field mutations for every generic encoded field; each mutation must change the preimage/digest and prevent equal replay.

### 6.2 Canonical command-body digest and replay

M60 computes `approval_body_digest` as SHA-256 of:

```text
ASCII domain: "ustc-source-approve-command/v1\0"
enc(source_id)
u64be(expected_authority_revision)
raw 32-byte expected_definition_digest
raw 32-byte expected_evidence_bundle_digest
enc(reviewer)
enc(review evidence id)
enc(permission evidence id)
enc(rate evidence id)
enc(parser_fixture evidence id)
```

The current B1 `SourceReviewReceipt` remains a five-ID provenance value and is not sufficient authority by itself. The successor approval boundary additionally owns one immutable `SourceApprovalEvidenceBundleV1`:

```text
SourceApprovalEvidenceBundleV1 {
  generation_id: SourceEvidenceGenerationId,
  source_id: SourceId,
  definition_digest: Sha256Digest,
  manifest_digest: Sha256Digest,
  review: SourceEvidenceBindingV1,
  permission: SourceEvidenceBindingV1,
  rate: SourceEvidenceBindingV1,
  parser_fixture: SourceParserEvidenceBindingV1,
}

SourceEvidenceDispositionV1 = Accepted | Rejected | Superseded

SourceEvidenceBindingV1 {
  generation_id: SourceEvidenceGenerationId,
  source_id: SourceId,
  definition_digest: Sha256Digest,
  manifest_digest: Sha256Digest,
  evidence_id: SourceReviewEvidenceId,
  content_digest: Sha256Digest,
  disposition: SourceEvidenceDispositionV1,
  accepted_by: SourceReviewerId,
  accepted_at: OffsetDateTime,
}

SourceParserEvidenceBindingV1 {
  binding: SourceEvidenceBindingV1,
  parser_identity: ParserIdentity,
  fixture_digest: Sha256Digest,
  oracle_digest: Sha256Digest,
  exact_source_body_digest: Sha256Digest,
}
```

The parser binding is self-authenticating rather than compared with an unstated retained artifact. Its nested `binding.content_digest` must equal SHA-256 over ASCII domain `"ustc-source-parser-evidence-binding/v1\0"`, then in exact order: `enc(binding.generation_id)`, `enc(binding.source_id)`, raw 32-byte `binding.definition_digest`, raw 32-byte `binding.manifest_digest`, `enc(binding.evidence_id)`, `enc` of the exact snake_case disposition tag, `enc(binding.accepted_by)`, checked signed Unix nanoseconds for `binding.accepted_at` as big-endian `i128`, `enc(parser_identity)`, raw 32-byte `fixture_digest`, raw 32-byte `oracle_digest`, and raw 32-byte `exact_source_body_digest`. The content-digest field itself is excluded from that preimage. M60 recomputes this equality both when recording and when approving a loaded bundle. `ParserBindingMismatch` means only this equality fails; artifact review provenance is represented by the four digests and accepted disposition, not by an undeclared second artifact read.

Evidence chronology is bounded by the admitted M00 clock observation, never by a caller clock. For each review/permission/rate/parser binding, M60 computes `accepted_at_unix_nanos = binding.accepted_at.unix_timestamp_nanos()` and `admission_upper_bound_unix_nanos = i128::from(VerifiedRequestAdmissionEvidence.admission_observed_at.as_unix_millis()) * 1_000_000`; that multiplication is checked even though the typed range fits. If any accepted timestamp is greater than the upper bound, the first such binding in declaration order yields `EvidenceTimestampInFuture`. Evidence recording compares against its record operation's sealed admission time, and approval independently compares the loaded immutable bundle against its approval operation's sealed admission time. This reason is terminal and replayable, authorizes no evidence insert/source CAS, and cannot be suppressed by a later clock read or by retrying the same command. Historical timestamps at or before the bound are not freshness proof; freshness remains an explicit later policy decision.

At M60 entry, before any command-ledger lookup or immutable-bundle load, M60 runs owner-private `validate_request_admission_bindings(context, VerifiedRequestAdmissionEvidence, command)`, requiring exact command/correlation/descriptor/policy/admission-time/actor/grant/event-key/event-digest coherence. Any mismatch returns `RepositoryCorrupt` with no ledger lookup or mutation. Only then M60 computes `approval_body_digest` and calls `classify_approval_command(command_id, approval_body_digest, &VerifiedRequestAdmissionEvidence)`. The repository compares the witness's complete sealed bindings against any retained terminal command row before projecting it: unequal body returns `ConflictingCommandReplay`; unequal sealed binding returns `RepositoryCorrupt`; only exact body plus exact binding returns `Replay(original)` without consulting current source or evidence state; `Unseen` may continue. This lookup is read-only and grants no authority. M60 next calls transactional `prepare_approval_evidence_load(command: SourceApproveCommandV1, approval_body_digest, VerifiedRequestAdmissionEvidence)`, passing the complete decoded command rather than attempting to reverse its digest. The transaction repeats replay/conflict classification and evaluates `SourceMissing`, `LegacyVersionOccupied`, `NotProposed`, `StaleAuthorityRevision` and `DefinitionDigestMismatch` in that order before any evidence I/O. A failing source guard atomically persists/returns the terminal rejection; a passing guard returns one owner-private linear `SourceApprovalEvidenceLoadPlanV1` bound to command/body/source/revision/definition/admission and the observed durable-state digest. A `Ready` plan reserves no source transition and authorizes only one immutable read; concurrent or post-crash preparation may mint another snapshot-bound plan, but final `transact_approval` serializes the command ledger/source CAS, invalidates stale bindings and returns the one terminal replay. Equal replay of an already terminal command returns the prior result and never another plan.

The canonical bundle digest starts with ASCII domain `"ustc-source-approval-evidence-bundle/v1\0"`, then encodes every field above in declaration order, using `enc` for text and closed disposition/parser tags, raw 32-byte digests and signed Unix nanoseconds as checked big-endian `i128`; R1 freezes independent golden bytes/digest before implementation. Every binding's duplicated generation/source/definition/manifest anchors must exactly equal the bundle anchors; every disposition must be exactly `Accepted`; and the parser binding's nested content digest must equal the canonical parser self-binding above. Only after `prepare_approval_evidence_load` returns `Ready(plan)`, M60 attempts the immutable bundle load by the plan-bound `expected_evidence_bundle_digest`, recomputes any present bundle digest, matches inner evidence IDs to `review_receipt`, and checks review/permission/rate `accepted_by` plus receipt reviewer against the authenticated context actor. It produces owner-private `SourceApprovalEvidenceObservationV1 = Decision(SourceApprovalEvidenceDecisionV1) | RepositoryUnavailable | RepositoryCorrupt`, where `SourceApprovalEvidenceDecisionV1 = Verified(VerifiedSourceApprovalEvidence) | Reject { reason: SourceApprovalEvidenceRejectionV1, evidence_observation_digest }` and `SourceApprovalEvidenceRejectionV1 = EvidenceBundleMissing | EvidenceBundleIncomplete | EvidenceBundleMismatch | EvidenceTimestampInFuture | ReviewerMismatch`. `SourceApprovalEvidenceLoadValueV1::Found(bundle)` enters digest/content validation; `Missing` maps to `Decision::Reject(EvidenceBundleMissing)`; `Err(Unavailable)`/`Err(Corrupt)` map only to `RepositoryUnavailable`/`RepositoryCorrupt` infrastructure observations respectively; a `Rejected` or `Superseded` binding disposition, or any binding generation unequal to the bundle generation, maps to `EvidenceBundleIncomplete`; bundle-digest/anchor/evidence-ID/parser-self-binding mismatch maps to `EvidenceBundleMismatch`; the first declaration-order timestamp beyond the sealed approval-admission upper bound maps to `EvidenceTimestampInFuture`; actor/receipt mismatch maps to `ReviewerMismatch` only after the preceding missing/incomplete/mismatch/time checks pass; sealed context/grant/witness incoherence was already rejected as `RepositoryCorrupt` before lookup and has no domain-rejection variant; M60 passes the exact plan plus every observation to `transact_approval`. The immutable store has no mutable current-generation or supersession index: `Superseded` is only an untrusted disposition tag rejected before storage, never a later mutation of an earlier bundle. That transaction resolves replay/conflict, revalidates every plan binding and transaction-currently reevaluates the same five source guards before inspecting the evidence observation. Thus source-first rejection occurs before evidence I/O for the initial snapshot and wins again if source authority changes during I/O; only if all source guards pass may an infrastructure observation return the corresponding non-terminal application error without ledger mutation, a validation rejection enter the durable ledger, or `Verified` permit the approval CAS. Failed preparation or final commit exposes no plan/partial record.

`parser_binding.accepted_by` is intentionally not required to equal the approving or evidence-recording context actor: parser/fixture conformance may be certified by a distinct reviewer. On approval, `ReviewerMismatch` covers the review/permission/rate bindings and source review receipt; parser self-binding digest inequality remains `EvidenceBundleMismatch`, while a binding generation unequal to the bundle generation is `EvidenceBundleIncomplete`. On evidence recording, the ordered reasons are exact: `BundleDigestMismatch` means expected versus recomputed canonical bundle digest inequality; `AnchorMismatch` means a binding's source/definition/manifest anchor differs from the bundle anchor; `EvidenceDispositionNotAccepted` means any binding disposition is `Rejected` or `Superseded`; `EvidenceTimestampInFuture` means the first declaration-order binding exceeds the record operation's sealed admission-time upper bound; `ReviewerMismatch` compares only review/permission/rate `accepted_by` against the recording context actor and never compares parser `accepted_by`; `ParserBindingMismatch` means the nested parser content digest differs from the canonical parser self-binding above; `CrossGeneration` means any binding generation differs from the bundle generation.

Plan revalidation is closed: the five source-row facts are re-read transaction-currently, while command/body, authenticated reviewer, request-admission event, operator grant/revision, policy snapshot and admission observation time are immutable bindings sealed into `VerifiedRequestAdmissionEvidence` and the linear plan. M60 compares those sealed bytes/digests for exact equality and never reopens a live M00 grant/policy decision after admission; an impossible internal mismatch is `RepositoryCorrupt` with no ledger mutation. A reviewer/evidence rejection already present in `SourceApprovalEvidenceObservationV1::Decision` remains subordinate to the re-read source guards and is interpreted only against that exact sealed plan, so concurrent source change and precomputed evidence rejection have one ordered outcome.

Bundle custody is not seeded out of band. R1 must also freeze internal operator operation `source.approval-evidence.record` with schema identity `schema:source-approval-evidence-record:v1`, the same `PlatformOperatorWrite -> PlatformAuthorityMutation`, capability `platform.source.approve`, and no public route. Before any Rust or M00 registration, the R1 owning patchset must freeze this operation's literal canonical JSON Schema bytes, byte count and SHA-256 under the same pre-trailing-LF convention as the approval/retrieval descriptors; its contract checker independently recomputes the literal and rejects a missing, duplicate or mismatched schema carrier. Its command is exactly `SourceApprovalEvidenceRecordCommandV1 { expected_bundle_digest: Sha256Digest, bundle: SourceApprovalEvidenceBundleV1 }`; M00's outer payload digest independently binds the admitted raw DTO but is intentionally not projected through `RequestAdmitted` or sealed context. M60 computes `record_body_digest` itself as SHA-256 over ASCII domain `"ustc-source-approval-evidence-record-command/v1\0"`, then raw 32-byte `expected_bundle_digest`, then the raw 32-byte canonical bundle digest recomputed under the preceding bundle rule; no caller, context or evidence witness supplies this digest. At either M60 approval/evidence-record entry, any context/witness sealed-binding mismatch returns `SourceApprovalApplicationError::RepositoryCorrupt` with no command/evidence/source ledger mutation; it never becomes a domain `Rejected` result. The record command ledger compares that exact M60 digest for equal/conflicting replay. M10 performs the same `RequestAdmitted` append/read-back gate and then calls only `M60SourceApprovalEvidencePort::record(&PlatformRequestContext, VerifiedRequestAdmissionEvidence, command)`. M60 recomputes the canonical bundle digest, duplicated anchors, disposition tags, binding generations and parser self-binding digest before its repository call. The immutable record transaction has a unique command ledger and the closed `Applied(SourceApprovalEvidenceRecordTerminalV1) | Replay(SourceApprovalEvidenceRecordTerminalV1)` result above. An unseen command with a new bundle yields `Accepted(..., Inserted)`; an unseen command naming an equal retained bundle yields `Accepted(..., AlreadySame)`; exact command/body/witness replay returns the byte-equivalent prior terminal inside `Replay`; unequal command body or witness is `ConflictingCommandReplay`/`RepositoryCorrupt` respectively. M60 computes `record_body_digest` from the typed command under the frozen domain above and moves the exact validated `VerifiedRequestAdmissionEvidence` unchanged into `transact_record` with the owner-private decision; the command ledger persists every sealed witness field beside every stored/rejected result for replay symmetry with source approval. Therefore a rejected branch needs no `validated_bundle` yet still reaches the same ledger transaction. The rejection reason is exactly `BundleDigestMismatch | AnchorMismatch | EvidenceDispositionNotAccepted | EvidenceTimestampInFuture | ReviewerMismatch | ParserBindingMismatch | CrossGeneration`, and validation executes strictly in that listed first-failure order. Only the first mismatch is persisted; later checks are not evaluated/projected. Every adjacent pair has a multi-failure mutation test proving the earlier reason wins. Each validated-but-rejected command persists that terminal reason for byte-equivalent replay and appends no bundle. Syntactically/schema-malformed input is rejected by M00 before this port and creates no M60 ledger row. Equal command/body or equal existing bundle replay is idempotent; unequal command replay is `ConflictingCommandReplay`; same digest with unequal retained bytes is `RepositoryCorrupt`. This operation records non-authority evidence only: it cannot create/revise/approve a source or mint retrieval authority. R3 fake-backed tests must exercise every rejection reason plus this admitted record path before approval; R5 executes it durably before `propose_exact_v2` and `source.approve`.

| Internal operation | M00 evidence gate before M60 | Bundle behavior | Admitted mutation |
|---|---|---|---|
| `source.approval-evidence.record` | append/read back exact `RequestAdmitted`; pass `VerifiedRequestAdmissionEvidence` | command carries full bundle; M60 validates then append-once stores or equal-replays it | immutable non-authority evidence only |
| `source.approve` | append/read back its own exact `RequestAdmitted`; pass a distinct witness | load/recompute exact already-stored bundle; mint `VerifiedSourceApprovalEvidence` only after all anchors/reviewer facts match | sole `Proposed -> Approved` CAS and approval receipt |

The two M00 events have distinct command IDs/keys and are never substituted for one another. Approval cannot ingest a missing bundle, while evidence recording cannot approve a source.

Before approval, `SourceAuthorityRepository::propose_exact_v2(definition: SourceDefinitionV2)` must insert the complete v2 definition as `Proposed` at initial `SourceAuthorityRevision = 1`, exactly matching the existing B1 initial-proposal rule while preserving a distinct storage/version discriminator. The repository, not the caller, computes the canonical v2 definition digest from the complete definition using §6.1 and returns/persists that digest. Equal v2 replay is idempotent; the same `SourceId` with unequal canonical definition bytes/digest conflicts; any existing v1 row rejects as `LegacyVersionOccupied`. `source.approve` never creates, revises or migrates a source row: an absent row returns `SourceMissing`, and the first approval for an otherwise unchanged v2 row therefore carries `expected_authority_revision = 1`. No v2 `revise` operation is declared or accepted by this packet; R1 must not invent one, so the only admissible R5 approval revision is `1`. Any future revision operation requires a separately versioned owning contract, explicit migration/revision algebra and a new reviewed packet.

The outer M00 payload digest still independently binds the admitted DTO. A durable M60 approval-command ledger and source transition share one transaction:

- unseen command + current exact revision/definition digest + verified exact evidence-bundle digest + matching reviewer/operator admission → CAS `Proposed → Approved`, increment revision, persist terminal result;
- exact command replay with equal `approval_body_digest` → return byte-equivalent original result without another transition;
- same command with unequal body → `ConflictingCommandReplay`;
- stale revision, changed definition, incomplete evidence, wrong lifecycle or corrupt persistence → persist/return a typed rejection and make no source transition;
- replay of a terminal rejection returns that same rejection even if authority later changes.

The repository record binds command/body digest, old/new revision, status, definition digest, evidence-bundle digest, reviewer/evidence IDs, complete `VerifiedRequestAdmissionEvidence` bindings and terminal result. Decode, digest or dual-index mismatch is `RepositoryCorrupt`; there is no empty/default fallback.

The terminal application algebra is exact:

```text
SourceApproveResultV1::Approved(SourceApprovalReceiptV1 {
  command_id, source_id, old_revision, new_revision,
  definition_digest, evidence_bundle_digest, reviewer,
  request_admission_evidence_key, request_admission_event_digest,
  operator_grant_id, operator_grant_revision, policy_snapshot_id,
})
| SourceApproveResultV1::Rejected(SourceApprovalRejectionV1 {
  command_id, source_id, reason,
})

SourceApprovalRejectionReasonV1 =
  SourceMissing | LegacyVersionOccupied | NotProposed | StaleAuthorityRevision |
  DefinitionDigestMismatch | EvidenceBundleMissing |
  EvidenceBundleIncomplete | EvidenceBundleMismatch |
  EvidenceTimestampInFuture | ReviewerMismatch | RevisionExhausted

SourceApprovalApplicationError =
  RequestAdmissionEvidenceUnavailable | RequestAdmissionEvidenceConflict |
  RequestAdmissionEvidenceMismatch | ConflictingCommandReplay |
  RepositoryUnavailable | RepositoryCorrupt | CommitFailed
```

Domain rejections are terminal ledger results and replay byte-equivalently. After M00/schema admission, `prepare_approval_evidence_load` resolves equal replay/conflicting command first, then evaluates the source-first guards `SourceMissing`, `LegacyVersionOccupied`, `NotProposed`, `StaleAuthorityRevision` and `DefinitionDigestMismatch` in that order before evidence I/O, terminalizing the first failure or returning one plan. `transact_approval` repeats replay/conflict and those same five guards against the plan's exact bindings before interpreting `SourceApprovalEvidenceObservationV1`. If those five pass and the observation is `RepositoryUnavailable | RepositoryCorrupt`, the matching non-terminal application error returns with no ledger mutation; if those five pass and the observation is `Decision`, the transaction evaluates `EvidenceBundleMissing`, `EvidenceBundleIncomplete`, `EvidenceBundleMismatch`, `EvidenceTimestampInFuture`, `ReviewerMismatch` and `RevisionExhausted` in that order. The first failing guard alone is persisted or returned; later outcomes are ignored even when already observed. Every adjacent domain pair plus the `DefinitionDigestMismatch -> evidence infrastructure -> EvidenceBundleMissing` boundary requires a multi-failure mutation proving the earlier outcome wins. Only if all guards pass may the CAS commit approval. Conflicting reuse and storage/corruption failures cannot be converted into a domain rejection or successful receipt; a failed commit exposes no partial source transition or terminal record.

`RevisionExhausted` fires only after a verified evidence decision when transaction-current `SourceAuthorityRevision == u64::MAX` makes checked `+1` impossible; it is the terminal rejection immediately after `ReviewerMismatch` and immediately before the approval CAS/commit. Adjacent multi-failure tests prove reviewer mismatch wins over exhaustion and exhaustion wins over injected CAS/commit failure.

`SourceApprovalReceiptV1`, `SourceApprovalEvidenceLoadPlanV1` and `VerifiedSourceApprovalEvidence` have private fields, redacted `Debug`, no Serde/`Default`/public constructor and are minted only by the M60 application/repository transaction. The plan is non-clone, linear, consumed exactly once by `transact_approval` and cannot be reconstructed from replay or public data. Read-only receipt accessors expose the exact receipt fields above; no caller can supply any of these authority carriers as input.

The proposed evidence IDs remain non-authoritative until durable creation:

```text
review: source-review-calendar-20135-20260902
permission: source-permission-develata-calendar-20135-20260902
rate: source-rate-calendar-20135-21600-20260902
parser_fixture rule: concatenate `source-parser-calendar-20135-v0-` with the
  64 lowercase hexadecimal characters of the eventually accepted fixture digest;
  no concrete evidence ID is assigned by this proposal
```

## 7. M60 durable port placement

M60 owns all public data-only port traits and transition logic. M90/adapters may implement physical storage/effects but cannot decide authority.

```text
SourceAuthorityRepository
  load_current(source_id)
    -> Result<VersionedSourceAuthorityLookupV2, SourceApprovalApplicationError>
  propose_exact_v2(definition: SourceDefinitionV2)
    -> Result<SourceProposalResultV2, SourceProposalErrorV2>
  classify_approval_command(command_id, approval_body_digest,
                            &VerifiedRequestAdmissionEvidence)
    -> Result<SourceApprovalCommandLookupV1, SourceApprovalApplicationError>
  prepare_approval_evidence_load(command: SourceApproveCommandV1,
                                 approval_body_digest,
                                 VerifiedRequestAdmissionEvidence)
    -> Result<SourceApprovalPrepareResultV1, SourceApprovalApplicationError>
  transact_approval(SourceApprovalEvidenceLoadPlanV1,
                    SourceApprovalEvidenceObservationV1)
    -> Result<SourceApproveResultV1, SourceApprovalApplicationError>

VersionedSourceAuthorityLookupV2 =
  Missing |
  LegacyVersionOccupied { source_id,
                          observed_definition_version: SourceDefinitionVersionTag } |
  CurrentV2 { definition: SourceDefinitionV2,
              observed_definition_version: SourceDefinitionVersionTag,
              canonical_definition_digest: Sha256Digest }

SourceProposalResultV2 =
  Proposed { source_id, authority_revision: SourceAuthorityRevision, definition_digest } |
  Replay { source_id, authority_revision: SourceAuthorityRevision, definition_digest }

SourceProposalErrorV2 =
  LegacyVersionOccupied |
  CanonicalUrlOccupied { canonical_url: SourceUrl,
                         existing_source_id: SourceId,
                         observed_definition_version: SourceDefinitionVersionTag } |
  ConflictingDefinition |
  RepositoryUnavailable | RepositoryCorrupt | CommitFailed

SourceApprovalCommandLookupV1 = Unseen | Replay(SourceApproveResultV1)

SourceApprovalPrepareResultV1 =
  Terminal(SourceApproveResultV1) |
  Ready(SourceApprovalEvidenceLoadPlanV1)

SourceApprovalEvidenceLoadPlanV1 {
  command_id, approval_body_digest, source_id,
  expected_revision, expected_definition_digest,
  expected_evidence_bundle_digest, reviewer, review_receipt,
  request_admission_evidence: VerifiedRequestAdmissionEvidence,
  observed_source_status, observed_revision,
  observed_definition_digest, expected_durable_state_digest
}

SourceApprovalEvidenceObservationV1 =
  Decision(SourceApprovalEvidenceDecisionV1) |
  RepositoryUnavailable |
  RepositoryCorrupt

SourceApprovalEvidenceDecisionV1 =
  Verified(VerifiedSourceApprovalEvidence) |
  Reject { reason: SourceApprovalEvidenceRejectionV1,
           evidence_observation_digest }

SourceApprovalEvidenceRejectionV1 =
  EvidenceBundleMissing | EvidenceBundleIncomplete |
  EvidenceBundleMismatch | EvidenceTimestampInFuture | ReviewerMismatch

SourceApprovalEvidenceRecordResultV1 =
  Applied(SourceApprovalEvidenceRecordTerminalV1) |
  Replay(SourceApprovalEvidenceRecordTerminalV1)

SourceApprovalEvidenceRecordTerminalV1 =
  Accepted { receipt: SourceApprovalEvidenceRecordReceiptV1,
             bundle_outcome: EvidenceBundleStoreOutcomeV1 } |
  Rejected(SourceApprovalEvidenceRecordRejectionReceiptV1)

EvidenceBundleStoreOutcomeV1 = Inserted | AlreadySame

SourceApprovalEvidenceRecordReceiptV1 {
  command_id, record_body_digest, evidence_bundle_digest,
  request_admission_evidence_key, request_admission_event_digest,
  durable_state_digest
}

SourceApprovalEvidenceRecordRejectionReceiptV1 {
  command_id, record_body_digest, expected_evidence_bundle_digest,
  request_admission_evidence_key, request_admission_event_digest,
  reason: SourceApprovalEvidenceRecordRejectionV1,
  validation_observation_digest, durable_state_digest
}

SourceApprovalEvidenceRecordRejectionV1 =
  BundleDigestMismatch | AnchorMismatch | EvidenceDispositionNotAccepted |
  EvidenceTimestampInFuture | ReviewerMismatch | ParserBindingMismatch |
  CrossGeneration

SourceApprovalEvidenceRecordDecisionV1 =
  Store(ValidatedSourceApprovalEvidenceBundleV1) |
  Reject { reason: SourceApprovalEvidenceRecordRejectionV1,
           validation_observation_digest }

SourceApprovalEvidenceLoadValueV1 =
  Found(SourceApprovalEvidenceBundleV1) | Missing

SourceApprovalEvidenceLoadErrorV1 = Unavailable | Corrupt

SourceApprovalEvidenceRepository
  transact_record(record_command_id, record_body_digest,
                  expected_evidence_bundle_digest,
                  VerifiedRequestAdmissionEvidence,
                  SourceApprovalEvidenceRecordDecisionV1)
    -> Result<SourceApprovalEvidenceRecordResultV1,
              SourceApprovalApplicationError>
  load_immutable_bundle(source_id, expected_evidence_bundle_digest)
    -> Result<SourceApprovalEvidenceLoadValueV1,
              SourceApprovalEvidenceLoadErrorV1>

M60SourceApprovalEvidencePort
  record(context, VerifiedRequestAdmissionEvidence,
         SourceApprovalEvidenceRecordCommandV1)
    -> Result<SourceApprovalEvidenceRecordResultV1, SourceApprovalApplicationError>

RetrievalAdmissionEnvelopeV1 {
  command: RetrievalAttemptCommand,
  authority: RetrievalAuthorityEvidence,
  request_admission: VerifiedRequestAdmissionEvidence,
}

SourceRetrievalUnitOfWorkPort
  decide_admission(context: &PlatformRequestContext,
                   VerifiedRequestAdmissionEvidence,
                   command: RetrievalAttemptCommand)
    -> Result<SourceAdmissionResultV1, SourceAdmissionErrorV1>
  start(ReservedPlan, SourceStartAuthorization, OwnerLeaseV1)
    -> Result<SourceStartResultV1, SourceStartErrorV1>
  record_transport_stopped(command, TransportStopReceiptV1,
                           ValidatedTransportResultMetadataV1)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  claim_snapshot_recovery(command, OwnerFenceWitnessV1,
                          OwnerLeaseV1, trusted_now)
    -> Result<SnapshotRecoveryClaimResultV1, SourcePostStartErrorV1>
  mark_snapshot_stored_and_complete_success(command,
                                              SnapshotWorkAuthorizationV1,
                                              RawSnapshotStored)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  complete_failure(command, SnapshotWorkAuthorizationV1,
                   SourceCompletionFailureDecisionV1)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  cancel_snapshot_work_before_dispatch(command, SnapshotWorkAuthorizationV1)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  cancel_started_after_drop(command, OwnerFenceWitnessV1)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  expire_reservation(command) -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  reap_abandoned_execution(command, OwnerFenceWitnessV1)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  load_by_command(command_id)
  load_by_attempt(attempt_id)
  list_recoverable(after_attempt_id, limit)

M60RetrievalCoordinator
  cancel_inflight_transport(attempt_id)
    -> Result<SourcePostStartResultV1, SourcePostStartErrorV1>
  tick_reservation_expiry()
    -> ReservationExpiryTickResultV1
  tick_abandoned_work_recovery()
    -> AbandonedWorkRecoveryTickResultV1

ReservationExpiryTickProgressV1 {
  trusted_now, fully_processed_prefix_count,
  acknowledged_expired_receipt_digests,
  uncertain_attempt_id?, next_reservation_deadline?
}

ReservationExpiryTickResultV1 =
  Complete(ReservationExpiryTickProgressV1) |
  Partial { progress: ReservationExpiryTickProgressV1,
            error: ReservationExpiryCoordinatorErrorV1 } |
  Failed { trusted_now?, uncertain_attempt_id?,
           error: ReservationExpiryCoordinatorErrorV1 }

ReservationExpiryCoordinatorErrorV1 =
  ClockUnavailable | RepositoryUnavailable | RepositoryCorrupt | ReadBackFailed

AbandonedWorkRecoveryTickProgressV1 {
  trusted_now, fully_processed_prefix_count,
  acknowledged_execution_abandoned_receipt_digests,
  acknowledged_snapshot_terminal_receipt_digests,
  uncertain_attempt_id?, next_recovery_deadline?
}

AbandonedWorkRecoveryTickResultV1 =
  Complete(AbandonedWorkRecoveryTickProgressV1) |
  Partial { progress: AbandonedWorkRecoveryTickProgressV1,
            error: AbandonedWorkRecoveryCoordinatorErrorV1 } |
  Failed { trusted_now?, uncertain_attempt_id?,
           error: AbandonedWorkRecoveryCoordinatorErrorV1 }

AbandonedWorkRecoveryCoordinatorErrorV1 =
  ClockUnavailable | OwnerStateUnavailable | RepositoryUnavailable |
  RepositoryCorrupt | RawSnapshotUnavailable | ReadBackFailed

RawSnapshotEvidencePort
  put_if_absent(snapshot_identity, source_id, attempt_id, bounded_bytes, sha256)
    -> Result<RawSnapshotPutResultV1, RawSnapshotEvidenceErrorV1>
  read_back(snapshot_identity)
    -> Result<RawSnapshotReadResultV1, RawSnapshotEvidenceErrorV1>

RawSnapshotPutResultV1 = Stored(RawSnapshotWriteReceipt) |
                         AlreadySame(RawSnapshotWriteReceipt) |
                         ExistingContentConflict
RawSnapshotReadResultV1 = Found(RawSnapshotReadObservation) | Missing
RawSnapshotEvidenceErrorV1 = MalformedRequest | RepositoryUnavailable |
                             RepositoryCorrupt | CommitFailed

ExecutionOwnerLeasePort
  register_attempt_owner(attempt_id, fresh_owner_id, trusted_now)
    -> Result<OwnerLeaseV1, OwnerLeaseErrorV1>
  renew_owner(OwnerLeaseV1, trusted_now)
    -> Result<OwnerLeaseV1, OwnerLeaseErrorV1>
  current_epoch(owner_id) -> Result<OwnerEpoch, OwnerLeaseErrorV1>
  advance_epoch_after_drop(owner_id, expected_epoch, DroppedFutureWitness)
    -> Result<OwnerFenceWitnessV1, OwnerLeaseErrorV1>
  fence_expired_owner(owner_id, expected_epoch, trusted_now)
    -> Result<OwnerFenceWitnessV1, OwnerLeaseErrorV1>
```

```text
OwnerLeaseV1 {
  attempt_id, owner_id, owner_epoch, registered_at, lease_expires_at,
  owner_incarnation_not_after, owner_lease_digest
}

SnapshotWorkAuthorizationV1 {
  command_id, attempt_id, source_id, success_metadata_digest,
  stop_receipt_digest, snapshot_owner_id, snapshot_owner_epoch,
  snapshot_lease_expires_at,
  origin: SnapshotWorkAuthorizationOriginV1,
  expected_durable_state_digest
}

SnapshotWorkAuthorizationOriginV1 = InitialAfterTransport | RecoveryAfterFence

SnapshotLeaseBusyReceipt {
  attempt_id, source_id, observed_owner_epoch, lease_expires_at,
  durable_state_digest
}

OwnerFenceCauseV1 = DroppedFuture { dropped_future_witness_digest } |
                    ExpiredLease { trusted_time_observation_digest }

OwnerFenceWitnessV1 {
  attempt_id, owner_id, prior_epoch, advanced_epoch,
  prior_lease_expires_at, cause: OwnerFenceCauseV1,
  fence_observation_digest
}

OwnerLeaseErrorV1 =
  MalformedOwner | OwnerMissing | OwnerAlreadyUsed | CrossAttemptOwnerReuse |
  OwnerEpochStale | LeaseStillActive |
  LeaseSpanExceeded | ConflictingFenceReplay | ArithmeticOverflow |
  RepositoryUnavailable | RepositoryCorrupt | CommitFailed
```

`VersionedSourceAuthorityLookupV2::LegacyVersionOccupied` requires `observed_definition_version == V1`; `CurrentV2` requires `V2`. `decide_admission` and `start` copy that exact tag into `observed_definition_version`; unsupported v1 therefore has only the v1 tag present and no v2 status/revision/digest/protocol projection. Any variant/tag mismatch is `RepositoryCorrupt`.

`propose_exact_v2` also preserves accepted canonical-URL uniqueness across both definition versions in the same transaction. It first applies source-ID precedence: same ID with v1 returns `LegacyVersionOccupied`; same ID with byte-equal v2 definition/digest returns `Replay`; same ID with unequal v2 definition returns `ConflictingDefinition`. Only when the source ID is absent does it query the unique canonical-URL index spanning all v1 and v2 rows. Any row with the same canonical `SourceUrl` under a different source ID returns `CanonicalUrlOccupied { canonical_url, existing_source_id, observed_definition_version: V1 | V2 }` without insertion; unknown/mismatched index/version state is `RepositoryCorrupt`. Only absence from both indexes permits the revision-1 v2 insert, and source row plus source-ID and canonical-URL indexes commit atomically. Callers cannot bypass the `21600`-second per-endpoint policy by selecting another source ID.

`ExecutionOwnerLeasePort` is the only attempt-owner fence authority. `register_attempt_owner` requires a fresh owner ID used by exactly one attempt row and never shared with a sibling attempt, sets a nonzero epoch, `lease_expires_at = checked(trusted_now + 60s)` and hard `owner_incarnation_not_after = checked(trusted_now + 300s)`. `renew_owner` is compare-and-set, sets at most `min(checked(trusted_now + 60s), owner_incarnation_not_after)`, and returns `LeaseSpanExceeded` rather than extending the incarnation cap; before that cap a healthy coordinator must drain/join the one scoped attempt, fence its epoch and register a fresh owner ID for later work. An owner that cannot renew must stop issuing work and synchronously drop/join its scoped operation before the retained lease expires. No epoch mutation can affect another attempt row because owner IDs are attempt-unique.

`LeaseStillActive` has one exact method trigger: `fence_expired_owner` returns it when trusted `now < retained lease_expires_at`. A current exact `(attempt_id, owner_id, epoch)` lease being active is the normal `renew_owner` prerequisite, not an error; `renew_owner` MUST NOT return `LeaseStillActive` for that condition. If the generic port nevertheless returns `LeaseStillActive` from `renew_owner`, M60 treats the method/variant mismatch as `SourcePostStartErrorV1::RepositoryCorrupt` with the ordinary error-zero-mutation and `Started` recovery semantics.

`advance_epoch_after_drop` and `fence_expired_owner` share one durable append-once advance ledger keyed by `(attempt_id, owner_id, expected_epoch)`. The former consumes the exact owner-private `DroppedFutureWitness` after synchronous future destruction; the latter admits a new `ExpiredLease` cause only when trusted `now >= retained_owner_lease_expires_at`. On their first call they atomically compare-and-increment the current epoch and persist one `OwnerFenceWitnessV1` with its exact cause in the same transaction. Equal replay returns that byte-equivalent retained witness without a second increment; startup or the live abandonment tick may therefore recover a row after crash/task-loss between drop-advance and cancellation once the retained owner lease expires. A current epoch greater than expected without the exact retained advance-ledger row is `ConflictingFenceReplay`/corruption, not synthesized authority. M60 independently reads back `current_epoch == advanced_epoch > prior_epoch` before use. Equal/stale epochs without exact replay, an unexpired lease on the new expired-lease path, failed CAS/read-back, unavailable state or corruption mint no witness.

After a successful transport return and before `record_transport_stopped`, M60 obtains `stopped_at` from the trusted clock. If that read fails, it returns `SourcePostStartErrorV1::ClockUnavailable`, calls neither `renew_owner` nor any post-start repository/raw method, keeps the in-memory transport result available only to the still-running scoped coordinator for bounded retry, and leaves the durable row `Started`; process/task loss is later handled by the live abandonment loop without transport reissue. With a trusted `stopped_at`, M60 computes the checked 50-second snapshot deadline plus five-second fence margin and calls `renew_owner` once when the current attempt-owner lease does not already cover that required instant. Renewal preserves the same owner ID/epoch and cannot exceed `owner_incarnation_not_after`. Its closed mapping is exhaustive: `LeaseSpanExceeded` or a successful renewed lease still short of the required instant maps to `SourcePostStartErrorV1::InsufficientOwnerLeaseCoverage`; `OwnerEpochStale` maps to `OwnerEpochStale`; `RepositoryUnavailable` maps to `RepositoryUnavailable`; `ArithmeticOverflow` maps to `ArithmeticOverflow`; `CommitFailed` maps to `CommitFailed`; and `MalformedOwner | OwnerMissing | OwnerAlreadyUsed | CrossAttemptOwnerReuse | LeaseStillActive | ConflictingFenceReplay | RepositoryCorrupt` maps to `RepositoryCorrupt`. Every error branch starts no raw work, writes no owner/journal/stopped/snapshot metadata and leaves the row `Started` for bounded abandonment recovery; a port implementation that reports an error after mutating the owner row is `RepositoryCorrupt`. A successful-but-short renewal has already committed exactly the same-owner/same-epoch `lease_expires_at` CAS returned by `renew_owner`; M60 then returns `InsufficientOwnerLeaseCoverage` without any additional owner mutation, journal/stopped/snapshot mutation or raw work, and the row remains `Started` for recovery. The exact source's 20-second transport bound, a renewal at stop and the 300-second incarnation cap make the 55-second post-stop requirement representable.

A successful `record_transport_stopped` transition first computes `snapshot_lease_expires_at = checked(stopped_at + 50s)` and `required_owner_lease_not_before = checked(snapshot_lease_expires_at + 5s)`. Its current attempt-owner lease and incarnation cap must both cover that required instant; otherwise it returns `SourcePostStartErrorV1::InsufficientOwnerLeaseCoverage`, writes no success metadata or snapshot authority, and leaves `Started` for bounded abandonment recovery. On success the atomic commit stores `snapshot_owner_id`, `snapshot_owner_epoch` and the 50-second deadline and returns `SnapshotWorkReady(SnapshotWorkAuthorizationV1 { origin: InitialAfterTransport, ... })` only to that first committer. The authorization is owner-private, non-Serde/non-clone and binds command/attempt/source, successful metadata and stop-receipt digests, owner/epoch, lease expiry, origin and durable-state digest. It must be current at every raw put/read and terminal transaction. `renew_owner` and ordinary raw/terminal operations cannot change that incarnation's row expiry or authorization binding; only a successful fenced `claim_snapshot_recovery` may replace an expired prior owner and deadline with a fresh incarnation.

The snapshot workflow has one aggregate `45s` initial budget: at most `20s` for `put_if_absent`, at most `20s` for the mandatory read-back, then a `5s` fence margin. Both `RawSnapshotEvidencePort` methods are synchronous, scoped, deadline-bounded, non-detaching adapter calls and must return within their 20-second bounds without leaving child work. One M60-private workflow frame owns the non-clone `SnapshotWorkAuthorizationV1` linearly across both calls; the adapter receives only the declared data-only snapshot identity/source/attempt/body/digest inputs. Before dispatching `put_if_absent`, M60 requires checked `trusted_now + 20s + 20s + 5s <= snapshot_lease_expires_at`; with less budget it starts no raw call and, while authority is current, terminalizes through the owner-minted `InsufficientSnapshotLeaseBudget` decision with exact `observed_at`, retained deadline and required `45` seconds. After `Stored | AlreadySame`, M60 obtains a fresh trusted observation and requires checked `trusted_now_after_put + 20s + 5s <= snapshot_lease_expires_at` before starting read-back. A conforming put that completes within its 20-second cap preserves this post-put budget. Clock unavailability/regression, adapter overrun or insufficient post-put budget constructs no cancellation/failure decision and leaves `TransportStopped`; the committed bytes are later reconciled by fenced recovery. The attempt-owner lease may renew only to keep fencing valid and cannot widen snapshot work. Completion/error/cancellation returns only after the current synchronous call and its physical work are joined/destroyed, so the five-second margin expires after any conforming old call is gone. Equal replay returns only the `TransportStopped` receipt and never reconstructs this authority.

`claim_snapshot_recovery` is the sole handoff. Before inspecting raw-snapshot storage, it requires both `trusted_now >= retained_snapshot_lease_expires_at` and an `OwnerFenceWitnessV1::ExpiredLease` for the exact retained attempt/snapshot owner/epoch. It then computes `fresh_snapshot_lease_expires_at = checked(trusted_now + 50s)` and `required_owner_lease_not_before = checked(fresh_snapshot_lease_expires_at + 5s)`. The caller's already-registered attempt-unique `OwnerLeaseV1` must be current, and both its lease expiry and incarnation cap must cover that required instant. Insufficient coverage maps exactly to `SourcePostStartErrorV1::InsufficientOwnerLeaseCoverage`; arithmetic overflow maps to `ArithmeticOverflow`; either result performs no claim/storage read and preserves the prior row for retry. A passing transaction atomically replaces the prior snapshot owner/epoch/deadline with the fresh bindings, advances the durable-state digest and returns one `Claimed(SnapshotWorkAuthorizationV1 { origin: RecoveryAfterFence, ... })` containing the fresh deadline. This recovery-origin authorization cannot enter `cancel_snapshot_work_before_dispatch` and cannot begin `put_if_absent`; its first raw operation must be the authoritative `read_back` that reconciles a possible prior-owner commit. `Found` is validated against retained success metadata and proceeds to exact success/failure terminalization; authoritative `Missing` may mint only `BodyUnavailableAfterFencedOwner` and terminalize the declared recovery failure. A cancellation request while recovery owns the authorization is ignored until that reconciliation path reaches a terminal receipt or infrastructure uncertainty leaves `TransportStopped` for a later fenced retry. An unexpired/live owner returns `Busy` without mutation; a terminal row returns its receipt; stale/cross-bound evidence fails closed. Consequently a recovery caller cannot observe `Missing` while the normal writer remains authoritative, an old writer cannot pass the epoch/state checks after handoff, and recovery receives a fresh 25-second read-back-plus-fence budget but no put budget or initial cancellation path. `OwnerLeaseV1`, `SnapshotWorkAuthorizationV1`, `SnapshotLeaseBusyReceipt`, `OwnerFenceWitnessV1`, `SourceCompletionFailureDecisionV1` and `OwnerLeaseErrorV1` are closed owner-private carriers; only M60 mints decisions from validated raw-store observations or the exact trusted-time insufficient-lease-budget predicate.

R1's structural checker must parse the `SourceAuthorityRepository` declaration block and require exactly one `propose_exact_v2(definition: SourceDefinitionV2)` method declaration; any overload, second declaration, caller-supplied digest parameter, v1-shaped substitute or prose-to-port signature mismatch fails. Prose references do not count as declarations. The repository recomputes the §6.1 v2 digest, applies source-ID precedence, then enforces one transaction-current canonical-URL uniqueness index across v1/v2 with exact `CanonicalUrlOccupied` projection before insertion. The same parser requires `validate_request_admission_bindings` before every command-ledger lookup, then exact `classify_approval_command(command_id, approval_body_digest, &VerifiedRequestAdmissionEvidence)` with retained sealed-binding comparison before replay, then transactional `prepare_approval_evidence_load(command: SourceApproveCommandV1, approval_body_digest, VerifiedRequestAdmissionEvidence)` with five source-first guards, before any evidence load; requires evidence load only from `Ready(SourceApprovalEvidenceLoadPlanV1)`; requires every evidence repository outcome to become `SourceApprovalEvidenceObservationV1` passed with that exact plan to `transact_approval`; and requires exact `transact_approval(...) -> Result<SourceApproveResultV1, SourceApprovalApplicationError>` and its final transaction to revalidate the plan/source guards before interpreting evidence infrastructure/content. It also requires the exact `RetrievalAdmissionEnvelopeV1` fields, exact `validate_retrieval_request_admission_bindings(context: &PlatformRequestContext, witness: &VerifiedRequestAdmissionEvidence, command: &RetrievalAttemptCommand)` before any M60 retrieval ledger/source lookup, and exact `decide_admission(context: &PlatformRequestContext, VerifiedRequestAdmissionEvidence, command: RetrievalAttemptCommand)` signature plus `Result<SourceAdmissionResultV1, SourceAdmissionErrorV1>` and `Result<SourceStartResultV1, SourceStartErrorV1>`, the three phase-specific `ClockUnavailable` application errors and their replay-first/zero-mutation mappings, the trusted-time fresh-deadline snapshot recovery/authorization plus completion-failure-decision signatures, and all declared typed owner-lease operations exactly once; it requires the exact pre-dispatch-only snapshot cancellation method plus linear M60 snapshot-workflow authorization custody and the synchronous bounded raw-call model, including exact `InitialAfterTransport | RecoveryAfterFence` authorization origin, recovery-first read-back/no-cancel/no-put, and pending-cancellation `Stored | AlreadySame` continuation through read-back/terminalization versus conflict/uncertain non-result branches, the one-second live reservation-expiry coordinator/start-race algebra, and the one-second live abandoned-work recovery coordinator for both `Started | TransportStopped`, each with exact `Complete | Partial | Failed` progress/error carriers; it requires the exact full `VerifiedRequestAdmissionEvidence` parameter on `transact_record`, including sealed `admission_observed_at`, the M60-owned `record_body_digest` domain, closed evidence-record result/rejection carriers, closed bundle-load outcome/error mapping, declaration-order future-timestamp rejection on record and approval, the closed execution/snapshot owner-coverage outcomes plus exhaustive `OwnerLeaseErrorV1` renewal-to-`SourcePostStartErrorV1` mapping with error-zero/successful-short-exact-CAS mutation semantics, and the closed 50-second snapshot lease/45-second two-call `InsufficientSnapshotLeaseBudget` decision/reason/predicate mapping; omission, overload, pre-transaction infrastructure return, reordered source/evidence precedence, an undeclared clock, or decoy prose fails. The parser also requires `platform-request-context/v1` to add exactly the four successor variants and two coherence pairs above, retain the fourteen rejection classes/mappings, and bind the retrieval descriptor to the M10/M60 executable consumer; naming the descriptor without declaring those enum/wire variants fails. The same R1 checker must require the exact `RetrievalTransportRequestV1` accessor list and `SourceTransportPortV1::transport` signature in the `source-retrieval/v1` owning section, prove the accepted v0 request/port block remains byte/API unchanged, reject `canonical_host_text` on the v0 type and bind the M60/M90 consumer to the v1 type/port.

`SourceRetrievalUnitOfWorkPort` transactions read the current source authority row and mutate source schedule + attempt journal atomically. Thus start-time status/revision/current rate and `last_attempt_started_at` cannot tear across stores. Source authority, command/attempt dual indexes and terminal rejection tombstones are durable before acknowledgement. At retrieval admission entry, M60 first runs owner-private `validate_retrieval_request_admission_bindings(context: &PlatformRequestContext, witness: &VerifiedRequestAdmissionEvidence, command: &RetrievalAttemptCommand)` under the same pre-ledger discipline used by approval: it compares command/correlation/actor/capability/grant/revision/operation/schema/permission/effect/adapter/policy/time/event-key/event-digest bindings before any M60 ledger/source lookup. It then derives `RetrievalAuthorityEvidence` only from the sealed context and stores the complete `RetrievalAdmissionEnvelopeV1`; retained equal replay requires byte-equivalent authority and admission witness, while any unequal command/envelope replay is `ConflictingCommandReplay` and cannot reserve or release a slot. Callers cannot construct either authority carrier.

The admission/start public result and error algebra is complete:

```text
SourceAdmissionResultV1 =
  Reserved(ReservedPlan) |
  Replay(AdmittedReplayReceipt) |
  Rejected(AdmissionRejectedReceipt)

SourceAdmissionErrorV1 =
  MalformedCommand | ConflictingCommandReplay | CrossBoundIdentity |
  ClockUnavailable | RepositoryUnavailable | RepositoryCorrupt | CommitFailed

SourceAdmissionRejectionReasonV1 =
  SourceMissing | UnsupportedSourceDefinitionVersion | SourceNotApproved |
  SourceAuthorityRevisionMismatch |
  DefinitionDigestMismatch | RetrievalProtocolMismatch |
  OperatorPolicyRevisionMismatch | PolicyDenied | ClockRegression |
  RateIntervalNotElapsed |
  SourceConcurrencyLimitReached | HostConcurrencyLimitReached |
  GlobalConcurrencyLimitReached

AdmissionRejectedReceipt {
  command_id, attempt_id, source_id, expected_authority_revision,
  expected_definition_digest, candidate_request_digest,
  reason: SourceAdmissionRejectionReasonV1,
  observed_at, observed_definition_version?, observed_source_status?, observed_authority_revision?,
  observed_definition_digest?, observed_protocol?, observed_operator_policy_revision?,
  last_attempt_started_at?, retry_not_before?, durable_state_digest
}

SourceStartResultV1 =
  FirstDispatch(EffectReadyPlan) |
  Replay(StartedReplayReceipt) |
  Rejected(StartRejectedReceipt) |
  ReservationExpired(ReservationExpiredReceipt)

StartRejectedReceipt {
  command_id, attempt_id, source_id,
  reason: SourceStartRejectionReasonV1,
  observed_owner_lease_expires_at?, required_owner_lease_not_before?,
  durable_state_digest
}

SourceStartRejectionReasonV1 =
  UnsupportedSourceDefinitionVersion | SourceNotApproved | SourceRevisionMismatch |
  DefinitionDigestMismatch |
  RequestDigestMismatch | AuthorizationExpired | AuthorizationMismatch |
  ClockRegression | RateNoLongerAdmitted | OwnerEpochStale |
  InsufficientExecutionOwnerLeaseCoverage | ArithmeticOverflow

SourceStartErrorV1 =
  MalformedCommand | ConflictingCommandReplay | CrossBoundIdentity |
  ClockUnavailable | RepositoryUnavailable | RepositoryCorrupt | CommitFailed

SourcePostStartResultV1 =
  SnapshotWorkReady(SnapshotWorkAuthorizationV1) |
  Applied(SourcePostStartReceiptV1) |
  Replay(SourcePostStartReceiptV1)

SnapshotRecoveryClaimResultV1 =
  Claimed(SnapshotWorkAuthorizationV1) |
  Busy(SnapshotLeaseBusyReceipt) |
  Replay(SourcePostStartReceiptV1)

SourcePostStartReceiptV1 =
  TransportStopped { command_id, attempt_id, source_id, request_digest,
                     success_metadata_digest, stop_receipt_digest,
                     snapshot_owner_id, snapshot_owner_epoch,
                     snapshot_lease_expires_at, durable_state_digest } |
  CompletedSuccess { command_id, attempt_id, source_id, body_digest,
                     snapshot_identity, snapshot_witness_digest, durable_state_digest } |
  CompletedFailure { command_id, attempt_id, source_id,
                     reason: CompletionFailureReasonV1, stop_receipt_digest,
                     completion_failure_decision_digest?,
                     durable_state_digest } |
  Cancelled { command_id, attempt_id, source_id, reason: CancellationReasonV1,
              dropped_future_witness_digest?, owner_fence_witness_digest?,
              prior_owner_epoch?, advanced_owner_epoch?,
              durable_state_digest } |
  ReservationExpired { command_id, attempt_id, source_id,
                       reservation_expires_at, durable_state_digest } |
  ExecutionAbandoned { command_id, attempt_id, source_id,
                       fenced_prior_owner_epoch, fenced_owner_epoch,
                       prior_owner_lease_expires_at,
                       fence_cause: OwnerFenceCauseV1, fence_witness_digest,
                       durable_state_digest }

CompletionFailureReasonV1 =
  Transport(SourceTransportError) | SnapshotBodyUnavailableAfterCrash |
  InsufficientSnapshotLeaseBudget |
  SnapshotContentConflict | SnapshotMissingAfterWrite |
  SnapshotReadbackMismatch

SourceCompletionFailureDecisionV1 =
  BodyUnavailableAfterFencedOwner { attempt_id, source_id, expected_state_digest,
    fence_witness_digest, missing_observation_digest } |
  InsufficientSnapshotLeaseBudget { attempt_id, source_id,
    expected_state_digest, observed_at, snapshot_lease_expires_at,
    required_budget_seconds } |
  ContentConflict { attempt_id, source_id, expected_state_digest,
    conflict_observation_digest } |
  MissingAfterWrite { attempt_id, source_id, expected_state_digest,
    write_receipt_digest, missing_observation_digest } |
  ReadbackMismatch { attempt_id, source_id, expected_state_digest,
    write_receipt_digest, read_observation_digest }

Decision-to-reason mapping is exact: `BodyUnavailableAfterFencedOwner -> SnapshotBodyUnavailableAfterCrash`, `InsufficientSnapshotLeaseBudget -> InsufficientSnapshotLeaseBudget`, `ContentConflict -> SnapshotContentConflict`, `MissingAfterWrite -> SnapshotMissingAfterWrite`, and `ReadbackMismatch -> SnapshotReadbackMismatch`. The insufficient-budget decision is M60-owner-minted only when checked arithmetic proves `observed_at + 20s + 20s + 5s > snapshot_lease_expires_at`; if any addition overflows, the method returns `SourcePostStartErrorV1::ArithmeticOverflow`, constructs no decision/raw future and preserves `TransportStopped` unchanged. `required_budget_seconds` must equal `45`, no raw future/call may have been constructed, and the current snapshot authorization plus observation/deadline digest must match the row. No other mapping exists; post-put budget uncertainty and raw-adapter uncertainty cannot construct a terminal decision and remain recoverable as `TransportStopped`.

`completion_failure_decision_digest` is SHA-256 over ASCII domain `"source-completion-failure-decision/v1\0"`, then the exact snake_case variant tag and every displayed variant field in declaration order: text/IDs use §6.1 `enc`, digests use raw 32-byte form, times use checked big-endian `i128`, and `required_budget_seconds` uses big-endian `u64`. It is computed by M60, persisted with the terminal receipt and recomputed on replay; callers cannot supply it.

CancellationReasonV1 = BeforeSnapshotWrite | DroppedFuture

SourcePostStartErrorV1 =
  MalformedCommand | ConflictingCommandReplay | CrossBoundIdentity |
  MissingAttempt | InvalidStateTransition | OwnerEpochStale |
  StopReceiptMismatch | SnapshotEvidenceMismatch |
  ClockUnavailable | InsufficientOwnerLeaseCoverage | ArithmeticOverflow |
  RepositoryUnavailable | RepositoryCorrupt | CommitFailed
```

Post-start method-to-result mapping is closed: successful first `record_transport_stopped` commits the `TransportStopped` receipt/lease and yields `SnapshotWorkReady`; equal replay yields only `Replay(TransportStopped)` and no second work authorization; transport error yields `Applied(CompletedFailure(Transport(error)))`; `mark_snapshot_stored_and_complete_success` requires current snapshot-work authority, atomically persists the snapshot witness, yields `Applied(CompletedSuccess)` and releases all slots; `complete_failure` requires the same current authority plus one owner-validated `SourceCompletionFailureDecisionV1` whose variant maps exactly to the corresponding non-transport failure reason; `cancel_snapshot_work_before_dispatch`, `expire_reservation` and `reap_abandoned_execution` yield exactly `Cancelled(BeforeSnapshotWrite)`, `ReservationExpired` and `ExecutionAbandoned`; owner-verifiable local future-drop cancellation yields `Cancelled(DroppedFuture)` through the distinct non-expiry transition only after the ledger-backed `DroppedFuture` fence witness and epoch read-back. `Applied` means the one atomic terminal transition committed; equal command/body replay returns `Replay` with the byte-equivalent persisted receipt and no release/effect repetition. `Cancelled(DroppedFuture)` requires matching dropped-future and owner-fence witness digests plus `advanced_owner_epoch > prior_owner_epoch`; `Cancelled(BeforeSnapshotWrite)` forbids every drop-only witness/epoch field. Each `SourceCompletionFailureDecisionV1` requires its displayed observations and forbids fields from every other variant; infrastructure uncertainty constructs no decision and leaves `TransportStopped` unchanged. Any method returning a result/receipt variant outside its mapping, missing/extra stop/snapshot/drop/failure witness, violating lease/cancellation/decision presence coherence, or decoding an unknown reason is `RepositoryCorrupt`. `SourcePostStartErrorV1` never acknowledges a partial transition; storage/commit uncertainty preserves the pre-call durable state for replay/recovery.

For `CompletedFailure(Transport(error))`, `completion_failure_decision_digest` is absent because the validated transport error carries its own failure observation digest. Every non-transport `CompletedFailure` requires the exact owner-minted decision digest and forbids absence; any other presence is `RepositoryCorrupt`.

`ReservedPlan`, `EffectReadyPlan` and `SourceStartAuthorization` are non-clone, non-Serde, private-field linear carriers. `AdmittedReplayReceipt`, `StartedReplayReceipt`, `AdmissionRejectedReceipt`, `StartRejectedReceipt` and `ReservationExpiredReceipt` are non-authoritative private-field state projections bound to command/attempt/source/revision/request digest and the durable state digest. Every valid admission guard maps to exactly one closed rejection reason above and persists the complete receipt plus both indexes before acknowledgement. `ClockRegression` preserves exactly accepted `source-retrieval/v0` §6.3 steps 1 and 3: trusted `now < last_attempt_started_at` returns that dedicated outcome before interval evaluation and never a rate retry. The receipt observation matrix projects the same fact as `observed_at = now`, `last_attempt_started_at = last`, absent `retry_not_before`; `RateIntervalNotElapsed` instead requires `last <= now` and `retry_not_before = checked(last + minimum_interval_seconds)`. `SourceMissing` requires `observed_at = trusted_now` because the replay-first unseen path has already acquired the trusted clock; every optional source/version/status/revision/digest/protocol/operator-policy observation plus `last_attempt_started_at` and `retry_not_before` is absent. `UnsupportedSourceDefinitionVersion` requires only the observed version discriminator and forbids v2 status/revision/digest/protocol projection; every other reason requires exact observed definition version `v2`, status, revision, definition digest and protocol; operator-policy reasons additionally require the observed operator-policy revision; all non-clock/non-rate reasons forbid both time-option fields. The definition-embedded source-use policy is already bound by authority revision plus definition digest and has no independent mutable revision. Equal replay of admission or start returns the corresponding byte-equivalent replay/terminal receipt and never reconstructs a reserved/effect carrier. Unknown reason tags, missing/extra receipt bindings or reason/optional-field incoherence are `RepositoryCorrupt`; malformed/cross-bound input mutates no row or index; commit/storage failures expose no partial transition.

A start coverage failure is representable and ordered. After execution-expiry arithmetic succeeds, M60 computes `required_owner_lease_not_before = checked(execution_lease_expires_at + 5s)`; overflow yields `ArithmeticOverflow`. If either the supplied current owner lease expiry or its hard incarnation cap is earlier than that instant, `start` persists `Rejected(StartRejectedReceipt { reason: InsufficientExecutionOwnerLeaseCoverage, observed_owner_lease_expires_at: Some(actual), required_owner_lease_not_before: Some(required), ... })`, releases the reserved source/host/global slots and mints no `EffectReadyPlan`. Both optional fields are present only for that reason and absent for every other reason; any mismatch is `RepositoryCorrupt`. `OwnerLeaseErrorV1::LeaseSpanExceeded` from a pre-start renewal is not silently remapped: the final transaction evaluates the retained lease and produces this domain rejection. The same insufficient-coverage condition in `record_transport_stopped` or `claim_snapshot_recovery` maps only to `SourcePostStartErrorV1::InsufficientOwnerLeaseCoverage`, performs no journal/owner/storage mutation and is retried or eventually handled by the live abandonment loop.

Simultaneous failures resolve by one frozen order. `decide_admission` checks: malformed/cross-bound/sealed-admission binding and checked arithmetic → dual command/attempt replay or conflict → trusted-clock acquisition → source missing → definition version → source status → authority revision → definition digest (including `SourceUsePolicyV1`) → retrieval protocol → operator-policy revision → policy denial → clock regression → interval not elapsed → source concurrency → host concurrency → global concurrency → reservation-expiry arithmetic → commit. `start` checks: malformed/cross-bound identity → equal replay or command/attempt/authorization conflict → trusted-clock acquisition → reservation expiry → definition version → source status → authority revision → definition digest/source-use policy → request digest → authorization identity → authorization validity window → clock regression → rate no longer admitted → owner epoch → execution-expiry arithmetic → execution-owner lease/incarnation coverage → commit. Equal terminal replay and conflicting replay therefore require no clock, but an unseen admission/start whose trusted-clock read fails returns the phase's exact `ClockUnavailable` application error before source read, journal/index/slot mutation or effect authority. The first failing step alone determines the persisted result; later facts are neither evaluated nor projected. Thus reservation expiry outranks source revocation/revision and authorization expiry in the same successfully clocked snapshot, while a valid reservation evaluates source facts before authorization time. Every fake/mutation test must cover at least one multi-failure vector for each ordered adjacent pair; implementations may not reorder guards for convenience.

`start` uses deliberate at-most-once dispatch semantics. Its durable `Started` record stores authorization/command/attempt/source identities, owner epoch, exact source authority revision/definition digest and operator-policy revision, exact serialized request bytes and digest, all non-authority bounds, `started_at`, execution expiry and `dispatch_state = IssuedOnce`. Only the process that commits the first `Started` transition receives `SourceStartResultV1::FirstDispatch(EffectReadyPlan)`. Equal replay returns `SourceStartResultV1::Replay(StartedReplayReceipt)` and never another plan; there is no load/reissue API. A crash after commit but before I/O may waste the attempt, consumes the source rate interval, and later becomes `ExecutionAbandoned` after owner fencing. This chooses no-duplicate-effects over liveness for the MVP; a future transactional outbox/retry design requires a new accepted protocol.

`RawSnapshotEvidencePort` identity is exactly `sha256("ustc-raw-source-snapshot/v1\0" || enc(source_id) || enc(attempt_id))`, using §6.1 `enc`; M60 computes it and M90 cannot choose or rewrite it. `RawSnapshotWriteReceipt` and `RawSnapshotReadObservation` are public data-only values containing exactly snapshot identity, source ID, attempt ID, byte count and SHA-256. `put_if_absent` atomically stores absent bytes, returns `AlreadySame` only when retained bytes, count and digest are equal, and returns `ExistingContentConflict` without mutation for the same identity with any unequal content. `Missing` is a successful read outcome; partial records, impossible receipt/read observations, digest disagreement or identity disagreement are `RepositoryCorrupt`, never `Missing`. `RawSnapshotStored` is owner-private and minted only after a `Stored | AlreadySame` receipt plus read-back prove all five fields and exact retained bytes against durable successful transport metadata. The journal stores the witness digest/identity, not raw bytes.

`cancel_snapshot_work_before_dispatch` is strictly pre-dispatch and accepts only `SnapshotWorkAuthorizationV1 { origin: InitialAfterTransport, ... }`. It consumes an otherwise unspent initial authorization; a `RecoveryAfterFence` authorization returns `SourcePostStartErrorV1::InvalidStateTransition` with no journal/raw/terminal mutation and remains obliged to perform recovery read-back. Because that carrier is non-clone and linear, starting
`put_if_absent` moves it into one scoped M60-private snapshot-workflow frame and makes the
cancellation method uncallable. The synchronous adapter methods receive only their declared
data-only arguments; the workflow frame retains the same private authorization across the
put result, mandatory read-back and terminal transaction. Once a raw call has begun, a cancellation request synchronously joins that scoped call until it returns within its 20-second bound and never mints cancellation authority. Result handling is closed. `Stored | AlreadySame` proves the put may have committed, so the pending cancellation is ignored for this workflow: M60 must continue immediately through the mandatory read-back and then atomically complete success or persist an exact owner-validated failure under the same current authorization. `ExistingContentConflict` mints the exact conflict failure decision and terminalizes without a read-back. A raw-adapter error or other uncertain non-result starts no next call and leaves the durable row `TransportStopped` for lease-expiry fencing and recovery read-back. Thus `cancel_snapshot_work_before_dispatch` is unreachable after dispatch, `Cancelled(BeforeSnapshotWrite)` proves that no raw call was dispatched, and every committed/equal or uncertain post-put path is reconciled without attaching immutable bytes to terminal cancellation.

The physical snapshot adapter returns only the closed data-only results above. M60 compares observations against the retained successful body and alone mints `RawSnapshotStored`; M90 cannot construct that witness. `list_recoverable` is bounded to `1..=256` rows, ordered by `RetrievalAttemptId`, and uses an exclusive cursor so recovery is finite and replayable. Startup runs pages to exhaustion before accepting retrieval admissions. For `TransportStopped`, it may inspect raw storage only after `claim_snapshot_recovery` fenced the expired prior owner and returned fresh snapshot-work authority; before that, `Busy` is not body absence. A repository/adapter unavailable or corrupt result keeps retrieval admission unavailable rather than silently leaking capacity. Runtime infrastructure uncertainty leaves `TransportStopped` unchanged and wakes the same recovery loop. Under `InitialAfterTransport`, existing-content conflict, missing after an acknowledged write, or read-back mismatch mints the matching owner-private failure decision because that workflow retains the exact put/write receipt. Under `RecoveryAfterFence`, no such write receipt exists: exact `Found` matching durable success metadata mints `RawSnapshotStored` and completes success; authoritative `Missing` alone may mint `BodyUnavailableAfterFencedOwner`; a `Found` identity/count/digest mismatch returns `SourcePostStartErrorV1::SnapshotEvidenceMismatch`, mints no write-receipt-dependent decision, changes no journal/terminal state, and leaves `TransportStopped` for explicit investigation or later recovery retry. Each valid owner-minted decision then releases slots through the terminal failure transaction.

The execution lease governs `Started`, where the transport owner may still exist; the distinct snapshot-owner lease governs `TransportStopped`, where transport resources are gone but a normal raw writer may still be live. Durable-state CAS serializes journal transitions but does not by itself serialize the external raw put, so no recovery caller may inspect or interpret storage merely by loading `TransportStopped`. The normal `InitialAfterTransport` holder either finishes put/read/terminal commit under its current epoch/lease or loses authority. Only after that lease expires and `fence_expired_owner` proves epoch advancement may `claim_snapshot_recovery` hand the row to a new owner. Every old owner call then fails current epoch/state-digest validation, while the new recovery owner may perform only the authoritative raw read-back reconciliation and exact terminalization; `put_if_absent` remains forbidden under `RecoveryAfterFence`. Thus `Missing` cannot race an authoritative slow normal put; adapter uncertainty may delay availability but cannot authorize a second retrieval effect, orphan-after-terminal write or duplicate slot release.

Before any raw snapshot write, `record_transport_stopped` accepts a closed result. `ValidatedTransportResultMetadataV1::Success { attempt_id, source_id, request_digest, status, response_head_digest, body_byte_count, body_sha256, validated_media_type, framing_mode, framing_complete, wire_byte_count, selected_peer_digest, dns_observation_digest, elapsed_milliseconds, stop_receipt_digest }` is representable from `RetrievalTransportSuccess` plus the retained plan/stop receipt and atomically enters `TransportStopped` only after proving the current snapshot owner lease and incarnation cap cover a checked 50-second snapshot lease plus five-second fence margin, storing that deadline and first-returning `SnapshotWorkAuthorizationV1`. Insufficient coverage returns the closed post-start error and leaves `Started` unchanged. `ValidatedTransportResultMetadataV1::Failure { attempt_id, source_id, request_digest, error_class: SourceTransportError, stop_receipt_digest }` intentionally requires no peer/DNS/elapsed payload that the accepted fieldless `SourceTransportError` cannot carry; partial physical observations are neither fabricated nor required. The failure transaction atomically persists this metadata and stop-receipt digest, enters `CompletedFailure`, and releases source/host/global slots—there is no durable intermediate `TransportStopped` failure state. A crash before that atomic commit leaves `Started` for fenced abandonment; a crash after it sees the terminal receipt. Only successful `TransportStopped` can enter snapshot storage. Raw body bytes are not stored in the journal. If the process crashes after durable success metadata but before raw put, startup must first expire/fence/read back the snapshot owner and atomically claim recovery; only its ensuing authoritative `Missing` may mint `BodyUnavailableAfterFencedOwner` and terminalize `SnapshotBodyUnavailableAfterCrash`. If raw put committed, the claimed recovery owner compares read-back against durable success metadata before M60 can mint `RawSnapshotStored`. Adapter `RepositoryUnavailable | CommitFailed` is non-terminal and constructs no failure decision; `RepositoryCorrupt` fails closed and keeps admissions unavailable for operator repair.

Public port carriers are data-only and expose no DB/TLS/HTTP/client type. Private linear carriers remain owner-private or crate-private; visibility is not widened merely for integration tests.

## 8. B3 state machine and edge semantics

### 8.1 Durable states

`SourceStartAuthorization` is owner-private and binds exactly: authorization ID, command ID, attempt ID, source ID, current authority revision, complete definition digest, serialized-request digest, authenticated operator identity, operator-policy revision, `AttemptOnly`, issued-at and not-after. `start` reloads the source and policy transaction-currently, requires equality for every bound field, requires `issued_at <= now <= not_after`, and atomically consumes the unique authorization ID with the `Started` commit. No authorization is transferable across a source, attempt, command, revision, definition or request generation.

`TransportStopReceiptV1` is non-clone, non-Serde and owner-private to the M60 coordinator. It binds invocation ID, attempt ID, serialized-request digest, resource-owner ID/epoch and terminal exchange class. M60 alone consumes `EffectReadyPlan`, retains its authority/lineage fields, derives one owned move-only data-only `RetrievalTransportRequestV1`, and invokes `SourceTransportPortV1::transport(RetrievalTransportRequestV1)`. The coordinator may mint the stop receipt only from completion of that exact future. M90 never receives, names or reconstructs `EffectReadyPlan`. The port future owns all DNS/socket/TLS/body tasks in one scoped resource tree: it may not detach work, and success, error, cancellation or drop returns only after synchronously cancelling, joining and destroying every owned resource. Public/fake transport observations are non-authoritative and cannot enter `record_transport_stopped`. M60 verifies the returned success/error against the retained internal plan plus receipt/Started-record identity and current owner epoch before either the successful `Started -> TransportStopped` transition or atomic failure terminalization; a boolean, elapsed timeout or caller-created observation is never a stop proof.

Accepted `SourceTransportPortV1` drop produces no observation/result, so it never mints `TransportStopReceiptV1`. M60 owns the in-flight future and exposes no caller drop handle. Each started attempt has its own never-shared owner ID. On local in-process cancellation, M60 first drops that exact future; only after drop returns—therefore after accepted synchronous resource destruction—does it mint owner-private, non-Serde/non-clone `DroppedFutureWitness { attempt_id, request_digest, owner_id, stored_epoch }` and call `advance_epoch_after_drop`, which atomically advances the attempt owner and persists an `OwnerFenceWitnessV1::DroppedFuture`. M60 reads back the exact newer epoch, then calls `cancel_started_after_drop`; that transaction validates witness cause/digest and Started identity, needs no execution-lease expiry, atomically enters `Cancelled`, persists the receipt and releases all slots. Failure before the advance leaves the original row/epoch; failure after the advance leaves the durable replayable witness. If the process then crashes, startup waits for the retained owner/execution leases, equal-replays that witness and may call `reap_abandoned_execution`; no row becomes permanently stale merely because cancellation was interrupted. A stale/equal epoch without exact ledger replay cannot terminalize. This owner-verifiable path handles early cancellation without fabricating a transport observation.

```text
AdmissionRejected
Admitted
ReservationExpired
StartRejected
Started
TransportStopped
CompletedSuccess
CompletedFailure
Cancelled
ExecutionAbandoned
```

### 8.2 Transition matrix

| Current | Command | Required guard | Next / durable effect |
|---|---|---|---|
| absent | `decide_admission` reject | exact command/attempt IDs unused; candidate digest valid; first failing ordered policy/status/clock/rate/concurrency guard rejects | `AdmissionRejected`; persist both indexes and exact terminal reason, including `ClockRegression` before rate; return `Rejected(AdmissionRejectedReceipt)`; reserve no slot |
| absent | malformed `decide_admission` command | unknown/invalid candidate digest, malformed identity or unrepresentable checked arithmetic | typed `MalformedCommand`; no attempt row, index or slot mutation |
| absent | unseen `decide_admission` clock unavailable | sealed admission bindings pass and no replay/conflict exists; first M60 trusted-clock read fails | typed `SourceAdmissionErrorV1::ClockUnavailable`; no source read, attempt row, index or slot mutation |
| absent | `decide_admission` accept | source `Approved`; expected revision/current policy match; interval/concurrency pass | `Admitted`; reserve source/host/global slots; `reservation_expires_at = admitted_at + 30s`; return `Reserved(ReservedPlan)` once |
| `Admitted` | equal admission replay | command/body and dual indexes exactly equal | return `Replay(AdmittedReplayReceipt)`; no second `ReservedPlan` or slot |
| `Admitted` | `start` | before reservation expiry; source still `Approved`; same revision/digest; `AttemptOnly`; checked execution expiry and `required_owner_lease_not_before = execution_lease_expires_at + 5s`; a fresh attempt-unique owner lease expiry and incarnation cap both cover that instant | `Started`; atomically set `started_at`, `last_attempt_started_at`, unique owner/epoch, `execution_lease_expires_at = started_at + maximum_elapsed_seconds + 5s`; insufficient coverage instead persists `InsufficientExecutionOwnerLeaseCoverage`, releases slots and returns no plan; success returns `FirstDispatch(EffectReadyPlan)` once |
| `Admitted` | `start` guard fails | first failing frozen guard: reservation expiry before source/revision/request/authorization/clock/rate/owner/arithmetic | `Rejected(StartRejectedReceipt)` or `ReservationExpired(ReservationExpiredReceipt)`; release slots; no transport carrier |
| `Admitted` | unseen `start` clock unavailable | replay/conflict checks pass; first M60 start-time trusted-clock read fails | typed `SourceStartErrorV1::ClockUnavailable`; row and slots remain `Admitted`; no authorization consumption or effect carrier |
| `Started` | equal start replay | command/body, authorization identity and dual indexes exactly equal | return `Replay(StartedReplayReceipt)`; no second `EffectReadyPlan` |
| `Started` | post-transport stopped-time clock unavailable | exact scoped transport call returned but M60 cannot obtain trusted `stopped_at` | typed `SourcePostStartErrorV1::ClockUnavailable`; no owner/journal/raw mutation; row remains `Started`; same in-memory result may retry only in the current scoped coordinator, otherwise abandonment recovery releases it without transport reissue |
| `Started` | `record_transport_stopped` success | exact attempt/owner epoch/request digest; owner-private stop receipt proves the exact scoped port future returned after synchronous resource destruction; `RetrievalTransportSuccess` validates; checked snapshot lease can be created and the registered owner lease covers it plus fence margin | `TransportStopped`; durably retain success/stop metadata plus snapshot owner/epoch/lease; first caller gets `SnapshotWorkReady(SnapshotWorkAuthorizationV1 { origin: InitialAfterTransport, ... })` |
| `Started` | `record_transport_stopped` failure | exact attempt/owner epoch/request digest; same stopped proof; fieldless `SourceTransportError` is closed and valid | atomically persist minimal failure metadata, enter `CompletedFailure`, release all slots; no recoverable intermediate |
| `Started` | `cancel_started_after_drop` | M60 owns and drops exact future; `OwnerFenceWitnessV1::DroppedFuture` matches exact attempt/owner/request witness; epoch read-back is newer; lease expiry not required | `Cancelled`; persist receipt and release slots; no stop receipt or transport observation is fabricated |
| `TransportStopped` | `claim_snapshot_recovery` | prior snapshot lease expired; `OwnerFenceWitnessV1::ExpiredLease` matches retained attempt/owner/epoch; fresh attempt-unique recovery owner lease expiry and incarnation cap both cover `checked(trusted_now + 50s + 5s)`; no raw-store read occurred before claim | atomically replace snapshot owner/epoch/deadline with `fresh_snapshot_lease_expires_at = checked(trusted_now + 50s)` and return one `Claimed(SnapshotWorkAuthorizationV1 { origin: RecoveryAfterFence, ... })`; live/unexpired owner returns `Busy`; insufficient owner coverage returns `InsufficientOwnerLeaseCoverage`, overflow returns `ArithmeticOverflow`, both without mutation |
| `TransportStopped` | `mark_snapshot_stored_and_complete_success` | current snapshot-work authorization; successful bounded body; immutable put + read-back identity/count/digest pass; witness matches attempt/body digest | atomically persist witness, enter `CompletedSuccess`, release slots; no durable `SnapshotStored` intermediate |
| `TransportStopped` | `complete_failure` | current snapshot-work authorization; exact owner-validated failure decision; no infrastructure uncertainty represented as terminal evidence | `CompletedFailure`; success/cancellation forbidden; release slots |
| `TransportStopped` | `cancel_snapshot_work_before_dispatch` | unspent current `InitialAfterTransport` snapshot-work authorization remains in M60 and no raw adapter call has begun; `RecoveryAfterFence` is forbidden | `Cancelled(BeforeSnapshotWrite)` only for initial origin; recovery origin returns `InvalidStateTransition` without mutation and must reconcile storage; persist no raw witness on valid initial cancellation; release slots |
| `Admitted` | `expire_reservation` | `now >= reservation_expires_at`; never started | `ReservationExpired`; release slots; generic B3 override authority, if a future source admits it, remains consumed; this exact source has no override path at all |
| `Started` | startup/live `reap_abandoned_execution` | `now >= execution_lease_expires_at` and retained owner lease expired; the bounded recovery coordinator obtained an exact ledger-backed `OwnerFenceWitnessV1`—new `ExpiredLease` or equal-replayed prior `DroppedFuture`—and read back current epoch equal to its newer epoch; no stopped proof | `ExecutionAbandoned`; release slots; success forbidden |

Timeout alone never proves resource destruction or owner death. Startup invokes the same two runtime ticks below and pages all nonterminal rows to exhaustion before opening admission. For each old `Admitted` row, the one-shot `ReservedPlan` is irrecoverable by design: the reservation tick never reconstructs or starts it, waits until `reservation_expires_at`, calls `expire_reservation`, reads back `ReservationExpired`, and releases source/host/global slots. For each old or live-orphaned `Started` row, the abandonment tick waits until both recorded execution and attempt-owner leases expire, first looks up the advance ledger by exact `(attempt_id, owner_id, stored_epoch)`, equal-replays a retained witness when present, and only when absent calls `fence_expired_owner(old_owner_id, stored_epoch, trusted_now)` to create a new expired-lease fence; it independently reads back the returned newer epoch and passes that exact ledger-backed `OwnerFenceWitnessV1` to `reap_abandoned_execution`. For each old or live-orphaned `TransportStopped` row it performs the analogous ledger-first snapshot/owner-lease fence, then registers a fresh owner ID bound only to that attempt with enough incarnation budget before `claim_snapshot_recovery(..., trusted_now)` creates a fresh 50-second snapshot deadline and reconciles raw storage. Current-epoch advance with no matching ledger row is corruption. An unexpired deadline, failed fence/read-back, unavailable owner/storage state or overflow keeps that row until the nearest retained deadline/next bounded tick; no timeout alone releases slots. Startup reaches terminal/recoverable handling for every row before serving a new retrieval admission. Clean shutdown drops/joins and fences each active attempt owner separately; restart and live recovery never infer death from PID change, task disappearance, elapsed timeout or a new owner ID alone.

The same reservation-expiry algorithm remains active after startup. While serving, M60 invokes `M60RetrievalCoordinator::tick_reservation_expiry` at least once per trusted second and immediately after every successful `Reserved` acknowledgement. Each tick pages the complete nonterminal inventory; the global cap makes at most four rows slot-owning at once. Rows are processed in stable `attempt_id` order by separate idempotent `expire_reservation` transactions followed by read-back; the tick is deliberately partial-progress, not all-or-nothing. `fully_processed_prefix_count` counts only a leading row whose fresh state was conclusively classified as a no-op or whose expiration terminal read-back succeeded; the uncertain currently attempted row is excluded. `Complete` means the full bounded inventory was scanned with no uncertainty, `uncertain_attempt_id` absent, exact `fully_processed_prefix_count` in `0..=4`, ordered acknowledged terminal-receipt digests in `0..=4`, and the exact nearest remaining deadline. `Partial` means `fully_processed_prefix_count >= 1` before a later error; its progress lists only expirations whose terminal read-back succeeded, carries the exact possibly-committed current attempt when uncertainty exists, and forbids `next_reservation_deadline`. `Failed` means zero fully processed prefix rows and no acknowledged expiration/read-back in this invocation and may still identify one uncertain current attempt; prior invocations and durable repository state are never rolled back by either error variant. `Failed` presence is exact: `ClockUnavailable` requires both optional fields absent; a repository failure before the first row requires `trusted_now` present and `uncertain_attempt_id` absent; first-row `ReadBackFailed` requires both present. `Failed` has no `next_reservation_deadline` field by construction.

For every `Admitted` row with `trusted_now >= reservation_expires_at`, it calls `expire_reservation` and requires byte-equivalent `ReservationExpired` read-back before treating slots as reusable. It expires the row even if an in-memory plan object still exists, because `start` is forbidden after the deadline. `start` and expiry serialize in the same unit of work: start-first changes the row to `Started`, so the tick's fresh read skips it as non-`Admitted` and returns no expiry result for that row; expiry-first returns `ReservationExpired` to any later start and no effect carrier exists. Clock/repository/read-back uncertainty returns `Partial` or `Failed`: every already acknowledged expiration remains terminal/released, every not-yet-processed row remains unchanged, and an uncertain current row is resolved by byte-equivalent `expire_reservation` replay/read-back on the next one-second tick before that invocation claims its receipt digest. No output fabricates release or rolls back prior progress. It is not an additional `decide_admission` guard: a simultaneous admission follows the existing `RepositoryUnavailable | RepositoryCorrupt` application errors when its own transaction cannot read trustworthy state, or the persisted source/host/global concurrency rejection when retained slots exhaust capacity. Thus loss of a one-shot plan while the process stays live is reclaimed within one coordinator tick after the 30-second durable deadline, without requiring restart.

The abandonment/recovery algorithm is equally live. While serving, M60 invokes `M60RetrievalCoordinator::tick_abandoned_work_recovery` at least once per trusted second, immediately after an owned transport/snapshot task exits without a terminal receipt, and when the nearest tracked execution/owner/snapshot deadline becomes due. It pages only `Started | TransportStopped` rows in stable `attempt_id` order, with the same `1..=256` cursor and current global slot cap. A due `Started` row is fence-ledger lookup/equal replay → optional `fence_expired_owner` → epoch read-back → idempotent `reap_abandoned_execution` → terminal read-back. A due `TransportStopped` row is analogous fence/read-back → fresh attempt-unique recovery owner registration → `claim_snapshot_recovery` → authoritative raw read-back reconciliation only (`put_if_absent` forbidden under `RecoveryAfterFence`) → terminal success/failure read-back. It never reissues transport. `fully_processed_prefix_count` excludes an uncertain current row; acknowledged digest lists contain only byte-equivalent terminal read-backs. `Complete | Partial | Failed` has the same prefix/uncertainty semantics as the reservation tick. `ClockUnavailable` has neither optional field; a pre-row owner/repository/storage failure has trusted time but no uncertain attempt; post-mutation read-back uncertainty carries the attempt. A claimed but nonterminal snapshot row is the uncertain current row and is retried under its retained fresh owner/deadline; prior terminal receipts remain released and are not rolled back. `next_recovery_deadline` is present only on `Complete` and is the minimum checked deadline among retained `Started | TransportStopped` rows. Thus a live coordinator panic or lost task cannot hold source/host/global slots beyond the two retained leases plus one bounded recovery tick, even when the process itself never restarts.

Byte-equivalent terminal-result replay applies to source approval and retrieval terminal/rejection receipts. Retrieval admission/start are deliberate linear-carrier exceptions: equal replay returns the state-bound `AdmittedReplayReceipt`/`StartedReplayReceipt`, never the original or a reconstructed `ReservedPlan`/`EffectReadyPlan`. Conflicting command body or command/attempt/authorization cross-binding fails. Malformed state, impossible transition, unknown enum, missing index, unequal duplicated index payload or digest mismatch yields `RepositoryCorrupt` and no mutation.

### 8.3 Suspension/revocation race

Suspend/revoke/revision before `start` forces `StartRejected`. A retrieval already atomically started while approved may stop and retain raw historical attempt evidence; later suspension/revocation does not rewrite history. It cannot create SourceRevision, baseline or publication authority: every later phase reloads source lifecycle/current revision separately and fails closed when not approved/current.

### 8.4 Crash boundaries

- crash before admitted transaction commit: no acknowledgement and no attempt row;
- crash after admit commit: replay returns `AdmittedReplayReceipt`, never another `ReservedPlan`; the live one-second expiry coordinator or startup drain waits the retained reservation deadline, commits/read-backs `ReservationExpired` and releases every slot before reuse;
- crash after start commit: replay returns `StartedReplayReceipt`; same attempt/owner epoch remains `Started` and no second effect carrier exists;
- crash or live coordinator loss after `advance_epoch_after_drop` commits but before `cancel_started_after_drop`: the append-once `DroppedFuture` fence witness remains bound to that attempt; after the retained leases expire, the next live/startup abandonment tick equal-replays it and completes `ExecutionAbandoned` without another epoch increment;
- crash or live coordinator loss after a new expired-lease fence commits but before `reap_abandoned_execution` or `claim_snapshot_recovery`: the next live/startup abandonment tick equal-replays the byte-identical `OwnerFenceWitnessV1` for retained `(attempt_id, owner_id, expected_epoch)`, reads back the already-advanced epoch and continues without a second increment;
- crash around transport failure terminalization: before the atomic failure commit the row stays `Started` and needs fenced abandonment; after commit it is terminal `CompletedFailure` with released slots;
- crash or live snapshot-task loss after successful `TransportStopped` commit but before raw put: the old snapshot lease first expires, the next live/startup abandonment tick fences/read-backs that owner and atomically claims snapshot work; only then may its first read return `Missing` and mint `BodyUnavailableAfterFencedOwner`, which terminalizes as `SnapshotBodyUnavailableAfterCrash` and releases slots;
- crash during or after raw put but before journal witness: recovery performs the same fenced claim before any read; exact `Found` bytes are verified against durable successful metadata, mint `RawSnapshotStored`, and call `mark_snapshot_stored_and_complete_success`; authoritative `Missing` alone mints `BodyUnavailableAfterFencedOwner` and terminalizes `SnapshotBodyUnavailableAfterCrash`; a `Found` identity/count/digest mismatch returns non-terminal `SnapshotEvidenceMismatch`, mints no failure decision, changes no journal/terminal state and leaves `TransportStopped` for explicit investigation or later fenced retry; corrupt or impossible read observations are `RepositoryCorrupt`; a still-`Started` attempt never advances from raw bytes and remains eligible only for fenced cancellation/abandonment;
- crash around snapshot completion: before the atomic witness/completion commit the row remains `TransportStopped` under its current snapshot owner lease; after owner death/expiry it can be fenced and reclaimed again; after commit it is terminal `CompletedSuccess` with released slots; no durable `SnapshotStored` intermediate exists;
- crash after terminal commit: replay returns the same terminal result.

The first retained B3 implementation proves this algebra using deterministic semantic fakes only. It performs no DNS/socket/TLS/HTTP, parser, revision, baseline or publication and does not activate the source.

## 9. M90 transport and physical adapter boundary

A later separately accepted slice may implement:

- M60 consumes `EffectReadyPlan`, retains its authority/lineage, derives one data-only `RetrievalTransportRequestV1`, and invokes `SourceTransportPortV1::transport(RetrievalTransportRequestV1)`; M90 performs one fresh DNS/socket/TLS/HTTP exchange in one scoped resource tree and returns only `RetrievalTransportSuccess` or transport-only `SourceTransportError` after every owned resource is synchronously cancelled/joined/destroyed; M60 validates the observation against the retained plan and alone then mints `TransportStopReceiptV1`.
- M90 physical implementations of the M60-owned source authority/unit-of-work/raw-snapshot/owner-lease ports.

The accepted `source-retrieval/v0` `RetrievalTransportRequest`, `SourceTransportPort`, their exact accessors and the statement “there are no additional accessors or conversions” remain byte/API unchanged. The future R1 owning patchset must instead define the distinct `source-retrieval/v1` successor surface below and obtain semantic acceptance for that owning contract before Rust:

```rust
pub struct RetrievalTransportRequestV1 {
    attempt_id: RetrievalAttemptId,
    source_id: SourceId,
    authority_revision: SourceAuthorityRevision,
    canonical_host: RetrievalDnsName,
    serialized_request: SerializedRetrievalRequest,
    expected_media_type: SourceMediaType,
    maximum_response_bytes: u32,
    maximum_elapsed_seconds: u32,
    protocol_version: SourceRetrievalProtocolVersionV2,
    public_ip_policy_version: PublicIpPolicyVersion,
}

pub fn RetrievalTransportRequestV1::attempt_id(&self) -> &RetrievalAttemptId
pub fn RetrievalTransportRequestV1::source_id(&self) -> &SourceId
pub fn RetrievalTransportRequestV1::authority_revision(&self) -> SourceAuthorityRevision
pub fn RetrievalTransportRequestV1::canonical_host(&self) -> &RetrievalDnsName
pub fn RetrievalTransportRequestV1::canonical_host_text(&self) -> &str
pub fn RetrievalTransportRequestV1::serialized_request(&self) -> &SerializedRetrievalRequest
pub fn RetrievalTransportRequestV1::expected_media_type(&self) -> &SourceMediaType
pub fn RetrievalTransportRequestV1::maximum_response_bytes(&self) -> u32
pub fn RetrievalTransportRequestV1::maximum_elapsed_seconds(&self) -> u32
pub fn RetrievalTransportRequestV1::protocol_version(&self) -> &SourceRetrievalProtocolVersionV2
pub fn RetrievalTransportRequestV1::public_ip_policy_version(&self) -> &PublicIpPolicyVersion

pub trait SourceTransportPortV1: Send + Sync {
    fn transport<'a>(
        &'a self,
        request: RetrievalTransportRequestV1,
    ) -> Pin<Box<dyn Future<Output = Result<RetrievalTransportSuccess,
                                          SourceTransportError>> + Send + 'a>>;
}
```

`RetrievalTransportRequestV1` remains private-field, owned, `Debug + Eq`, not `Clone`/`Copy`/Serde/`Default`, and has a `pub(crate)` constructor used only by M60 inside platform-core. Its field names and non-authority meanings are the v0 set, except that `protocol_version` deliberately uses the two-variant `SourceRetrievalProtocolVersionV2` introduced by the same R1 `source-import/v2` owning patchset; M60 copies that exact transaction-current definition tag, with no conversion to or mutation of accepted one-variant `SourceRetrievalProtocolVersion`. `canonical_host_text` is the sole additional v1 accessor beyond the exact v0 accessor set. Its implementation and the v1 type must remain in the same owning `source_retrieval` module as `RetrievalDnsName`, where the retained private text is reachable; `RetrievalDnsName::as_str` remains module-private and `RetrievalDnsName` gains no public text accessor or conversion. The v1 structural checker requires the exact field/accessor/port declaration above, rejects an accessor or enum widening on the v0 type, rejects every extra v1 accessor/conversion/constructor, requires the two-variant protocol type and requires the M60 coordinator plus M90 adapter consumer to use only the v1 type/port.

M60 constructs `canonical_host` and the exact serialized Host line from the same transaction-current definition, independently recomputes the complete serialized-request bytes/digest and proves their typed host equality before minting `RetrievalTransportRequestV1`. M90 cannot select or add URL, host, path, User-Agent, header, redirect, retry, proxy, limit, rate, lifecycle or authority. It uses `canonical_host_text()` only for DNS lookup and TLS SNI, then sends M60's serialized bytes exactly; M90 neither parses/reconstructs/compares the Host line nor widens constructors, and TLS/HTTP client types do not cross the port.

Durable composition requires one transactional adapter boundary for source authority + attempt schedule/journal and one immutable deterministic snapshot adapter with read-back. Startup runs integrity checks before serving commands; corruption is terminal/unavailable, not empty state. A later adapter spike must compare memory, TLS roots, DNS/peer binding, proxy/redirect disablement, exact HTTP/1.1 bytes and cancellation/resource destruction. No dependency is selected by this proposal.

Diagnostics expose only IDs, byte count, digests, status/error class, timing class and redacted network metadata. Raw body never enters logs, repository, tests or product responses.

## 10. Acceptance projections and non-promotion

The accepted docs-first projection adds these exact planned rows without promotion:

```text
SRC-016 | source | atomic source retrieval admission/start uses durable dual command/attempt indexes, separate reservation/execution/snapshot-owner leases, denial-finalized M00 reservations, continuous runtime/startup reservation expiry, actual-start rate time, explicit ClockRegression, frozen multi-failure precedence, complete post-start result/replay algebra, attempt-unique owner IDs, ledger-backed drop/expired-lease fencing and startup dead-owner recovery, transaction-current source authority, at-most-once effect dispatch, exact stopped-resource proof, durable validated-body metadata, atomic transport-failure terminalization, deterministic closed raw-snapshot adapter algebra, pre-dispatch-only cancellation with uncertain/post-put reconciliation, claimed snapshot recovery before raw-store inspection, owner-validated failure decisions, startup recovery before admission, atomic snapshot-success completion and corrupt-replay rejection against deterministic fakes | future source_retrieval_admission integration tests | pr | planned | backend
SRC-017 | source | ustc-teach-calendar-fall-2026 binds exact URL, exact source-import/v2 definition-embedded SourceUsePolicyV1/revision/digest, bounded-use/rate/retention evidence, identified request bytes without claiming unproved v0 causality, no rate override before reservation, synthetic parser fixture plus exact-source oracle conformance, durable RequestAdmitted read-back, admitted immutable evidence-bundle record before M00 operator-only approval, repository-computed definition digest and zero wholesale-raw publication before one admitted retrieval | future manual-security plus source-specific integration tests | core-demo | planned | security
```

Existing truth remains:

```text
SRC-001 implemented only at bounded B1 lifecycle scope
SRC-015 implemented only at bounded offline pure-B2 scope
SRC-010/SRC-011/SRC-012/SRC-014 remain planned/non-pass
M60 remains planned
```

The exact R1 owning-contract patchset must update, and be reviewed plus semantically accepted as one authority object, before Rust:

```text
docs/contracts/platform-request-context.md   // v1 approval + retrieval permission/effect pairs and denial mapping
docs/contracts/platform-control-evidence.md // request-admitted evidence gate/receipt binding
docs/contracts/permissions.md                // exact platform-operator capability/grant semantics
docs/contracts/interfaces.md                 // source approval/evidence/retrieval descriptors; no public route
docs/contracts/source-import.md              // distinct source-import/v2 successor; accepted v1 remains unchanged
docs/contracts/source-retrieval.md           // v1 wire + B3 ports/state machine
docs/contracts/module-boundaries.md          // M00/M10/M60/M90 ownership
docs/plan/05-campus-trust-kernel.md
docs/plan/modules/00-module-map.md           // update authoritative M60 contract-version registry row
docs/plan/modules/10-platform-control-identity.md
docs/plan/modules/70-campus-trust-source-pipeline.md
docs/tasks/01-execution-roadmap.md
docs/acceptance/platform-baseline.md
docs/acceptance/matrix.tsv                   // exact planned SRC-016/SRC-017 rows
```

The list above is the later R1 owning-contract patchset, not this proposal generation's changed-path scope or edit authority. It also corrects stale text describing implemented B1 code as future and broad M90 dependency wording that hides M60 semantic authority, and includes the authoritative `docs/plan/modules/00-module-map.md` M60 registry row. Before Develata's R1 semantic decision, the prepared candidate may label v2/v1 only as proposed and is not mergeable. After one exact R1 semantic-acceptance receipt, a marker-external status-only promotion must update the final merge candidate's owning contracts, acceptance carriers and module-map M60 row together so unchanged v1/v0 remain accepted compatibility surfaces and v2/v1 are labeled accepted/current successors under that exact receipt—not proposed. A focused promotion-delta review and complete R1 checker/read-back must pass before merge or R2/R3. R1's checker rejects omission, a final merged row that still says proposed, or any owner/module-map/current-version disagreement. This task file can authorize only preparation of that future R1 object; it never becomes sole or peer semantic authority. No acceptance status is promoted by this proposal, docs-only review or fake-backed tests.

## 11. Phase sequence

```text
R0. Develata gives or withholds non-authoritative direction to prepare one exact
    R1 owning-contract patchset from this proposal. R0 accepts no runtime/API/
    authority semantics and authorizes no Rust/network/source-status mutation.
R1. Prepare source-import/v2, source-retrieval/v1 and all docs/plan/contracts/
    acceptance projections as one exact proposed patchset, preserving source-import/v1;
    independently review it, then obtain Develata semantic acceptance of that exact
    owning object. Apply only a marker-external status promotion that projects the
    receipt into every owning status carrier and the module-map M60 row as
    accepted/current, run focused delta review plus complete R1 checks/read-back, and
    only then merge. Until that final promotion lands, current v0 owners remain
    authoritative and R2/R3 are blocked.
R2. offline source-retrieval/v1 serializer/checkers + exact synthetic parser fixture
    bytes/digest/oracle; retain mutations for missing/duplicate root, missing/duplicate/
    contradictory fields, invalid dates/week count, active/executable content,
    structural reordering and output-bound overflow; replay retained §2 bytes for
    parser conformance; no network/ports/approval/runtime.
R3. M10/M60 RequestAdmitted-gated source.approval-evidence.record + source.approve,
    durable authority/evidence/B3 ports and state algebra against fakes; no M90/live source.
R4. separately reviewed M90 transport/physical adapters with restart/read-back;
    source stays Proposed.
Gate before R5. Exact executable R1/R3/R4 tests must pass for: every adjacent
    first-failure pair in evidence-record, approval, admission and start precedence;
    evidence-record `EvidenceDispositionNotAccepted` versus `EvidenceTimestampInFuture`,
    `EvidenceTimestampInFuture` versus `ReviewerMismatch`, `ReviewerMismatch` versus
    `ParserBindingMismatch` and `ParserBindingMismatch` versus `CrossGeneration` are
    named required pairs; approval future timestamp versus reviewer mismatch is also named;
    equal/unequal command replay and every result/error/receipt presence rule;
    approval replay before evidence load; exact `transact_approval` result/error signature;
    prepare-plan concurrent/stale/final-replay paths; approval/record entry sealed-binding mismatch -> `RepositoryCorrupt` with zero ledger mutation;
    source missing/version/status/revision/definition
    rejection before evidence-store outage/corruption and outage before evidence-content
    rejection when source guards pass;
    reviewer mismatch persisted through the rejection ledger and every sealed operator/context/witness mismatch rejected as `RepositoryCorrupt` before any ledger lookup/mutation;
    RevisionExhausted only at transaction-current u64::MAX, including its two
    adjacent evidence-rejection/CAS-failure pairs;
    RequestAdmitted append/read-back failure; evidence-record M00 key/event
    persistence/equal replay/mismatch and `record_body_digest` exact M60 domain over expected/recomputed bundle digests, full sealed-witness replay mismatch rejection including exact `context.observed_at()` projection, record/approval declaration-order future timestamp rejection with no caller clock, and unchanged payload-omitting `RequestAdmitted`; source/grant/schema mismatch;
    retrieval `decide_admission` exact context/witness/command signature, M10
    RequestAdmitted append/read-back failure, pre-ledger sealed-binding mismatch,
    complete `RetrievalAdmissionEnvelopeV1` persistence, equal replay under the
    same authority/admission witness and unequal-witness replay conflict;
    admission/start replay without a clock, unseen admission/start trusted-clock
    failure to their exact `ClockUnavailable` errors before source/journal/slot
    mutation, and post-transport stopped-time `ClockUnavailable` preserving
    `Started` with no owner/journal/raw mutation or transport reissue;
    `SourceMissing` admission receipt keeps required `observed_at = trusted_now`
    while every optional source/rate observation is absent; absent required time or
    any optional source/time presence is rejected as incoherent;
    exact `Accepted | Rejected | Superseded` evidence-disposition decoding, only
    `Accepted` bundle insertion, both non-accepted tags reaching durable
    `EvidenceDispositionNotAccepted`, parser self-binding preimage/digest golden and
    one-axis mismatch, cross-generation against the bundle generation only, and no
    mutable current-generation/supersession index;
    accepted source-retrieval/v0 request/port/accessor/API fixtures unchanged;
    source-retrieval/v1 exact request accessor list, `canonical_host_text` value,
    both `SourceRetrievalProtocolVersionV2` tags carried exactly, v1 port signature
    and M60/M90 consumer compile; any v0 accessor addition, accepted one-variant protocol widening, public
    `RetrievalDnsName` text conversion and every extra v1 accessor/constructor reject;
    source-import/v1 byte/API fixtures unchanged, v1-as-v2 decode rejected,
    same-SourceId v1/v2 collision rejected as LegacyVersionOccupied, and the
    source-import/v2 generic field-by-field encoder plus calendar golden; one-axis
    mutations of source ID, owner, URL, authority, each numeric limit/media/protocol/
    public-IP field and each use-policy field must all change the digest and prevent
    equal replay; exact enum wire tags are used without automatic case conversion;
    source-ID precedence and canonical-URL uniqueness are exercised across v1/v1,
    v1/v2, v2/v1 and v2/v2 rows, with a different source ID on the same URL returning
    exact `CanonicalUrlOccupied` and mutating no row/index;
    exact R1 inclusion of `docs/plan/modules/00-module-map.md`; pre-decision
    candidate labels v2/v1 proposed and non-mergeable, while the post-decision final
    merge candidate labels unchanged v1/v0 accepted compatibility plus v2/v1
    accepted/current successors under the exact R1 receipt across module map, owning
    contracts and acceptance carriers; stale proposed or mixed status rejects;
    every SourceUsePolicyV1 enum/digest mismatch, unknown/no-default decode, caller
    override and invalid AttemptAndRateOverride authorization rejected before slot/
    Admitted/override consumption except the required terminal rejection tombstone;
    dropped-future attempt-unique owner advance-ledger success/equal-replay/conflict,
    non-expiry cancellation, crash-after-advance-before-cancel and proof that one
    attempt's fence cannot alter any sibling attempt;
    live one-second and startup expiration/read-back of every stranded Admitted
    reservation, dropped-plan recovery, start-first skip/expiry-first replay races,
    complete/partial/failed tick receipts and Failed optional-field matrix, failure after one acknowledged expiration,
    uncertain-current read-back followed by equal replay, startup
    registration, bounded owner-incarnation renewal cap, expired-owner
    fence/read-back, byte-equivalent fence replay after crash-before-journal,
    conflicting-fence rejection and startup/live Started reaping; live one-second
    `Started | TransportStopped` abandoned-work pages, exact complete/partial/failed
    progress/uncertainty carriers, panic/task-loss recovery and no transport reissue;
    start-time `InsufficientExecutionOwnerLeaseCoverage` receipt/presence/order and
    post-start `InsufficientOwnerLeaseCoverage` no-mutation mapping; every
    `OwnerLeaseErrorV1` variant and successful-short renewal maps to the exact closed
    post-start variant above; one-axis mutations prove error branches change no
    owner/journal/stopped/snapshot/raw state, while successful-short changes only the completed
    same-owner/same-epoch renewal CAS and changes no journal/stopped/snapshot/raw state;
    an active current lease renews successfully, `fence_expired_owner` before expiry returns
    `LeaseStillActive`, and an adversarial `renew_owner -> LeaseStillActive` maps to
    `RepositoryCorrupt` without mutation;
    transport-failure atomic terminalization; per-incarnation hard 50-second snapshot deadline,
    trusted-time fresh recovery deadline/owner-lease coverage, aggregate 45-second
    put-plus-read-back budget and post-put 25-second read/fence gate, owner-minted typed
    `InsufficientSnapshotLeaseBudget` boundary/equal-replay/malformed-decision rejection,
    checked-add overflow returning `ArithmeticOverflow` with no decision/raw/mutation,
    pre-dispatch snapshot cancellation versus first raw-call dispatch,
    exact snapshot authorization origin: initial origin alone may cancel before put,
    recovery origin rejects cancellation/put without mutation and must first read back;
    whole-packet sweep forbids every recovery-origin write/put/cancel license;
    recovery `Found` mismatch maps to `SnapshotEvidenceMismatch` with no
    write-receipt-dependent decision or terminal mutation, while exact `Found` succeeds
    and authoritative `Missing` alone mints `BodyUnavailableAfterFencedOwner`;
    prior-owner committed bytes followed by recovery cancellation cannot terminalize
    `Cancelled` and must reconcile to success or exact owner-validated failure;
    in-progress synchronous raw-call join with no terminal cancellation; a pending
    cancellation plus `Stored | AlreadySame` must continue through read-back and
    terminal reconciliation, `ExistingContentConflict` must take the exact failure
    path without read-back, and raw-adapter uncertainty must start no next call and
    preserve `TransportStopped`; snapshot put/read-back normal path;
    every closed snapshot put/read result/error, identity/content conflict, each
    owner-validated completion-failure variant, normal-writer versus recovery race,
    unexpired-lease Busy, expired-owner fenced claim, stale owner rejection,
    startup/live drain, crash or live task loss before/after raw put and before/after atomic success
    completion; corrupt journal/snapshot/replay; all deterministic
    fakes proving zero network effect.
    Missing, skipped, zero-case or non-biting mutation evidence blocks R5.
R5. durably execute the M00-admitted evidence-record operation, then `propose_exact_v2`
    the fresh v2 definition at revision 1, then execute M00-admitted approval with the same
    complete bundle and exactly one rate-compliant retrieval. If transport succeeds,
    raw put/read-back and atomic snapshot completion are mandatory. If transport fails,
    the terminal failure receipt closes only that attempt: no raw read-back, revision,
    baseline, publication or R6 promotion is claimed. A later success attempt requires
    a separately authorized command after the 21600-second interval; no automatic retry.
    The first successful path verifies the production User-Agent and exact v1 bytes.
R6. later parser revision/conflict/freshness/baseline/product composition.
```

No later phase proves an earlier missing gate. Manual `curl` reconnaissance is not M90 conformance, M60 admission, SourceRevision or product acceptance.

## 12. Proposal scope, gates and stops

This proposal generation changes exactly:

```text
docs/tasks/m60-calendar-source-activation-readiness.md
```

It changes no current plan/contract/acceptance/Rust/Cargo/CI/workflow/source-status/raw-evidence/config/deployment file. `scripts/check_repo_contracts.py` is a generic gate, not packet-specific evidence.

Pre-commit freeze requires exact base `54d758fbf2f1c08df2e1993919287569b501b115`, one staged path, mode `100644`, index blob equal to worktree hash, non-zero staged addition, zero unstaged/untracked paths, `git diff --cached --check`, marker byte/digest recomputation, no CR, citation-ledger evidence, evidence-manifest `11/11`, exact manifest-directory membership of manifest plus those `11` files, and the separate exclusion-receipt digest/mode check. An external controller receipt records literal commands/outcomes. Independent reviewers receive an immutable candidate copy, not the mutable producer checkout; unless a lane separately reads the listed restricted local paths, source-evidence hash/mode/existence rows are controller-verified attestations rather than bundled reviewer evidence. Semantic reviewers must label any such unperformed recomputation `NEEDS_VERIFICATION`, while the parent closeout replays it mechanically.

A scoped local commit is admitted only after mandatory Codex review and parent-reconciled final gates. Push, Draft PR, CI, merge and all authority/effect actions require separate operation-specific Develata authorization.

Stop before mutation when:

- source-owner evidence contradicts the bounded posture or source requires credentials/redirect/query/alternate host/compression/proxy/caller-controlled headers;
- v1 weakens a v0 security/framing/deadline invariant;
- parser proof requires wholesale raw HTML in Git;
- source approval cannot use exact M00 authenticated operator authority;
- M00 `RequestAdmitted` evidence cannot append/read back before either M60 mutation port, or denial cannot release an owned idempotency reservation;
- the immutable approval-evidence bundle cannot be created through its admitted append-once operation before approval;
- source authority/journal/snapshot transactions cannot preserve the state/replay matrix;
- M90 would own semantics or leak concrete adapter types;
- implementation needs any unaccepted contract, dependency, lifecycle, authority or acceptance change;
- `origin/main` moves before local commit or separately authorized shipping;
- another fetch would violate the `21600`-second interval.
<!-- M60_CALENDAR_SOURCE_ACTIVATION_PACKET:END -->

## R48 independent review receipt

```text
receipt_status: ISSUED_DELTA_BOUND
reviewed_object: marker-delimited semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
producer_head: e09347908869b227819af94767dbeab00e854124
packet_sha256: 9a7583db7104b3b8f500a87ff9eb5f7260157bab6fe776fc5fb7a0c8c77368b5
packet_bytes: 153508
candidate_sha256: 0dad9cd13bf0fa3328fe6138d030a5ba2893a64939d3ef97cd06c233d557ee05
candidate_bytes: 240402
index_tree: 05d51ea5cbef448a46fb7399180ade3711691219
index_blob: f154178c6a899f96fd88b7f193e3ac0706616a39
mandatory_reviewer: codex-reviewer
mandatory_provider: custom
mandatory_model: codex-auto-review
mandatory_session_id: 20260903_001950_a135fc
mandatory_terminal_verdict: PASS
mandatory_result_sha256: 9a8fc9ad83a4f4df0543079b4c562cce5f12a7d49fe81ab117e3742b01285a4f
independent_reviewer: deepseek-reviewer
independent_provider: deepseek
independent_model: deepseek-v4-flash
independent_session_id: 20260903_002103_f1b0b2
independent_terminal_verdict: PASS
independent_result_sha256: 36f2798b3cf7f858e98afd6ad5b801112415b82d84cda9b4507666ea56d725e7
review_completed_at: 2026-09-03T00:24:16+08:00
parent_identity_recomputation: PASS
receipt_delta_binding: PASS
receipt_delta_binding_session_id: 20260903_002747_95e27e
receipt_delta_binding_result_sha256: 754b3a464e29422c64747344c680ba949dc41caab72469328ede3e9dd26c7097
receipt_delta_binding_completed_at: 2026-09-03T00:28:52+08:00
```

R48 Codex and DeepSeek found no construction blocker. DeepSeek's recovery-clause self-containment and adapter-observation wording notes remain non-blocking clarity backlog because the normative origin/mapping/gate clauses are already closed and mechanically swept. This receipt is non-authoritative and grants no source, owning-contract, Rust, network, merge, deployment, release or publication authority.

## R44 independent review receipt

```text
receipt_status: ISSUED_DELTA_BOUND
reviewed_object: marker-delimited semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
producer_head: 471114f6b5897e32630d9f7f635fd00b4ca4bf2e
packet_sha256: c81a8246578724b54e0096521cfa0ec842ad0b47410279d0a3195a0f5f234a7d
packet_bytes: 148403
candidate_sha256: d45b37e9e640a6af6e4b46653749fb6b68a7ce7e4a1be604bc1669a262f1a6ad
candidate_bytes: 227122
index_tree: 7bcd7311cd57e27449d75520d58121eb8ada5c0b
index_blob: 01e16c2186ccf81ebf38f36d0f0945af64a14b6d
mandatory_reviewer: codex-reviewer
mandatory_provider: custom
mandatory_model: codex-auto-review
mandatory_session_id: 20260902_224323_41f058
mandatory_terminal_verdict: PASS
mandatory_result_sha256: 3c82950c75d910a4b86cc67bc6bedae5a5c412bca5a097f2d0092c33c77f0549
independent_reviewer: deepseek-reviewer
independent_provider: deepseek
independent_model: deepseek-v4-flash
independent_session_id: 20260902_224324_41e369
independent_terminal_verdict: PASS
independent_result_sha256: 824245c617b8e1bff92c49b0cf3562ff04ada97f66289f4fc65710f7bb11ac9d
review_completed_at: 2026-09-02T22:47:12+08:00
parent_identity_recomputation: PASS
receipt_delta_binding: PASS
receipt_delta_binding_session_id: 20260902_225245_e5e5a4
receipt_delta_binding_result_sha256: 49d2aefaa32304120ad190f8e21a8f9fefa11f001aca4c97718b0e6a81681cf0
receipt_delta_binding_completed_at: 2026-09-02T22:53:47+08:00
```

R44 Codex and DeepSeek found no construction blocker. DeepSeek's exact-wire-spelling wording and transition-row restatement notes are classified as non-blocking clarity backlog because the generic `wire_tag` encoder, literal golden and reason-specific receipt rule are already unambiguous. This receipt is non-authoritative and grants no source, owning-contract, Rust, network, merge, deployment, release or publication authority.

## R43 independent review receipt

```text
receipt_status: ISSUED_DELTA_BOUND
reviewed_object: marker-delimited semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
producer_head: 163fa950581cc212101565d1a52b7c9072c979d1
packet_sha256: 709caea57cbe45d055ff9121a4f022516cb52dfc25ce59c86a4f498bd03797bb
packet_bytes: 146386
candidate_sha256: 5f393128a421e98459c9fb61abb28440518ebae66641f590f37f8b6d85be241d
candidate_bytes: 220738
index_tree: 1754883648faac5fbf4868cce85caf20cb94203b
index_blob: 2b549c20931ae528c55ad8af8e592f671bedf99a
mandatory_reviewer: codex-reviewer
mandatory_provider: custom
mandatory_model: codex-auto-review
mandatory_session_id: 20260902_214219_005bf5
mandatory_terminal_verdict: PASS
mandatory_result_sha256: 7cb363ae9a33140c7b54a6fbcf33ee9c2f6cecf7855372954d6bef98c01c7515
independent_reviewer: deepseek-reviewer
independent_provider: deepseek
independent_model: deepseek-v4-flash
independent_substantive_session_id: 20260902_214703_6ac185
independent_substantive_explicit_verdict: PASS
independent_substantive_result_sha256: ec7b6e11581feabdbbe0e3406cebcf2f9413b7542f4c32cddeb4dd47d79f127f
independent_output_finalization_session_id: 20260902_215251_349285
independent_output_finalization_verdict: PASS
independent_output_finalization_result_sha256: e45c859b84ea24e30b76c475cecf1de7cce0efcbbae32ba03a231570f8b8a240
review_completed_at: 2026-09-02T21:54:51+08:00
parent_identity_recomputation: PASS
receipt_delta_binding: PASS
receipt_delta_binding_session_id: 20260902_215815_16cc7f
receipt_delta_binding_result_sha256: 5cec2fc7cdd59f231ff482bfc940e36a07a1f4f58c1a1766c43795045c20f878
receipt_delta_binding_completed_at: 2026-09-02T21:59:29+08:00
```

The R43 Codex and substantive DeepSeek blocker-only reviews found no construction blocker. DeepSeek's first result used the task prompt's plural `BLOCKERS:` header; the immutable substantive PASS was therefore followed by a read-only output-finalization run that satisfied the runner's singular-header schema. Its long-line and normalization remarks are non-blocking proof-carrier backlog under the product-first stop rule. This receipt records evidence only and grants no source, owning-contract, Rust, network, merge, deployment, release or publication authority.

## R42 independent review receipt

```text
receipt_status: ISSUED_DELTA_BOUND
reviewed_object: marker-delimited semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
producer_head: e82ea5562f057931401763fff0e1c23930b181c6
packet_sha256: fc19ad759e518bb848884c7a615db61cdc7ad890b936bb821b3292df398d884a
packet_bytes: 144664
candidate_sha256: a0e78477de6b39a20518a7ad53871f01d025075e208babacfb0502c3b51df094
candidate_bytes: 214830
index_tree: 36c0dce8ab5d05bc16380b2710301a824d55cdb8
index_blob: 58e27215216ac3d7b204957cc2ef6c3266fdb87b
mandatory_reviewer: codex-reviewer
mandatory_provider: custom
mandatory_model: codex-auto-review
mandatory_session_id: 20260902_204146_68cfd6
mandatory_terminal_verdict: PASS
mandatory_result_sha256: c6a61bb8fb80bff6a783de694ecef25abe48dd930f1ab08bd5a9f07586c74781
independent_reviewer: deepseek-reviewer
independent_provider: deepseek
independent_model: deepseek-v4-flash
independent_session_id: 20260902_204147_3ab7d5
independent_terminal_verdict: PASS
independent_result_sha256: 05c63e050b555275a9a987f798a815d4ed46cc7788feda304879460176c6c175
review_completed_at: 2026-09-02T20:49:06+08:00
parent_identity_recomputation: PASS
receipt_delta_binding: PASS
receipt_delta_binding_session_id: 20260902_205150_a3dda9
receipt_delta_binding_result_sha256: e78afc30726bbe4d115e004e0da64c09c98e0a385ed86c00de1e899b2752e75f
receipt_delta_binding_completed_at: 2026-09-02T20:53:16+08:00
```

The R42 Codex/DeepSeek blocker-only lanes found no construction blocker. DeepSeek's line-length readability should-fix and its two residual precision notes are classified as non-blocking proof-carrier/clarity backlog under the product-first stop rule: the complete semantics are present and parent mechanical identity/source checks passed. This marker-external receipt records review evidence only; it does not approve a source, accept the future R1 owning patchset, authorize Rust/network/live retrieval, promote an acceptance row, or authorize merge/deployment/release/publication.

## R41 independent review receipt

```text
receipt_schema: m60-source-activation-independent-review/v1
reviewed_candidate_generation: R41
reviewed_candidate_path: /opt/data/tmp/uca-m60-source-review-r41/candidate.md
reviewed_candidate_sha256: 6623a88f145f58f1c1e2245ca7417b1d33ca4f5fa0f57569840383f93bd015d1
reviewed_candidate_bytes: 205522
reviewed_packet_sha256: 074d6acf036e1886652cabf9a330e224dcf497d8bca458c5973c14037a4c45ae
reviewed_packet_bytes: 139512
reviewed_index_blob: 79a8d13aad912146c1021ba75a5745b43021b662
reviewed_index_tree: 01b08caccf999ac1708aba377eed72d096118fdd
reviewed_base_commit: 54d758fbf2f1c08df2e1993919287569b501b115
reviewed_base_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
codex_profile: codex-reviewer
codex_model: codex-auto-review
codex_verdict: PASS
codex_blocker: NONE
codex_result_sha256: ff0413779dd51582ec23202c05052c53cc2c1cfaf084bbcc2354102ec788d4a8
codex_usage_sha256: cbc663c9c36fa2767a8820fe6dd1dac699bc20b04f6ca7e5859d712ab2b113bd
deepseek_profile: deepseek-reviewer
deepseek_provider_model: deepseek/deepseek-v4-flash
deepseek_verdict: PASS
deepseek_blocker: NONE
deepseek_result_sha256: 933a85f2daf509edee45f009665b2b71986a1009af685a4d73dfc65eb032e90f
deepseek_summary_sha256: db217262c4b80390e6f855d37f75249bfa394e22e343bbc2cf3150900c1298cf
deepseek_usage_sha256: fb06a3dc9633cc13a11c18cc14dc3f438b87edc7769afb5b2a4fb8b1c7a74375
deepseek_route_verified: true
review_completed_at: 2026-09-02T19:53:46+08:00
receipt_delta_binding: PASS
receipt_delta_binding_old_sha256: 6623a88f145f58f1c1e2245ca7417b1d33ca4f5fa0f57569840383f93bd015d1
receipt_delta_binding_reviewed_sha256: ece5ad38a006b17d24545eb01ce2cac840667a3fedc33c8a44accbd0ab84c332
receipt_delta_binding_result_sha256: b5a153bb03a558fb5416e1cd7805ce83d91d58c2753d76f5365484f68498fbc0
receipt_delta_binding_usage_sha256: 05ffd66fdb86851f8f9b70b23e0744523d5e05dcb97a6e326f3314a274f73ffb
receipt_delta_binding_completed_at: 2026-09-02T19:58:46+08:00
proposal_only: true
source_approved: false
live_retrieval_or_network_effect: false
rust_implementation_authorized: false
merge_authorized_by_review: false
```

## Review ancestry

R0–R3 were proposal generations bound to base `bd67dd042f8cb3f32eccaad364fffa5fd76aba96`; none is current after `origin/main` advanced. Their still-valid findings were imported into R4: exact successful probe identity, canonical oracle bytes, source attribution calibration, staged candidate custody, source-retrieval v1 version closure, reservation/execution-lease separation, raw snapshot before success, durable source/journal/snapshot placement, canonical-host representability, approval replay/digest semantics, exact B3 transitions and acceptance projection closure.

R3 mandatory `codex-reviewer` was `BLOCKED` on approval-command specificity, B3/M90 construction detail and exact acceptance/projection rows. R4/R5 repaired successive construction findings. R6 then added explicit at-most-once lost-work semantics, durable Started/result metadata, generation-bound evidence records, exact permission/effect/schema/dispatcher/grant algebra and owner-private scoped-future stop receipts. Mandatory R6 Codex remained `BLOCKED` only on missing complete start result/error algebra and contradictory generic replay prose. R6 DeepSeek independently found an undisclosed third pre-policy ICS request/unbound sensitive-header custody plus an under-specified publication-time selector. R7 closed those findings; its mandatory Codex remained `BLOCKED` on an incomplete admission-rejection receipt/reason algebra and pre-authorization replay disclosure, while DeepSeek returned `PASS` with evidence-packaging and wording improvements. R8 closed both blockers and mandatory Codex returned `PASS`; its optional DeepSeek runner produced a complete advisory result beginning with prose rather than the required first-line verdict, so the wrapper correctly recorded `NO_VERDICT`. That advisory exposed a three-asterisk placeholder and lookup-order ambiguity. R9 attempted the repair, but focused mandatory Codex correctly found the placeholder still present in its immutable review copy. R10 removed every literal placeholder, but the reviewer lane's long-line reader abbreviated the inline contract identifier and therefore conservatively returned `BLOCKED`. R11 puts the exact identifier `platform-request-context/v0`, section 6 on a dedicated short line and leaves the accepted algorithm plus schema-identity/schema-digest extension unchanged. Focused mandatory Codex returned terminal `PASS` on exact R11 packet digest `bd637e3ab894a6ac1ac459bb15cf538919fd7557b5610be3d2c70d38713c1569`.

PR #66 GitHub Codex then found two source-backed blockers on committed R11: the proposed M90 call incorrectly received M60's `EffectReadyPlan`, and R0 would have accepted semantics before their owning contracts. R12 restores the accepted M60/M90 boundary by consuming the carrier inside M60 and passing only data-only `RetrievalTransportRequest`; it also makes R0 non-authoritative direction and requires the exact R1 owning-contract patchset to be independently reviewed and semantically accepted as one authority object before merge or Rust. The R11 receipt remains historical and does not approve R12. Focused mandatory Codex returned terminal `PASS` on exact R12 packet digest `801b549d2a1afa1a3585e017c1f9799386bf61445061981fc70b9239bd068c2e`.

Exact-head PR #66 GitHub Codex review of committed R12 then found seven construction blockers: missing durable `RequestAdmitted` evidence before M60 mutation, leaked M00 reservation capacity on post-reservation denial, no admitted immutable evidence-bundle creation path, missing `ClockRegression`, missing M00 blueprint projection, ambiguous multi-failure precedence and contradictory `propose_exact` signatures. R13 closes all seven with read-back-bound M00 causation witnesses, fenced rejection finalization, `source.approval-evidence.record`, complete clock outcome, exact R1 owner list, frozen rejection order and repository-computed one-argument `propose_exact`. R11/R12 receipts are historical only and do not approve R13. No independent terminal verdict exists for R13 yet.

Focused R13 mandatory review remained `BLOCKED` because the review lane's redacted long-line projection hid the inherited M00 stage; it also requested explicit finalizer-method/result binding and exact v0 clock semantics. R14 puts the inherited `platform-request-context/v0` §§5–6 three-step stage on a dedicated line, binds denial release to existing `AdmissionPorts::finalize_idempotency`, reproduces `source-retrieval/v0` §6.3 clock/option semantics, and adds cross-operation plus single-signature drift tables. R11–R13 receipts/verdicts remain historical only. No independent terminal verdict exists for R14 yet.

Focused R14 mandatory review remained `BLOCKED` because its reader again redacted one prose token and because `AlreadySame` lacked an explicit return/capacity branch. R15 labels the inherited stage only as A1/A2/A3 and states the exact `AlreadySame -> promote_persisted_prior -> PriorRejected` result, no second release, and no remaining owned reservation after the winning finalizer. All earlier substantive repairs remain unchanged. No independent terminal verdict exists for R15 yet.

Focused R15 mandatory review still rendered the first arrow in `A1/A2/A3` as a placeholder despite raw packet validation proving zero literal placeholders. R16 removes symbolic arrows from that order and spells it as “perform A1 first; perform A2 second; perform A3 third”; no semantic or finalizer change accompanies the renderer-safe wording. No independent terminal verdict exists for R16 yet.

Focused R16 mandatory review then redacted the `A1` label itself. R17 removes all A-prefixed labels and gives a plain numbered list: envelope hash, four-way reserve/retrieve, prior/in-flight classification. The following paragraph requires those numbered operations to complete in ascending order. No other semantic change accompanies this renderer-safe form. Focused mandatory Codex returned terminal `PASS` on exact R17 packet digest `b2de4e2f3bcb9446ef6c1862ed0878f7978d487f096e9f0f1d64b903e4685b76`.

Exact-head PR #66 GitHub Codex review of committed R17 found four further construction blockers: fieldless `SourceTransportError` could not supply required failure digests/timing, evidence-record validation had no closed rejection result, durable transport failure could strand slots after restart, and the Campus Trust owning plan was absent from R1. R18 makes failure metadata minimal and representable, atomically terminalizes durable failures with slot release, adds closed evidence-record rejection reasons, and adds `docs/plan/05-campus-trust-kernel.md` to the owner patchset. The R17 receipt remains historical and does not approve R18. Focused mandatory Codex returned terminal `PASS` on exact R18 packet digest `6696dfc2e82afd3b880c9811a60f04e8d02da0ee5c162edefcab7eb3738698d9`.

Exact-head PR #66 GitHub Codex review of committed R18 found four further construction blockers: future drop could strand slots in-process, approval operation ID had two spellings, post-start transitions lacked closed result/error receipts, and evidence-record rejection precedence was unordered. R19 adds M60-owned drop/epoch-fence/reap cancellation, uses exact `source.approve` and `source.approval-evidence.record` IDs, freezes complete post-start result/receipt/error mappings, and orders all evidence-record rejection reasons. The R18 receipt remains historical and does not approve R19. Focused mandatory Codex returned terminal `PASS` on exact R19 packet digest `cd32f614751ec31032c140a7eb9992e37f1c7ce4bd1b9cce44c3bedcdfef8079`; implementation mutation-test execution remains an explicit future R1/R3 gate, while parent packet-digest recomputation passed.

Exact-head PR #66 GitHub Codex review of committed R19 found six further construction blockers: no M00 retrieval-effect descriptor, rejected evidence-record/approval decisions could not reach their ledgers, early drop cancellation incorrectly reused lease-expiry reaping, recovery could strand `SnapshotStored`, and approval rejection precedence was unordered. R20 froze the retrieval descriptor/capability/consumer, passed owner-private validated-or-rejected decisions into both ledger transactions, added a distinct dropped-future cancellation transaction, atomically combined snapshot witness with `CompletedSuccess`, and ordered approval rejection guards. Mandatory R20 review of packet `f1cf09f772f9e8360a6b069db825d72c08c21f8213a535723b49a7e89c197a4e` remained `BLOCKED`: attempt-command API v0 versus outbound wire v1 was not distinguished, and the retrieval descriptor lacked literal canonical schema bytes/digest; it also requested explicit infrastructure-failure and pre-R5 executable-test behavior. R21 distinguishes those version axes, freezes the `1028`-byte schema and digest, classifies immutable-bundle infrastructure failures as non-terminal application errors, and expands the executable gate. The R19 receipt remains historical and does not approve R21.

A delayed three-lane R3 delegation then fanned in against superseded packet `e84580351ee07df2848a536e8a02d5813b368c7a5e9bd401ae16c8d8648c10e7`. It is advisory evidence, not an independent vote on current bytes. Exact-current classification found its reservation expiry, durable rejection, owner fencing, approval algebra, owner-projection sequence, durable unit-of-work and phase-separation blockers already superseded by R12–R20. R21 carries forward the still-reproducible findings: demote the unproved User-Agent causal claim; embed retention/publication/no-override policy in the revision/digest-bound definition; close raw-snapshot adapter results/identity/conflicts; state why post-stop recovery needs no execution lease; and require the complete parser/policy/snapshot mutation matrix. The R20 in-progress receipt is superseded and cannot approve R21.

Exact-head PR #66 GitHub Codex review of committed R21 found seven further construction blockers: admission/start ports omitted their declared error types; approval replay still depended on current evidence storage; snapshot completion failure lacked a typed cause; recovery could race the normal raw writer; reviewer mismatch had contradictory pre-ledger behavior; restart never advanced a crashed owner's epoch; and R5 required raw read-back even after transport failure. R22 makes admission/start return typed `Result`s, classifies the approval command ledger before evidence load, passes an owner-validated failure decision, adds a separately fenced snapshot-owner lease/handoff and startup dead-owner fencing, routes reviewer mismatch through the rejection ledger, and makes R5 raw read-back conditional on transport success. The R21 receipts remain historical and do not approve R22.

Mandatory Codex and optional DeepSeek both returned terminal `PASS` on exact R22 packet `8ddd6bb44b0a667a38717834ded538ec0ca5f12a5f30e573d19d3c33e5542e71`. DeepSeek identified two non-blocking but parent-confirmed construction ambiguities: owner renewal did not say whether it could extend the row snapshot deadline, and a crash after owner fencing but before journal transition could strand the row. R23 makes the snapshot deadline hard/non-renewable, bounds owner incarnations, gives the owner fence an append-once byte-replay ledger, adds the missing crash transition/test, and pins literal evidence-record schema plus adjacent precedence checks for R1. R22 receipts are historical and do not approve changed R23 bytes.

Mandatory R23 Codex returned `PASS`, while independent DeepSeek returned `BLOCKED` on a parent-reproduced epoch-algebra counterexample: a live `advance_epoch` had no append-once witness and a shared process owner could strand sibling or interrupted-cancellation rows. R24 uses an attempt-unique owner ID, replaces the bare advance with `advance_epoch_after_drop`, and makes drop/expired-lease advancement share one atomic append-once `OwnerFenceWitnessV1` ledger. Interrupted cancellation can now equal-replay its exact witness after lease expiry, while current-epoch advance without a matching ledger row remains corruption. R23 verdicts are historical and do not approve changed R24 bytes.

Mandatory R24 Codex and independent DeepSeek both returned `PASS` on exact packet `16e50c848ad222b4b6a04cbb70e0cb168320083728ea7a24d1a9618dbc045233`; the receipt-only delta also received mandatory `PASS`. Exact-head GitHub Codex then found three parent-confirmed construction blockers: evidence-repository outage bypassed the declared source-first approval rejection order, snapshot recovery inherited an expired hard deadline, and startup did not expire stranded `Admitted` reservations. R25 passes all evidence outcomes into the transaction so source guards win atomically, mints a fresh checked 30-second deadline on fenced snapshot recovery with owner-lease margin, and drains/read-backs every expired admitted reservation before reopening admission. It also adopts the two non-blocking R24 precision notes by removing the angle-bracket evidence-ID token and making startup advance-ledger lookup-first explicit. R24 receipts are historical and do not approve changed R25 bytes.

Mandatory R25 Codex and independent DeepSeek both returned `PASS` on exact packet `cb75d9709be3d6ee44089c6dd8d8bf79d06bc16fc03152637b33f2d53db83f83`; the receipt-only delta also received mandatory `PASS`. Exact-head GitHub Codex on `b1daadc46606f83015c83fec1b5313287d82c051` then found four parent-confirmed construction blockers: approval preparation lacked the complete command bindings it had to inspect; admitted reservations expired only on restart; snapshot cancellation could orphan already-committed raw bytes; and the incompatible successor definition/protocol shape reused accepted `source-import/v1`. R26 passes the complete `SourceApproveCommandV1` plus admission witness into preparation, adds a deterministic one-second live reservation-expiry coordinator, makes snapshot cancellation pre-dispatch-only while uncertain/post-put paths reconcile under recovery, and introduces a distinct `source-import/v2` definition/policy/protocol/repository row with no implicit v1 migration. R25 receipts are historical and do not approve changed R26 bytes.

Mandatory R26 Codex returned `BLOCKED` on one parent-confirmed representability gap: the no-dispatch insufficient snapshot-budget path still named removed generic `Explicit` failure despite the closed decision enum. Independent DeepSeek returned `PASS` on the same R26 packet and identified two non-blocking precision gaps in tick-failure wording and final approval-plan revalidation, plus three small R1 proof notes. R27 adds owner-minted `InsufficientSnapshotLeaseBudget` decision/reason/receipt presence and exact 25-second predicate, binds tick failures to existing repository/concurrency outcomes rather than an undeclared admission guard, closes sealed M00 plan revalidation, defines the `RevisionExhausted` trigger/precedence, and persists evidence-record M00 key/event bindings. R26 verdicts are historical and do not approve changed R27 bytes.

Mandatory R27 Codex returned `BLOCKED` on one parent-confirmed tick-algebra contradiction: a later row error claimed every row/slot stayed unchanged even after an earlier per-row expiry had already committed and read back. Independent DeepSeek returned `PASS` on the same R27 packet and identified non-blocking precision gaps for evidence-record body-digest identity, checked-add overflow and start-first tick handling. R28 makes tick execution explicitly stable-order partial progress with closed `Complete | Partial | Failed` carriers, preserves/reports acknowledged earlier commits, identifies an uncertain current attempt for idempotent replay, and never claims rollback. It also binds `record_body_digest` to the exact read-back M00 payload digest, maps checked-add overflow to `ArithmeticOverflow` with no mutation, and makes a start-first tick skip the now-`Started` row. R27 verdicts are historical and do not approve changed R28 bytes.

Mandatory R28 Codex returned `PASS`; independent DeepSeek returned `BLOCKED` on one parent-confirmed representability gap: accepted `RequestAdmitted` intentionally omits payload digest, while R28 required `record_body_digest` from that event but neither the closed witness nor another M60 input carried it. DeepSeek also identified a non-blocking first-row-uncertainty discriminator mismatch and an undeclared future `revise` reference. R29 adds `payload_digest` to owner-private `VerifiedRequestAdmissionEvidence`, minted from the schema-admitted `PlatformRequestContext` only after the byte-equal payload-omitting event read-back; M60 requires exact context/witness equality and passes that digest to `transact_record`. R29 defines tick prefix count as fully processed rows so first-row uncertainty is `Failed`, and explicitly keeps v2 revise out of scope. R28 verdicts are historical and do not approve changed R29 bytes.

Mandatory R29 Codex and independent DeepSeek both returned `PASS`. DeepSeek identified two non-blocking but implementation-relevant signature/outcome gaps plus three small carrier-presence notes. R30 adds the exact `transact_approval -> Result<SourceApproveResultV1, SourceApprovalApplicationError>` signature, maps approval/evidence-record sealed-binding mismatch to `RepositoryCorrupt` with zero ledger mutation, persists request payload digest in approval plan/receipt/repository binding, closes `Failed` tick optional-field presence, and source-verifies the existing sealed context payload accessor without widening `RequestAdmitted`. R29 verdicts are historical and do not approve changed R30 bytes.

Mandatory R30 Codex returned `BLOCKED` because approval replay could bypass sealed-binding mismatch validation. Independent DeepSeek returned `BLOCKED` after direct source read-back disproved R30's claimed sealed-context payload accessor; it also identified missing closed evidence-record/load carriers and legacy-version projection precision. R31 validates context/witness bindings before any ledger lookup, passes the complete private witness into replay classification/record transactions, and compares retained sealed bindings before replay. It removes the nonexistent payload carrier, defines an M60-owned canonical record-command digest over expected/recomputed bundle digests, adds closed evidence-record terminal/load algebras, and pins v1/v2 version projection plus parser-reviewer independence. R29/R30 payload PASS attestations are explicitly superseded. R30 verdicts are historical and do not approve changed R31 bytes.

Mandatory R31 Codex returned `PASS`. The optional DeepSeek wrapper recorded `NO_VERDICT` because the result placed explanatory prose before its explicit line-three `VERDICT: PASS`; that schema-invalid body remains advisory, not an independent vote. Parent read-back confirmed its two precision findings and one adjacent representability gap: record-side reviewer/parser triggers were ambiguous, receipt-reviewer mismatch could be read as preceding bundle presence/content, and `OperatorAdmissionMismatch` required grant lineage absent from the typed bundle. R32 defines the record-side first-failure predicates, delays reviewer mismatch until the preceding bundle checks pass, removes the unreachable operator domain variant in favor of the already sealed `RepositoryCorrupt` entry error, and preserves the accepted v0 envelope preimage rather than appending pre-descriptor schema facts. R31 verdicts are historical and do not approve changed R32 bytes.

Mandatory R32 Codex returned `PASS`. The optional DeepSeek wrapper again recorded `NO_VERDICT` because its otherwise explicit `VERDICT: PASS` appeared on line three rather than byte zero; its two `SHOULD_FIX` findings remain advisory. Parent read-back confirmed both and found the stronger adjacent contradiction: `SourceEvidenceBindingV1.disposition` was a singleton `Accepted` field while the record algebra required reachable `EvidenceDispositionNotAccepted`. R33 introduces the closed `Accepted | Rejected | Superseded` input enum, stores only `Accepted`, defines the exact canonical parser self-binding digest/comparator, removes any undeclared retained-generation or mutable supersession-store semantics, and binds the corresponding R1 golden/mutation gates. R32 verdicts are historical and do not approve changed R33 bytes.

R33 mandatory Codex and heterogeneous DeepSeek each returned exact-object `PASS` with `BLOCKER: NONE`. Parent read-back accepted DeepSeek's three precision notes as non-blocking future R1 proof-carrier hardening: literal disposition wire tags, recursive bundle-field flattening prose and explicit per-operation precedence asymmetry. Under the product-first terminal stop rule they do not justify another semantic proposal generation because no current contract, API, authority, replay or representability failure was found. R33 remains proposal-only and not semantically accepted.

## R33 independent review receipt

```text
receipt_schema: m60-source-proposal-independent-review/v1
issued_at: 2026-09-02T16:34:23+08:00
review_stage: POST_EDIT_STAGED_CANDIDATE
candidate_generation: R33
candidate_sha256: b1324eaab377391f2f6137ca0286c5febc6c4197c9e82d20e233acbcdccb8515
candidate_bytes: 169988
packet_sha256: 434406e0b00a0d43224ff27c1cf2277a0a51dd751a44c4e83fd1bce11bfc8b0a
packet_bytes: 121197
candidate_manifest_sha256: d656c80c403fd243f67e6bb3d5c6eafb95b4d7157f3836186824362601822347
review_prompt_sha256: 35eeb557ffaf4fc10922972a3dcc4163f35fb1272c4e1c835e33c175ab6f61e2
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
staged_tree: ad6b62e7eb33f9aa47999abf5b3fbb2521177bd9
staged_blob: de05673278c893c0520923e7b96ef2ee8c90257a
mandatory_codex_verdict: PASS
mandatory_codex_blocker: NONE
mandatory_codex_result_sha256: 5c5dbb68d833c88042fde8835703392dbfe6d36b0edca92ffba8ab73161883a3
mandatory_codex_usage_sha256: d9011b7c8fd5d55a641fddae601230d4ac7f80430da80b117664deee8b28dfe2
mandatory_codex_provider_model: custom/codex-auto-review
heterogeneous_deepseek_verdict: PASS
heterogeneous_deepseek_blocker: NONE
heterogeneous_deepseek_result_sha256: 1b494a59ff518eacf2ac4fb508652572e34bda20e56f3d1851d7e14388da5e29
heterogeneous_deepseek_summary_sha256: 45b20e2a2fcfed368247abbec8a792e1d6b44c4354295a8147182483cecdb5cb
heterogeneous_deepseek_usage_sha256: 163b738f5ba6138ecff507421127ba62be1cd553e5b9e4066078adb2b7931a16
heterogeneous_deepseek_provider_model: deepseek/deepseek-v4-flash
parent_mechanical_readback: PASS
semantic_packet_delta_after_review: NONE
receipt_delta_binding: PASS
receipt_delta_result_sha256: 9678b6367a4cb9206d4d637b68bbd9abd2ec6062cc4225fd03ee72969a20d277
receipt_delta_usage_sha256: 92465847acd22617e993f7669b531651a33208b88af2d9fe1cc36f1bcbe0f1ef
receipt_delta_carry_forward: YES
authority_effect: NONE
```

This marker-external receipt records review evidence only. It does not approve the source, accept the future R1 owning-contract patchset, authorize Rust implementation, permit DNS/socket/HTTP/live retrieval, promote any acceptance row, or authorize merge/deployment/release/publication.

Exact-head GitHub Codex on committed R33 found five parent-confirmed construction blockers: no live `Started` reaper, unrepresented owner-lease coverage failures, undeclared M00 retrieval permission/effect variants, an impossible two-call snapshot lease budget, and future-dated evidence accepted without a trusted admission-time bound. R34 added a one-second live `Started | TransportStopped` recovery tick, closed start/post-start lease-coverage outcomes, exact `platform-request-context/v1` variants/serde/coherence/denial/consumer rules, a 50-second lease with aggregate 45-second put/read/fence budget, and sealed record/approval future-timestamp rejection. The superseded R33 exact-head CI/review state does not approve changed R34 bytes.

Mandatory R34 Codex then returned `BLOCKED` on a source-backed representability gap: the candidate invoked `RetrievalTransportRequest::canonical_host_text`, but accepted `source-retrieval/v0` exposes only `canonical_host() -> &RetrievalDnsName`, says no additional accessors/conversions exist, and `RetrievalDnsName::as_str` is module-private. Independent DeepSeek returned `PASS` but identified three parent-confirmed precision gaps: the canonical approval-order sentence omitted `EvidenceTimestampInFuture`, synchronous raw-snapshot port methods were described as futures, and post-transport owner renewal was implicit. R35 preserves v0 byte/API exactness, proposes an explicit `source-retrieval/v1` `RetrievalTransportRequestV1`/`SourceTransportPortV1` successor with the sole host-text accessor and checker/consumer gates, restores the complete approval order, aligns raw-workflow/cancellation prose with synchronous adapter calls, and makes the bounded renewal step explicit. R34 verdicts are historical and do not approve changed R35 bytes.

Mandatory R35 Codex returned `PASS`; independent DeepSeek returned `BLOCKED` on a parent-confirmed successor-type contradiction: `RetrievalTransportRequestV1.protocol_version` still used accepted one-variant `SourceRetrievalProtocolVersion`, so it could not carry this source's identified v1 protocol selected by `SourceDefinitionV2`. DeepSeek also identified two precision gaps: renewal error mapping remained generic and M90 Host equality wording implied forbidden wire parsing. R36 uses the two-variant `SourceRetrievalProtocolVersionV2` field/accessor while preserving the accepted enum, freezes exact mapping of renewal failure to closed post-start errors with zero mutation, and moves typed Host equality validation to M60 before transport so M90 only uses host text for DNS/SNI and sends exact bytes. R35 verdicts are historical and do not approve changed R36 bytes.

Mandatory R36 Codex returned `BLOCKED` on one parent-confirmed closed-algebra typo: the renewal paragraph mapped owner-state unavailability to undeclared `SourcePostStartErrorV1::OwnerStateUnavailable`, while the declared post-start algebra uses `RepositoryUnavailable`. The R36 DeepSeek lane returned `NO_VERDICT` before reading the candidate because the first-party provider responded HTTP 402 Insufficient Balance; this is review infrastructure, not semantic evidence. R37 maps the branch exactly to `SourcePostStartErrorV1::RepositoryUnavailable` and changes no other semantic carrier. R36 verdicts do not approve changed R37 bytes.

Mandatory R37 Codex returned `BLOCKED` because the renewal paragraph still mapped only a subset of the closed generic `OwnerLeaseErrorV1` algebra. R38 maps short/active/span cases to `InsufficientOwnerLeaseCoverage`, stale epoch to `OwnerEpochStale`, unavailable/overflow/commit to their same-named post-start variants, and impossible internal owner identity/reuse/corruption cases to `RepositoryCorrupt`; every error or short-success branch was described as zero-mutation. Mandatory R38 Codex then found the sole omitted declared variant, `ConflictingFenceReplay`; R39 added it to the `RepositoryCorrupt` bucket. Mandatory R39 Codex and the recharged independent DeepSeek lane both returned `PASS`, but DeepSeek's detailed result exposed a parent-confirmed contradiction in the R39 verification wording: a successful-but-short renewal legitimately commits its same-owner/same-epoch lease CAS and therefore cannot satisfy a blanket no-owner-change assertion. R40 split error-zero from successful-short exact-CAS mutation semantics. Mandatory R40 Codex returned `PASS`; independent DeepSeek also returned `PASS` but identified a parent-confirmed method reachability gap: `LeaseStillActive` had no exact trigger and, if interpreted as rejecting ordinary active-lease renewal, could make the successful snapshot path dead. R41 reserves `LeaseStillActive` for pre-expiry `fence_expired_owner`, makes active current leases renewable, maps an impossible `renew_owner -> LeaseStillActive` result to `RepositoryCorrupt`, and adds non-vacuous method-specific tests. R37-R40 verdicts do not approve changed R41 bytes.

Mandatory R41 Codex and independent DeepSeek both returned `PASS`; the marker-external receipt and finalization deltas also returned mandatory `PASS`, and committed head `e82ea5562f057931401763fff0e1c23930b181c6` passed exact-head CI/governance. Exact-head GitHub Codex then found three parent-confirmed proposal-construction blockers: retrieval admission had no sealed context/admission-witness parameter despite persisting operator/grant/policy authority, the complete phase error algebras omitted trusted-clock unavailability, and the exact R1 authority patchset omitted the authoritative module-map M60 version registry. R42 passes `&PlatformRequestContext` plus full `VerifiedRequestAdmissionEvidence` into retrieval admission and persists an owner-private `RetrievalAdmissionEnvelopeV1`, adds replay-first zero-mutation `ClockUnavailable` errors for admission/start/post-transport time acquisition, and includes `docs/plan/modules/00-module-map.md` with exact version-projection/checker gates. R41 reviews, receipts, CI and resolved threads are historical and do not approve changed R42 bytes. R42 later received exact candidate PASS verdicts from mandatory Codex and independent DeepSeek plus marker-external receipt/delta binding; the issued R42 receipt above is the authoritative review carrier.

Committed R42 head `163fa950581cc212101565d1a52b7c9072c979d1` passed exact-head CI/governance and all prior threads were resolved. Exact-head GitHub Codex then found two parent-confirmed construction blockers: a pending cancellation after `put_if_absent` returned `Stored | AlreadySame` was simultaneously told to stop before read-back and to complete mandatory reconciliation, and the final R1 module-map checker still required successor versions to remain proposed after their exact semantic-acceptance gate. R43 makes pending-cancellation result handling closed—committed/equal put continues through read-back/terminalization, conflict takes its exact failure path, and uncertain raw error leaves `TransportStopped`—and splits R1 pre-decision proposed status from the post-receipt final merge candidate where every owning carrier labels v2/v1 accepted/current. R42 reviews, receipts, CI and GitHub thread closure are historical and do not approve changed R43 bytes. R43 then received exact candidate PASS verdicts from mandatory Codex and substantive independent DeepSeek; the issued R43 receipt above carries those identities and remains non-authoritative.

Committed R43 head `471114f6b5897e32630d9f7f635fd00b4ca4bf2e` passed exact-head CI/governance and all prior threads were resolved. Exact-head GitHub Codex then found two parent-confirmed construction blockers: the v2 definition preimage substituted this calendar candidate's literals for generic owner/authority/policy fields, permitting unequal definitions to hash identically, and `SourceMissing` simultaneously required non-optional `observed_at` while forbidding all time observations after trusted-clock acquisition. R44 defines one generic field-by-field `SourceDefinitionV2` encoder and retains this source only as its independent golden vector with one-axis mutation obligations; it also requires `SourceMissing.observed_at = trusted_now` while every optional source/rate observation is absent. R43 reviews, receipts, CI and GitHub thread closure are historical and do not approve changed R44 bytes. R44 then received exact candidate PASS verdicts from mandatory Codex and independent DeepSeek; the issued receipt above carries those identities and remains non-authoritative.

Committed R44 head `e09347908869b227819af94767dbeab00e854124` passed exact-head CI/governance and all prior threads were resolved. Exact-head GitHub Codex then found three parent-confirmed construction blockers: recovery authorization could take the initial pre-put cancellation path and orphan prior-owner committed bytes; v2 proposal omitted accepted cross-version canonical-URL uniqueness; and generic digest prose incorrectly required snake_case for exact PascalCase protocol/public-IP wire tags. R45 adds a closed initial-versus-recovery authorization origin, forbids recovery cancel/put before authoritative read-back, preserves one canonical-URL index across v1/v2 with exact collision precedence/outcome, and freezes exact enum wire tags with no automatic case conversion. R44 reviews, receipts, CI and GitHub thread closure are historical and do not approve changed R45 bytes.

Mandatory R45 Codex returned `PASS`; independent DeepSeek returned `BLOCKED` on one parent-confirmed residual contradiction in the live abandonment tick, which still licensed recovery-origin raw `put_if_absent` "as needed" despite R45's no-recovery-put invariant. R46 replaces that step with authoritative read-back reconciliation only and explicitly repeats that `put_if_absent` is forbidden under `RecoveryAfterFence`; no other semantic carrier changes. R45 verdicts do not approve changed R46 bytes.

Mandatory R46 Codex returned `PASS`; independent DeepSeek returned `BLOCKED` on one remaining whole-packet recovery-write license in §7 and identified a parent-confirmed incomplete recovery `Found`-mismatch mapping that could require an unavailable write receipt. R47 scopes write-receipt failure decisions to `InitialAfterTransport`, restricts `RecoveryAfterFence` to authoritative read-back and terminalization everywhere, maps recovery `Found` mismatch to non-terminal `SnapshotEvidenceMismatch`, and adds a whole-packet no-recovery-write/cancel sweep. R46 verdicts do not approve changed R47 bytes.

Mandatory R47 Codex and independent DeepSeek both returned `BLOCKED` on the same final crash-boundary residue, which still mapped recovery read conflict/mismatch to a write-receipt-dependent failure decision. R48 mirrors the closed recovery algebra there: exact `Found` succeeds; `Missing` alone mints `BodyUnavailableAfterFencedOwner`; `Found` mismatch returns non-terminal `SnapshotEvidenceMismatch`; corrupt/impossible observations are `RepositoryCorrupt`. R47 verdicts do not approve changed R48 bytes. R48 then received exact candidate PASS verdicts from mandatory Codex and independent DeepSeek; the issued receipt above carries those identities and remains non-authoritative.

## R11 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
packet_sha256: bd637e3ab894a6ac1ac459bb15cf538919fd7557b5610be3d2c70d38713c1569
packet_bytes: 49282
mandatory_reviewer: codex-reviewer
provider: custom
model: codex-auto-review
review_session_id: 20260902_042841_aacb01
review_completed_at_utc: 2026-09-01T20:29:23Z
terminal_verdict: PASS
blockers: none
should_fix: none
needs_verification: none
optional_deepseek_r8: NO_VERDICT (runner schema rejected result whose first line was prose); advisory findings were repaired before R11
parent_postflight: packet digest/scope/marker/no-placeholder/base/currentness/evidence-manifest/exclusion-receipt/oracle/goldens PASS
authority_effect: proposal reviewed only; semantic acceptance, source approval, source activation, implementation, push, merge and network effect remain separate gates
```

## R12 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R12 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
packet_sha256: 801b549d2a1afa1a3585e017c1f9799386bf61445061981fc70b9239bd068c2e
packet_bytes: 50100
mandatory_reviewer: codex-reviewer
provider: custom
model: codex-auto-review
review_session_id: 20260902_044224_558d7d
review_completed_at_utc: 2026-09-01T20:43:34Z
terminal_verdict: PASS
blockers: none
should_fix: none
needs_verification: none
review_scope: PR #66 M60/M90 carrier ownership and R0/R1 authority sequencing repair
parent_postflight: packet digest/scope/marker/no-placeholder/source-contract/currentness/evidence-manifest/exclusion-receipt/oracle/goldens PASS
authority_effect: reviewed proposal only; R0 is non-authoritative direction; accepted semantics require a separately reviewed and accepted R1 owner patchset
```

## R17 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R17 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
packet_sha256: b2de4e2f3bcb9446ef6c1862ed0878f7978d487f096e9f0f1d64b903e4685b76
packet_bytes: 58161
mandatory_reviewer: codex-reviewer
provider: custom
model: codex-auto-review
review_session_id: 20260902_052412_9e1d06
review_completed_at_utc: 2026-09-01T21:24:56Z
terminal_verdict: PASS
blockers: none
should_fix: none
needs_verification: none
review_scope: exact-head PR findings plus renderer-safe inherited M00 order/finalizer closure
parent_postflight: packet digest/scope/no-placeholder/source-readback/evidence-manifest/exclusion-receipt/oracle/goldens PASS
authority_effect: reviewed proposal only; R0 remains non-authoritative direction and R1 owner-patch acceptance is separate; no accepted contract, source approval/activation, implementation or network effect is granted
```

## R18 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R18 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
packet_sha256: 6696dfc2e82afd3b880c9811a60f04e8d02da0ee5c162edefcab7eb3738698d9
packet_bytes: 59901
mandatory_reviewer: codex-reviewer
provider: custom
model: codex-auto-review
review_session_id: 20260902_054450_9de57a
review_completed_at_utc: 2026-09-01T21:45:24Z
terminal_verdict: PASS
blockers: none
should_fix: none
needs_verification: none
review_scope: exact-head transport-failure/evidence-record/Campus-Trust owner repair
parent_postflight: packet digest/scope/no-placeholder/source-readback/evidence-manifest/exclusion-receipt/oracle/goldens PASS
authority_effect: reviewed proposal only; R0 remains non-authoritative direction; no accepted contract, source approval/activation, implementation or network effect is granted
```

## R19 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R19 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
packet_sha256: cd32f614751ec31032c140a7eb9992e37f1c7ce4bd1b9cce44c3bedcdfef8079
packet_bytes: 64660
mandatory_reviewer: codex-reviewer
provider: custom
model: codex-auto-review
review_session_id: 20260902_060410_0a5fb0
review_completed_at_utc: 2026-09-01T22:05:23Z
terminal_verdict: PASS
blockers: none
should_fix: none
needs_verification: future R1/R3 mutation-test execution; parent packet-digest recomputation PASS
review_scope: exact-head live-drop/operation-ID/post-start/evidence-precedence repair
parent_postflight: packet digest/scope/no-placeholder/source-readback/evidence-manifest/exclusion-receipt/oracle/goldens PASS
authority_effect: reviewed proposal only; R0 remains non-authoritative direction; no accepted contract, source approval/activation, implementation or network effect is granted
```

## R21 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R21 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
packet_sha256: ab93adbac394d3963a0e12fa5a9d2249b90cb4b39b97c2aa458a0947f08a94a6
packet_bytes: 78061
candidate_sha256: d86413957b06e4bdb977c876e66d9d89e293915e00676c5e1f324f83a72dee70
candidate_manifest_sha256: 2299f9852f8acefabd9e2b8c3454cba031401eaceadbe80d898ee49b9720e6a2
mandatory_reviewer_profile: codex-reviewer
mandatory_reviewer_route: custom/codex-auto-review
mandatory_review_session_id: 20260902_064717_2750d9
mandatory_review_result_sha256: 2d21aacb6a472d6a7931fc9849bab6d33f0b288ad6e3893e9bbae2e2dd7e787f
mandatory_terminal_verdict: PASS
mandatory_blockers: none
mandatory_should_fix: none
optional_reviewer_profile: deepseek-reviewer
optional_reviewer_route: deepseek/deepseek-v4-flash
optional_review_session_id: 20260902_064812_f1691d
optional_review_result_sha256: 73fe1cf2bbd71e3cb554dda44dba95a4ff0bcf4c6d95fc7c60ae39ae6aefaf83
optional_terminal_verdict: PASS
optional_blockers: none
parent_mechanical_recomputation: PASS (packet 78061/ab93adba; retrieval schema 1028/5320abe8; definition preimage 436/c2025139)
review_identity_note: mandatory NEEDS_VERIFICATION prose transposed characters in the displayed definition hash; it is not used as an identity carrier, while the review prompt, candidate and parent recomputation bind the exact c20251390524990f05c27c6ad1aed84c0e4c48a71a639ffbacdcbf139b908396 value
receipt_issued_at_utc: 2026-09-01T22:51:24Z
authority_effect: reviewed proposal only; R0 remains non-authoritative direction; no accepted contract, source approval/activation, Rust implementation, push/merge or network effect is granted
```

## R24 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R24 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
packet_sha256: 16e50c848ad222b4b6a04cbb70e0cb168320083728ea7a24d1a9618dbc045233
packet_bytes: 93837
candidate_sha256: 86311964106a356ea324398b21550404fa3b6ab28f142cd6a710bc72b1e7d7c7
candidate_manifest_sha256: 24da34ec7fc9e0c6f5ad4fc2070d8e0eb26c128a62538d0e48e99e06f1c9c101
mandatory_reviewer_profile: codex-reviewer
mandatory_reviewer_route: custom/codex-auto-review
mandatory_review_session_id: 20260902_121208_d6a460
mandatory_review_result_sha256: 8534489b173784a20885ad49e50b2f492c0fa2d6f798543a8da2ea95e433d4be
mandatory_terminal_verdict: PASS
mandatory_blockers: none
mandatory_should_fix: none
optional_reviewer_profile: deepseek-reviewer
optional_reviewer_route: deepseek/deepseek-v4-flash
optional_review_session_id: 20260902_121208_821bd6
optional_review_result_sha256: 0071e6aae6046a64e2e10b965c86f814890ca9ea5816e2c740f2867c9145e883
optional_terminal_verdict: PASS
optional_blockers: none
optional_should_fix: renderer-safe parser-fixture suffix wording and explicit startup advance-ledger lookup-first wording; non-blocking precision backlog for the R1 owning patchset, not current proposal construction blockers
parent_mechanical_recomputation: PASS (packet 93837/16e50c84; approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; definition preimage 436/c2025139; evidence manifest 11/11)
parent_counterexample_probe: PASS (attempt-unique owners; drop-advance replay; unledgered advance rejected)
receipt_issued_at_utc: 2026-09-02T04:15:33Z
authority_effect: reviewed proposal only; R0 remains non-authoritative direction; no accepted contract, source approval/activation, Rust implementation, push/merge or network effect is granted
```

## R25 independent review receipt

```text
receipt_status: ISSUED
reviewed_object: marker-delimited R25 semantic packet
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
packet_sha256: cb75d9709be3d6ee44089c6dd8d8bf79d06bc16fc03152637b33f2d53db83f83
packet_bytes: 99080
candidate_sha256: 6e250ef81daeb799e7a192cd1cdf1dd17e44b413627c1502e833687180bfa2b7
candidate_manifest_sha256: f6911d2ffa2f5989fd28c3554d41daa7752a9400f2a8932c7f61e5b8f111fce8
mandatory_reviewer_profile: codex-reviewer
mandatory_reviewer_route: custom/codex-auto-review
mandatory_review_session_id: 20260902_124945_98570d
mandatory_review_result_sha256: 7d73f373a5cf8f1f73f560c1e67f83d73f7762cf32d76b574d25f266ff3677a5
mandatory_terminal_verdict: PASS
mandatory_blockers: none
mandatory_should_fix: none
optional_reviewer_profile: deepseek-reviewer
optional_reviewer_route: deepseek/deepseek-v4-flash
optional_review_session_id: 20260902_124946_f4079c
optional_review_result_sha256: 5dd72fcf9b61659a27ee7bf4c49206d28b80e1ab98974863cf7cf01cbcff9e91
optional_terminal_verdict: PASS
optional_blockers: none
optional_should_fix: none
optional_nice_to_have: bind the snapshot fenceable owner-lease delay explicitly; bind stale/cross-bound recovery to a named closed outcome; define RevisionExhausted and its adjacent test pair; non-blocking R1 owning-patchset precision backlog
parent_mechanical_recomputation: PASS (packet 99080/cb75d970; approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; definition preimage 436/c2025139; evidence manifest 11/11)
parent_three_finding_replay: PASS (source-first preparation/final recheck; fresh bounded recovery incarnation; startup Admitted expiry/read-back)
receipt_issued_at_utc: 2026-09-02T04:53:15Z
authority_effect: reviewed proposal only; R0 remains non-authoritative direction; no accepted contract, source approval/activation, Rust implementation, push/merge or network effect is granted
```

## R21 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
packet_sha256: ab93adbac394d3963a0e12fa5a9d2249b90cb4b39b97c2aa458a0947f08a94a6
packet_bytes: 78061
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
citation_ledger_evidence: PASS
changed_path_scope: PASS (one staged path; zero unstaged/untracked paths)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
evidence_manifest_directory: EXACT (manifest + 11 entries; mode 0700/0600)
excluded_probe_receipt: 5/5 bound; sha256 da34d8908f4ceca1921585234333063489bb3c72d146a934b8062504b6ea156f; mode 0700/0600
oracle_display_vs_artifact: BYTE_IDENTICAL; 812 bytes; sha256 07ac00567dcfd7bd7b832c3120b7649205c3dadb5b2b8999df69f9eff6223c75
origin_main_currentness: PASS
parent_source_backing_readback: PASS (platform-request-context §§5–6/finalizer; platform-control-evidence §6; source-retrieval §6.3; M00/M60 blueprints; module boundary)
whole_file_sha256_and_index_blob: recorded externally in the immutable R21 candidate manifest to avoid self-reference
```

## R22 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b36bdc930aa62971865c2891d6a308b8d7e4ae50
packet_sha256: 8ddd6bb44b0a667a38717834ded538ec0ca5f12a5f30e573d19d3c33e5542e71
packet_bytes: 89739
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r22_seven_finding_repair_assertions: PASS
origin_main_currentness: PASS
parent_golden_recomputation: PASS (retrieval schema 1028/5320abe8; definition preimage 436/c2025139)
whole_file_sha256_and_index_blob: recorded externally in the immutable R22 candidate manifest to avoid self-reference
```

## R23 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b36bdc930aa62971865c2891d6a308b8d7e4ae50
packet_sha256: 69fe3e954c7985bcde4d206593d3f4855430a404687254d23fa2642b2587adb7
packet_bytes: 92059
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r23_hard_snapshot_deadline_and_fence_replay_assertions: PASS
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; definition preimage 436/c2025139)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R23 candidate manifest to avoid self-reference
```

## R24 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b36bdc930aa62971865c2891d6a308b8d7e4ae50
packet_sha256: 16e50c848ad222b4b6a04cbb70e0cb168320083728ea7a24d1a9618dbc045233
packet_bytes: 93837
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r24_interrupted_cancellation_counterexample_probe: PASS
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; definition preimage 436/c2025139)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R24 candidate manifest to avoid self-reference
```

## R25 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 7b24c61e838ab3c02477a33c09c4eb30962147bf
packet_sha256: cb75d9709be3d6ee44089c6dd8d8bf79d06bc16fc03152637b33f2d53db83f83
packet_bytes: 99080
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r25_source_first_outage_fresh_recovery_lease_startup_admitted_assertions: PASS
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; definition preimage 436/c2025139)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R25 candidate manifest to avoid self-reference
```

## R26 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: 96358774b7b9bb0e4a313c4dd83637690e428c60c9d70247219679b683a2accc
packet_bytes: 106269
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r26_complete_approval_bindings_live_expiry_post_put_reconciliation_v2_versioning_assertions: PASS
r26_transition_oracle: PASS (lost-plan live expiry; start/expiry serialization; pre-dispatch cancellation/post-put recovery; v1/v2 separation)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R26 candidate manifest to avoid self-reference
```

## R27 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: b595bd16453a5306d57c5ef361fafe2c59bba00543ce373eddbf2b466a640c7d
packet_bytes: 110222
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r27_insufficient_budget_tick_outcome_plan_revalidation_revision_record_bindings_assertions: PASS
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R27 candidate manifest to avoid self-reference
```

## R28 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: 7fa48d22d341c0ca79077307b4440aba88a828a6d72c068f25fd593766539b7a
packet_bytes: 112293
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r28_partial_progress_record_digest_overflow_start_skip_assertions: PASS
r28_partial_progress_oracle: PASS (acknowledged prior expiry + uncertain current row + idempotent retry; first-row uncertainty -> Failed)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R28 candidate manifest to avoid self-reference
```

## R29 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: ca12b0461e5354f4848e01a41c2f25d6e37bfeab2c0a65967a25c033c96efee5
packet_bytes: 113494
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r29_payload_digest_transport_tick_boundary_no_revise_assertions: FAIL/SUPERSEDED (sealed PlatformRequestContext has no payload accessor; R31 uses M60 canonical record-body digest)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R29 candidate manifest to avoid self-reference
```

## R30 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: 8af13675750cf6ef0e809ace129af92472a2551255c02e8fc4d4bc6cc9029544
packet_bytes: 114756
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r30_signature_outcome_payload_presence_assertions: FAIL/SUPERSEDED (payload carrier premise was false)
source_backing_readback: FAIL/SUPERSEDED (BuildRequestContextCommand carries payload_digest, but sealed PlatformRequestContext and RequestAdmitted omit it)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R30 candidate manifest to avoid self-reference
```

## R31 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: e4fc6591a2531840b7ce4ce3670bd7192e42eb4b76f546890b9eabb086692574
packet_bytes: 118286
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r31_replay_witness_record_digest_version_load_assertions: PASS
source_backing_readback: PASS (BuildRequestContextCommand carries payload_digest; sealed PlatformRequestContext and RequestAdmitted omit it)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R31 candidate manifest to avoid self-reference
```

## R32 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: 78022ea9a51b02668ffec1bb8484007c626faf89450747232e75af811c7126e2
packet_bytes: 119378
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r32_record_reviewer_precedence_operator_variant_envelope_assertions: PASS
source_backing_readback: PASS (BuildRequestContextCommand carries payload_digest; sealed PlatformRequestContext and RequestAdmitted omit it)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R32 candidate manifest to avoid self-reference
```

## R33 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: b1daadc46606f83015c83fec1b5313287d82c051
packet_sha256: 434406e0b00a0d43224ff27c1cf2277a0a51dd751a44c4e83fd1bce11bfc8b0a
packet_bytes: 121197
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r33_disposition_parser_binding_supersession_assertions: PASS
source_backing_readback: PASS (BuildRequestContextCommand carries payload_digest; sealed PlatformRequestContext and RequestAdmitted omit it)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R33 candidate manifest to avoid self-reference
```

## R34 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: 882daa376a61c21cd16f854a16e6aec8c5ea2d508b261ce053fe8e9b0077912c
packet_bytes: 131630
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r34_exact_head_five_blocker_closure_assertions: PASS
source_backing_readback: PASS (PlatformRequestContext exposes observed_at; SessionInstant exposes as_unix_millis; v0 omits privileged external variants and requires a successor)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R34 candidate manifest to avoid self-reference
```

## R35 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: f3da105115ca35b84115ec17abd1f7545df193895b56ebafee76adac251ac802
packet_bytes: 136450
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r35_transport_v1_raw_sync_renewal_order_assertions: PASS
source_backing_readback: PASS (v0 transport request exposes no host text; RetrievalDnsName::as_str is module-private; raw snapshot port methods are synchronous)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R35 candidate manifest to avoid self-reference
```

## R36 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: ca9dfae748ed7bf09b81c1681d89600e84782b20580a3b7ba3427f636e88639b
packet_bytes: 137545
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r36_successor_protocol_renewal_host_assertions: PASS
source_backing_readback: PASS (accepted SourceRetrievalProtocolVersion is one-variant; source-import/v2 successor is two-variant; M90 has no authority to reconstruct Host)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R36 candidate manifest to avoid self-reference
```

## R37 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: b7373e4a61cb355898bef133f7fc5023c34a31ee852f9246a77872a20da6a488
packet_bytes: 137582
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r37_closed_post_start_error_mapping_assertions: PASS
source_backing_readback: PASS (SourcePostStartErrorV1 declares RepositoryUnavailable, not OwnerStateUnavailable)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R37 candidate manifest to avoid self-reference
```

## R38 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: 73d7ec2a3fec64790dfda6f30df02fd88d1f1f6e37bd95ccb0ddebd7c5bce8ff
packet_bytes: 138233
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r38_exhaustive_owner_renewal_mapping_assertions: PASS
source_backing_readback: PASS (every OwnerLeaseErrorV1 variant maps to a declared SourcePostStartErrorV1 variant)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R38 candidate manifest to avoid self-reference
```

## R39 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: 1cdca1bbcc015a4ef23a8a1febc7bfded45a2b941ab2213072d0ef7ec73e4f77
packet_bytes: 138258
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r39_owner_renewal_conflicting_fence_mapping_assertions: PASS
source_backing_readback: PASS (ConflictingFenceReplay is included in the exhaustive generic OwnerLeaseErrorV1 mapping)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R39 candidate manifest to avoid self-reference
```

## R40 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: 1537dd7b07228693571f4175c72747c0a790086fe9ec135dc0a08548f82b44ae
packet_bytes: 138715
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r40_owner_renewal_mutation_scope_assertions: PASS
source_backing_readback: PASS (error renewal and successful-short renewal have distinct mutation semantics)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R40 candidate manifest to avoid self-reference
```

## R41 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 33a575b0f37931c18053a2795ed50871c809c63e
packet_sha256: 074d6acf036e1886652cabf9a330e224dcf497d8bca458c5973c14037a4c45ae
packet_bytes: 139512
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
generic_repo_contract_check: PASS (not packet-specific evidence)
staged_candidate_diff_check_against_bound_base: PASS
packet_marker_scope_mode_cr_check: PASS
r41_lease_still_active_trigger_assertions: PASS
source_backing_readback: PASS (LeaseStillActive trigger and renew_owner method mismatch are explicit and non-vacuous)
origin_main_currentness: PASS
parent_golden_recomputation: PASS (approval schema 1371/c76081e0; retrieval schema 1028/5320abe8; source-import/v2 definition preimage 436/101c2c89)
evidence_manifest: 11/11 PASS; sha256 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1; exact mode 0700/0600
whole_file_sha256_and_index_blob: recorded externally in the immutable R41 candidate manifest to avoid self-reference
```

## R42 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R42
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: e82ea5562f057931401763fff0e1c23930b181c6
semantic_packet_bytes: 144664
semantic_packet_sha256: fc19ad759e518bb848884c7a615db61cdc7ad890b936bb821b3292df398d884a
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r42_retrieval_admission_clock_module_map_assertions: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R42 candidate manifest to avoid self-reference
```

## R43 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R43
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 163fa950581cc212101565d1a52b7c9072c979d1
semantic_packet_bytes: 146386
semantic_packet_sha256: 709caea57cbe45d055ff9121a4f022516cb52dfc25ce59c86a4f498bd03797bb
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r43_cancellation_and_r1_status_promotion_assertions: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R43 candidate manifest to avoid self-reference
```

## R44 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R44
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: 471114f6b5897e32630d9f7f635fd00b4ca4bf2e
semantic_packet_bytes: 148403
semantic_packet_sha256: c81a8246578724b54e0096521cfa0ec842ad0b47410279d0a3195a0f5f234a7d
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r44_generic_definition_digest_and_source_missing_receipt_assertions: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R44 candidate manifest to avoid self-reference
```

## R45 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R45
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: e09347908869b227819af94767dbeab00e854124
semantic_packet_bytes: 152256
semantic_packet_sha256: 70be01384ae7c1cad8f7f3fad73ca782154ffe6cd5f4d77fa04545f323708ec5
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r45_recovery_origin_url_uniqueness_wire_tag_assertions: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R45 candidate manifest to avoid self-reference
```

## R46 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R46
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: e09347908869b227819af94767dbeab00e854124
semantic_packet_bytes: 152322
semantic_packet_sha256: 30497f58dbcc57ac60fdc02fc7418158b94a78d0cbe228ae253ab08838373d1d
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r46_recovery_tick_no_put_assertion: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R46 candidate manifest to avoid self-reference
```

## R47 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R47
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: e09347908869b227819af94767dbeab00e854124
semantic_packet_bytes: 153176
semantic_packet_sha256: 57cf4d3c779862211ffb75162bbc74cac16c6b7314812afdff360980f4ab2ed6
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r47_whole_packet_recovery_read_only_assertion: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R47 candidate manifest to avoid self-reference
```

## R48 pre-review evidence

```text
review_stage: POST_EDIT_STAGED_CANDIDATE
review_generation: R48
source_commit: 54d758fbf2f1c08df2e1993919287569b501b115
source_tree: 973b999d14feb91f5ebe84b1712006e18e21baeb
superseded_remote_head: e09347908869b227819af94767dbeab00e854124
semantic_packet_bytes: 153508
semantic_packet_sha256: 9a7583db7104b3b8f500a87ff9eb5f7260157bab6fe776fc5fb7a0c8c77368b5
changed_path: docs/tasks/m60-calendar-source-activation-readiness.md
repo_contract_checker: PASS
staged_candidate_diff_check_against_bound_base: PASS
foreign_dirtiness_check: PASS
r48_crash_boundary_recovery_mapping_assertion: PASS
source_evidence_manifest_sha256: 6c2bec479be09ec9c9d7cabb7ed5d41b0ab6cacfec7a0a1b8aee1c54e6ee5aa1
source_evidence_entries: 11/11 PASS
whole_file_sha256_and_index_blob: recorded externally in the immutable R48 candidate manifest to avoid self-reference
```

## Sources

[1]: https://www.teach.ustc.edu.cn/calendar/20135.html
[2]: https://www.teach.ustc.edu.cn/category/calendar
[3]: https://www.teach.ustc.edu.cn/robots.txt
[4]: https://www.teach.ustc.edu.cn/about/responsibility
