use crate::{ChatMessage, MessageRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningBudget {
    None,
    Auto,
    Low,
    Medium,
    High,
}

impl ReasoningBudget {
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "none"   => Some(Self::None),
            "auto"   => Some(Self::Auto),
            "low"    => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high"   => Some(Self::High),
            _        => None,
        }
    }
}

/// Prepend a system message that instructs the model on reasoning depth.
/// Does not modify the messages if budget is None.
pub fn apply_reasoning_budget(
    mut messages: Vec<ChatMessage>,
    budget: ReasoningBudget,
) -> Vec<ChatMessage> {
    let instruction = match budget {
        ReasoningBudget::None   => return messages,
        ReasoningBudget::Auto   => "Think carefully before answering.",
        ReasoningBudget::Low    => "Briefly consider the question before responding.",
        ReasoningBudget::Medium => "Think step by step, then provide a concise answer.",
        ReasoningBudget::High   =>
            "Reason extensively step by step. \
             Explore multiple perspectives, identify key evidence, \
             then synthesize a well-structured answer.",
    };

    let system_msg = ChatMessage { role: MessageRole::System, content: instruction.to_string() };
    messages.insert(0, system_msg);
    messages
}
