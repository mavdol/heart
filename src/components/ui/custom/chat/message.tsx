import { useTheme } from "@/providers/theme.provider";
import AnimatedMeshGradient from "./animatedMeshGradient";
import TypingDots from "./typingDots";
import { Message as MessageInterface } from "@/interfaces/chat.interface";
import { EmotionState } from "@/interfaces/emotion.interface";
import { getEmotionGradient } from "@/lib/emotion-gradients";

export default function Message({
  message,
  currentEmotion,
  islastMessage,
}: {
  message: MessageInterface;
  currentEmotion: EmotionState;
  islastMessage: boolean;
}) {
  const { isDark } = useTheme();

  const gradientColors = getEmotionGradient(currentEmotion, isDark);

  return (
    <div
      className={`flex gap-3 items-start animate-in fade-in slide-in-from-bottom-4 duration-500 ${
        message.role === "user" ? "flex-row-reverse" : "flex-row"
      }`}
      id={islastMessage ? "last-message" : ""}
    >
      {message.role === "assistant" && (
        <div
          className="w-10 h-10 rounded-full relative overflow-hidden"
          title={`valence: ${currentEmotion.valence}, arousal: ${currentEmotion.arousal}`}
        >
          <AnimatedMeshGradient
            colors={gradientColors}
            speed={2}
            darkenTop={false}
          />
        </div>
      )}

      {/* Writing animation */}
      {message.isWriting && (
        <div className="flex items-center py-2 px-1 bg-muted/80 rounded-tl-sm  backdrop-blur-sm rounded-2xl">
          <TypingDots
            speed={1.5}
            size="w-1.5 h-1.5"
            color="text-muted-foreground/50"
            delay={0.2}
            dotCount={3}
          />
        </div>
      )}
      {!message.isWriting && (
        <div
          className={`flex flex-col max-w-[75%]  ${
            message.role === "user" ? "items-end" : "items-start"
          }`}
        >
          <div
            className={`rounded-2xl px-4 py-3 shadow-sm ${
              message.role === "user"
                ? "bg-primary text-primary-foreground rounded-tr-sm"
                : "bg-muted/80 backdrop-blur-sm rounded-tl-sm text-foreground"
            }`}
          >
            <p className="text-sm leading-relaxed whitespace-pre-wrap">
              {message.content}
            </p>
          </div>
          {/* <span className="text-xs text-muted-foreground mt-1.5 px-1">
          {message.timestamp}
        </span> */}
        </div>
      )}
    </div>
  );
}
