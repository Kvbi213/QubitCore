use qubit_protocol::{ChatMessage, MessageRole};
use uuid::Uuid;

pub struct ContextCompressor;

impl ContextCompressor {
    pub fn compress(messages: Vec<ChatMessage>, max_chars: usize) -> Vec<ChatMessage> {
        let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        if total_chars <= max_chars / 2 || messages.len() <= 20 {
            return messages;
        }

        let split_idx = messages.len() - 20;
        let (head, tail) = messages.split_at(split_idx);
        let summary_budget = (max_chars / 5).max(128);

        let mut summary = String::from("[COMPRESSED HISTORY SUMMARY]\n");
        for m in head {
            let line = format!("- {:?}: {}\n", m.role, m.content.chars().take(80).collect::<String>());
            if summary.len() + line.len() > summary_budget {
                break;
            }
            summary.push_str(&line);
        }

        let session_id = tail.first().map(|m| m.session_id).unwrap_or_else(Uuid::new_v4);
        let tainted = head.iter().any(|m| m.tainted);
        let mut out = Vec::with_capacity(21);
        out.push(ChatMessage::new(session_id, MessageRole::System, summary, tainted));
        out.extend_from_slice(tail);
        out
    }
}
