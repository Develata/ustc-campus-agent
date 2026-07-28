//! Canonical `platform-identity/v0` value types owned by `M00-B1 identity-types`.
//!
//! This module owns six bounded, opaque, nominally distinct platform ID kinds and the one
//! construction-error taxonomy they share. It mints no value, reads no clock, opens no
//! transport and touches no store: every value arrives as caller-supplied text and is either
//! accepted verbatim or rejected.
//!
//! A syntactically valid ID proves shape only. It never proves that the referenced tenant,
//! user, session, request, command or correlation chain exists, is in scope, is authenticated
//! or is authorized. Those decisions belong to later `M00` batches, `M10` and each owning
//! domain module.
//!
//! Rejected input may itself be credential material, so no error variant, `Display`, `Debug`
//! or Serde diagnostic produced by this module retains or echoes it.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Maximum encoded length, in UTF-8 bytes, of any `platform-identity/v0` value.
const MAX_IDENTITY_BYTES: usize = 128;

/// Which grammar rule rejected a candidate identity value.
///
/// Each variant carries a fixed bound or a byte offset only. No variant carries the rejected
/// input, a fragment of it, or the offending byte itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityValueErrorKind {
    /// The candidate had zero bytes.
    Empty,
    /// The candidate exceeded the fixed encoded-length bound.
    TooLong {
        /// The fixed maximum encoded length, in UTF-8 bytes.
        max_bytes: usize,
    },
    /// The first byte was not ASCII alphanumeric.
    InvalidStart,
    /// An interior byte was neither ASCII alphanumeric nor one of `.`, `_`, `:`, `-`.
    InvalidCharacter {
        /// Zero-based index of the first offending byte within the rejected UTF-8 bytes.
        byte_index: usize,
    },
    /// The final byte was not ASCII alphanumeric.
    InvalidEnd,
}

/// Why one `platform-identity/v0` construction failed.
///
/// The error names the Rust value kind that rejected the input and the grammar rule that
/// rejected it. It deliberately has no `source`, so no rejected input can be reached by
/// walking the error chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityValueError {
    value_kind: &'static str,
    kind: IdentityValueErrorKind,
}

impl IdentityValueError {
    /// Returns the Rust type name of the ID kind that rejected the input, such as `"TenantId"`.
    #[must_use]
    pub const fn value_kind(&self) -> &'static str {
        self.value_kind
    }

    /// Returns the grammar rule that rejected the input.
    #[must_use]
    pub const fn kind(&self) -> IdentityValueErrorKind {
        self.kind
    }
}

impl fmt::Display for IdentityValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_kind = self.value_kind;
        match self.kind {
            IdentityValueErrorKind::Empty => {
                write!(formatter, "{value_kind} rejected: value is empty")
            }
            IdentityValueErrorKind::TooLong { max_bytes } => write!(
                formatter,
                "{value_kind} rejected: encoded length exceeds {max_bytes} bytes"
            ),
            IdentityValueErrorKind::InvalidStart => write!(
                formatter,
                "{value_kind} rejected: first byte is not ASCII alphanumeric"
            ),
            IdentityValueErrorKind::InvalidCharacter { byte_index } => write!(
                formatter,
                "{value_kind} rejected: byte {byte_index} is not permitted"
            ),
            IdentityValueErrorKind::InvalidEnd => write!(
                formatter,
                "{value_kind} rejected: final byte is not ASCII alphanumeric"
            ),
        }
    }
}

impl Error for IdentityValueError {}

/// Boundary bytes are ASCII alphanumeric only.
const fn is_boundary_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

/// Interior bytes add the four canonical delimiters. Repetition carries no meaning.
const fn is_interior_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b':')
}

/// Applies the shared grammar in the exact precedence frozen by `platform-identity/v0` §5.
///
/// Precondition: `value` is well-formed UTF-8, guaranteed by `&str`.
/// Postcondition: `Ok(())` exactly when `value` matches
/// `^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$`.
/// Invariant: exactly one left-to-right pass over the interior, and no allocation.
fn classify(value: &str) -> Result<(), IdentityValueErrorKind> {
    let bytes = value.as_bytes();
    let Some((&first, after_first)) = bytes.split_first() else {
        return Err(IdentityValueErrorKind::Empty);
    };
    if bytes.len() > MAX_IDENTITY_BYTES {
        return Err(IdentityValueErrorKind::TooLong {
            max_bytes: MAX_IDENTITY_BYTES,
        });
    }
    if !is_boundary_byte(first) {
        return Err(IdentityValueErrorKind::InvalidStart);
    }
    // A one-byte value is fully decided by the first-byte rule; the interior range is then
    // empty and there is no separate final byte.
    let Some((&last, interior)) = after_first.split_last() else {
        return Ok(());
    };
    for (offset, &byte) in interior.iter().enumerate() {
        if !is_interior_byte(byte) {
            return Err(IdentityValueErrorKind::InvalidCharacter {
                byte_index: offset + 1,
            });
        }
    }
    if !is_boundary_byte(last) {
        return Err(IdentityValueErrorKind::InvalidEnd);
    }
    Ok(())
}

/// Defines one nominal ID kind. Private on purpose: the six kinds below are the whole
/// public surface, and no downstream crate may mint a seventh through this macro.
macro_rules! identity_value {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        // A NAMED-FIELD struct, deliberately not a tuple struct.
        //
        // A tuple struct's constructor is a VALUE, not only a syntax: `let ctor = $name;`
        // binds it and `ctor(text)` then builds the private newtype without writing `$name(`
        // or `Self(` anywhere at the construction site. Counting construction expressions is
        // therefore not a closure while that value exists — it counts one spelling of the
        // constructor and misses every alias, local binding, argument and return of it.
        //
        // A named-field struct has no constructor function item at all. `let ctor = $name;`
        // does not compile, so the ONLY way to produce one of these is a struct-literal
        // expression, and a struct literal cannot be bound, aliased, passed or returned. The
        // class is closed by the language rather than by a scan that has to predict spellings.
        pub struct $name {
            value: String,
        }

        impl $name {
            #[doc = concat!("Parses one canonical `", stringify!($name), "`.")]
            ///
            /// This is the single validator. Every other construction and deserialization path
            /// on this type delegates here, so all of them share one grammar and one error
            /// precedence.
            ///
            /// # Errors
            ///
            /// Returns [`IdentityValueError`] when `value` does not match the
            /// `platform-identity/v0` grammar. The error names this kind and the failing rule
            /// and never contains the rejected input.
            pub fn parse(value: impl Into<String>) -> Result<Self, IdentityValueError> {
                let value = value.into();
                match classify(&value) {
                    Ok(()) => Ok(Self { value }),
                    Err(kind) => Err(IdentityValueError {
                        value_kind: stringify!($name),
                        kind,
                    }),
                }
            }

            /// Returns the exact canonical bytes, with case and delimiters preserved.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.value
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdentityValueError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IdentityValueError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl FromStr for $name {
            type Err = IdentityValueError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            /// Deserializes the canonical string, then applies the one checked constructor.
            ///
            /// A hand-written `Visitor` is deliberately NOT used. Every implemented `visit_*`
            /// method is an independent construction path, so a visitor has to be enumerated
            /// and each arm proven to validate — and the next unenumerated arm (`visit_bytes`,
            /// `visit_borrowed_str`, …) reopens the hole. Deferring to `String`'s own
            /// `Deserialize` leaves exactly one construction path in this impl: whatever entry
            /// point the deserializer chooses, it produces a `String` that this line then hands
            /// to `parse`. There is no second arm to keep in step, so the property is
            /// structural rather than something evidence has to re-check per method.
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                $name::parse(value).map_err(de::Error::custom)
            }
        }
    };
}

identity_value! {
    /// One platform tenant.
    ///
    /// It proves no organization metadata, membership or permission. A prefix or delimiter run
    /// inside the value conveys no tenant class.
    ///
    /// The private backing field cannot be constructed directly:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::TenantId;
    ///
    /// let tenant = TenantId { value: String::from("tenant:example") };
    /// ```
    ///
    /// A default identity value does not exist:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::TenantId;
    ///
    /// let tenant = TenantId::default();
    /// ```
    ///
    /// There is no unchecked constructor:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::TenantId;
    ///
    /// let tenant = TenantId::new("tenant:example");
    /// ```
    TenantId
}

identity_value! {
    /// One platform-managed user subject, meaningful only together with a tenant.
    ///
    /// It is never a verbatim external username or CAS/OIDC subject, and it proves no
    /// authentication or role.
    ///
    /// The backing string cannot be mutated:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::UserId;
    ///
    /// fn rewrite(user: &mut UserId) {
    ///     user.as_mut_str().make_ascii_uppercase();
    /// }
    /// ```
    ///
    /// The value does not dereference to its backing string:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::UserId;
    ///
    /// fn borrow(user: &UserId) -> &str {
    ///     &**user
    /// }
    /// ```
    UserId
}

identity_value! {
    /// One platform session identity.
    ///
    /// It proves no active, authenticated, unexpired or unrevoked session state.
    ///
    /// One identity kind cannot convert into another:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::{SessionId, TenantId};
    ///
    /// fn widen(session: SessionId) -> TenantId {
    ///     TenantId::from(session)
    /// }
    /// ```
    SessionId
}

identity_value! {
    /// One ingress-attempt identity.
    ///
    /// It proves no admission, authorization or command acceptance.
    ///
    /// Identifier shape is not interpreted:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::RequestId;
    ///
    /// fn classify(request: &RequestId) -> &str {
    ///     request.prefix()
    /// }
    /// ```
    RequestId
}

identity_value! {
    /// One platform command identity.
    ///
    /// It proves no persistence, idempotent success or domain authorization.
    ///
    /// The type name is not a constructor value:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::CommandId;
    ///
    /// let build = CommandId;
    /// let command = build(String::from("command:example"));
    /// ```
    CommandId
}

identity_value! {
    /// One audit/operation correlation-chain identity.
    ///
    /// It proves no idempotency, authorization or causal adjacency. Causation identity is owned
    /// by the later `request-context` batch and is deliberately absent here.
    ///
    /// There is no tuple constructor call:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::identity::CorrelationId;
    ///
    /// let correlation = CorrelationId(String::from("correlation:example"));
    /// ```
    ///
    /// `Debug` renders the named-field form, which is the one thing about the representation an
    /// outside caller can actually observe. Both `compile_fail` proofs above hold for a TUPLE
    /// struct too, because the field is private either way — a `compile_fail` fence asserts only
    /// that some error occurred, not which one. This proof is what a revert to a tuple struct
    /// breaks:
    ///
    /// ```
    /// use ustc_campus_agent_core::identity::CorrelationId;
    ///
    /// let correlation = CorrelationId::parse("correlation:example").expect("valid");
    /// assert_eq!(
    ///     format!("{correlation:?}"),
    ///     "CorrelationId { value: \"correlation:example\" }"
    /// );
    /// ```
    CorrelationId
}
