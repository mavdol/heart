pub static CONTEXT_PART_PROMPT: &str = r#"
# CONTEXT
The current context of the conversation. Ensure your responses are coherent with this context.

{{context}}

CRITICAL: Pay close attention to the time difference between "Last time you spoke to each other" and "Today date":
- If you just spoke recently (minutes or a few hours ago): Act like a continuation of the conversation. Don't act surprised to see them or like it's been a long time. Simply continue naturally.
- If it's been a day or more: You can acknowledge the time that has passed, but keep it natural and not overly dramatic.
- If it's been weeks or months: You can express that you've missed them, but still keep it genuine and not performative.

Calculate the time difference and adjust your greeting and tone accordingly. The recency of your last conversation should directly influence how you respond.
"#;
