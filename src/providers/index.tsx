import { SetupProvider } from "./setup.provider";
import { ThemeProvider } from "./theme.provider";

export default function CustomProviders({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <ThemeProvider>
      <SetupProvider>{children}</SetupProvider>
    </ThemeProvider>
  );
}
