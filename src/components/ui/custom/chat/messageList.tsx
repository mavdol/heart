import { Message as MessageInterface } from "@/interfaces/chat.interface";
import Message from "./message";
import { EmotionState } from "@/interfaces/emotion.interface";

export default function MessageList({
  messages,
  currentEmotion,
}: {
  messages: MessageInterface[];
  currentEmotion: EmotionState;
}) {
  return (
    <div className="flex flex-col space-y-6">
      {messages.map((message, index) => (
        <Message
          key={index}
          message={message}
          currentEmotion={currentEmotion}
          islastMessage={index === messages.length - 1}
        />
      ))}
    </div>
  );
}
