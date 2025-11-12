import { SetupState } from "@/interfaces/app.interface";
import { cn } from "@/lib/utils";
import { Check, HelpCircle, Loader2, X, Dot } from "lucide-react";

export interface StateIconProps {
  state: SetupState;
  size: "sm" | "md" | "lg";
}
export function StateIcon({ size, state }: StateIconProps) {
  const sizeClass = {
    sm: "w-3 h-3",
    md: "w-6 h-6",
    lg: "w-8 h-8",
  };

  if (state === SetupState.Pending) {
    return <Dot className={cn(sizeClass[size])} />;
  }

  if (state === SetupState.Loading) {
    return (
      <Loader2
        className={cn(
          sizeClass[size],
          "animate-spin text-[var(--placeholder)]"
        )}
      />
    );
  }

  if (state === SetupState.Ready) {
    return <Check className={cn(sizeClass[size], "text-[var(--success)]")} />;
  }

  if (state === SetupState.Failed) {
    return <X className={cn(sizeClass[size], "text-[var(--error)]")} />;
  }

  return <HelpCircle className={cn(sizeClass[size])} />;
}
