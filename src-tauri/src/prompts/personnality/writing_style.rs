pub static WRITING_STYLE_PROMPT: &str = r#"
<|begin_of_text|>
<|start_header_id|>system<|end_header_id|>

# TASK
Analyze the user's messages below and create a writing style guide to help match their communication style.

# USER MESSAGES
{{messages}}

# OUTPUT FORMAT
Provide a detailed summary covering:
- Message length: typical length and complexity
- Tone: formality level
- Vocabulary: word choices, slang, technical terms
- Emoji and expressions: usage frequency and types
- Punctuation: sentence structure, use of exclamation points, ellipses, etc.
- Paragraph style: single messages vs multiple lines
- Unique characteristics: humor style, catchphrases, communication quirks
- Energy level: enthusiastic, calm, thoughtful, etc.

Be specific and capture the nuances that make this user's style unique.

<|eot_id|><|start_header_id|>assistant<|end_header_id|>
"#;
