//! plan_ref: docs/plan/modules/60-model-provider-integration.md#121-bounded-chat--first-party-plugin-implementation-slice
//! Strict OpenAI-compatible Responses wire serialization and parsing.

use super::*;

pub(super) fn serialize_request(
    model: &str,
    request: ModelInvocationRequest,
) -> Result<Value, ModelInvocationError> {
    match request {
        ModelInvocationRequest::Initial {
            developer_instruction,
            user_message,
            tools,
            max_output_tokens,
        } => {
            validate_output_tokens(max_output_tokens)?;
            if developer_instruction.trim().is_empty()
                || developer_instruction.len() > MAX_CHAT_ANSWER_BYTES
                || user_message.trim().is_empty()
                || user_message.len() > MAX_CHAT_MESSAGE_BYTES
                || tools.len() != 2
            {
                return Err(ModelInvocationError::InvalidRequest);
            }

            let serialized_tools = serialize_tools(tools)?;

            Ok(json!({
                "model": model,
                "store": false,
                "parallel_tool_calls": false,
                "max_output_tokens": max_output_tokens,
                "input": [
                    {
                        "role": "developer",
                        "content": [{"type": "input_text", "text": developer_instruction}],
                    },
                    {
                        "role": "user",
                        "content": [{"type": "input_text", "text": user_message}],
                    },
                ],
                "tools": serialized_tools,
            }))
        }
        ModelInvocationRequest::ToolContinuation {
            prior_response_id,
            developer_instruction,
            user_message,
            tool_call,
            tool_output,
            tools,
            max_output_tokens,
        } => {
            validate_output_tokens(max_output_tokens)?;
            if !valid_metadata(&prior_response_id)
                || developer_instruction.trim().is_empty()
                || developer_instruction.len() > MAX_CHAT_ANSWER_BYTES
                || user_message.trim().is_empty()
                || user_message.len() > MAX_CHAT_MESSAGE_BYTES
                || !valid_metadata(&tool_call.call_id)
                || !valid_metadata(&tool_call.name)
                || tool_call.arguments_json.trim().is_empty()
                || tool_call.arguments_json.len() > MAX_CHAT_TOOL_ARGUMENT_BYTES
                || !valid_metadata(&tool_output.call_id)
                || tool_call.call_id != tool_output.call_id
                || tool_call.name != tool_output.name
                || !matches!(
                    tool_output.name.as_str(),
                    USTC_AFFAIRS_LOOKUP_TOOL | USTC_COURSE_ADVICE_TOOL
                )
                || tool_output.output_json.trim().is_empty()
                || tool_output.output_json.len() > MAX_CHAT_TOOL_OUTPUT_BYTES
            {
                return Err(ModelInvocationError::InvalidRequest);
            }
            let tool_arguments = serde_json::from_str::<Value>(&tool_call.arguments_json)
                .map_err(|_| ModelInvocationError::InvalidRequest)?;
            if !tool_arguments.is_object() {
                return Err(ModelInvocationError::InvalidRequest);
            }
            let serialized_tools = serialize_tools(tools)?;
            Ok(json!({
                "model": model,
                "store": false,
                "parallel_tool_calls": false,
                "max_output_tokens": max_output_tokens,
                "input": [
                    {
                        "role": "developer",
                        "content": [{"type": "input_text", "text": developer_instruction}],
                    },
                    {
                        "role": "user",
                        "content": [{"type": "input_text", "text": user_message}],
                    },
                    {
                        "type": "function_call",
                        "call_id": tool_call.call_id,
                        "name": tool_call.name,
                        "arguments": tool_call.arguments_json,
                    },
                    {
                        "type": "function_call_output",
                        "call_id": tool_output.call_id,
                        "output": tool_output.output_json,
                    }
                ],
                "tools": serialized_tools,
            }))
        }
    }
}

fn serialize_tools(
    tools: Vec<ustc_campus_agent_runtime::chat::ModelToolDefinition>,
) -> Result<Vec<Value>, ModelInvocationError> {
    if tools.len() != 2 {
        return Err(ModelInvocationError::InvalidRequest);
    }
    let mut saw_affairs = false;
    let mut saw_course = false;
    let mut serialized = Vec::with_capacity(tools.len());
    for tool in tools {
        if !tool.strict || tool.description.trim().is_empty() {
            return Err(ModelInvocationError::InvalidRequest);
        }
        match tool.name.as_str() {
            USTC_AFFAIRS_LOOKUP_TOOL if !saw_affairs => saw_affairs = true,
            USTC_COURSE_ADVICE_TOOL if !saw_course => saw_course = true,
            _ => return Err(ModelInvocationError::InvalidRequest),
        }
        let parameters = serde_json::from_str::<Value>(&tool.parameters_json)
            .map_err(|_| ModelInvocationError::InvalidRequest)?;
        if !parameters.is_object() {
            return Err(ModelInvocationError::InvalidRequest);
        }
        serialized.push(json!({
            "type": "function",
            "name": tool.name,
            "description": tool.description,
            "parameters": parameters,
            "strict": true,
        }));
    }
    if !saw_affairs || !saw_course {
        return Err(ModelInvocationError::InvalidRequest);
    }
    Ok(serialized)
}

fn validate_output_tokens(max_output_tokens: u32) -> Result<(), ModelInvocationError> {
    if (1..=MAX_CHAT_OUTPUT_TOKENS).contains(&max_output_tokens) {
        Ok(())
    } else {
        Err(ModelInvocationError::InvalidRequest)
    }
}

pub(super) fn parse_response(
    bytes: &[u8],
) -> Result<ModelInvocationResponse, ModelInvocationError> {
    let value = serde_json::from_slice::<Value>(bytes)
        .map_err(|_| ModelInvocationError::MalformedResponse)?;
    let object = value
        .as_object()
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if object.get("status").and_then(Value::as_str) != Some("completed") {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let response_id = required_metadata(object.get("id"))?;
    let model = required_metadata(object.get("model"))?;
    let outputs = object
        .get("output")
        .and_then(Value::as_array)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    let mut actionable = Vec::new();
    for output in outputs {
        let output = output
            .as_object()
            .ok_or(ModelInvocationError::MalformedResponse)?;
        match output.get("type").and_then(Value::as_str) {
            Some("reasoning") => {}
            Some("message" | "function_call") => actionable.push(output),
            _ => return Err(ModelInvocationError::MalformedResponse),
        }
    }
    let [output] = actionable.as_slice() else {
        return Err(ModelInvocationError::MalformedResponse);
    };

    let normalized = match output.get("type").and_then(Value::as_str) {
        Some("message") => parse_message_output(output)?,
        Some("function_call") => parse_function_call_output(output)?,
        _ => return Err(ModelInvocationError::MalformedResponse),
    };

    Ok(ModelInvocationResponse {
        response_id,
        model,
        status: ModelResponseStatus::Completed,
        outputs: vec![normalized],
    })
}

fn parse_message_output(
    output: &serde_json::Map<String, Value>,
) -> Result<ModelOutput, ModelInvocationError> {
    if output.get("role").and_then(Value::as_str) != Some("assistant")
        || output.get("status").and_then(Value::as_str) != Some("completed")
    {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let content = output
        .get("content")
        .and_then(Value::as_array)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    let [part] = content.as_slice() else {
        return Err(ModelInvocationError::MalformedResponse);
    };
    let part = part
        .as_object()
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if part.get("type").and_then(Value::as_str) != Some("output_text") {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let text = part
        .get("text")
        .and_then(Value::as_str)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if text.trim().is_empty() || text.len() > MAX_CHAT_ANSWER_BYTES {
        return Err(ModelInvocationError::MalformedResponse);
    }
    Ok(ModelOutput::Text(text.to_owned()))
}

fn parse_function_call_output(
    output: &serde_json::Map<String, Value>,
) -> Result<ModelOutput, ModelInvocationError> {
    if output.get("status").and_then(Value::as_str) != Some("completed") {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let call_id = required_metadata(output.get("call_id"))?;
    let name = required_metadata(output.get("name"))?;
    let arguments_json = output
        .get("arguments")
        .and_then(Value::as_str)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if arguments_json.is_empty() || arguments_json.len() > MAX_CHAT_TOOL_ARGUMENT_BYTES {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let arguments = serde_json::from_str::<Value>(arguments_json)
        .map_err(|_| ModelInvocationError::MalformedResponse)?;
    if !arguments.is_object() {
        return Err(ModelInvocationError::MalformedResponse);
    }
    Ok(ModelOutput::ToolCall(ModelToolCall {
        call_id,
        name,
        arguments_json: arguments_json.to_owned(),
    }))
}

fn required_metadata(value: Option<&Value>) -> Result<String, ModelInvocationError> {
    let value = value
        .and_then(Value::as_str)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if valid_metadata(value) {
        Ok(value.to_owned())
    } else {
        Err(ModelInvocationError::MalformedResponse)
    }
}

fn valid_metadata(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_METADATA_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

pub(super) fn map_transport_error(error: reqwest::Error) -> ModelInvocationError {
    if error.is_timeout() {
        ModelInvocationError::Timeout
    } else {
        ModelInvocationError::Unavailable
    }
}
