use super::*;
fn update(revision: u64, text: &str) -> RootPromptUpdateDto {
    RootPromptUpdateDto {
        schema: "agent-root-prompt-update/v1".to_owned(),
        expected_revision: revision,
        text: text.to_owned(),
    }
}
#[test]
fn root_prompt_owner_cas_clear_restart_and_v1_migration() {
    let f = Fixture::new();
    let store = f.store();
    let initial = fs::read(f.path()).expect("v1 bytes");
    assert_eq!(
        store
            .root_prompt(&f.tenant, &f.user)
            .expect("default")
            .revision,
        0
    );
    assert_eq!(fs::read(f.path()).expect("read"), initial);
    let saved = store
        .update_root_prompt(&f.tenant, &f.user, update(0, "  用中文\n简洁回答  "))
        .expect("save");
    assert_eq!(saved.text, "用中文\n简洁回答");
    assert_eq!(saved.revision, 1);
    assert_eq!(
        store
            .update_root_prompt(&f.tenant, &f.user, update(0, &saved.text))
            .expect("exact repeat"),
        saved
    );
    assert_eq!(
        store.update_root_prompt(&f.tenant, &f.user, update(0, "other")),
        Err(ConversationError::RevisionConflict)
    );
    for (tenant, user) in [
        (f.tenant.clone(), UserId::parse("user:bob").expect("bob")),
        (
            TenantId::parse("tenant:other").expect("other"),
            f.user.clone(),
        ),
    ] {
        assert_eq!(
            store
                .root_prompt(&tenant, &user)
                .expect("isolated")
                .revision,
            0
        );
    }
    drop(store);
    let store = f.store();
    assert_eq!(
        store.root_prompt(&f.tenant, &f.user).expect("reopen"),
        saved
    );
    let cleared = store
        .update_root_prompt(&f.tenant, &f.user, update(1, " \n\t"))
        .expect("clear");
    assert_eq!(cleared.revision, 2);
    assert!(cleared.text.is_empty());
    assert_eq!(
        store.update_root_prompt(&f.tenant, &f.user, update(0, &saved.text)),
        Err(ConversationError::RevisionConflict)
    );
    drop(store);
    let store = f.store();
    assert_eq!(
        store
            .root_prompt(&f.tenant, &f.user)
            .expect("clear persists"),
        cleared
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(f.path()).expect("disk"))
            .expect("json")["version"],
        2
    );
}
#[test]
fn root_prompt_limits_controls_and_state_validation() {
    let f = Fixture::new();
    let store = f.store();
    for text in [
        "a".repeat(8193),
        "\0".to_owned(),
        "bad\u{202e}".to_owned(),
        " ".repeat(8193),
    ] {
        assert_eq!(
            store.update_root_prompt(&f.tenant, &f.user, update(0, &text)),
            Err(ConversationError::InvalidIntent)
        );
    }
    store
        .update_root_prompt(&f.tenant, &f.user, update(0, &"a".repeat(8192)))
        .expect("8192 bytes");
    let state = store.inner.lock().expect("lock").state.clone();
    let mut invalid = state.clone();
    invalid.version = 1;
    assert_eq!(
        validate_state(&invalid),
        Err(ConversationError::Unavailable)
    );
    let mut invalid = state.clone();
    invalid.root_prompts = None;
    assert_eq!(
        validate_state(&invalid),
        Err(ConversationError::Unavailable)
    );
    let mut invalid = state;
    let prompts = invalid.root_prompts.as_mut().expect("v2");
    prompts.push(prompts[0].clone());
    assert_eq!(
        validate_state(&invalid),
        Err(ConversationError::Unavailable)
    );
}
#[test]
fn root_prompt_snapshot_changes_only_next_turn_and_replay_has_no_admission() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    store
        .update_root_prompt(&f.tenant, &f.user, update(0, "old instruction"))
        .expect("save");
    let first_intent = intent("first", 0, "hello");
    let BeginTurn::New {
        request,
        title_request,
        ..
    } = store
        .begin_admitted(
            &f.tenant,
            &f.user,
            &c.id,
            first_intent.clone(),
            false,
            |request| {
                assert_eq!(
                    request.saved_root_prompt.as_deref(),
                    Some("old instruction")
                );
                Ok(())
            },
        )
        .expect("first")
    else {
        panic!("new")
    };
    assert_eq!(title_request.as_deref(), Some("hello"));
    store
        .update_root_prompt(&f.tenant, &f.user, update(1, "new instruction"))
        .expect("save newer");
    assert_eq!(
        request.saved_root_prompt.as_deref(),
        Some("old instruction")
    );
    assert!(
        !serde_json::to_string(&request)
            .expect("serialize")
            .contains("old instruction")
    );
    store
        .finish(&f.tenant, &f.user, &c.id, "first", Ok(response("answer")))
        .expect("finish");
    assert!(matches!(
        store.begin_admitted(&f.tenant, &f.user, &c.id, first_intent, false, |_| panic!(
            "replay cannot admit provider"
        )),
        Ok(BeginTurn::Replay(_))
    ));
    let BeginTurn::New { request, .. } = store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("second", 2, "next"),
            false,
        )
        .expect("next")
    else {
        panic!("new")
    };
    assert_eq!(
        request.saved_root_prompt.as_deref(),
        Some("new instruction")
    );
    assert!(
        !serde_json::to_string(
            &store
                .get(&f.tenant, &f.user, &c.id)
                .expect("public history")
        )
        .expect("json")
        .contains("instruction")
    );
}
