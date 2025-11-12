import { invoke } from "@tauri-apps/api/core";

import { InvokeResponse } from "@/interfaces/api.interface";
import { Message } from "@/interfaces/chat.interface";
import { useCallback, useEffect, useRef, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { EmotionState } from "@/interfaces/emotion.interface";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface Brain {
  isThinking: boolean;
  firstMessageSent: boolean;
  isInitialized: boolean;
  currentEmotion: EmotionState;
  currentRequestId: number;
  processNewMessage: (
    messages: Message[],
    requestId: number
  ) => Promise<Message>;
  welcomeBackMessage: () => Promise<Message>;
  destroyBrain: () => Promise<void>;
}

export function useBrain(): Brain {
  const [isThinking, setIsThinking] = useState<boolean>(false);
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [firstMessageSent, setFirstMessageSent] = useState<boolean>(false);
  const [currentEmotion, setCurrentEmotion] = useState<EmotionState>({
    valence: 0.0,
    arousal: 0.0,
  });
  const currentRequestIdRef = useRef<number>(0);
  const canRefreshCurrentEmotionRef = useRef<boolean>(true);

  const initialize = useCallback(async () => {
    const store = await Store.load("heart.json");
    const firstMessageSentValue =
      (await store.get("first_message_sent")) || false;
    setFirstMessageSent(firstMessageSentValue as boolean);
    setIsInitialized(true);
  }, []);

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (canRefreshCurrentEmotionRef.current) {
      refreshCurrentEmotion();
      canRefreshCurrentEmotionRef.current = false;
    }
  }, [canRefreshCurrentEmotionRef.current]);

  const welcomeBackMessage = async () => {
    setIsThinking(true);

    const response = await invoke<InvokeResponse<Message>>(
      "process_welcome_back_message",
      {}
    );

    setIsThinking(false);
    return response.data;
  };

  const processNewMessage = async (messages: Message[], requestId: number) => {
    setIsThinking(true);

    const response = await invoke<InvokeResponse<Message>>(
      "process_new_message",
      { messages }
    );

    canRefreshCurrentEmotionRef.current = true;

    if (requestId === currentRequestIdRef.current) {
      setIsThinking(false);
    }

    return response.data;
  };

  const refreshCurrentEmotion = useCallback(async () => {
    const response = await invoke<InvokeResponse<EmotionState>>(
      "current_emotion",
      {}
    );

    setCurrentEmotion(response.data);
    console.log("Current emotion refreshed", response.data);

    return response.data;
  }, []);

  const destroyBrain = async () => {
    const window = getCurrentWindow();
    try {
      await invoke<InvokeResponse<void>>("destroy_brain", {});
      await window.close();
    } catch (error) {
      console.error("Error destroying brain", error);
      await window.close();
    }
  };

  return {
    isThinking,
    firstMessageSent,
    isInitialized,
    currentEmotion,
    currentRequestId: currentRequestIdRef.current,
    processNewMessage,
    welcomeBackMessage,
    destroyBrain,
  };
}
