use aion_agent::engine::AgentError as AionrsAgentError;
use aion_providers::ProviderError;
use aionui_api_types::{
    AgentErrorCode, AgentErrorOwnership, AgentErrorResolution, AgentErrorResolutionKind, AgentErrorResolutionTarget,
};

use crate::protocol::send_error::AgentSendError;

pub(super) fn aionrs_engine_error_to_send_error(error: &AionrsAgentError) -> AgentSendError {
    let public_error = aionrs_engine_error_to_public_error(error);
    aionrs_public_error_to_send_error(&public_error)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AionrsPublicError {
    pub(super) code: AionrsPublicErrorCode,
    pub(super) message: String,
    pub(super) ownership: AionrsPublicErrorOwnership,
    pub(super) details: Vec<AionrsPublicErrorDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum AionrsPublicErrorCode {
    ProviderCredentialMissing,
    ProviderAuthFailed,
    ProviderPermissionDenied,
    ProviderBillingRequired,
    ProviderQuotaExceeded,
    ProviderRateLimited,
    ProviderModelNotFound,
    ProviderModelUnsupported,
    ProviderModelUnavailable,
    ProviderEndpointNotFound,
    ProviderContextTooLarge,
    ProviderToolSchemaInvalid,
    ProviderToolCallInvalid,
    ProviderContentBlocked,
    ProviderTimeout,
    ProviderStreamInterrupted,
    ProviderTransportFailed,
    ProviderServerError,
    ProviderInvalidRequest,
    ProviderResponseParseFailed,
    ProviderEmptyResponse,
    ProviderUnknownError,
    ConfigInvalid,
    ConfigProfileNotFound,
    ConfigProviderAliasInvalid,
    ConfigEnvMissing,
    ConfigFileReadFailed,
    ConfigFileParseFailed,
    ConfigFileWriteFailed,
    BootstrapFailed,
    BootstrapProviderInitFailed,
    BootstrapToolInitFailed,
    BootstrapMcpInitFailed,
    BootstrapMemoryInitFailed,
    SessionCreateFailed,
    SessionLoadFailed,
    SessionSaveFailed,
    SessionListFailed,
    SessionIndexFailed,
    SessionNotFound,
    ToolNotFound,
    ToolInputInvalid,
    ToolExecutionFailed,
    ToolTimeout,
    ToolPermissionDenied,
    McpServerUnavailable,
    McpServerConfigInvalid,
    McpProtocolError,
    McpCapabilityDiscoveryFailed,
    McpInvocationFailed,
    ApprovalRejected,
    ApprovalTimeout,
    ApprovalChannelClosed,
    CompactionFailed,
    CompactionContextStillTooLarge,
    CommandFailed,
    UserAborted,
    ProtocolInvalidCommand,
    ProtocolParseFailed,
    ProtocolClientDisconnected,
    ProtocolStateViolation,
    InternalError,
    InternalInvariantViolation,
}

impl AionrsPublicErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::ProviderCredentialMissing => "provider_credential_missing",
            Self::ProviderAuthFailed => "provider_auth_failed",
            Self::ProviderPermissionDenied => "provider_permission_denied",
            Self::ProviderBillingRequired => "provider_billing_required",
            Self::ProviderQuotaExceeded => "provider_quota_exceeded",
            Self::ProviderRateLimited => "provider_rate_limited",
            Self::ProviderModelNotFound => "provider_model_not_found",
            Self::ProviderModelUnsupported => "provider_model_unsupported",
            Self::ProviderModelUnavailable => "provider_model_unavailable",
            Self::ProviderEndpointNotFound => "provider_endpoint_not_found",
            Self::ProviderContextTooLarge => "provider_context_too_large",
            Self::ProviderToolSchemaInvalid => "provider_tool_schema_invalid",
            Self::ProviderToolCallInvalid => "provider_tool_call_invalid",
            Self::ProviderContentBlocked => "provider_content_blocked",
            Self::ProviderTimeout => "provider_timeout",
            Self::ProviderStreamInterrupted => "provider_stream_interrupted",
            Self::ProviderTransportFailed => "provider_transport_failed",
            Self::ProviderServerError => "provider_server_error",
            Self::ProviderInvalidRequest => "provider_invalid_request",
            Self::ProviderResponseParseFailed => "provider_response_parse_failed",
            Self::ProviderEmptyResponse => "provider_empty_response",
            Self::ProviderUnknownError => "provider_unknown_error",
            Self::ConfigInvalid => "config_invalid",
            Self::ConfigProfileNotFound => "config_profile_not_found",
            Self::ConfigProviderAliasInvalid => "config_provider_alias_invalid",
            Self::ConfigEnvMissing => "config_env_missing",
            Self::ConfigFileReadFailed => "config_file_read_failed",
            Self::ConfigFileParseFailed => "config_file_parse_failed",
            Self::ConfigFileWriteFailed => "config_file_write_failed",
            Self::BootstrapFailed => "bootstrap_failed",
            Self::BootstrapProviderInitFailed => "bootstrap_provider_init_failed",
            Self::BootstrapToolInitFailed => "bootstrap_tool_init_failed",
            Self::BootstrapMcpInitFailed => "bootstrap_mcp_init_failed",
            Self::BootstrapMemoryInitFailed => "bootstrap_memory_init_failed",
            Self::SessionCreateFailed => "session_create_failed",
            Self::SessionLoadFailed => "session_load_failed",
            Self::SessionSaveFailed => "session_save_failed",
            Self::SessionListFailed => "session_list_failed",
            Self::SessionIndexFailed => "session_index_failed",
            Self::SessionNotFound => "session_not_found",
            Self::ToolNotFound => "tool_not_found",
            Self::ToolInputInvalid => "tool_input_invalid",
            Self::ToolExecutionFailed => "tool_execution_failed",
            Self::ToolTimeout => "tool_timeout",
            Self::ToolPermissionDenied => "tool_permission_denied",
            Self::McpServerUnavailable => "mcp_server_unavailable",
            Self::McpServerConfigInvalid => "mcp_server_config_invalid",
            Self::McpProtocolError => "mcp_protocol_error",
            Self::McpCapabilityDiscoveryFailed => "mcp_capability_discovery_failed",
            Self::McpInvocationFailed => "mcp_invocation_failed",
            Self::ApprovalRejected => "approval_rejected",
            Self::ApprovalTimeout => "approval_timeout",
            Self::ApprovalChannelClosed => "approval_channel_closed",
            Self::CompactionFailed => "compaction_failed",
            Self::CompactionContextStillTooLarge => "compaction_context_still_too_large",
            Self::CommandFailed => "command_failed",
            Self::UserAborted => "user_aborted",
            Self::ProtocolInvalidCommand => "protocol_invalid_command",
            Self::ProtocolParseFailed => "protocol_parse_failed",
            Self::ProtocolClientDisconnected => "protocol_client_disconnected",
            Self::ProtocolStateViolation => "protocol_state_violation",
            Self::InternalError => "internal_error",
            Self::InternalInvariantViolation => "internal_invariant_violation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum AionrsPublicErrorOwnership {
    User,
    Provider,
    Aionrs,
    Host,
    Tool,
    McpServer,
    Unknown,
}

impl AionrsPublicErrorOwnership {
    fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Provider => "provider",
            Self::Aionrs => "aionrs",
            Self::Host => "host",
            Self::Tool => "tool",
            Self::McpServer => "mcp_server",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum AionrsPublicErrorDetailKey {
    Provider,
    Model,
    Status,
    RequestId,
    Phase,
    ConfigKey,
    Profile,
    SessionId,
    ToolName,
    McpServer,
    McpCapability,
    Command,
    RawCode,
    RawType,
}

impl AionrsPublicErrorDetailKey {
    fn as_str(self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::Model => "model",
            Self::Status => "status",
            Self::RequestId => "request_id",
            Self::Phase => "phase",
            Self::ConfigKey => "config_key",
            Self::Profile => "profile",
            Self::SessionId => "session_id",
            Self::ToolName => "tool_name",
            Self::McpServer => "mcp_server",
            Self::McpCapability => "mcp_capability",
            Self::Command => "command",
            Self::RawCode => "raw_code",
            Self::RawType => "raw_type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AionrsPublicErrorDetail {
    pub(super) key: AionrsPublicErrorDetailKey,
    pub(super) value: String,
}

impl AionrsPublicErrorDetail {
    pub(super) fn new(key: AionrsPublicErrorDetailKey, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

pub(super) fn aionrs_public_error_to_send_error(error: &AionrsPublicError) -> AgentSendError {
    let detail = aionrs_public_error_detail(error);

    match error.code {
        AionrsPublicErrorCode::ProviderCredentialMissing | AionrsPublicErrorCode::ProviderAuthFailed => {
            provider_send_error(
                "The model provider rejected the request",
                AgentErrorCode::UserLlmProviderAuthFailed,
                detail,
                false,
                AgentErrorResolutionKind::CheckProviderCredentials,
                Some(AgentErrorResolutionTarget::ProviderSettings),
            )
        }
        AionrsPublicErrorCode::ProviderPermissionDenied => provider_send_error(
            "The model provider denied access to the request",
            AgentErrorCode::UserLlmProviderPermissionDenied,
            detail,
            false,
            AgentErrorResolutionKind::CheckProviderCredentials,
            Some(AgentErrorResolutionTarget::ProviderSettings),
        ),
        AionrsPublicErrorCode::ProviderBillingRequired | AionrsPublicErrorCode::ProviderQuotaExceeded => {
            provider_send_error(
                "The model provider account requires billing attention",
                AgentErrorCode::UserLlmProviderBillingRequired,
                detail,
                false,
                AgentErrorResolutionKind::CheckProviderBilling,
                Some(AgentErrorResolutionTarget::ProviderSettings),
            )
        }
        AionrsPublicErrorCode::ProviderRateLimited => provider_send_error(
            "The model provider throttled the request",
            AgentErrorCode::UserLlmProviderRateLimited,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ProviderModelNotFound => provider_send_error(
            "The configured model was not found by the provider",
            AgentErrorCode::UserLlmProviderModelNotFound,
            detail,
            false,
            AgentErrorResolutionKind::ChangeModel,
            Some(AgentErrorResolutionTarget::ProviderSettings),
        ),
        AionrsPublicErrorCode::ProviderModelUnsupported => provider_send_error(
            "The configured model does not support this request",
            AgentErrorCode::UserLlmProviderUnsupportedModel,
            detail,
            false,
            AgentErrorResolutionKind::ChangeModel,
            Some(AgentErrorResolutionTarget::ProviderSettings),
        ),
        AionrsPublicErrorCode::ProviderModelUnavailable => provider_send_error(
            "The model provider returned a server error",
            AgentErrorCode::UserLlmProviderGatewayError,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ProviderEndpointNotFound => provider_send_error(
            "The model provider endpoint was not found",
            AgentErrorCode::UserLlmProviderEndpointNotFound,
            detail,
            false,
            AgentErrorResolutionKind::CheckProviderBaseUrl,
            Some(AgentErrorResolutionTarget::ProviderSettings),
        ),
        AionrsPublicErrorCode::ProviderContextTooLarge | AionrsPublicErrorCode::CompactionContextStillTooLarge => {
            provider_send_error(
                "The request is too large for the configured model context window",
                AgentErrorCode::UserLlmProviderContextTooLarge,
                detail,
                false,
                AgentErrorResolutionKind::ReduceContext,
                None,
            )
        }
        AionrsPublicErrorCode::ProviderToolSchemaInvalid => provider_send_error(
            "The model provider rejected an internal tool schema",
            AgentErrorCode::UserLlmProviderInvalidToolSchema,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ProviderToolCallInvalid => provider_send_error(
            "The model provider returned invalid tool calls",
            AgentErrorCode::UserLlmProviderInvalidRequest,
            detail,
            false,
            AgentErrorResolutionKind::ChangeModel,
            Some(AgentErrorResolutionTarget::ProviderSettings),
        ),
        AionrsPublicErrorCode::ProviderInvalidRequest => provider_send_error(
            "The model provider rejected the request",
            AgentErrorCode::UserLlmProviderInvalidRequest,
            detail,
            false,
            AgentErrorResolutionKind::SendFeedback,
            Some(AgentErrorResolutionTarget::Feedback),
        ),
        AionrsPublicErrorCode::ProviderContentBlocked => provider_send_error(
            "The model provider blocked the response content",
            AgentErrorCode::UserLlmProviderInvalidRequest,
            detail,
            false,
            AgentErrorResolutionKind::SendFeedback,
            Some(AgentErrorResolutionTarget::Feedback),
        ),
        AionrsPublicErrorCode::ProviderTimeout => provider_send_error(
            "The model provider did not respond in time",
            AgentErrorCode::UserLlmProviderTimeout,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ProviderStreamInterrupted => provider_send_error(
            "The model provider stream was interrupted",
            AgentErrorCode::UserLlmProviderNetworkError,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ProviderTransportFailed => provider_send_error(
            "The model provider could not be reached",
            AgentErrorCode::UserLlmProviderNetworkError,
            detail,
            true,
            AgentErrorResolutionKind::CheckProviderBaseUrl,
            Some(AgentErrorResolutionTarget::ProviderSettings),
        ),
        AionrsPublicErrorCode::ProviderEmptyResponse => provider_send_error(
            "The model provider returned an empty response",
            AgentErrorCode::UserLlmProviderEmptyResponse,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ProviderServerError
        | AionrsPublicErrorCode::ProviderResponseParseFailed
        | AionrsPublicErrorCode::ProviderUnknownError => provider_send_error(
            "The model provider returned a server error",
            AgentErrorCode::UserLlmProviderGatewayError,
            detail,
            true,
            AgentErrorResolutionKind::Retry,
            None,
        ),
        AionrsPublicErrorCode::ConfigInvalid
        | AionrsPublicErrorCode::ConfigProfileNotFound
        | AionrsPublicErrorCode::ConfigProviderAliasInvalid
        | AionrsPublicErrorCode::ConfigEnvMissing
        | AionrsPublicErrorCode::ConfigFileReadFailed
        | AionrsPublicErrorCode::ConfigFileParseFailed
        | AionrsPublicErrorCode::ConfigFileWriteFailed
        | AionrsPublicErrorCode::BootstrapFailed
        | AionrsPublicErrorCode::BootstrapProviderInitFailed
        | AionrsPublicErrorCode::BootstrapToolInitFailed
        | AionrsPublicErrorCode::BootstrapMcpInitFailed
        | AionrsPublicErrorCode::BootstrapMemoryInitFailed
        | AionrsPublicErrorCode::SessionCreateFailed
        | AionrsPublicErrorCode::SessionLoadFailed
        | AionrsPublicErrorCode::SessionSaveFailed
        | AionrsPublicErrorCode::SessionListFailed
        | AionrsPublicErrorCode::SessionIndexFailed
        | AionrsPublicErrorCode::SessionNotFound
        | AionrsPublicErrorCode::ToolNotFound
        | AionrsPublicErrorCode::ToolInputInvalid
        | AionrsPublicErrorCode::ToolExecutionFailed
        | AionrsPublicErrorCode::ToolTimeout
        | AionrsPublicErrorCode::ToolPermissionDenied
        | AionrsPublicErrorCode::McpServerUnavailable
        | AionrsPublicErrorCode::McpServerConfigInvalid
        | AionrsPublicErrorCode::McpProtocolError
        | AionrsPublicErrorCode::McpCapabilityDiscoveryFailed
        | AionrsPublicErrorCode::McpInvocationFailed
        | AionrsPublicErrorCode::ApprovalRejected
        | AionrsPublicErrorCode::ApprovalTimeout
        | AionrsPublicErrorCode::ApprovalChannelClosed
        | AionrsPublicErrorCode::CompactionFailed
        | AionrsPublicErrorCode::CommandFailed
        | AionrsPublicErrorCode::UserAborted
        | AionrsPublicErrorCode::ProtocolInvalidCommand
        | AionrsPublicErrorCode::ProtocolParseFailed
        | AionrsPublicErrorCode::ProtocolClientDisconnected
        | AionrsPublicErrorCode::ProtocolStateViolation
        | AionrsPublicErrorCode::InternalError
        | AionrsPublicErrorCode::InternalInvariantViolation => unknown_upstream_send_error(detail),
    }
}

fn aionrs_engine_error_to_public_error(error: &AionrsAgentError) -> AionrsPublicError {
    match error {
        AionrsAgentError::Provider(provider_error) => aionrs_provider_error_to_public_error(provider_error),
        AionrsAgentError::RepeatedMalformedToolCall { count, limit } => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderToolCallInvalid,
            message: "The model provider repeatedly returned malformed tool calls".to_owned(),
            ownership: AionrsPublicErrorOwnership::Provider,
            details: vec![
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::Phase, "tool_call_decode"),
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::RawType, "repeated_malformed_tool_call"),
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::RawCode, format!("{count}/{limit}")),
            ],
        },
        AionrsAgentError::ContextTooLong { input_tokens, limit } => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderContextTooLarge,
            message: "The request is too large for the configured model context window".to_owned(),
            ownership: AionrsPublicErrorOwnership::User,
            details: vec![
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::Phase, "request_build"),
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::RawCode, format!("{input_tokens}/{limit}")),
            ],
        },
        AionrsAgentError::ApiError(message) => AionrsPublicError {
            code: AionrsPublicErrorCode::InternalError,
            message: message.clone(),
            ownership: AionrsPublicErrorOwnership::Aionrs,
            details: vec![AionrsPublicErrorDetail::new(
                AionrsPublicErrorDetailKey::RawType,
                "api_error",
            )],
        },
        AionrsAgentError::UserAborted => AionrsPublicError {
            code: AionrsPublicErrorCode::UserAborted,
            message: "The user aborted the operation".to_owned(),
            ownership: AionrsPublicErrorOwnership::User,
            details: vec![],
        },
    }
}

fn aionrs_provider_error_to_public_error(error: &ProviderError) -> AionrsPublicError {
    match error {
        ProviderError::Api { status, message } => AionrsPublicError {
            code: aionrs_provider_status_to_public_code(*status),
            message: message.clone(),
            ownership: AionrsPublicErrorOwnership::Provider,
            details: vec![
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::Status, status.to_string()),
                AionrsPublicErrorDetail::new(AionrsPublicErrorDetailKey::RawType, "provider_api"),
            ],
        },
        ProviderError::RateLimited { retry_after_ms } => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderRateLimited,
            message: "The model provider throttled the request".to_owned(),
            ownership: AionrsPublicErrorOwnership::Provider,
            details: vec![AionrsPublicErrorDetail::new(
                AionrsPublicErrorDetailKey::RawCode,
                retry_after_ms.to_string(),
            )],
        },
        ProviderError::PromptTooLong(message) => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderContextTooLarge,
            message: message.clone(),
            ownership: AionrsPublicErrorOwnership::User,
            details: vec![AionrsPublicErrorDetail::new(
                AionrsPublicErrorDetailKey::RawType,
                "prompt_too_long",
            )],
        },
        ProviderError::Connection(message) => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderTransportFailed,
            message: message.clone(),
            ownership: AionrsPublicErrorOwnership::Provider,
            details: vec![AionrsPublicErrorDetail::new(
                AionrsPublicErrorDetailKey::RawType,
                "provider_connection",
            )],
        },
        ProviderError::Http(error) => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderTransportFailed,
            message: error.to_string(),
            ownership: AionrsPublicErrorOwnership::Provider,
            details: vec![AionrsPublicErrorDetail::new(
                AionrsPublicErrorDetailKey::RawType,
                "provider_http",
            )],
        },
        ProviderError::Parse(message) => AionrsPublicError {
            code: AionrsPublicErrorCode::ProviderResponseParseFailed,
            message: message.clone(),
            ownership: AionrsPublicErrorOwnership::Provider,
            details: vec![AionrsPublicErrorDetail::new(
                AionrsPublicErrorDetailKey::RawType,
                "provider_parse",
            )],
        },
    }
}

fn aionrs_provider_status_to_public_code(status: u16) -> AionrsPublicErrorCode {
    match status {
        400 => AionrsPublicErrorCode::ProviderInvalidRequest,
        401 => AionrsPublicErrorCode::ProviderAuthFailed,
        402 => AionrsPublicErrorCode::ProviderBillingRequired,
        403 => AionrsPublicErrorCode::ProviderPermissionDenied,
        404 => AionrsPublicErrorCode::ProviderEndpointNotFound,
        408 | 504 => AionrsPublicErrorCode::ProviderTimeout,
        429 => AionrsPublicErrorCode::ProviderRateLimited,
        500..=599 => AionrsPublicErrorCode::ProviderServerError,
        _ => AionrsPublicErrorCode::ProviderUnknownError,
    }
}

fn aionrs_public_error_detail(error: &AionrsPublicError) -> String {
    let mut parts = vec![
        format!("Aionrs public error: code={}", error.code.as_str()),
        format!("ownership={}", error.ownership.as_str()),
    ];

    if !error.message.is_empty() {
        parts.push(format!("message={}", error.message));
    }

    for detail in &error.details {
        parts.push(format!("{}={}", detail.key.as_str(), detail.value));
    }

    parts.join("; ")
}

fn provider_send_error(
    message: &'static str,
    code: AgentErrorCode,
    detail: String,
    retryable: bool,
    resolution_kind: AgentErrorResolutionKind,
    resolution_target: Option<AgentErrorResolutionTarget>,
) -> AgentSendError {
    AgentSendError::new(
        message,
        code,
        AgentErrorOwnership::UserLlmProvider,
        Some(detail),
        retryable,
        false,
        Some(AgentErrorResolution::new(resolution_kind, resolution_target)),
    )
}

fn unknown_upstream_send_error(detail: String) -> AgentSendError {
    AgentSendError::new(
        "The upstream Agent failed while handling the request",
        AgentErrorCode::UnknownUpstreamError,
        AgentErrorOwnership::UnknownUpstream,
        Some(detail),
        true,
        true,
        Some(AgentErrorResolution::new(
            AgentErrorResolutionKind::SendFeedback,
            Some(AgentErrorResolutionTarget::Feedback),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public_error(
        code: AionrsPublicErrorCode,
        message: &str,
        ownership: AionrsPublicErrorOwnership,
        details: Vec<AionrsPublicErrorDetail>,
    ) -> AionrsPublicError {
        AionrsPublicError {
            code,
            message: message.to_owned(),
            ownership,
            details,
        }
    }

    fn public_detail(key: AionrsPublicErrorDetailKey, value: &str) -> AionrsPublicErrorDetail {
        AionrsPublicErrorDetail::new(key, value)
    }

    #[test]
    fn aionrs_structured_malformed_tool_call_error_is_provider_error() {
        let error = AionrsAgentError::RepeatedMalformedToolCall { count: 3, limit: 3 };
        let send_error = aionrs_engine_error_to_send_error(&error);

        assert_eq!(
            send_error.code(),
            Some(aionui_api_types::AgentErrorCode::UserLlmProviderInvalidRequest)
        );
        assert_eq!(
            send_error.ownership(),
            Some(aionui_api_types::AgentErrorOwnership::UserLlmProvider)
        );
        assert_eq!(send_error.stream_error().retryable, Some(false));
    }

    #[test]
    fn aionrs_provider_connection_error_is_user_llm_provider_error() {
        let error = AionrsAgentError::Provider(ProviderError::Connection(
            "Signable request error: failed to create canonical request".to_owned(),
        ));
        let send_error = aionrs_engine_error_to_send_error(&error);

        assert_eq!(
            send_error.code(),
            Some(aionui_api_types::AgentErrorCode::UserLlmProviderNetworkError)
        );
        assert_eq!(
            send_error.ownership(),
            Some(aionui_api_types::AgentErrorOwnership::UserLlmProvider)
        );
        assert_eq!(send_error.stream_error().retryable, Some(true));
    }

    #[test]
    fn aionrs_api_connection_error_is_user_llm_provider_network_error() {
        let error = AionrsAgentError::Provider(ProviderError::Connection("error decoding response body".to_owned()));
        let send_error = aionrs_engine_error_to_send_error(&error);

        assert_eq!(
            send_error.code(),
            Some(aionui_api_types::AgentErrorCode::UserLlmProviderNetworkError)
        );
        assert_eq!(
            send_error.ownership(),
            Some(aionui_api_types::AgentErrorOwnership::UserLlmProvider)
        );
        assert_eq!(send_error.stream_error().retryable, Some(true));
    }

    #[test]
    fn aionrs_provider_status_error_uses_status_instead_of_message_text() {
        let error = AionrsAgentError::Provider(ProviderError::Api {
            status: 401,
            message: "credentials failed".to_owned(),
        });
        let send_error = aionrs_engine_error_to_send_error(&error);

        assert_eq!(
            send_error.code(),
            Some(aionui_api_types::AgentErrorCode::UserLlmProviderAuthFailed)
        );
        assert_eq!(
            send_error.ownership(),
            Some(aionui_api_types::AgentErrorOwnership::UserLlmProvider)
        );
        assert_eq!(send_error.stream_error().retryable, Some(false));
    }

    #[test]
    fn aionrs_context_too_long_is_provider_context_error() {
        let error = AionrsAgentError::ContextTooLong {
            input_tokens: 120_000,
            limit: 100_000,
        };
        let send_error = aionrs_engine_error_to_send_error(&error);

        assert_eq!(
            send_error.code(),
            Some(aionui_api_types::AgentErrorCode::UserLlmProviderContextTooLarge)
        );
        assert_eq!(
            send_error.ownership(),
            Some(aionui_api_types::AgentErrorOwnership::UserLlmProvider)
        );
        assert_eq!(send_error.stream_error().retryable, Some(false));
    }

    #[test]
    fn aionrs_repeated_malformed_tool_call_is_user_llm_provider_error() {
        let error = AionrsAgentError::RepeatedMalformedToolCall { count: 3, limit: 3 };
        let send_error = aionrs_engine_error_to_send_error(&error);

        assert_eq!(
            send_error.code(),
            Some(aionui_api_types::AgentErrorCode::UserLlmProviderInvalidRequest)
        );
        assert_eq!(
            send_error.ownership(),
            Some(aionui_api_types::AgentErrorOwnership::UserLlmProvider)
        );
        assert_eq!(send_error.stream_error().retryable, Some(false));
    }

    #[test]
    fn aionrs_public_provider_auth_maps_without_message_classification() {
        let error = public_error(
            AionrsPublicErrorCode::ProviderAuthFailed,
            "not auth",
            AionrsPublicErrorOwnership::Provider,
            vec![public_detail(AionrsPublicErrorDetailKey::Provider, "openai")],
        );

        let send_error = aionrs_public_error_to_send_error(&error);

        assert_eq!(send_error.code(), Some(AgentErrorCode::UserLlmProviderAuthFailed));
        assert_eq!(send_error.ownership(), Some(AgentErrorOwnership::UserLlmProvider));
        assert_eq!(send_error.stream_error().retryable, Some(false));
        assert_eq!(
            send_error.stream_error().resolution.map(|value| value.kind),
            Some(AgentErrorResolutionKind::CheckProviderCredentials)
        );
        assert_eq!(
            send_error.stream_error().resolution.and_then(|value| value.target),
            Some(AgentErrorResolutionTarget::ProviderSettings)
        );
    }

    #[test]
    fn aionrs_public_provider_rate_limited_maps_retryable() {
        let error = public_error(
            AionrsPublicErrorCode::ProviderRateLimited,
            "provider unavailable",
            AionrsPublicErrorOwnership::Provider,
            vec![],
        );

        let send_error = aionrs_public_error_to_send_error(&error);

        assert_eq!(send_error.code(), Some(AgentErrorCode::UserLlmProviderRateLimited));
        assert_eq!(send_error.ownership(), Some(AgentErrorOwnership::UserLlmProvider));
        assert_eq!(send_error.stream_error().retryable, Some(true));
        assert_eq!(
            send_error.stream_error().resolution.map(|value| value.kind),
            Some(AgentErrorResolutionKind::Retry)
        );
    }

    #[test]
    fn aionrs_public_provider_stream_interrupted_maps_network_retry() {
        let error = public_error(
            AionrsPublicErrorCode::ProviderStreamInterrupted,
            "provider interrupted after headers",
            AionrsPublicErrorOwnership::Provider,
            vec![],
        );

        let send_error = aionrs_public_error_to_send_error(&error);

        assert_eq!(send_error.code(), Some(AgentErrorCode::UserLlmProviderNetworkError));
        assert_eq!(send_error.ownership(), Some(AgentErrorOwnership::UserLlmProvider));
        assert_eq!(send_error.stream_error().retryable, Some(true));
        assert_eq!(
            send_error.stream_error().resolution.map(|value| value.kind),
            Some(AgentErrorResolutionKind::Retry)
        );
    }

    #[test]
    fn aionrs_public_provider_tool_call_invalid_maps_invalid_request() {
        let error = public_error(
            AionrsPublicErrorCode::ProviderToolCallInvalid,
            "provider tool call invalid",
            AionrsPublicErrorOwnership::Provider,
            vec![],
        );

        let send_error = aionrs_public_error_to_send_error(&error);

        assert_eq!(send_error.code(), Some(AgentErrorCode::UserLlmProviderInvalidRequest));
        assert_eq!(send_error.ownership(), Some(AgentErrorOwnership::UserLlmProvider));
        assert_eq!(send_error.stream_error().retryable, Some(false));
    }

    #[test]
    fn aionrs_public_mcp_invocation_maps_to_unknown_upstream_without_tool_confusion() {
        let error = public_error(
            AionrsPublicErrorCode::McpInvocationFailed,
            "mcp failed",
            AionrsPublicErrorOwnership::McpServer,
            vec![
                public_detail(AionrsPublicErrorDetailKey::McpServer, "filesystem"),
                public_detail(AionrsPublicErrorDetailKey::McpCapability, "read_file"),
            ],
        );

        let send_error = aionrs_public_error_to_send_error(&error);
        let detail = send_error.stream_error().detail.as_deref().unwrap_or_default();

        assert_eq!(send_error.code(), Some(AgentErrorCode::UnknownUpstreamError));
        assert_eq!(send_error.ownership(), Some(AgentErrorOwnership::UnknownUpstream));
        assert!(detail.find("mcp_invocation_failed").is_some());
        assert!(detail.find("mcp_capability=read_file").is_some());
        assert!(detail.find("mcp_server=filesystem").is_some());
        let forbidden_detail_key = ["mcp", "tool"].join("_");
        assert!(detail.find(&forbidden_detail_key).is_none());
    }

    #[test]
    fn aionrs_public_command_failed_maps_unknown_upstream_or_user_agent() {
        let error = public_error(
            AionrsPublicErrorCode::CommandFailed,
            "command failed",
            AionrsPublicErrorOwnership::Host,
            vec![public_detail(AionrsPublicErrorDetailKey::Command, "aion run")],
        );

        let send_error = aionrs_public_error_to_send_error(&error);

        assert_eq!(send_error.code(), Some(AgentErrorCode::UnknownUpstreamError));
        assert_eq!(send_error.ownership(), Some(AgentErrorOwnership::UnknownUpstream));
        assert_eq!(send_error.stream_error().retryable, Some(true));
    }
}
