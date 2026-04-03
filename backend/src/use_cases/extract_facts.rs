use crate::config::PipesConfig;
use crate::models::chat::{ChatFacts, ChatMessage, MessageRole};
use crate::providers::chat_storage::ChatStorage;
use crate::providers::llm;
use arti_pipes::executor::PromptExecutor;
use chrono::Utc;

pub async fn command(
    chat_storage: ChatStorage,
    config: PipesConfig,
    messages: Vec<ChatMessage>,
    existing_facts: Option<ChatFacts>,
) {
    let prompt = build_extraction_prompt(&messages, &existing_facts);

    let executor = llm::create_executor(&config, &config.models[0].id);

    let result = match executor.execute_raw(prompt).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Fact extraction LLM error: {:?}", e);
            return;
        }
    };

    let output = match result.output.await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            eprintln!("Fact extraction output error: {:?}", e);
            return;
        }
        Err(e) => {
            eprintln!("Fact extraction join error: {:?}", e);
            return;
        }
    };

    let response_text = output.result;
    let facts = parse_facts_response(&response_text, &existing_facts);

    if let Err(e) = chat_storage.save_chat_facts(&facts).await {
        eprintln!("Failed to save chat facts: {:?}", e);
    }
}

fn build_extraction_prompt(messages: &[ChatMessage], existing: &Option<ChatFacts>) -> String {
    let mut prompt = String::from(
        "<|system|>\n\
        You are a fact extraction assistant. Analyze the conversation and produce:\n\
        1. A brief summary of the conversation (1-3 sentences)\n\
        2. A list of key facts, topics, and user preferences mentioned\n\n\
        Output format (use exactly this format):\n\
        SUMMARY: <your summary>\n\
        FACTS:\n\
        - fact 1\n\
        - fact 2\n\
        ...\n\n",
    );

    if let Some(existing) = existing {
        prompt.push_str("Previous summary: ");
        prompt.push_str(&existing.summary);
        prompt.push('\n');
        if !existing.facts.is_empty() {
            prompt.push_str("Previous facts:\n");
            for fact in &existing.facts {
                prompt.push_str("- ");
                prompt.push_str(fact);
                prompt.push('\n');
            }
        }
        prompt.push_str("\nUpdate the summary and facts based on the new messages below.\n\n");
    }

    prompt.push_str("<|conversation|>\n");

    let start = if messages.len() > 20 {
        messages.len() - 20
    } else {
        0
    };
    for msg in &messages[start..] {
        match msg.role {
            MessageRole::User => prompt.push_str(&format!("<|user|>\n{}\n", msg.content)),
            MessageRole::Assistant => {
                prompt.push_str(&format!("<|assistant|>\n{}\n", msg.content))
            }
        }
    }

    prompt
}

fn parse_facts_response(response: &str, existing: &Option<ChatFacts>) -> ChatFacts {
    let mut summary = String::new();
    let mut facts = Vec::new();
    let mut in_facts = false;

    for line in response.lines() {
        let trimmed = line.trim();
        if let Some(s) = trimmed.strip_prefix("SUMMARY:") {
            summary = s.trim().to_string();
            in_facts = false;
        } else if trimmed == "FACTS:" {
            in_facts = true;
        } else if in_facts {
            if let Some(fact) = trimmed.strip_prefix("- ") {
                if !fact.is_empty() {
                    facts.push(fact.to_string());
                }
            }
        }
    }

    if summary.is_empty() {
        if let Some(existing) = existing {
            summary = existing.summary.clone();
        }
    }
    if facts.is_empty() {
        if let Some(existing) = existing {
            facts = existing.facts.clone();
        }
    }

    ChatFacts {
        summary,
        facts,
        updated_at: Utc::now(),
    }
}
