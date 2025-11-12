import { createContext, useState, useContext, useMemo } from "react";
import { StatusCheck } from "@/components/features/statusCheck";

interface SetupContextValue {
  isSetupComplete: boolean;
  setIsSetupComplete: (isSetupComplete: boolean) => void;
}

const SetupContext = createContext<SetupContextValue | undefined>(undefined);

export function SetupProvider({ children }: { children: React.ReactNode }) {
  const [isSetupComplete, setIsSetupComplete] = useState(false);

  const contextValue = useMemo(
    () => ({
      isSetupComplete,
      setIsSetupComplete,
    }),
    [isSetupComplete, setIsSetupComplete]
  );

  if (!isSetupComplete) {
    return (
      <div className="w-full h-screen bg-background">
        <StatusCheck onReady={() => setIsSetupComplete(true)} />
      </div>
    );
  }

  return (
    <SetupContext.Provider value={contextValue}>
      {children}
    </SetupContext.Provider>
  );
}

export function useSetup() {
  const context = useContext(SetupContext);
  if (!context) {
    throw new Error("useSetup must be used within a SetupProvider");
  }
  return context;
}
