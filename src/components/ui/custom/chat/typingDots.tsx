import { cn } from "@/lib/utils";

interface TypingDotsProps {
  speed?: number;
  size?: string;
  color?: string;
  gap?: number;
  delay?: number;
  className?: string;
  dotCount?: number;
}

export default function TypingDots({
  speed = 1.4,
  size = "w-2 h-2",
  color = "text-muted-foreground",
  gap = 4,
  delay = 0.2,
  className,
  dotCount = 3,
}: TypingDotsProps) {
  return (
    <div
      className={cn("flex items-center py-2 px-3", color, className)}
      style={{ gap: `${gap}px` }}
    >
      {Array.from({ length: dotCount }).map((_, index) => (
        <div
          key={index}
          className={cn("rounded-full bg-current animate-typing-bounce", size)}
          style={{
            animationDuration: `${speed}s`,
            animationDelay: `${index * delay}s`,
          }}
        />
      ))}
    </div>
  );
}
