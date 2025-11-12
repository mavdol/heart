import { SetupState } from "@/interfaces/app.interface";
import { StateIcon } from "@/components/ui/custom/stateIcon";
import { useApp } from "@/hooks/useApp";
import { cn } from "@/lib/utils";
import { useEffect, useRef } from "react";

export function StatusCheck({ onReady }: { onReady: () => void }) {
  const isChecking = useRef(false);
  const {
    ollamaInstallationStatus,
    ollamaRunningStatus,
    modelInstallationStatus,
    neuralAffectMatrixInstallationStatus,
    checkOllamaInstallationStatus,
    checkOllamaRunningStatus,
    checkModelInstallationStatus,
    checkNeuralAffectMatrixInstallationStatus,
    setModelInstallationStatus,
    modelDownloadProgress,
    ready,
  } = useApp();

  useEffect(() => {
    if (isChecking.current) return;

    if (ollamaInstallationStatus === SetupState.Pending) {
      isChecking.current = true;
      checkOllamaInstallationStatus().finally(() => {
        isChecking.current = false;
      });

      return;
    }

    if (
      ollamaRunningStatus === SetupState.Pending &&
      ollamaInstallationStatus === SetupState.Ready
    ) {
      isChecking.current = true;
      checkOllamaRunningStatus().finally(() => {
        isChecking.current = false;
      });
      return;
    }

    if (
      modelInstallationStatus === SetupState.Pending &&
      ollamaRunningStatus === SetupState.Ready
    ) {
      isChecking.current = true;
      checkModelInstallationStatus().finally(() => {
        isChecking.current = false;
      });
      return;
    }

    if (
      neuralAffectMatrixInstallationStatus === SetupState.Pending &&
      modelInstallationStatus === SetupState.Ready
    ) {
      isChecking.current = true;
      checkNeuralAffectMatrixInstallationStatus().finally(() => {
        isChecking.current = false;
      });
      return;
    }
  }, [
    ollamaInstallationStatus,
    ollamaRunningStatus,
    modelInstallationStatus,
    neuralAffectMatrixInstallationStatus,
    checkOllamaInstallationStatus,
    checkOllamaRunningStatus,
    checkModelInstallationStatus,
    checkNeuralAffectMatrixInstallationStatus,
  ]);

  useEffect(() => {
    if (
      modelDownloadProgress &&
      modelDownloadProgress.status === "complete" &&
      modelInstallationStatus === SetupState.Loading
    ) {
      setModelInstallationStatus(SetupState.Ready);
    }
  }, [modelDownloadProgress]);

  useEffect(() => {
    if (ready) {
      onReady();
    }
  }, [ready]);

  return (
    <div className="w-full h-full flex items-center justify-center">
      <div
        className={cn(
          "flex flex-col items-start justify-start gap-2 w-64 h-36 text-sm text-foreground"
        )}
      >
        <div
          className={cn(
            "flex flex-row gap-2 items-center transition-all duration-200 animate-slideUp",
            ollamaInstallationStatus === SetupState.Pending &&
              "text-muted-foreground font-light"
          )}
        >
          <StateIcon size="sm" state={ollamaInstallationStatus} />
          <div className="flex flex-col items-start justify-start h-auto">
            <p className="text-sm font-light m-0">Ollama installation</p>

            {ollamaInstallationStatus === SetupState.Failed && (
              <p className="text-xs font-light m-0 text-[var(--placeholder)] animate-slide-up duration-fast">
                Ollama isn't installed
              </p>
            )}
          </div>
        </div>

        <div
          className={cn(
            "flex flex-row gap-2 items-center transition-all duration-200",
            ollamaRunningStatus === SetupState.Pending &&
              "text-muted-foreground font-light"
          )}
        >
          <StateIcon size="sm" state={ollamaRunningStatus} />
          <div className="flex flex-col items-start justify-start h-auto">
            <p className="text-sm font-light m-0">Ollama running</p>
            {ollamaRunningStatus === SetupState.Failed && (
              <p className="text-xs font-light m-0 text-[var(--placeholder)]">
                Ollama isn't running
              </p>
            )}
          </div>
        </div>

        <div
          className={cn(
            "flex flex-row gap-2 items-center transition-all duration-200",
            modelInstallationStatus === SetupState.Pending &&
              "text-muted-foreground font-light"
          )}
        >
          <StateIcon size="sm" state={modelInstallationStatus} />

          <div className="flex flex-col items-start justify-start h-auto">
            <p className="text-sm font-light m-0">Ollama model Download</p>
            {modelDownloadProgress &&
              modelDownloadProgress.status === "starting" && (
                <p className="text-xs font-light m-0 text-[var(--placeholder)] animate-slide-up duration-fast">
                  Starting to download llama model
                </p>
              )}
            {modelDownloadProgress &&
              modelDownloadProgress.status === "downloading" && (
                <p className="capitalize text-xs font-light m-0 text-[var(--placeholder)] animate-slide-up duration-fast">
                  {modelDownloadProgress?.message}
                </p>
              )}
            {modelInstallationStatus === SetupState.Failed && (
              <p className="text-xs font-light m-0 text-[var(--placeholder)] animate-slide-up duration-fast">
                Failed to pull ollama model
              </p>
            )}
          </div>
        </div>

        <div
          className={cn(
            "flex flex-row gap-2 items-center transition-all duration-200",
            neuralAffectMatrixInstallationStatus === SetupState.Pending &&
              "text-muted-foreground font-light"
          )}
        >
          <StateIcon size="sm" state={neuralAffectMatrixInstallationStatus} />
          <div className="flex flex-col items-start justify-start h-auto">
            <p className="text-sm font-light m-0">Neural affect matrix</p>
            {neuralAffectMatrixInstallationStatus === SetupState.Failed && (
              <p className="text-xs font-light m-0 text-[var(--placeholder)]">
                Neural affect matrix failed to start
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
