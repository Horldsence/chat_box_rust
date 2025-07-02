import React, { useState, useMemo } from "react";
import { Conversations } from "@ant-design/x";
import { Button, Dropdown, Modal, Input, message, Tooltip } from "antd";
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  MoreOutlined,
  MessageOutlined,
  ClockCircleOutlined,
  ExclamationCircleOutlined,
} from "@ant-design/icons";
import type { GetProp, MenuProps } from "antd";
import type { Conversation } from "../types";
import { utils } from "../utils/api";

type ConversationsProps = GetProp<typeof Conversations, "items">;

interface AntdXConversationListProps {
  conversations: Conversation[];
  selectedConversationId: number | null;
  onSelectConversation: (conversationId: number) => Promise<void>;
  onCreateConversation: (title?: string) => Promise<Conversation | null>;
  onDeleteConversation: (conversationId: number) => Promise<void>;
  onEditConversation?: (id: number, newTitle: string) => Promise<void>;
  isLoading?: boolean;
  className?: string;
}

export const AntdXConversationList: React.FC<AntdXConversationListProps> = ({
  conversations,
  selectedConversationId,
  onSelectConversation,
  onCreateConversation,
  onDeleteConversation,
  onEditConversation,
  isLoading = false,
  className,
}) => {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [deleteModalVisible, setDeleteModalVisible] = useState(false);
  const [deletingId, setDeletingId] = useState<number | null>(null);

  // Convert conversations to Ant Design X format
  const conversationItems: ConversationsProps = useMemo(() => {
    return conversations.map((conv) => ({
      key: conv.id.toString(),
      label: conv.title,
      timestamp: conv.timestamp,
      description: conv.last_message
        ? utils.truncateText(conv.last_message, 50)
        : "暂无消息",
      avatar: {
        icon: <MessageOutlined />,
        style: {
          backgroundColor:
            selectedConversationId === conv.id ? "#1677ff" : "#f0f0f0",
        },
      },
      extra: (
        <div
          className="flex items-center gap-1"
          onClick={(e) => e.stopPropagation()}
        >
          <Tooltip title={utils.formatTimestamp(conv.timestamp)}>
            <ClockCircleOutlined className="text-gray-400 text-xs" />
          </Tooltip>
          <Dropdown
            menu={{
              items: getMenuItems(),
              onClick: ({ key }) => handleMenuClick(key, conv.id, conv.title),
            }}
            trigger={["click"]}
            placement="bottomRight"
          >
            <Button
              type="text"
              size="small"
              icon={<MoreOutlined />}
              className="opacity-0 group-hover:opacity-100 transition-opacity"
            />
          </Dropdown>
        </div>
      ),
    }));
  }, [conversations, selectedConversationId]);

  // Menu items for conversation actions
  const getMenuItems = (): MenuProps["items"] => [
    {
      key: "edit",
      label: "重命名",
      icon: <EditOutlined />,
    },
    {
      key: "delete",
      label: "删除",
      icon: <DeleteOutlined />,
      danger: true,
    },
  ];

  // Handle menu item clicks
  const handleMenuClick = (
    key: string,
    conversationId: number,
    title: string,
  ) => {
    switch (key) {
      case "edit":
        setEditingId(conversationId);
        setEditTitle(title);
        break;
      case "delete":
        setDeletingId(conversationId);
        setDeleteModalVisible(true);
        break;
    }
  };

  // Handle conversation selection
  const handleConversationChange = async (activeKey: string) => {
    const conversationId = parseInt(activeKey);
    try {
      await onSelectConversation(conversationId);
    } catch (error) {
      message.error("Failed to select conversation");
      console.error("Error selecting conversation:", error);
    }
  };

  // Handle new conversation
  const handleNewConversation = async () => {
    try {
      await onCreateConversation();
      message.success("新对话已创建");
    } catch (error) {
      message.error("创建对话失败");
      console.error("Error creating conversation:", error);
    }
  };

  // Handle edit conversation
  const handleEditSubmit = async () => {
    if (!editingId || !editTitle.trim()) return;

    try {
      if (onEditConversation) {
        await onEditConversation(editingId, editTitle.trim());
        message.success("对话已重命名");
      }
    } catch (error) {
      message.error("重命名失败");
      console.error("Error editing conversation:", error);
    } finally {
      setEditingId(null);
      setEditTitle("");
    }
  };

  // Handle delete conversation
  const handleDeleteConfirm = async () => {
    if (!deletingId) return;

    try {
      await onDeleteConversation(deletingId);
      message.success("对话已删除");
    } catch (error) {
      message.error("删除对话失败");
      console.error("Error deleting conversation:", error);
    } finally {
      setDeleteModalVisible(false);
      setDeletingId(null);
    }
  };

  return (
    <div className={`h-full flex flex-col ${className || ""}`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            对话列表
          </h2>
          <span className="text-sm text-gray-500 dark:text-gray-400">
            {conversations.length} 个对话
          </span>
        </div>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          block
          onClick={handleNewConversation}
          loading={isLoading}
          size="large"
        >
          新建对话
        </Button>
      </div>

      {/* Conversations List */}
      <div className="flex-1 overflow-hidden">
        {conversations.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full p-8 text-center">
            <MessageOutlined className="text-4xl text-gray-300 dark:text-gray-600 mb-4" />
            <h3 className="text-lg font-medium text-gray-500 dark:text-gray-400 mb-2">
              暂无对话
            </h3>
            <p className="text-sm text-gray-400 dark:text-gray-500 mb-4">
              点击"新建对话"开始聊天
            </p>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleNewConversation}
              loading={isLoading}
            >
              创建第一个对话
            </Button>
          </div>
        ) : (
          <Conversations
            items={conversationItems}
            activeKey={selectedConversationId?.toString()}
            onActiveChange={handleConversationChange}
            style={{ height: "100%" }}
            className="conversation-list"
          />
        )}
      </div>

      {/* Edit Modal */}
      <Modal
        title="重命名对话"
        open={editingId !== null}
        onOk={handleEditSubmit}
        onCancel={() => {
          setEditingId(null);
          setEditTitle("");
        }}
        okText="确认"
        cancelText="取消"
        width={400}
      >
        <div className="py-4">
          <Input
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            placeholder="请输入新的对话标题"
            maxLength={50}
            showCount
            onPressEnter={handleEditSubmit}
            autoFocus
          />
        </div>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        title="删除对话"
        open={deleteModalVisible}
        onOk={handleDeleteConfirm}
        onCancel={() => {
          setDeleteModalVisible(false);
          setDeletingId(null);
        }}
        okText="删除"
        cancelText="取消"
        okButtonProps={{ danger: true }}
        width={400}
      >
        <div className="py-4">
          <div className="flex items-center gap-3 mb-4">
            <ExclamationCircleOutlined className="text-orange-500 text-xl" />
            <span className="text-gray-700 dark:text-gray-300">
              确定要删除这个对话吗？
            </span>
          </div>
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-3">
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
              <strong>注意：</strong>
            </p>
            <ul className="text-sm text-gray-600 dark:text-gray-400 space-y-1">
              <li>• 删除后将无法恢复</li>
              <li>• 所有聊天记录将被永久删除</li>
              <li>• 相关的附件和设置也会被清除</li>
            </ul>
          </div>
        </div>
      </Modal>

      <style>{`
        .conversation-list .ant-conversations-item {
          transition: all 0.2s ease;
        }

        .conversation-list .ant-conversations-item:hover {
          background-color: rgba(0, 0, 0, 0.02);
        }

        .conversation-list .ant-conversations-item-active {
          background-color: #e6f4ff;
          border-color: #1677ff;
        }

        .dark .conversation-list .ant-conversations-item:hover {
          background-color: rgba(255, 255, 255, 0.05);
        }

        .dark .conversation-list .ant-conversations-item-active {
          background-color: rgba(22, 119, 255, 0.1);
        }
      `}</style>
    </div>
  );
};

export default AntdXConversationList;
