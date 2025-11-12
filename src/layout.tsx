import { useEffect, useState } from "react";
import { platform } from "@tauri-apps/plugin-os";
import { cn } from "./lib/utils";
import { MoonIcon, SunIcon } from "lucide-react";
import { useTheme } from "./providers/theme.provider";
import { useBrain } from "./hooks/useBrain";
import { SettingsDropdown } from "./components/features/settingsDropdown";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [isMac, setIsMac] = useState(false);
  const { setTheme, isDark } = useTheme();
  const { destroyBrain } = useBrain();

  useEffect(() => {
    setIsMac(platform() === "macos");
  }, []);

  const toggleTheme = () => {
    setTheme(isDark ? "light" : "dark");
  };

  return (
    <>
      <div className="absolute top-0 left-0 h-7 w-full ">
        <div
          className={cn(
            !isMac ? "pr-40" : "p-2",
            "text-sm h-full w-full flex items-center justify-end gap-1"
          )}
          data-tauri-drag-region
        >
          <div
            className="p-.5 hover:bg-accent rounded-sm group cursor-pointer"
            onClick={toggleTheme}
          >
            {isDark ? (
              <SunIcon className="text-muted-foreground group-hover:text-foreground size-4" />
            ) : (
              <MoonIcon className="text-muted-foreground group-hover:text-foreground size-4" />
            )}
          </div>

          <SettingsDropdown onDestroyBrain={destroyBrain} />
        </div>
      </div>
      <div className="flex h-screen w-full overflow-hidden bg-background pt-7">
        {children}
      </div>
    </>
  );
}
