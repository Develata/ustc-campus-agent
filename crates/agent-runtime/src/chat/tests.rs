//! Unit tests for the bounded provider-neutral chat loop.

use super::*;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};

struct ScriptedModel {
    results: Mutex<VecDeque<Result<ModelInvocationResponse, ModelInvocationError>>>,
    requests: Mutex<Vec<ModelInvocationRequest>>,
}

impl ScriptedModel {
    fn new(results: Vec<Result<ModelInvocationResponse, ModelInvocationError>>) -> Self {
        Self {
            results: Mutex::new(results.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<ModelInvocationRequest> {
        self.requests.lock().expect("request lock").clone()
    }
}

impl ModelInvocationPort for ScriptedModel {
    fn invoke(&self, request: ModelInvocationRequest) -> ModelInvocationFuture<'_> {
        self.requests.lock().expect("request lock").push(request);
        let result = self
            .results
            .lock()
            .expect("script lock")
            .pop_front()
            .expect("scripted provider result");
        Box::pin(async move { result })
    }
}

struct ScriptedTool {
    result: Result<ToolExecutionOutput, ChatToolError>,
    calls: Mutex<Vec<ModelToolCall>>,
}

impl ScriptedTool {
    fn successful() -> Self {
        Self {
            result: Ok(ToolExecutionOutput {
                output_json: r#"{"procedure_id":"proc-011","status":"found"}"#.to_owned(),
            }),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn failing(error: ChatToolError) -> Self {
        Self {
            result: Err(error),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<ModelToolCall> {
        self.calls.lock().expect("tool calls lock").clone()
    }
}

impl ChatToolPort for ScriptedTool {
    fn execute(&self, call: &ModelToolCall) -> Result<ToolExecutionOutput, ChatToolError> {
        self.calls
            .lock()
            .expect("tool calls lock")
            .push(call.clone());
        self.result.clone()
    }
}

fn completed(response_id: &str, outputs: Vec<ModelOutput>) -> ModelInvocationResponse {
    ModelInvocationResponse {
        response_id: response_id.to_owned(),
        model: "model-test".to_owned(),
        status: ModelResponseStatus::Completed,
        outputs,
    }
}

fn tool_call(name: &str) -> ModelOutput {
    ModelOutput::ToolCall(ModelToolCall {
        call_id: "call-1".to_owned(),
        name: name.to_owned(),
        arguments_json: r#"{"procedure_id":"proc-011"}"#.to_owned(),
    })
}

fn block_on_ready<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("scripted future must be immediately ready"),
    }
}

#[test]
fn direct_chat_trims_input_and_returns_ungrounded_text() {
    let model = ScriptedModel::new(vec![Ok(completed(
        "resp-1",
        vec![ModelOutput::Text("  Direct answer.  ".to_owned())],
    ))]);
    let tools = ScriptedTool::successful();
    let engine = BoundedChatEngine::new(&model, &tools);

    let result = block_on_ready(engine.chat(ChatRequest {
        message: "  hello campus  ".to_owned(),
    }))
    .expect("direct chat");

    assert_eq!(result.answer, "Direct answer.");
    assert_eq!(result.model, "model-test");
    assert!(result.used_tools.is_empty());
    assert!(!result.grounded);
    assert!(tools.calls().is_empty());
    let requests = model.requests();
    let [
        ModelInvocationRequest::Initial {
            user_message,
            tools,
            max_output_tokens,
            ..
        },
    ] = requests.as_slice()
    else {
        panic!("one initial request expected");
    };
    assert_eq!(user_message, "hello campus");
    assert_eq!(*max_output_tokens, MAX_CHAT_OUTPUT_TOKENS);
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().all(|tool| tool.strict));
    assert_eq!(tools[0].name, USTC_AFFAIRS_LOOKUP_TOOL);
    assert_eq!(tools[1].name, USTC_COURSE_ADVICE_TOOL);
    let course_schema: serde_json::Value =
        serde_json::from_str(&tools[1].parameters_json).expect("course tool schema JSON");
    assert_eq!(course_schema["properties"]["min_credits"]["minimum"], 1);
}

#[test]
fn one_tool_round_is_correlated_and_marks_success_grounded() {
    let model = ScriptedModel::new(vec![
        Ok(completed(
            "resp-tool",
            vec![tool_call(USTC_AFFAIRS_LOOKUP_TOOL)],
        )),
        Ok(completed(
            "resp-final",
            vec![ModelOutput::Text("Grounded answer".to_owned())],
        )),
    ]);
    let tools = ScriptedTool::successful();
    let engine = BoundedChatEngine::new(&model, &tools);

    let result = block_on_ready(engine.chat(ChatRequest {
        message: "look it up".to_owned(),
    }))
    .expect("tool chat");

    assert_eq!(result.used_tools, vec![USTC_AFFAIRS_LOOKUP_TOOL]);
    assert!(result.grounded);
    assert_eq!(tools.calls().len(), 1);
    let requests = model.requests();
    let [
        _,
        ModelInvocationRequest::ToolContinuation {
            prior_response_id,
            tool_output,
            max_output_tokens,
            ..
        },
    ] = requests.as_slice()
    else {
        panic!("initial request and one continuation expected");
    };
    assert_eq!(prior_response_id, "resp-tool");
    assert_eq!(tool_output.call_id, "call-1");
    assert_eq!(tool_output.name, USTC_AFFAIRS_LOOKUP_TOOL);
    assert_eq!(*max_output_tokens, MAX_CHAT_OUTPUT_TOKENS);
}

#[test]
fn invalid_messages_reach_no_provider_or_tool() {
    for message in ["   ".to_owned(), "x".repeat(MAX_CHAT_MESSAGE_BYTES + 1)] {
        let model = ScriptedModel::new(Vec::new());
        let tools = ScriptedTool::successful();
        let engine = BoundedChatEngine::new(&model, &tools);
        let result = block_on_ready(engine.chat(ChatRequest { message }));
        assert_eq!(result, Err(ChatError::InvalidMessage));
        assert!(model.requests().is_empty());
        assert!(tools.calls().is_empty());
    }
}

#[test]
fn mixed_or_multiple_outputs_are_rejected_before_tool_execution() {
    let cases = vec![
        Vec::new(),
        vec![
            ModelOutput::Text("answer".to_owned()),
            tool_call(USTC_AFFAIRS_LOOKUP_TOOL),
        ],
        vec![
            tool_call(USTC_AFFAIRS_LOOKUP_TOOL),
            tool_call(USTC_COURSE_ADVICE_TOOL),
        ],
    ];
    for outputs in cases {
        let model = ScriptedModel::new(vec![Ok(completed("resp-1", outputs))]);
        let tools = ScriptedTool::successful();
        let engine = BoundedChatEngine::new(&model, &tools);
        let result = block_on_ready(engine.chat(ChatRequest {
            message: "hello".to_owned(),
        }));
        assert_eq!(result, Err(ChatError::InvalidProviderOutcome));
        assert!(tools.calls().is_empty());
    }
}

#[test]
fn unknown_and_malformed_tool_calls_fail_closed() {
    let unknown = ScriptedModel::new(vec![Ok(completed(
        "resp-1",
        vec![tool_call("unregistered_tool")],
    ))]);
    let tools = ScriptedTool::successful();
    let engine = BoundedChatEngine::new(&unknown, &tools);
    assert_eq!(
        block_on_ready(engine.chat(ChatRequest {
            message: "hello".to_owned(),
        })),
        Err(ChatError::UnknownTool)
    );
    assert!(tools.calls().is_empty());

    let malformed = ScriptedModel::new(vec![Ok(completed(
        "resp-2",
        vec![ModelOutput::ToolCall(ModelToolCall {
            call_id: String::new(),
            name: USTC_AFFAIRS_LOOKUP_TOOL.to_owned(),
            arguments_json: "{}".to_owned(),
        })],
    ))]);
    let engine = BoundedChatEngine::new(&malformed, &tools);
    assert_eq!(
        block_on_ready(engine.chat(ChatRequest {
            message: "hello".to_owned(),
        })),
        Err(ChatError::MalformedToolCall)
    );
    assert!(tools.calls().is_empty());
}

#[test]
fn any_second_tool_round_is_rejected_after_one_execution() {
    let model = ScriptedModel::new(vec![
        Ok(completed(
            "resp-tool",
            vec![tool_call(USTC_AFFAIRS_LOOKUP_TOOL)],
        )),
        Ok(completed(
            "resp-tool-2",
            vec![tool_call(USTC_COURSE_ADVICE_TOOL)],
        )),
    ]);
    let tools = ScriptedTool::successful();
    let engine = BoundedChatEngine::new(&model, &tools);
    assert_eq!(
        block_on_ready(engine.chat(ChatRequest {
            message: "hello".to_owned(),
        })),
        Err(ChatError::SecondToolRound)
    );
    assert_eq!(tools.calls().len(), 1);
    assert_eq!(model.requests().len(), 2);
}

#[test]
fn tool_denial_and_malformed_arguments_do_not_continue() {
    for (tool_error, expected) in [
        (ChatToolError::Denied, ChatError::ToolDenied),
        (
            ChatToolError::MalformedArguments,
            ChatError::MalformedToolArguments,
        ),
        (ChatToolError::Unavailable, ChatError::ToolFailed),
    ] {
        let model = ScriptedModel::new(vec![Ok(completed(
            "resp-tool",
            vec![tool_call(USTC_COURSE_ADVICE_TOOL)],
        ))]);
        let tools = ScriptedTool::failing(tool_error);
        let engine = BoundedChatEngine::new(&model, &tools);
        assert_eq!(
            block_on_ready(engine.chat(ChatRequest {
                message: "advise me".to_owned(),
            })),
            Err(expected)
        );
        assert_eq!(model.requests().len(), 1);
        assert_eq!(tools.calls().len(), 1);
    }
}

#[test]
fn provider_failure_and_nonterminal_outcome_are_stable_errors() {
    let tools = ScriptedTool::successful();
    let failed = ScriptedModel::new(vec![Err(ModelInvocationError::Unavailable)]);
    let engine = BoundedChatEngine::new(&failed, &tools);
    assert_eq!(
        block_on_ready(engine.chat(ChatRequest {
            message: "hello".to_owned(),
        })),
        Err(ChatError::ProviderUnavailable)
    );

    let nonterminal = ScriptedModel::new(vec![Ok(ModelInvocationResponse {
        response_id: "resp-running".to_owned(),
        model: "model-test".to_owned(),
        status: ModelResponseStatus::NonTerminal,
        outputs: Vec::new(),
    })]);
    let engine = BoundedChatEngine::new(&nonterminal, &tools);
    assert_eq!(
        block_on_ready(engine.chat(ChatRequest {
            message: "hello".to_owned(),
        })),
        Err(ChatError::NonTerminalProviderResponse)
    );
}

#[test]
fn safe_errors_never_render_external_content() {
    let forbidden = [
        "secret-key",
        "https://provider.invalid",
        "provider-body",
        "tool-output",
        "model-answer",
    ];
    for error in [
        ChatError::InvalidMessage,
        ChatError::ProviderUnavailable,
        ChatError::MalformedProviderResponse,
        ChatError::ToolDenied,
        ChatError::SecondToolRound,
    ] {
        let rendered = format!("{error:?} {error}");
        assert!(forbidden.iter().all(|value| !rendered.contains(value)));
    }
}
