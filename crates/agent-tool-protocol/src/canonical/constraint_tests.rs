use super::*;

fn schema(node: UnvalidatedSchemaNodeV0) -> ValidatedToolInputSchemaV0 {
    ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".into(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: vec![("x".into(), node)],
            required: vec!["x".into()],
        },
    })
    .expect("bounded schema")
}
fn argument(node: UnvalidatedArgumentValueV0) -> CanonicalArgumentValueV0 {
    CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![("x".into(), node)]))
        .expect("bounded argument")
}
fn member(schema: &ValidatedToolInputSchemaV0) -> &ValidatedSchemaNodeV0 {
    let ValidatedSchemaNodeV0::Object { properties, .. } = schema.root() else {
        panic!("object");
    };
    &properties["x"]
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn scalar_constraints_preserve_legacy_golden_and_encode_new_tags() {
    let old = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".into(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: vec![
                (
                    "query".into(),
                    UnvalidatedSchemaNodeV0::String { enum_values: None },
                ),
                ("count".into(), UnvalidatedSchemaNodeV0::Integer),
            ],
            required: vec!["query".into()],
        },
    })
    .expect("legacy schema");
    assert_eq!(
        old.digest().as_str(),
        "sha256:8a91a2fdad047d1bcfc4ac0392778f7125afce4faf637ed3aac4fd535fd1db2e"
    );
    assert_eq!(old.canonical_bytes().len(), 80);
    let bounded = schema(UnvalidatedSchemaNodeV0::BoundedString {
        enum_values: None,
        min_length: Some(1),
        max_length: Some(2),
    });
    assert_eq!(
        hex(bounded.canonical_bytes()),
        concat!(
            "746f6f6c2d696e7075742d736368656d612f763000",
            "010000000000000001000000000000000178",
            "0700010000000000000001010000000000000002",
            "0000000000000001000000000000000178"
        )
    );
    let integer = schema(UnvalidatedSchemaNodeV0::BoundedInteger {
        minimum: Some(-1),
        maximum: None,
    });
    let number = schema(UnvalidatedSchemaNodeV0::BoundedNumber {
        minimum: Some(-1.0),
        maximum: None,
    });
    assert_eq!(
        hex(integer.canonical_bytes()),
        concat!(
            "746f6f6c2d696e7075742d736368656d612f763000",
            "010000000000000001000000000000000178",
            "0801ffffffffffffffff00",
            "0000000000000001000000000000000178"
        )
    );
    assert_eq!(
        hex(number.canonical_bytes()),
        concat!(
            "746f6f6c2d696e7075742d736368656d612f763000",
            "010000000000000001000000000000000178",
            "0901bff000000000000000",
            "0000000000000001000000000000000178"
        )
    );
    assert_ne!(integer.digest(), number.digest());
    for changed in [
        UnvalidatedSchemaNodeV0::BoundedString {
            enum_values: None,
            min_length: Some(0),
            max_length: Some(2),
        },
        UnvalidatedSchemaNodeV0::BoundedString {
            enum_values: None,
            min_length: Some(1),
            max_length: Some(3),
        },
        UnvalidatedSchemaNodeV0::BoundedString {
            enum_values: Some(vec!["a".into()]),
            min_length: Some(1),
            max_length: Some(2),
        },
    ] {
        assert_ne!(bounded.digest(), schema(changed).digest());
    }
    for (first, changed) in [(1, 2), (i64::MIN, i64::MIN + 1)] {
        assert_ne!(
            schema(UnvalidatedSchemaNodeV0::BoundedInteger {
                minimum: Some(first),
                maximum: None
            })
            .digest(),
            schema(UnvalidatedSchemaNodeV0::BoundedInteger {
                minimum: Some(changed),
                maximum: None
            })
            .digest()
        );
    }
    assert_ne!(
        number.digest(),
        schema(UnvalidatedSchemaNodeV0::BoundedNumber {
            minimum: Some(-0.5),
            maximum: None
        })
        .digest()
    );
    assert_ne!(
        integer.digest(),
        schema(UnvalidatedSchemaNodeV0::BoundedInteger {
            minimum: Some(-1),
            maximum: Some(2)
        })
        .digest()
    );
    assert_ne!(
        number.digest(),
        schema(UnvalidatedSchemaNodeV0::BoundedNumber {
            minimum: Some(-1.0),
            maximum: Some(2.0)
        })
        .digest()
    );
}

#[test]
fn scalar_constraints_use_unicode_scalars_and_intersect_enum() {
    let single = schema(UnvalidatedSchemaNodeV0::BoundedString {
        enum_values: None,
        min_length: Some(1),
        max_length: Some(1),
    });
    for text in ["中", "😀", "é"] {
        assert!(single.accepts(&argument(UnvalidatedArgumentValueV0::String(text.into()))));
    }
    for text in ["", "e\u{301}", "中文", "👩‍🔬"] {
        assert!(!single.accepts(&argument(UnvalidatedArgumentValueV0::String(text.into()))));
    }
    let choices = schema(UnvalidatedSchemaNodeV0::BoundedString {
        enum_values: Some(vec!["a".into(), "ab".into()]),
        min_length: Some(2),
        max_length: Some(2),
    });
    assert!(member(&choices).accepts_string_value("ab"));
    assert!(!member(&choices).accepts_string_value("a"));
    assert!(!member(&choices).accepts_string_value("xy"));
    assert!(!single.accepts(&argument(UnvalidatedArgumentValueV0::Integer("1".into()))));
    assert!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::String("中".repeat(1366)))
            .is_err(),
        "scalar constraint does not relax 4096 UTF-8 byte cap"
    );
}

#[test]
fn scalar_constraints_preserve_integer_precision_and_input_tags() {
    let integers = schema(UnvalidatedSchemaNodeV0::BoundedInteger {
        minimum: Some(i64::MIN),
        maximum: Some(i64::MAX),
    });
    for value in [i64::MIN, i64::MAX] {
        assert!(
            integers.accepts(&argument(UnvalidatedArgumentValueV0::Integer(
                value.to_string()
            )))
        );
    }
    assert!(!integers.accepts(&argument(UnvalidatedArgumentValueV0::Number("1.0".into()))));
    assert!(member(&integers).accepts_output_number(1.0));
    assert!(!member(&integers).accepts_output_number(1.5));
    assert!(!member(&integers).accepts_output_number(9_223_372_036_854_775_808.0));
    assert!(!member(&integers).accepts_output_integer(i128::from(u64::MAX)));
    let exact = schema(UnvalidatedSchemaNodeV0::BoundedInteger {
        minimum: Some(9_007_199_254_740_993),
        maximum: Some(9_007_199_254_740_993),
    });
    assert!(member(&exact).accepts_output_integer(9_007_199_254_740_993));
    assert!(!member(&exact).accepts_output_number(9_007_199_254_740_992.0));
    assert!(!member(&exact).accepts_output_integer(9_007_199_254_740_992));
}

#[test]
fn scalar_constraints_check_binary64_bounds_and_exact_mixed_comparisons() {
    let numbers = schema(UnvalidatedSchemaNodeV0::BoundedNumber {
        minimum: Some(-0.5),
        maximum: Some(9_007_199_254_740_992.0),
    });
    assert!(numbers.accepts(&argument(UnvalidatedArgumentValueV0::Number("-0.5".into()))));
    assert!(
        !numbers.accepts(&argument(UnvalidatedArgumentValueV0::Number(
            "-0.5001".into()
        )))
    );
    assert!(!numbers.accepts(&argument(UnvalidatedArgumentValueV0::Integer("1".into()))));
    let node = member(&numbers);
    assert!(node.accepts_output_integer(9_007_199_254_740_992));
    assert!(!node.accepts_output_integer(9_007_199_254_740_993));
    assert!(!node.accepts_output_integer(-1));
    assert!(node.accepts_output_integer(0));
    assert!(!node.accepts_output_number(f64::INFINITY));
    assert!(!node.accepts_output_number(f64::NAN));
    assert_eq!(
        compare_integer_number(i128::MIN, i128::MIN as f64),
        Some(std::cmp::Ordering::Equal)
    );
    assert_eq!(
        compare_integer_number(i128::MAX, -(i128::MIN as f64)),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        compare_integer_number(0, -0.0),
        Some(std::cmp::Ordering::Equal)
    );
    assert_eq!(
        compare_integer_number(-1, -1.5),
        Some(std::cmp::Ordering::Greater)
    );
    let positive_zero = schema(UnvalidatedSchemaNodeV0::BoundedNumber {
        minimum: Some(0.0),
        maximum: None,
    });
    let negative_zero = schema(UnvalidatedSchemaNodeV0::BoundedNumber {
        minimum: Some(-0.0),
        maximum: None,
    });
    assert_eq!(positive_zero, negative_zero);
}

#[test]
fn scalar_constraints_reject_empty_invalid_and_reversed_bounds() {
    for node in [
        UnvalidatedSchemaNodeV0::BoundedInteger {
            minimum: None,
            maximum: None,
        },
        UnvalidatedSchemaNodeV0::BoundedString {
            enum_values: None,
            min_length: None,
            max_length: None,
        },
        UnvalidatedSchemaNodeV0::BoundedNumber {
            minimum: None,
            maximum: None,
        },
        UnvalidatedSchemaNodeV0::BoundedInteger {
            minimum: Some(2),
            maximum: Some(1),
        },
        UnvalidatedSchemaNodeV0::BoundedString {
            enum_values: None,
            min_length: Some(2),
            max_length: Some(1),
        },
        UnvalidatedSchemaNodeV0::BoundedNumber {
            minimum: Some(f64::NAN),
            maximum: None,
        },
        UnvalidatedSchemaNodeV0::BoundedNumber {
            minimum: None,
            maximum: Some(f64::INFINITY),
        },
        UnvalidatedSchemaNodeV0::BoundedNumber {
            minimum: Some(2.0),
            maximum: Some(1.0),
        },
    ] {
        assert_eq!(
            ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
                dialect: "tool-input-schema/v0".into(),
                root: UnvalidatedSchemaNodeV0::Object {
                    properties: vec![("x".into(), node)],
                    required: vec![],
                },
            })
            .expect_err("invalid bounded schema"),
            SchemaConstructionError::SchemaMalformed
        );
    }
}
