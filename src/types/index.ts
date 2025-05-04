export interface Message {
  id: number;
  conversation_id: number;
  content: string;
  sender: "user" | "bot";
  timestamp: number;
}

export interface Conversation {
  id: number;
  title: string;
  avatar: string;
  lastMessage: string;
  timestamp: number;
}

export interface MessageChunk {
  conversation_id: number;
  content: string;
  is_complete: boolean;
}
