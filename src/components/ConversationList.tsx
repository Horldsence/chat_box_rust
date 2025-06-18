import React, { useState } from "react";
import {
  MessageSquare,
  Plus,
  Trash2,
  Search,
  Edit2,
  MoreVertical,
} from "lucide-react";
import { cn } from "../utils/cn";
import { utils } from "../utils/api";
import type { Conversation } from "../types";

interface ConversationItemProps {
  conversation: Conversation;
  isSelected: boolean;
  onSelect: () => void;
  onDelete: () => void;
  onEdit?: (id: number, newTitle: string) => void;
}

function ConversationItem({
  conversation,
  isSelected,
  onSelect,
  onDelete,
  onEdit,
}: ConversationItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(conversation.title);
  const [showMenu, setShowMenu] = useState(false);

  const handleEdit = () => {
    if (isEditing && editTitle.trim() && editTitle !== conversation.title) {
      onEdit?.(conversation.id, editTitle.trim());
    }
    setIsEditing(!isEditing);
    setShowMenu(false);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleEdit();
    } else if (e.key === "Escape") {
      setEditTitle(conversation.title);
      setIsEditing(false);
    }
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowMenu(false);
    onDelete();
  };

  return (
    <div
      className={cn(
        "relative group cursor-pointer transition-colors p-3 rounded-lg mx-2 mb-1",
        isSelected
          ? "bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500"
          : "hover:bg-gray-50 dark:hover:bg-gray-700/50",
      )}
      onClick={onSelect}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <MessageSquare
              size={14}
              className={cn(
                "flex-shrink-0",
                isSelected
                  ? "text-blue-500"
                  : "text-gray-400 dark:text-gray-500",
              )}
            />
            {isEditing ? (
              <input
                type="text"
                value={editTitle}
                onChange={(e) => setEditTitle(e.target.value)}
                onBlur={handleEdit}
                onKeyDown={handleKeyPress}
                className="flex-1 px-2 py-1 text-sm bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                autoFocus
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <h3
                className={cn(
                  "font-medium text-sm truncate",
                  isSelected
                    ? "text-gray-900 dark:text-gray-100"
                    : "text-gray-700 dark:text-gray-300",
                )}
                title={conversation.title}
              >
                {conversation.title}
              </h3>
            )}
          </div>

          {conversation.last_message && (
            <p className="text-xs text-gray-500 dark:text-gray-400 truncate mb-1">
              {conversation.last_message}
            </p>
          )}

          <span className="text-xs text-gray-400 dark:text-gray-500">
            {utils.formatRelativeTime(conversation.timestamp)}
          </span>
        </div>

        {/* Menu Button */}
        <div className="relative">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setShowMenu(!showMenu);
            }}
            className={cn(
              "p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-opacity",
              showMenu || isSelected
                ? "opacity-100"
                : "opacity-0 group-hover:opacity-100",
            )}
          >
            <MoreVertical
              size={14}
              className="text-gray-500 dark:text-gray-400"
            />
          </button>

          {/* Dropdown Menu */}
          {showMenu && (
            <div className="absolute right-0 top-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-10 py-1 min-w-32">
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setIsEditing(true);
                  setShowMenu(false);
                }}
                className="w-full px-3 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2"
              >
                <Edit2 size={14} />
                Edit
              </button>
              <button
                onClick={handleDelete}
                className="w-full px-3 py-2 text-left text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
              >
                <Trash2 size={14} />
                Delete
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Click outside to close menu */}
      {showMenu && (
        <div className="fixed inset-0 z-5" onClick={() => setShowMenu(false)} />
      )}
    </div>
  );
}

interface ConversationListProps {
  conversations: Conversation[];
  selectedConversationId: number | null;
  onSelectConversation: (id: number) => void;
  onCreateConversation: () => void;
  onDeleteConversation: (id: number) => void;
  onEditConversation?: (id: number, newTitle: string) => void;
  isLoading?: boolean;
  className?: string;
}

export function ConversationList({
  conversations,
  selectedConversationId,
  onSelectConversation,
  onCreateConversation,
  onDeleteConversation,
  onEditConversation,
  isLoading = false,
  className,
}: ConversationListProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredConversations = conversations.filter(
    (conv) =>
      conv.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (conv.last_message &&
        conv.last_message.toLowerCase().includes(searchQuery.toLowerCase())),
  );

  const handleDeleteConversation = async (id: number) => {
    const conversation = conversations.find((c) => c.id === id);
    if (!conversation) return;

    const confirmed = window.confirm(
      `Are you sure you want to delete "${conversation.title}"? This action cannot be undone.`,
    );

    if (confirmed) {
      onDeleteConversation(id);
    }
  };

  return (
    <div
      className={cn(
        "flex flex-col h-full bg-white dark:bg-gray-800",
        className,
      )}
    >
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Conversations
          </h2>
          <button
            onClick={onCreateConversation}
            className="p-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 transition-colors"
            title="New conversation"
          >
            <Plus size={16} />
          </button>
        </div>

        {/* Search */}
        <div className="relative">
          <Search
            size={16}
            className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 dark:text-gray-500"
          />
          <input
            type="text"
            placeholder="Search conversations..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 text-sm bg-gray-50 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Conversation List */}
      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex flex-col items-center justify-center p-8 text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mb-4" />
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Loading conversations...
            </p>
          </div>
        ) : filteredConversations.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-8 text-center">
            {searchQuery ? (
              <>
                <Search
                  size={32}
                  className="text-gray-300 dark:text-gray-600 mb-4"
                />
                <p className="text-sm text-gray-500 dark:text-gray-400 mb-2">
                  No conversations found
                </p>
                <p className="text-xs text-gray-400 dark:text-gray-500">
                  Try adjusting your search terms
                </p>
              </>
            ) : (
              <>
                <MessageSquare
                  size={32}
                  className="text-gray-300 dark:text-gray-600 mb-4"
                />
                <p className="text-sm text-gray-500 dark:text-gray-400 mb-2">
                  No conversations yet
                </p>
                <p className="text-xs text-gray-400 dark:text-gray-500 mb-4">
                  Start a new conversation to begin chatting
                </p>
                <button
                  onClick={onCreateConversation}
                  className="inline-flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors text-sm"
                >
                  <Plus size={14} />
                  New Chat
                </button>
              </>
            )}
          </div>
        ) : (
          <div className="py-2">
            {filteredConversations.map((conversation) => (
              <ConversationItem
                key={conversation.id}
                conversation={conversation}
                isSelected={selectedConversationId === conversation.id}
                onSelect={() => onSelectConversation(conversation.id)}
                onDelete={() => handleDeleteConversation(conversation.id)}
                onEdit={onEditConversation}
              />
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      {conversations.length > 0 && (
        <div className="p-3 border-t border-gray-200 dark:border-gray-700">
          <div className="text-xs text-gray-500 dark:text-gray-400 text-center">
            {conversations.length} conversation
            {conversations.length !== 1 ? "s" : ""}
            {searchQuery &&
              filteredConversations.length !== conversations.length && (
                <span> • {filteredConversations.length} shown</span>
              )}
          </div>
        </div>
      )}
    </div>
  );
}
