# Platform identity value contract

## Metadata

- `Status`: Accepted `M00-B1` contract; implemented with passing evidence
- `Version`: `platform-identity/v0`
- `Last Review`: `2026-07-27`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for authority partition and [`module-boundaries.md`](module-boundaries.md) for cross-module ownership
- `Acceptance`: implemented `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015`, `AUTH-016`; catalog-only `AUTH-013` is deferred to `M00-B3 request-context`
- `Primary Code`: `crates/platform-core/src/identity.rs`; `invocation::{TenantId, UserId}` are compatibility re-exports of these canonical values, not parallel identities

## 1. Scope and authority

`platform-identity/v0` freezes the small, framework-free values needed before `M00` can construct sessions or admitted request contexts. It owns:

- six canonical bounded platform ID representations;
- nominal separation between semantically different ID kinds;
- one shared construction-error taxonomy;
- validation, conversion, serialization and diagnostic behavior for those values.

It does not authenticate a subject, compose a tenant-scoped actor, open a session, assign a role, authorize a domain operation, generate an ID, persist a record or infer authority from text. Those decisions remain with later `M00` batches, `M10`, an owning domain module or an adapter as named by their contracts.

The module mints no value. It imports no clock, random-number generator, transport, database, framework or authentication-adapter type. These values are domain primitives, not transport DTOs, database rows, framework handles, credentials or user-facing labels.

## 2. Canonical value set

`M00-B1` introduces exactly these public value kinds:

| Value | Meaning | Explicitly does not prove |
|---|---|---|
| `TenantId` | one platform tenant | organization metadata, membership or permission |
| `UserId` | one platform-managed user subject, meaningful only with a tenant | external username, CAS/OIDC subject, authentication or role |
| `SessionId` | one platform session identity | active, authenticated, unexpired or unrevoked session state |
| `RequestId` | one ingress-attempt identity | admission, authorization or command acceptance |
| `CommandId` | one platform command identity | persistence, idempotent success or domain authorization |
| `CorrelationId` | one audit/operation correlation-chain identity | idempotency, authorization or causal adjacency |

`CausationId` and any tenant-scoped actor key are owned by the later `request-context` small module. `PlatformPolicySnapshotId` is owned by the later `policy-reference` small module. `platform-identity/v0` does not define them.

External `(issuer, subject)` evidence is owned by the later authentication-adapter/session contract and carries its own separately bounded type. A platform `UserId` is never a verbatim provider subject, and this bound is not widened to accommodate one.

## 3. Shared identifier grammar

Each of the six ID newtypes wraps one canonical string whose UTF-8 bytes satisfy:

```regex
^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$
```

Normative consequences:

1. encoded length is `1..=128` bytes;
2. the first and last byte are ASCII alphanumeric;
3. interior bytes are ASCII alphanumeric or one of `.`, `_`, `:`, `-`;
4. whitespace, control characters, non-ASCII text and every other punctuation byte are rejected;
5. case is significant;
6. no trimming, Unicode normalization, case folding, delimiter rewriting or alternate spelling occurs;
7. repeated interior delimiters are legal and retain no semantic meaning;
8. a prefix or delimiter pattern conveys no tenant class, role, provider, authorization, identifier kind or lifecycle state.

The grammar permits opaque and prefixed generator output whose first and last symbols are ASCII alphanumeric; hexadecimal, base32/Crockford, ULID and UUID qualify. Output from alphabets such as base64url or default Nano ID that can place `-` or `_` at an endpoint must be re-encoded before use. Retrying generation until a value happens to conform is not an accepted mitigation. Generation and collision policy are later adapter/port concerns; every generated value must still pass the same constructor.

## 4. Public construction and representation

Each ID kind is an owned nominal Rust value with one private backing field, declared as a named-field struct for the reason given later in this section. It provides an inherent checked `parse(value: impl Into<String>) -> Result<Self, IdentityValueError>` as the single canonical validator. The following public paths all delegate to `parse` and therefore share one grammar and error precedence:

- `TryFrom<String>`;
- `TryFrom<&str>`;
- `FromStr`;
- Serde deserialization from one string.

The inherent `parse` preserves existing invocation fixture call sites while tenant/user definitions converge. It is the checked constructor, not an unchecked compatibility path.

Each kind provides read-only access through `as_str()` and exact `Display`. Serialization emits exactly the canonical string. `Clone`, `Eq`, `Ord` and `Hash` operate on exact bytes. `Debug` retains the nominal type name and renders the named-field form (`TenantId { value: "…" }`); the value type does not silently redact or rewrite its bytes. `Debug` output is diagnostic, not an encoding: only `Display` and Serde are byte-exact carriers, and neither is affected by the representation.

The public API must not provide:

- `Default`;
- a public unchecked constructor;
- a lossy or infallible conversion from arbitrary text;
- cross-kind `From` conversions;
- `Deref` or mutable access to the backing string;
- segment, prefix or delimiter interpretation APIs;
- framework, database, auth-provider or transport-specific traits in the domain module.

Serde offers several entry points into the same type and the deserializer chooses between them, so **naming those entry points cannot close the class**. Every implemented `visit_*` method is an independent construction path: pinning `visit_str` and `visit_string` leaves `visit_bytes` free, and a branch inside a shared helper that still *contains* the parse call bypasses it for a chosen value while every named delegation remains intact. Both were demonstrated against evidence built that way.

The rule therefore sits one level below the entry points, at the thing every path must reach. `Deserialize` is implemented without a hand-written visitor at all — the canonical string is deserialized once through `String`'s own implementation, then handed to the checked `parse` exactly once — so whichever entry point a deserializer picks, there is one construction path and no second arm to keep in step. A value whose field is private can only be produced by its own literal syntax inside the defining module, so the module is additionally required to contain **exactly one such construction expression, inside `parse`**. An extra visitor arm, an early return, a branch, a decoy helper or a future trait implementation all have to build the value somewhere, and that expression is counted wherever it is written.

The counting is only a closure if it counts every *spelling* of the constructor. The field is private to the module, not to the macro expansion, so the six concrete kind names construct exactly as the generator's `$name` placeholder and `Self` do — a bare `fn f() -> TenantId { TenantId(s) }` written beside the generator bypasses `parse` while naming neither placeholder. The admitted construction forms are therefore derived from the frozen kind list rather than repeated beside it, so a seventh kind cannot be added in one place and forgotten in the other. For the same reason the module's **function inventory** is frozen: a bare helper is invisible to the `pub` scan and to the `mod`/`use`/`type` item accounting, and it is exactly where such a construction would sit. The module carries no `Visitor`, no `visit_*` method and no `deserialize_any`.

**The representation is part of the rule.** Counting construction expressions is only a closure if the constructor *is* an expression, and for a tuple struct it is not: the constructor is also a **value**. `let ctor = $name; ctor(text)` fills the private field while writing neither `$name(` nor `Self(` at the construction site, so it satisfies every count and every spelling above, and it can be bound, aliased, passed as an argument or returned before it is ever called — there is no site for a scan to find. That was demonstrated against evidence built the other way: with the checked constructor itself unchanged in every other respect, an ordinary downstream caller obtained and displayed a `TenantId` holding an invalid payload while the repository checker, the Python carrier tests, the exact Rust binding, the broad Rust suite and the doctests all passed.

The six kinds are therefore **named-field structs with one private field**, not tuple structs. A named-field struct has no constructor function item at all, so `let ctor = TenantId;` does not compile and the only way to produce one is a struct-literal expression — syntax, which cannot be bound. The tuple form is rejected outright rather than merely absent. Two rustdoc `compile_fail` proofs cover the API half — neither the tuple-call spelling nor the struct-literal spelling is usable from outside the crate — but they are honestly only *privacy* proofs: a `compile_fail` fence asserts that some error occurred, not which one, and both hold for a tuple struct as well, since the field is private either way. (The `E0423`/`E0451` diagnostics were confirmed by compiling each form directly, not by the fences.) The representation itself is pinned mechanically by the evidence carriers, plus one rustdoc proof of the single thing an outside caller can observe about it: `Debug` renders the named-field form.

**Function bodies are accounted for exactly.** A name inventory freezes the module's shape but says nothing about what each function does, and a containment check says nothing about what else it does: an early return placed above the admitted call keeps every containment rule satisfied and never reaches it. One construction site inside `parse` does not close that either — a single `Ok(Self { value })` reached through an inverted guard (`if value != <chosen> { classify(&value)?; }`) is still one site, in the right function, guarding nothing. Every function of the module is therefore pinned to its exact body, in source order, the same total accounting the item, `pub`, `impl`, attribute, derive and macro-arm surfaces already carry, one level further down. The cost is that any change to this module is drift that must be mirrored in both carriers; that is the intended price of a frozen `v0` implementation.

That rule has one stated limit rather than an implied completeness. Bodies are compared after comments and literal *payloads* are stripped, so it pins control flow and token shape, not the bytes inside a literal: replacing `b':'` with `b'?'` inside `is_interior_byte` leaves the frozen body unchanged. An **exhaustive grammar oracle** in the bound suite covers the runtime half — all 256 byte values through the leading, interior, final and one-byte positions, rather than a hand-picked corpus — but an oracle carries a delimiter table of its own, so on its own it only proves that production and oracle agree.

**Agreement among mutable carriers is not evidence.** Production, the oracle, both bound corpora, the tenant/user fixtures, their digests and the projection goldens are all editable in one change. Moved together from `:` to `?`, every mechanical gate stayed green while `a?b` was accepted and `a:b` rejected — demonstrated against evidence built that way. What was missing was not another carrier but a **root outside the mutable set**.

Grammar semantics are therefore bound in one authority chain whose root is this document:

```text
accepted contract grammar (§3 of this contract)
        │ exact semantic cross-check
evidence-side semantic table
        │ exact semantic extraction
production grammar + exhaustive oracle + bound corpora
```

The evidence-side table carries `regex`, `max_bytes`, the boundary class, the interior delimiter set, normalization and case sensitivity. It is not the authority: every field is cross-checked against §3 before being used. §3's regex must exist **exactly once** as a fenced normative carrier and is **parsed structurally** — its boundary and interior character classes are expanded to byte sets, the leading class must equal the trailing one, the interior class must extend the boundary class by exactly the admitted delimiters with no repeated byte, and the repetition bound must yield the admitted maximum. Each remaining field is bound to its own **anchored** normative-consequence line by list position and exact text, never to a substring found somewhere in the document, so one surviving mention cannot prove a value that has moved everywhere it is used.

The same table is then extracted from the implementation and the oracle, read from comment-stripped but literal-**preserving** source, because those bytes are exactly what the body fingerprint drops: the declared length bound, the whole admitted body of the boundary predicate, the interior delimiter byte literals as an ordered list so a duplicated delimiter is distinguishable from the correct set, the implementation's own restatement of the regex, and the oracle's single delimiter table bound to its admitted function body rather than to a literal appearing anywhere in the file. The bound valid corpus must exercise every delimiter the contract admits.

Consequences worth stating plainly. Changing the delimiter in production alone fails; changing production and the oracle together fails; changing production, oracle, corpora, fixtures, digests and goldens together fails, and fails specifically as a grammar-contract mismatch; changing the evidence-side table as well fails against this document. Changing this document is a `platform-identity/v0` change under §9 — which is the point, since it is then visible as one.

**A declared value is not an effective one.** The rules above bind the *declaration* `const MAX_IDENTITY_BYTES: usize = 128;` to §3 and freeze the deciding function's exact body. Neither closes the length bound, because the body fingerprint is itself one of the mutable carriers and a body may legally introduce a second semantic constant. A body that declares a local `const EFFECTIVE_MAX_IDENTITY_BYTES: usize = 129;`, compares and reports through it, and keeps the contract-bound carrier alive as `let _ = MAX_IDENTITY_BYTES;` — so no unused-item lint fires — leaves this document, both evidence-side tables and every declared `128` untouched. With both body fingerprints and the bound suite's corpus constant co-mutated with it, the full gate chain stayed green while an external caller parsed a 129-byte ID and was told the maximum was 129. Demonstrated against evidence built that way, not argued.

So **effective use is proven by elimination against the contract-bound name**, not by agreement between snapshots. Both carriers require, over comment- and literal-stripped source: the name is bound exactly once in the module, by a brace-depth-zero `const` whose value is §3's number written in plain digits (a computed value such as `127 + 1` does not match at all and fails closed); the name occurs nowhere else outside the deciding function; inside that function it occurs exactly twice — as the entire right-hand side of the module's only length comparison, with the frozen operator, and as the entire reported bound; that function declares no item, binds no name that could shadow it, spells no integer other than the byte-index offset, and measures exactly one `let bytes = value.as_bytes();`; and across the whole module there is exactly one length comparison and exactly one constructed `max_bytes` field, the enum variant's own field type excepted. Nothing is then left for a second bound to come from — no local constant, no `let`, no alias, no helper, no literal — and renaming the carrier does not help, because whatever name the checker binds is the one compared against §3.

**An occurring comparison is not a deciding one.** Requiring that the comparison *occur* in the deciding function leaves the branch it belongs to free to be disabled around it. Wrapping the guard — `if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES { … }`, opaque rather than a bare `false` so no lint collapses it — keeps the unique module constant, the two in-function occurrences, the comparison tuple, the reported bound, the admitted literal set, the subject binding and every module-wide count exactly as this document requires, while the effective maximum becomes unbounded. With both body fingerprints co-mutated, that tree passed the contract checker, the whole suite, `fmt`, `clippy -D warnings` and every cargo gate while an external crate parsed a 200-byte identity through the public API. Demonstrated, not argued.

So the guard is bound as **one structural unit**: the entire controlling condition between `if` and its opening brace, and the immediate rejection branch, matched as a single token sequence assembled from this document's own names, at the deciding function's own statement depth. Anything before or after the comparison, any `&&`, `||`, `!`, `== false`, helper call, macro condition, added predicate, alternate operator or operand, `if let` chain, alternate `else` branch, or a copy nested one block deeper, is not that sequence and is refused. The rejection branch must be exactly one immediate `return Err` of the `TooLong` variant reporting the contract-bound symbol in the `max_bytes` field, and that variant may be spelled nowhere in the module but its declaration, its rendering and this one branch, so a second rejection path cannot report a second bound while the admitted one is disabled.

**A prefix, suffix or wrapper condition around the max comparison belongs to the max-bound closure and is rejected.** It is not covered by the unrelated-control-flow limit stated below.

Three further carriers close the runtime half. The bound suite's corpus constant, from which every length fixture derives, is pinned to §3 by the checker and asserted equal to the suite's contract-parsed bound, so a co-mutated copy cannot make the runtime agree with a wrong implementation. The boundary itself is driven through the public API using the contract-parsed number rather than that corpus constant — the last admitted length must parse and be retained, and the first inadmissible one must be refused while reporting exactly the contract's bound. The over-length multibyte fixture asserts only its byte length and that its character count is strictly smaller, both derived, because hand-written counts were co-mutated once and kept agreeing with the wrong implementation.

**A call site is not a proof body.** Pinning only that the acceptance test *calls* its bound helper left the helper's load-bearing tail free to be deleted while the call survived; the same co-mutation also made the generic corpus macro `continue` past every over-length row while keeping each substring the checker pins inside it, because a substring is not a case that still reaches it. So the runtime proof lives in a dedicated function whose **entire body** is bound token-for-token by the always-run checker, from this document's names — the two candidate lengths, the `Ok`/`Err` polarity of each, and the reported bound — with string literals stripped, since the assertion prose is not the proof. That function must be called exactly once, as a plain statement of its caller rather than under a condition, in a caller that cannot leave early before it; and the generic corpus macro may not transfer control past a row at all.

**A bound guard is not a bound procedure.** The residue the paragraph above left to review turned out to be reachable mechanically, so it is closed rather than described. A step this document does not name could still be added *ahead* of the guard: `if value == "aaa…129" { return Ok(()); }` leaves the guard, the constant, every count and every elimination rule intact, and because literal payloads are stripped before all of them, both frozen fingerprints could be synchronized to `if value == { return Ok(()); }`. Checker, suite, `fmt`, `clippy -D warnings`, every cargo gate and every doctest stayed green while a 129-byte value parsed; keyed to 200 bytes instead, the same accept is invisible to a boundary proof that only drives the bound and one past it.

So the deciding function is bound to **exactly the decision procedure §5 names**, step for step and in that order, as a single token sequence assembled from this document's own names — the empty check, the length rejection above, the start-byte rule, the one-byte early return, the indexed interior scan, the end-byte rule, and canonical acceptance. A step the contract does not name is refused outright. The deciding function may additionally hold **no string literal at all**: it tests length and per-byte class, never a whole value, and a literal there is a value-keyed branch that every other rule reads past.

**`?` is control transfer too.** Banning `continue` and `return` where reachability is claimed left `?` open: a proof helper changed to return `Result`, a caller writing `let _ = helper();`, and one `black_box(Err::<(),()>(()))?` exit before the proof runs while spelling neither word — and the same shape inside an ignored closure wrapped around the corpus loop skips every over-length row, as does a `break`, while AUTH-011's own evidence calls survive `if black_box(false) { … }` around them with each registered carrier substring in place. So `?`, `break`, `continue` and `return` are all refused wherever a carrier claims something downstream of it executes; load-bearing proof helpers must be declared taking no argument and returning nothing, so no caller can discard the outcome; each proof and each of AUTH-011's evidence calls must appear as a plain statement of its caller's own body, which `let _ = …` and a never-entered wrapper are not; and each corpus loop must be a statement of the macro arm rather than of a closure nested inside it.

Structural rules can only refuse a value-keyed branch they can see, so one further control is behavioural and independent of all of them: every length up to twice the bound, under two canonical seeds, is driven through the public API and must be admitted exactly when this document admits it, and refused reporting exactly §3's number otherwise. An accept keyed to *any* over-bound length fails there whatever the source looks like.

**A bound call is not a call to the bound helper.** Requiring each proof to appear as a plain statement of its caller is a rule about tokens; which function that statement runs is a question of **name resolution**, and Rust resolves lexically, so an item declared in the caller's own body binds the name ahead of the module's. A local `fn r#assert_no_length_past_the_bound_is_accepted() {}` beside a decoy `let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();` keeps the real helper used — so no unused-item lint fires — leaves the call and every statement-position rule exactly as they were, and runs a no-op; checker, suite, `fmt`, `clippy -D warnings` and every cargo gate stayed green while neither the runtime proof nor the length sweep executed at all. Enumerating spellings cannot close that, because a raw identifier is the same name to Rust and a different string to every textual rule. Two facts close it instead: a shadow needs a **declaration**, so no caller of a load-bearing helper may declare an item at all; and a declaration must **write the name**, so no such caller may spell a load-bearing name more than the once its own call spends. `use x as helper;`, `let helper = …`, `const helper: …`, a nested `mod`, a `macro_rules!` and a closure parameter are each refused by one or the other, in any spelling; and each load-bearing helper is declared exactly once in the evidence module, so every rule that binds "the" helper's body binds the body its call resolves to. Raw identifiers are normalized to the plain name before any of this is read.

**A bound loop is not a bound sweep.** Freezing the sweep's token sequence fixes the loops and leaves free what they range over: `const RUNTIME_PROOF_SEEDS: [&str; 0] = [];` left every frozen token in place and swept nothing, and `RUNTIME_PROOF_SWEEP = GRAMMAR_MAX_BYTES` swept nothing *past* the bound, which is the half that matters — both green through every gate. So the sweep's carriers are bound as values, not only as names: each is declared exactly once at module level in a shape this document fixes, which refuses an alias, a helper call, a `const` expression, a macro expansion and a `cfg` twin without enumerating them; the seeds are two distinct single-byte values the grammar itself admits; and the span is twice §3's number. The sweep additionally **counts its own extent** and asserts both counts against §3's number rather than against either carrier, so an emptied or shortened carrier fails at runtime even where no structural rule sees it.

What remains is worth stating rather than implying. An accept keyed to a specific non-repeating value that the sweep does not generate is caught by the decision-procedure binding and the literal prohibition, not by execution; defeating it requires co-mutating the production source, both carriers' assembled procedures and both frozen fingerprints in one change. That cost is the closure, and it is a cost rather than an impossibility.

Runtime evidence is otherwise a secondary control rather than the closure. The bound tests drive an owning string deserializer and a bytes deserializer — two entry points `serde_json::from_str` never reaches — for all six kinds, and an invalid value must be rejected with exactly the error the checked constructor reports, so it inherits `parse`'s deterministic error class and non-echo guarantee by identity rather than by a weaker assertion. A finite set of exercised entry points is not a substitute for the structural rules above; it observes them holding.

Serde is an admitted stable value-encoding foundation and is exempt from the framework prohibition. A caller may explicitly read `as_str()` and construct another kind through its validator, but that act creates no authority and must not be used as a convenience conversion inside platform code.

That "must not provide" list is illustrative, not exhaustive, so the admitted surface is frozen positively instead. The identity module's entire public surface is exactly:

```text
public functions      parse, as_str                        (per ID kind)
                      value_kind, kind                     (on IdentityValueError,
                                                            both const fn)
trait implementations Display, TryFrom<String>, TryFrom<&str>, FromStr,
                      Serialize, Deserialize               (per ID kind)
                      Display, Error                       (on IdentityValueError)
derived traits        Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash   (per ID kind)
                      Debug, Clone, Copy, PartialEq, Eq    (on both error types)
public items          the six ID kinds, IdentityValueError, IdentityValueErrorKind
```

Anything absent from that list is forbidden, including a public alias or re-export (`pub type`, `pub use`), a public constant, a public module, trait or union, restricted visibility such as `pub(crate)`, and any additional inherent method or trait implementation. That list is the module's implementations in full: there is no private helper implementation behind it, and in particular no Serde visitor, for the reasons given above.

Evidence must enforce this as a **complete allowlist over the declaration grammar**, not as a list of forbidden spellings, and not as a scan for selected declaration forms. Every `pub` and `impl` token in the module must be classified and matched against the admitted set, function qualifiers included, so that `pub async fn`, `pub extern "Rust" fn`, `pub union` or any future `pub` form fails rather than passing unseen. An unclassifiable declaration must fail closed. Proving one spelling such as `new` is absent proves nothing about `from_unchecked`, `AsMut<String>`, a cross-kind `From` or a `pub type` alias for a deferred kind.

The frozen surface belongs to the six value kinds, not to one file. Anything in a sibling module of `platform-core` that adds externally reachable API to those kinds is equally forbidden: an **inherent** implementation, which Rust's orphan rule does not restrict to the defining file; and any alias or re-export that gives an admitted kind a second name or path.

Alias bindings are rejected rather than resolved. No sibling source may `use` or `type`-alias an admitted kind or the identity module itself, whether the binding is public or private, because a local alias does not change Rust's self type — `use crate::identity::TenantId as Tenant; impl AsRef<str> for Tenant { … }` is a real implementation for the governed type that no comparison against the kind's own name will see. A whole-module re-export such as `pub use crate::identity as identity_alias;` names no kind at all yet republishes every one of them, so the module path is governed exactly like the type names. The only admitted cross-file binding is the invocation compatibility re-export named in §6, spelled without renaming.

Evidence must identify an implementation's self type structurally. A `where` clause follows the self type rather than belonging to it, and an `impl` token's position on a line proves nothing about whether it is an item — a preceding `fn` on the same line can be a decoy. Every `impl` token in every pinned source file must therefore be resolved to its self type and checked.

For the same reason the compiled module graph is pinned rather than the file extension. Which files Cargo compiles into `platform-core` is decided by its non-inline `mod` declarations, so those are frozen exactly: `lib.rs` declares `identity` and `invocation` and nothing else, and neither of those declares a submodule. Pinning the declarations pins the compiled set semantically, so no attribute spelling can introduce a module the evidence never reads — not `#[path]`, not `#[cfg_attr(all(), path = "hidden.txt")]`, and not a future one, whatever file extension it targets. Enumerating `*.rs` files cannot do this on its own, because a module's source need not end in `.rs`.

Pinning module *names* is nevertheless not the same as pinning module *sources*, and rejecting a re-export by the spelling `crate::identity` is not the same as accounting for the use tree that contains it. An attribute on an admitted declaration redirects where that name is compiled from (`#[path = "identity_hidden.txt"] pub mod identity;` keeps the admitted name), and a grouped use tree republishes the module without ever spelling the searched path (`pub use crate::{identity as identity_alias, invocation as invocation_alias};`, and equally its `self::`, unqualified, nested and `{…}`-rooted forms). Both are closed the same way the public surface is: by **total accounting rather than pattern search**. Every `mod`, `use` and `type` item of every governed source is enumerated in source order, complete with its visibility and its bracket-balanced attribute envelope, and compared against an exact allowlist. An added item fails, a removed item fails, an attributed module declaration is a different fingerprint from a bare one, and a use tree is one fingerprint whatever spelling produced it. This deliberately freezes ordinary imports too: changing the protocol import list of `invocation.rs` is drift that must be mirrored in the evidence carriers, which is the intended cost of a frozen `v0` surface.

Governed sources additionally carry no `cfg_attr`, no `extern crate` and splice no items (`include!`; the identity module also excludes `include_str!` and `include_bytes!`). Otherwise a textual allowlist, however complete over the declaration grammar, can be satisfied while arbitrary public items live in a file it never reads. `extern crate self as x;` re-roots the crate under a second public name, so it is governed as an item — `extern` joins `mod`, `use` and `type` in the accounted keyword set — as well as by the carrier rule.

**A comment is a token separator, and every rule above is a rule about tokens.** Evidence must therefore preserve token boundaries when it removes comments and literals: a stripper that deletes them welds the neighbours together, so `extern/**/crate` becomes the single identifier `externcrate`, which no `extern crate` scan and no `extern` token match can see while Rust still reads two keywords and compiles the item. Each removed span becomes one space. For the same reason, any carrier that must remain a pattern rather than an allowlist is matched over a token sequence and not as a substring: `# /*x*/ ! [cfg(any())]` is the same crate-level exclusion as `#![cfg(any())]`, and `include /* x */ !("f.rs")` is the same splice as `include!("f.rs")`. Where a carrier is really an item or a macro, it is additionally accounted for by the corresponding allowlist rather than screened at all — the macro *invocation* names of every governed source are pinned, so a splicing macro is rejected by name whatever its spelling. Raw identifiers are excluded from the keyword scan in both directions: `r#type` is an ordinary field or module name, not a `type` item.

A macro is the remaining item category that can add API to a governed type without naming it in a `use`, a `type` or an `impl` header, because the definition sees only a metavariable and the invocation site sees only a macro call. Sibling macro definitions are therefore pinned by name, and no sibling macro invocation may pass an admitted kind.

For the same reason the sibling `impl` surface is an allowlist rather than a scan for the governed kind names. A blanket `impl<T> Extension for T` names no kind and covers all six, so rejecting only implementations that spell a kind leaves it green. The complete `impl` surface of each sibling is frozen instead; these are M20 items, and a genuine M20 addition is drift that must be admitted explicitly rather than arriving unseen.

Source-level accounting must also cover macro-generated items. The module admits exactly one private value-generating macro with exactly one match arm; its matcher grammar and every invocation are frozen, and an invocation passes exactly one kind name. Otherwise the macro can be widened to forward an arbitrary item — adding a real public trait implementation with no new macro definition and no top-level `impl` token. For the same reason, an `impl` token may be treated as a type-position `impl Trait` only inside a function signature, never merely because the preceding character is punctuation.

Because a bound test that is ignored or de-registered still exits zero, evidence must also pin the attribute envelope of each bound test: exactly one `#[test]` and no other attribute. Attribute parsing must be bracket-balanced, since a wrapped `#[cfg_attr(..., ignore)]` or `#[cfg(any(...))]` is invisible to line-based scanning while still suppressing execution.

**Counting an evidence carrier is not the same as pinning what it means.** A macro *definition* rebinds every call site that uses its name while changing no invocation name at all: a test-local `macro_rules! assert_eq` that expands to an uncalled closure keeps the suite compiling, keeps every admitted invocation name, and makes every equality claim in the evidence type-check-only. Macro definitions are therefore pinned by name wherever invocations are pinned, and no governed source or bound test file may redefine a macro the evidence asserts through.

An admitted helper is pinned as a *complete* executable carrier, not by its name and the presence of one matcher line. Rust selects the first arm whose matcher matches, so an earlier `($ignored:expr)` arm intercepts every existing `helper!(TenantId)` call — a type path is also an expression path — while the real `($kind:ty)` arm below it stays present but unread. The complete arm-matcher list is therefore frozen to exactly one `($kind:ty)` arm, and that sole arm's body must still carry the checks the grammar oracle depends on, so a body gutted to a no-op cannot pass while production is arbitrarily wrong.

Rebinding a name needs no definition at all. A block-local `use std::assert as assert_eq;`, dropped after an in-suite guard has already run, rebinds `assert_eq!` for the rest of its scope while adding no `macro_rules!` and changing no invocation name, so neither a definition pin nor an invocation-name pin nor a guard that ran earlier can see it. The bound test file's `use`/`type`/`mod` items are therefore accounted for in full against an exact allowlist — the same total accounting the governed sources carry — so any such alias, top-level or inside a block after any guard, is drift that fails.

That is still a rule about spelling, so the suite also proves the property directly. A guard reached from more than one bound test asserts that the assertion macros evaluate their arguments and still fail on a false claim, and reports its own failure through the path-qualified `::core::panic!`, which no local definition can shadow. A spelling rule can be evaded by a spelling nobody predicted; this one fails whenever the property is false, however it was made false.

Where the same rule is implemented in two carriers, agreement must be checked rather than claimed. Both carriers compare their lexer against one committed adversarial corpus — comment-split keywords, byte-char literals, raw identifiers, nested use trees, restricted visibility, non-ASCII identifiers, and each macro's complete arm-matcher list — so a divergence fails whichever side drifted, instead of surviving until a reviewer happens to probe the right input. Every class in that corpus has produced a real divergence or bypass at some point in this contract's evidence. The corpus is the retained fingerprint of *both* lexers, so the repository checker itself — not only the separate unit test — recomputes every corpus field from its own lexer; a mutated stripper that drifted the corpus along with it would otherwise leave the checker green, because nothing the acceptance rows run would have compared them.

Exclusion of the whole evidence crate counts the same way. The bound test file carries no inner attribute — a crate-level `#![cfg(any())]` makes every bound command report `running 0 tests` at exit zero and silences any guard written inside the suite — and it splices no source of its own (`#[path]`, `include!`, `extern crate`), so its guards cannot be replaced from a file the evidence never reads.

An attribute NAME is an ordinary identifier, and Rust accepts an identifier written as a raw identifier. `#[r#derive(Default)]` derives exactly as `#[derive(Default)]` does, `#[r#ignore]` suppresses a bound test, `#[r#default]` selects an enum default — and none of them contains the substring any scan for the plain spelling looks for. Screening for the raw form would only move the boundary one spelling further out, so attributes are **accounted for in full**: every attribute of every governed source is parsed, its name normalized (`r#` removed, a `$` metavariable retained), and the resulting name set matched against an exact allowlist. An attribute nobody predicted is drift because it is unadmitted, not because it was foreseen and blacklisted.

A `compile_fail` fence proves only that *something* failed to compile. Swapping its body for an unrelated type error keeps the fence, its category prose and the case count intact while the API the category denies becomes reachable, so each proof is additionally pinned to the expression it advertises: the "no `Default`" proof must contain a `::default()` call, the "no unchecked constructor" proof a `::new(` call, and so on. A proof that no longer exercises the denied API fails even though it still fails to compile.

An attribute head is also a token sequence, not a substring, everywhere it is read — not only for the inner `#![`. Rust tolerates whitespace, and therefore a comment, between `#` and `[`, so `# [derive(Copy)]` derives exactly as `#[derive(Copy)]` does, and `# [ignore]` suppresses a bound test exactly as `#[ignore]` does. A derive is the sharpest case: it synthesizes a trait implementation that appears nowhere as text, so no `use`/`type`/`impl` accounting can see it, and a literal `#[derive(` scan that missed a spaced spelling would admit an extra public impl while the frozen-surface allowlist stayed unchanged. So the derive surface is extracted by matching `#` `[` `derive` `(` whitespace-tolerantly over the whole source, the bound test's attribute envelope is collected the same way, and the same tolerance applies to `macro_rules ! name`, whose `!` a comment can likewise separate from its keyword. The two lexers' derive extraction, like their arm-matcher lists, is compared case for case against the shared corpus.

One bound is stated rather than papered over: the comment-and-literal stripper treats every backslash escape inside a `char` literal as two characters, so a `'\xHH'` or `'\u{…}'` escape is not recognized and its bytes survive into the stripped stream. This cannot hide an item — the leaked text carries only balanced or non-delimiter characters and every governed source is free of such escapes — and both lexers agree on it, so the corpus differential stays green; it is recorded here as a known limit of the stripper, not a guarantee that arbitrary char-literal escapes are handled.

Pinning dependency NAMES pins nothing about what those names resolve to. `semver = { path = "crates/fake-semver" }` keeps an admitted name while Cargo compiles an attacker-authored crate, and every Rust scan still reads `semver::Version`. Three carriers close that, each catching what the one above it misses: dependency **specifications** are compared value for value, so a `path`, `git` or alternate-registry key is drift rather than a silent redirect; `[patch]`, `[replace]`, `[source]` and a `.cargo/config*` source replacement are rejected everywhere, because each redirects a dependency without editing any dependency line; and the **resolved identity** in the committed `Cargo.lock` is pinned, so a redirect that survived the first two is still caught by the source URL disappearing. The direct dependencies of the governed package are pinned to `crates.io` by exact source URL, and `ustc-agent-tool-protocol` to its exact in-repo path.

The resolved graph is read from `Cargo.lock` rather than from `cargo metadata --locked --offline`. That command resolves against exactly this file, and the CI job that runs the repository checker on every pull request installs Python only, with no Rust toolchain — so invoking cargo there would make the gate unrunnable. Parsing the lock keeps the rule inside the carrier that every acceptance row already runs.

Cargo, not Rust, decides which file becomes which target, and it can be redirected without touching one line of Rust. `[lib] path`, `[package] build`, `[[bin]]`, `[[example]]`, `[[bench]]` and `[[test]]` each name a source outside anything a Rust scan reads. Screening for one key leaves the next open, so the owning manifest is pinned by exact key sets — admitted tables, admitted `[package]` keys, the exact `[lib]` target path, and the exact dependency and dev-dependency name sets — and the package's complete file inventory outside the separately digest-governed fixture tree is pinned as well. The pin is on the manifest's *meaning*, not on its text: a comment, a blank line and a reordered dependency change which bytes are in the file without changing which files Cargo compiles, and a guard that rejected them would fail the frozen-surface gate for an edit that never touched the surface.

Where the same rule is carried in two places, the two carriers must decide the same question. A structural rule mirrored by a literal text comparison is not a second carrier for that rule; it is a second, stricter rule that will diverge on the first legitimate edit. A rule applied to one governed source but not its siblings is not two carriers either: it is one carrier plus a gap, and the gap is where the same class reappears one file over. Every source-level rule here therefore names the set of sources it governs, and both carriers iterate that same set.

One asymmetry is deliberate and stated rather than papered over: the dependency **specification** and redirect-table rules live only in the repository checker, because the resolved-graph facts they guard are also pinned by the lockfile rule that both carriers implement. A redirect that evades the specification rules still has to change `Cargo.lock`, which both carriers read.

That last carrier is also the reason every acceptance row in §8 runs the repository checker before its Rust leg. **A Rust test cannot prove that it ran.** Redirecting the `[[test]] platform_identity` target, or renaming a bound function, makes `--exact` match nothing, which cargo reports as `running 0 tests` at exit zero — and any guard written inside the suite is exactly what has been replaced. Only an out-of-band carrier can detect that, so the checker that pins the manifest target set and the bound function names is part of each binding rather than a separate courtesy check.

Adding to this surface changes `platform-identity/v0` under §9.

## 5. Deterministic validation errors

All six ID constructors return one shared `IdentityValueError` taxonomy:

```text
Empty
TooLong { max_bytes: 128 }
InvalidStart
InvalidCharacter { byte_index }
InvalidEnd
```

That taxonomy has one frozen public representation. `IdentityValueErrorKind` is the public enum owning exactly the five failure variants above and their payloads. `IdentityValueError` is the returned wrapper; its fields are private and it carries exactly two facts — the static Rust value-kind name of the ID kind that rejected the input, and one `IdentityValueErrorKind`. It exposes exactly two read-only accessors:

```text
value_kind() -> &'static str
kind()       -> IdentityValueErrorKind
```

`value_kind()` returns the Rust type name, such as `"TenantId"`. No second public enum exists to carry that name, and neither type gains a field for the rejected input. This fixes representation only: the grammar, precedence, Serde shape and nominal kind set are unchanged, so it is not a `platform-identity/v0` version change under §9.

Error precedence is deterministic:

1. empty input;
2. byte length greater than 128;
3. invalid first byte;
4. first invalid byte in the half-open interior range `bytes[1..len-1]`, scanned left to right;
5. invalid final byte at `bytes[len-1]`.

For a one-byte input, the first-byte rule is the complete character check. For length at least two, the interior range excludes both endpoints. If all earlier checks pass, any non-alphanumeric final byte returns `InvalidEnd`, whether that byte would be a legal interior delimiter or an otherwise forbidden byte. A multibyte non-ASCII suffix may therefore return `InvalidCharacter` for its first invalid byte inside the interior range before the final-byte rule is reached.

`byte_index` is a zero-based byte index into the rejected input. An error may report the value kind, failure variant, fixed bound and byte index; it must not retain, format or log the rejected input. `Display` and `Debug` must not contain the complete rejected input, quote any input-derived fragment or render the offending byte.

Every failed construction returns no partial value. Serde uses `parse` and the same error precedence; it cannot bypass the constructor with a derived unchecked field decode.

## 6. Existing-type convergence

`crates/platform-core/src/invocation.rs` currently defines local `TenantId`, `UserId` and invocation-specific `PolicySnapshotId` values for the bounded P0a invocation proof. `M00-B1` must converge only the first two:

1. canonical `TenantId` and `UserId` move to the platform identity module;
2. invocation code imports and compatibility-re-exports those exact tenant/user types so existing `invocation::TenantId` and `invocation::UserId` paths remain valid without a second wrapper;
3. invocation `PolicySnapshotId` remains M20-owned and unrenamed; it identifies an `InvocationPolicySnapshot` and MUST NOT alias any future platform-policy identity;
4. existing invocation-specific IDs such as `InstallationId`, `GrantSnapshotId`, `RunId` and `TurnId` remain outside this contract until their owning contracts migrate them;
5. fixture data that violates `platform-identity/v0` fails migration explicitly; implementation must not preserve it through an unchecked compatibility path.

`M00-B1` is incomplete if duplicate tenant or user identity definitions remain publicly usable inside `platform-core`. Converging M20 policy identity is explicitly forbidden by this contract.

This convergence is done. `invocation.rs` no longer defines local tenant/user values and re-exports the canonical ones, so `identity::TenantId` and `invocation::TenantId` are one nominal type, as are the user values. `PolicySnapshotId` and every other invocation-local ID keep their M20 constructor, error type and 256-byte bound. Every existing tenant/user fixture value already satisfied this grammar, so no fixture was migrated and no unchecked compatibility path exists.

## 7. Failure and security boundaries

- Invalid text is rejected before any session, request, command or persistence operation exists.
- IDs are opaque references, not secrets; credential/token/password material must never be placed in them.
- Because rejected input may itself be secret material, validation errors omit the raw input.
- A syntactically valid ID never proves that the referenced object exists or is in scope.
- Deserialization success is shape validation only; every authority-bearing operation performs its owning lookup and state checks.
- No same-text or same-prefix fallback converts one ID kind into another.

## 8. Acceptance projection

| Case | Required proof | Binding |
|---|---|---|
| `AUTH-011` | each of the six ID kinds enforces the exact bounded grammar, deterministic error precedence and validating Serde path through `parse` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_values_enforce_canonical_bounds_and_errors -- --exact` |
| `AUTH-012` | the six ID kinds are byte-exact in string, Serde, ordering and hashing behavior; compile-fail API checks reject private-field construction, `Default`, unchecked construction, mutable backing access, cross-kind conversion and identifier-shape parsing | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_values_are_exact_and_nominal -- --exact && cargo test --locked -p ustc-campus-agent-core --doc identity` |
| `AUTH-014` | construction errors expose only value kind, failure class, fixed bound and byte index; `Display` and `Debug` contain no complete rejected input, input-derived fragment or offending-byte rendering | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_errors_never_echo_rejected_input -- --exact` |
| `AUTH-015` | the identity module mints no identifier and declares no clock, random, transport, database, framework or authentication-adapter dependency | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_module_has_no_generation_or_adapter_surface -- --exact` |
| `AUTH-016` | Market invocation authority consumes the M00-owned tenant/user definitions with no duplicate public tenant/user identity, while invocation policy-snapshot identity remains M20-owned | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity market_invocation_authority_uses_m00_identity_definitions -- --exact` |

All five active rows are `implemented`: the named Rust tests, the rustdoc `compile_fail` API proofs and the implementation-specific `check_platform_identity_implementation` checker rules exist and pass. `AUTH-013` remains catalog-only for the future request-context batch; retaining its stable ID does not make it current evidence.

These five rows prove `M00-B1` only. They do not prove any session, actor, request context, policy reference, generator, port, adapter or integration behavior, and they do not advance `M00` past `partial-evidence`.

## 9. Change rule

Changing the accepted byte grammar, maximum length, error precedence, Serde shape or nominal kind set changes `platform-identity/v0`. Such a change requires:

1. an owning-contract update;
2. acceptance-row and fixture review;
3. migration impact review for persisted or externally transported values;
4. implementation and downstream consumer evidence on the same revision.

Adding causation, actor, policy-reference or authenticated/service/administrator semantics is a later owning contract, not an incidental extension of these text values.
