import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { ArrowUp } from "lucide-react";
import { useState, useEffect, useRef, useCallback } from "react";
import { useTheme } from "@/providers/theme.provider";
import MessageList from "@/components/ui/custom/chat/messageList";
import { Message } from "@/interfaces/chat.interface";
import { useBrain } from "@/hooks/useBrain";
import { welcomeMessage } from "@/lib/chat.helper";

export default function Chat() {
  const [message, setMessage] = useState("");
  const { setTheme, theme } = useTheme();
  const {
    processNewMessage,
    welcomeBackMessage,
    isThinking,
    currentEmotion,
    firstMessageSent,
    isInitialized: isBrainInitialized,
  } = useBrain();

  const chatInitializedRef = useRef<boolean>(false);
  const sendTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const requestIdRef = useRef<number>(0);

  const [messages, setMessages] = useState<Message[]>(
    firstMessageSent
      ? [
          { role: "system", content: "{{system placeholder}}" }, // Don't remove system placeholder
          {
            role: "assistant",
            content: welcomeMessage,
          },
        ]
      : [{ role: "system", content: "{{system placeholder}}" }] // Don't remove system placeholder
  );
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    requestAnimationFrame(() => {
      const lastMessage = document.getElementById("last-message");
      if (lastMessage) {
        lastMessage.scrollIntoView({ behavior: "smooth", block: "end" });
      } else if (messagesEndRef.current) {
        messagesEndRef.current.scrollIntoView({ behavior: "smooth" });
      }
    });
  }, [messages]);

  useEffect(() => {
    return () => {
      if (sendTimeoutRef.current) {
        clearTimeout(sendTimeoutRef.current);
      }
    };
  }, []);

  const handleSend = async () => {
    if (!message.trim()) return;

    if (sendTimeoutRef.current) {
      clearTimeout(sendTimeoutRef.current);
      sendTimeoutRef.current = null;
    }

    if (isThinking) {
      requestIdRef.current += 1;
      setMessages((prevMessages) => prevMessages.filter((m) => !m.isWriting));
    }

    const newMessages: Message[] = [
      ...messages.filter((m) => !m.isWriting),
      { role: "user", content: message },
    ];

    setMessages(newMessages);
    setMessage("");

    sendTimeoutRef.current = setTimeout(async () => {
      requestIdRef.current += 1;
      const currentRequestId = requestIdRef.current;

      setMessages((prevMessages) => [
        ...prevMessages,
        { role: "assistant", content: "Thinking...", isWriting: true },
      ]);

      const response = await processNewMessage(newMessages, currentRequestId);

      if (currentRequestId === requestIdRef.current) {
        setMessages((prevMessages) =>
          prevMessages.map((m, index) => {
            return index === prevMessages.length - 1
              ? { role: "assistant", content: response.content }
              : m;
          })
        );
      }

      sendTimeoutRef.current = null;
    }, 1000);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleWelcomeBackMessage = useCallback(async () => {
    setMessages((prevMessages) => [
      ...prevMessages,
      { role: "assistant", content: "Thinking...", isWriting: true },
    ]);

    const response = await welcomeBackMessage();

    setMessages((prevMessages) =>
      prevMessages.map((m, index) => {
        return index === prevMessages.length - 1
          ? { role: "assistant", content: response.content }
          : m;
      })
    );
  }, [welcomeBackMessage]);

  useEffect(() => {
    if (!isBrainInitialized) {
      return;
    }

    if (chatInitializedRef.current) {
      return;
    }

    chatInitializedRef.current = true;

    if (!firstMessageSent) {
      setMessages([{ role: "assistant", content: welcomeMessage }]);
    } else {
      handleWelcomeBackMessage();
    }
  }, [isBrainInitialized, firstMessageSent, handleWelcomeBackMessage]);

  return (
    <div className="flex flex-col w-full h-full">
      <div
        ref={scrollContainerRef}
        className="flex-1 overflow-y-auto px-8 py-6"
      >
        <div className="max-w-3xl mx-auto">
          <MessageList
            messages={messages.filter((m) => m.role !== "system")}
            currentEmotion={currentEmotion}
          />
          <div ref={messagesEndRef} />
        </div>
      </div>

      <div className=" bg-background/95 backdrop-blur-sm px-8">
        <div className="max-w-3xl mx-auto pb-4 flex flex-col gap-3">
          <div className="relative text-foreground">
            <Textarea
              placeholder="Type your message..."
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              onKeyDown={handleKeyDown}
              className="min-h-[80px] max-h-[200px] resize-none pr-14 rounded-xl border-2 focus-visible:ring-2 transition-all"
              rows={1}
            />
            <Button
              onClick={handleSend}
              size="icon"
              className="absolute bottom-2 right-2 h-8 w-8 rounded-full  transition-all hover:scale-101 disabled:opacity-50 disabled:scale-100"
              disabled={!message.trim()}
            >
              <ArrowUp className="w-4 h-4" />
            </Button>
          </div>
          <p
            className="text-xs text-muted-foreground text-left"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
          >
            Press{" "}
            <kbd className="px-1.5 py-0.5 text-xs rounded bg-muted">Enter</kbd>{" "}
            to send,{" "}
            <kbd className="px-1.5 py-0.5 text-xs rounded bg-muted">
              Shift + Enter
            </kbd>{" "}
            for new line
          </p>
        </div>
      </div>
    </div>
  );
}
