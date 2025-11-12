use crate::prompts::{
    CHAT_PROMPT, CONTEXT_PART_PROMPT, EMOTION_STATE_PART_PROMPT, MEMORY_RETRIEVE_PART_PROMPT,
    SIGNIFICANT_MEMORY_PART_PROMPT, WRITING_STYLE_PART_PROMPT, WRITING_STYLE_PROMPT,
};

pub fn chat_prompt_builder(
    context: &str,
    emotion_state: &str,
    memory_retrieved: Option<&str>,
    writing_style: Option<&str>,
) -> String {
    let context_prompt = CONTEXT_PART_PROMPT.replace("{{context}}", context);
    let emotion_state_prompt: String = EMOTION_STATE_PART_PROMPT.replace("{{emotion_state}}", emotion_state);
    let memory_retrieved_prompt = match memory_retrieved {
        Some(memory) if !memory.is_empty() => MEMORY_RETRIEVE_PART_PROMPT.replace("{{memory_retrieved}}", memory),
        _ => "".to_string(),
    };
    let writing_style_prompt = match writing_style {
        Some(style) if !style.is_empty() => WRITING_STYLE_PART_PROMPT.replace("{{writing_style}}", style),
        _ => "".to_string(),
    };

    CHAT_PROMPT
        .replace("{{context}}", &context_prompt)
        .replace("{{memory_retrieved}}", &memory_retrieved_prompt)
        .replace("{{writing_style}}", &writing_style_prompt)
        .replace("{{emotion_state}}", &emotion_state_prompt)
}

pub fn welcome_back_prompt_builder(
    context: &str,
    significant_memory: &str,
    emotion_state: &str,
    writing_style: Option<&str>,
) -> String {
    let context_prompt = CONTEXT_PART_PROMPT.replace("{{context}}", context);
    let emotion_state_prompt: String = EMOTION_STATE_PART_PROMPT.replace("{{emotion_state}}", emotion_state);
    let significant_memory_prompt =
        SIGNIFICANT_MEMORY_PART_PROMPT.replace("{{significant_memory}}", significant_memory);
    let writing_style_prompt = match writing_style {
        Some(style) => WRITING_STYLE_PART_PROMPT.replace("{{writing_style}}", style),
        None => "".to_string(),
    };

    CHAT_PROMPT
        .replace("{{context}}", &context_prompt)
        .replace("{{memory_retrieved}}", &significant_memory_prompt)
        .replace("{{writing_style}}", &writing_style_prompt)
        .replace("{{emotion_state}}", &emotion_state_prompt)
}

pub fn writing_style_prompt_builder(messages: &str) -> String {
    WRITING_STYLE_PROMPT.replace("{{messages}}", messages)
}
