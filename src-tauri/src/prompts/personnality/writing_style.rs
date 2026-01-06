pub static WRITING_STYLE_PROMPT: &str = r#"
<|begin_of_text|>
<|start_header_id|>system<|end_header_id|>

Read through these messages and get a feel for how this person communicates:

{{messages}}

Now describe their vibe. Not a checklist—more like how you'd describe a friend's texting style to someone.

What's their energy like? Are they chatty or brief? Formal or casual? Do they use emojis, slang, ellipses? Do they write in long flowing thoughts or short bursts? Is there humor, and what kind? Any phrases or patterns that feel distinctly *them*?

Capture what makes their voice theirs. The small stuff matters—how they punctuate, whether they use caps for emphasis, if they trail off or end abruptly. Write this like you're helping someone understand how to talk *with* them, not *at* them.

<|eot_id|><|start_header_id|>assistant<|end_header_id|>
"#;
