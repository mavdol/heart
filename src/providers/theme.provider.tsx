import {
  createContext,
  useState,
  useEffect,
  useContext,
  useRef,
  useCallback,
  useMemo,
} from "react";
import { Store } from "@tauri-apps/plugin-store";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface ThemeContextValue {
  theme: string;
  isLoading: boolean;
  setTheme: (mode: string) => Promise<void>;
  isDark: boolean;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState("light");
  const [isLoading, setIsLoading] = useState(true);
  const storeRef = useRef<Store | null>(null);

  useEffect(() => {
    const initTheme = async () => {
      try {
        if (!storeRef.current) {
          storeRef.current = await Store.load("settings.json");
        }

        const savedTheme = (await storeRef.current.get("theme")) as
          | string
          | null;

        if (savedTheme && savedTheme !== "system") {
          setTheme(savedTheme);
        } else {
          const appWindow = getCurrentWindow();
          const theme = await appWindow.theme();
          setTheme(theme as string);
        }
      } catch (error) {
        console.error("Failed to load theme:", error);
      } finally {
        setIsLoading(false);
      }
    };

    initTheme();
  }, []);

  useEffect(() => {
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [theme]);

  const setThemeMode = useCallback(async (mode: string) => {
    setTheme(mode);

    try {
      if (storeRef.current) {
        await storeRef.current.set("theme", mode);
        await storeRef.current.save();
      }
    } catch (error) {
      console.error("Failed to save theme:", error);
    }
  }, []);

  const contextValue = useMemo(
    () => ({
      theme,
      isLoading,
      setTheme: setThemeMode,
      isDark: theme === "dark",
    }),
    [theme, isLoading, setThemeMode]
  );

  return (
    <ThemeContext.Provider value={contextValue}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}
