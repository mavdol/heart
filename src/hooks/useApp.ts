import { InvokeResponse } from "@/interfaces/api.interface";
import { ModelDownloadProgress, SetupState } from "@/interfaces/app.interface";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export interface App {
  ollamaInstallationStatus: SetupState;
  ollamaRunningStatus: SetupState;
  modelInstallationStatus: SetupState;
  neuralAffectMatrixInstallationStatus: SetupState;
  modelDownloadProgress: ModelDownloadProgress | null;
  ready: boolean;
  checkOllamaInstallationStatus: () => Promise<void>;
  checkOllamaRunningStatus: () => Promise<void>;
  checkModelInstallationStatus: () => Promise<void>;
  setModelInstallationStatus: (status: SetupState) => void;
  checkNeuralAffectMatrixInstallationStatus: () => Promise<void>;
}

export function useApp(): App {
  const [ollamaInstallationStatus, setOllamaInstallationStatus] = useState(
    SetupState.Pending
  );
  const [ollamaRunningStatus, setOllamaRunningStatus] = useState(
    SetupState.Pending
  );
  const [modelInstallationStatus, setModelInstallationStatus] = useState(
    SetupState.Pending
  );
  const [
    neuralAffectMatrixInstallationStatus,
    setNeuralAffectMatrixInstallationStatus,
  ] = useState(SetupState.Pending);

  const [modelDownloadProgress, setModelDownloadProgress] =
    useState<ModelDownloadProgress | null>(null);
  const [ready, setReady] = useState(false);

  const checkOllamaInstallationStatus = async () => {
    setOllamaInstallationStatus(SetupState.Loading);

    try {
      const ollamaInstallationStatus = await invoke<InvokeResponse<boolean>>(
        "check_ollama_installed"
      );

      setOllamaInstallationStatus(
        ollamaInstallationStatus.data ? SetupState.Ready : SetupState.Failed
      );
    } catch (error) {
      setOllamaInstallationStatus(SetupState.Failed);
    }
  };

  const checkOllamaRunningStatus = async () => {
    setOllamaRunningStatus(SetupState.Loading);
    try {
      const ollamaRunningStatus = await invoke<InvokeResponse<boolean>>(
        "check_ollama_running"
      );
      setOllamaRunningStatus(
        ollamaRunningStatus.data ? SetupState.Ready : SetupState.Failed
      );
    } catch (error) {
      setOllamaRunningStatus(SetupState.Failed);
    }
  };

  const checkModelInstallationStatus = async () => {
    setModelInstallationStatus(SetupState.Loading);
    try {
      const modelInstallationStatus = await invoke<InvokeResponse<boolean>>(
        "check_model_installed"
      );

      if (!modelInstallationStatus.data) {
        await downloadModel();
      } else {
        setModelInstallationStatus(SetupState.Ready);
      }
    } catch (error) {
      setModelInstallationStatus(SetupState.Failed);
    }
  };

  const checkNeuralAffectMatrixInstallationStatus = async () => {
    setNeuralAffectMatrixInstallationStatus(SetupState.Loading);
    try {
      const neuralAffectMatrixInstallationStatus = await invoke<
        InvokeResponse<boolean>
      >("check_neural_affect_matrix_running");
      setNeuralAffectMatrixInstallationStatus(
        neuralAffectMatrixInstallationStatus.data
          ? SetupState.Ready
          : SetupState.Failed
      );
    } catch (error) {
      setNeuralAffectMatrixInstallationStatus(SetupState.Failed);
    }
  };

  const downloadModel = async () => {
    await invoke<InvokeResponse<boolean>>("download_model");
  };

  useEffect(() => {
    const unlisten = listen<ModelDownloadProgress>(
      "model-download-progress",
      (event) => {
        const { status, message } = event.payload;
        setModelDownloadProgress({ status, message });
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    setReady(
      ollamaInstallationStatus === SetupState.Ready &&
        ollamaRunningStatus === SetupState.Ready &&
        modelInstallationStatus === SetupState.Ready &&
        neuralAffectMatrixInstallationStatus === SetupState.Ready
    );
  }, [
    ollamaInstallationStatus,
    ollamaRunningStatus,
    modelInstallationStatus,
    neuralAffectMatrixInstallationStatus,
    ready,
  ]);

  return {
    ollamaInstallationStatus,
    ollamaRunningStatus,
    modelInstallationStatus,
    neuralAffectMatrixInstallationStatus,
    modelDownloadProgress,
    ready,
    checkOllamaInstallationStatus,
    checkOllamaRunningStatus,
    checkModelInstallationStatus,
    setModelInstallationStatus,
    checkNeuralAffectMatrixInstallationStatus,
  };
}
