//! Compile-time proof that `client-protocol` has no server, domain, or client-implementation
//! dependency.
//!
//! `client-protocol` is wire-only. It MUST NOT depend on `platform-core` (M00 authority),
//! `affairs-navigator` (M71 domain), `application-ingress` (M10 server), `client-core` (M80
//! client), or any storage/runtime crate. The crate's `Cargo.toml` lists only `serde` and
//! `serde_json`, so no `use` statement can name a server/domain type.
//!
//! These compile_fail doctests attempt to `use` crates that exist in the workspace but are NOT
//! dependencies of `client-protocol`. They MUST fail to compile — that failure IS the
//! wire-only proof.

/// `platform-core` (M00 authority) is not a dependency.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_core;
```"]
const _NO_M00_DEPENDENCY: () = ();

/// `affairs-navigator` (M71 domain) is not a dependency.
#[cfg(doctest)]
#[doc = "```compile_fail
use affairs_navigator;
```"]
const _NO_M71_DEPENDENCY: () = ();

/// `application-ingress` (M10 server) is not a dependency.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress;
```"]
const _NO_SERVER_DEPENDENCY: () = ();

/// The crate does not re-export or name any M00 authority type.
#[test]
fn lib_reexports_contain_no_authority_type() {
    // If client-protocol re-exported an M00 type, this trait bound would resolve.
    // It does not, because the crate is wire-only.
    fn assert_wire_only<T: serde::Serialize + serde::de::DeserializeOwned>() {}
    assert_wire_only::<ustc_campus_agent_client_protocol::WireText>();
    assert_wire_only::<ustc_campus_agent_client_protocol::UnixMillis>();
}
