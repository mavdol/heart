use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionData {
    pub label: String,
    pub valence: f32,
    pub arousal: f32,
}

pub fn load_emotion_data() -> Result<Vec<EmotionData>, serde_json::Error> {
    const EMOTION_DATA: &str = include_str!("classify-emotion.json");
    serde_json::from_str(EMOTION_DATA)
}

pub fn get_emotion_label(valence: f32, arousal: f32, max_distance: f32) -> Result<String, serde_json::Error> {
    let emotions = load_emotion_data()?;

    let closest = emotions
        .into_iter()
        .map(|emotion| {
            let distance = ((emotion.valence - valence).powi(2) + (emotion.arousal - arousal).powi(2)).sqrt();
            (emotion, distance)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    match closest {
        Some((emotion, distance)) if distance <= max_distance => Ok(emotion.label),
        _ => Ok("neutral".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_emotion_label() {
        let result = get_emotion_label(0.5, -0.5, 0.7).unwrap();
        assert_eq!(result, "happy");
    }
}
