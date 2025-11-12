pub static MEMORY_RETRIEVE_PART_PROMPT: &str = r#"
# RELATED MEMORIES
The following memories and conversation snippets have been retrieved dynamically from your memory and may be relevant to the current conversation:
{{memory_retrieved}}
"#;
