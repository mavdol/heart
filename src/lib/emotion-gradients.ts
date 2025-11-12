import { EmotionState } from "@/interfaces/emotion.interface";

interface GradientColors {
  light: string[];
  dark: string[];
}

const LOW_VALENCE_LOW_AROUSAL: GradientColors = {
  light: ["#6B7280", "#9CA3AF", "#4B5563", "#8B92A0"],
  dark: ["#E5E7EB", "#D1D5DB", "#F3F4F6", "#C9CDD3"],
};

const LOW_VALENCE_HIGH_AROUSAL: GradientColors = {
  light: ["#8B4747", "#D47474", "#6B3939", "#B86A6A"],
  dark: ["#ffbfbf", "#ff8787", "#ffd4d4", "#ffafaf"],
};

const HIGH_VALENCE_HIGH_AROUSAL: GradientColors = {
  light: ["#47A86B", "#74D4A5", "#39855B", "#6AB88F"],
  dark: ["#c2ffdf", "#87ffb8", "#d4fff0", "#afffd4"],
};

const HIGH_VALENCE_LOW_AROUSAL: GradientColors = {
  light: ["#4788A8", "#74B5D4", "#396985", "#6A9AB8"],
  dark: ["#c2e8ff", "#87d4ff", "#d4f0ff", "#afdeff"],
};

function interpolateColor(
  color1: string,
  color2: string,
  factor: number
): string {
  const c1 = parseInt(color1.slice(1), 16);
  const c2 = parseInt(color2.slice(1), 16);

  const r1 = (c1 >> 16) & 0xff;
  const g1 = (c1 >> 8) & 0xff;
  const b1 = c1 & 0xff;

  const r2 = (c2 >> 16) & 0xff;
  const g2 = (c2 >> 8) & 0xff;
  const b2 = c2 & 0xff;

  const r = Math.round(r1 + (r2 - r1) * factor);
  const g = Math.round(g1 + (g2 - g1) * factor);
  const b = Math.round(b1 + (b2 - b1) * factor);

  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

function interpolateGradients(
  gradient1: string[],
  gradient2: string[],
  factor: number
): string[] {
  const length = Math.max(gradient1.length, gradient2.length);
  const result: string[] = [];

  for (let i = 0; i < length; i++) {
    const color1 = gradient1[i % gradient1.length];
    const color2 = gradient2[i % gradient2.length];
    result.push(interpolateColor(color1, color2, factor));
  }

  return result;
}

export function getEmotionGradient(
  emotion: EmotionState,
  isDark: boolean
): string[] {
  const { valence, arousal } = emotion;
  const theme = isDark ? "light" : "dark";

  const isPositiveValence = valence > 0;
  const isHighArousal = arousal > 0;

  let baseGradient: string[];
  let targetGradient: string[] | null = null;
  let interpolationFactor = 0;

  if (!isPositiveValence && !isHighArousal) {
    baseGradient = LOW_VALENCE_LOW_AROUSAL[theme];
  } else if (!isPositiveValence && isHighArousal) {
    const arousalIntensity = Math.min(Math.abs(arousal), 1);
    baseGradient = LOW_VALENCE_LOW_AROUSAL[theme];
    targetGradient = LOW_VALENCE_HIGH_AROUSAL[theme];
    interpolationFactor = arousalIntensity;
  } else if (isPositiveValence && isHighArousal) {
    const intensityValence = Math.min(Math.abs(valence), 1);
    const intensityArousal = Math.min(Math.abs(arousal), 1);
    const intensity = (intensityValence + intensityArousal) / 2;
    baseGradient = LOW_VALENCE_LOW_AROUSAL[theme];
    targetGradient = HIGH_VALENCE_HIGH_AROUSAL[theme];
    interpolationFactor = intensity;
  } else {
    const intensityValence = Math.min(Math.abs(valence), 1);
    baseGradient = LOW_VALENCE_LOW_AROUSAL[theme];
    targetGradient = HIGH_VALENCE_LOW_AROUSAL[theme];
    interpolationFactor = intensityValence;
  }

  if (targetGradient) {
    return interpolateGradients(
      baseGradient,
      targetGradient,
      interpolationFactor
    );
  }

  return baseGradient;
}
