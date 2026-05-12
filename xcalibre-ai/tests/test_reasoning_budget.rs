use xcalibre_ai::reasoning::{ReasoningBudget, apply_reasoning_budget};
use xcalibre_ai::ChatMessage;

#[test]
fn test_budget_none_adds_no_system_message() {
    let messages: Vec<ChatMessage> = vec![];
    let result = apply_reasoning_budget(messages.clone(), ReasoningBudget::None);
    assert_eq!(result.len(), messages.len());
}

#[test]
fn test_budget_auto_adds_system_message() {
    let messages: Vec<ChatMessage> = vec![];
    let result = apply_reasoning_budget(messages, ReasoningBudget::Auto);
    assert!(result.len() >= 1, "Auto budget must add at least a system message");
}

#[test]
fn test_budget_high_adds_detailed_reasoning_prompt() {
    let messages: Vec<ChatMessage> = vec![];
    let result = apply_reasoning_budget(messages, ReasoningBudget::High);
    assert!(result.iter().any(|m| {
        m.content.to_lowercase().contains("step") ||
        m.content.to_lowercase().contains("reason")
    }), "High budget must add reasoning instructions");
}

#[test]
fn test_budget_from_str() {
    assert_eq!(ReasoningBudget::from_str("none"),    Some(ReasoningBudget::None));
    assert_eq!(ReasoningBudget::from_str("auto"),    Some(ReasoningBudget::Auto));
    assert_eq!(ReasoningBudget::from_str("low"),     Some(ReasoningBudget::Low));
    assert_eq!(ReasoningBudget::from_str("medium"),  Some(ReasoningBudget::Medium));
    assert_eq!(ReasoningBudget::from_str("high"),    Some(ReasoningBudget::High));
    assert_eq!(ReasoningBudget::from_str("unknown"), None);
}
