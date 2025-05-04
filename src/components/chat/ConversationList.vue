<template>
    <div class="conversation-list">
        <!-- 头部 -->
        <div class="list-header">
            <h1>聊天</h1>
            <button class="new-chat-btn" @click="showCreateDialog">
                <img :src="addIcon" alt="New chat" />
                新对话
            </button>
        </div>

        <!-- 搜索框 -->
        <div class="search-container">
            <img :src="searchIcon" alt="Search" class="search-icon" />
            <input type="text" v-model="searchQuery" placeholder="搜索对话..." class="search-input" />
        </div>

        <!-- 对话列表 -->
        <div class="conversations">
            <div v-for="conversation in filteredConversations" :key="conversation.id" class="conversation-item"
                :class="{ active: conversation.id === currentConversationId }"
                @click="selectConversation(conversation.id)">
                <div class="conversation-avatar">
                    <img :src="defaultAvatar" alt="Conversation" />
                </div>

                <div class="conversation-info">
                    <div class="conversation-title">{{ conversation.title }}</div>
                    <div class="conversation-preview">{{ conversation.lastMessage }}</div>
                </div>

                <div class="conversation-meta">
                    <div class="conversation-time">
                        {{ formatTime(conversation.timestamp) }}
                    </div>
                    <button class="delete-btn" @click.stop="confirmDelete(conversation)"
                        :aria-label="'删除 ' + conversation.title">
                        <img :src="trashIcon" alt="Delete" />
                    </button>
                </div>
            </div>

            <div v-if="filteredConversations.length === 0" class="empty-list">
                <p v-if="searchQuery">没有找到匹配的对话</p>
                <p v-else>没有对话，点击"新对话"开始聊天</p>
            </div>
        </div>

        <!-- 创建对话对话框 -->
        <el-dialog v-model="createDialogVisible" title="创建新对话" width="300px" :append-to-body="true">
            <el-input v-model="newConversationTitle" placeholder="对话标题" autofocus />
            <template #footer>
                <span class="dialog-footer">
                    <el-button @click="createDialogVisible = false">取消</el-button>
                    <el-button type="primary" @click="createConversation">
                        创建
                    </el-button>
                </span>
            </template>
        </el-dialog>

        <!-- 删除确认 -->
        <el-dialog v-model="deleteDialogVisible" title="删除对话" width="300px" :append-to-body="true">
            <p>确定要删除"{{ conversationToDelete?.title }}"吗？此操作不可撤销。</p>
            <template #footer>
                <span class="dialog-footer">
                    <el-button @click="deleteDialogVisible = false">取消</el-button>
                    <el-button type="danger" @click="deleteConversation">
                        删除
                    </el-button>
                </span>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Conversation } from '../../types';

// 图标导入
import defaultAvatar from '../../assets/account.svg';
import addIcon from '../../assets/plus.svg'; // 需添加此图标
import searchIcon from '../../assets/text-search.svg';
import trashIcon from '../../assets/trash-can.svg'; // 需添加此图标

const props = defineProps<{
    conversations: Conversation[];
    currentConversationId: number | null;
}>();

const emit = defineEmits<{
    select: [id: number];
    create: [title: string];
    delete: [id: number];
}>();

// 状态
const searchQuery = ref('');
const createDialogVisible = ref(false);
const deleteDialogVisible = ref(false);
const newConversationTitle = ref('');
const conversationToDelete = ref<Conversation | null>(null);

// 过滤后的对话列表
const filteredConversations = computed(() => {
    if (!searchQuery.value) {
        // 按时间戳排序，最新的在前面
        return [...props.conversations].sort((a, b) => b.timestamp - a.timestamp);
    }

    const query = searchQuery.value.toLowerCase();
    return props.conversations
        .filter(conv =>
            conv.title.toLowerCase().includes(query) ||
            conv.lastMessage.toLowerCase().includes(query)
        )
        .sort((a, b) => b.timestamp - a.timestamp);
});

// 格式化时间
const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp);
    const now = new Date();

    // 如果是今天
    if (date.toDateString() === now.toDateString()) {
        return date.toLocaleTimeString('zh-CN', {
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    // 如果是昨天
    const yesterday = new Date(now);
    yesterday.setDate(now.getDate() - 1);
    if (date.toDateString() === yesterday.toDateString()) {
        return '昨天';
    }

    // 如果是7天内
    const days = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    if (now.getTime() - date.getTime() < 7 * 24 * 60 * 60 * 1000) {
        return days[date.getDay()];
    }

    // 其他情况显示日期
    return date.toLocaleDateString('zh-CN', {
        month: 'short',
        day: 'numeric'
    });
};

// 选择对话
const selectConversation = (id: number) => {
    emit('select', id);
};

// 显示创建对话框
const showCreateDialog = () => {
    newConversationTitle.value = '';
    createDialogVisible.value = true;
};

// 创建对话
const createConversation = () => {
    if (newConversationTitle.value.trim()) {
        emit('create', newConversationTitle.value.trim());
        createDialogVisible.value = false;
    }
};

// 确认删除
const confirmDelete = (conversation: Conversation) => {
    conversationToDelete.value = conversation;
    deleteDialogVisible.value = true;
};

// 删除对话
const deleteConversation = () => {
    if (conversationToDelete.value) {
        emit('delete', conversationToDelete.value.id);
        deleteDialogVisible.value = false;
    }
};
</script>

<style scoped>
.conversation-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #ffffff;
    border-right: 1px solid #eaeaea;
}

.list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #eaeaea;
}

.list-header h1 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
}

.new-chat-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: #1976d2;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 14px;
    cursor: pointer;
    transition: background-color 0.2s;
}

.new-chat-btn:hover {
    background-color: #1565c0;
}

.new-chat-btn img {
    width: 16px;
    height: 16px;
    filter: brightness(0) invert(1);
}

.search-container {
    position: relative;
    padding: 12px 20px;
    border-bottom: 1px solid #eaeaea;
}

.search-icon {
    position: absolute;
    left: 30px;
    top: 50%;
    transform: translateY(-50%);
    width: 16px;
    height: 16px;
    opacity: 0.5;
}

.search-input {
    width: 100%;
    padding: 8px 12px 8px 36px;
    border-radius: 8px;
    border: 1px solid #e0e0e0;
    font-size: 14px;
    outline: none;
    transition: border-color 0.2s;
}

.search-input:focus {
    border-color: #1976d2;
}

.conversations {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
}

.conversation-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 20px;
    cursor: pointer;
    transition: background-color 0.2s;
    border-left: 3px solid transparent;
}

.conversation-item:hover {
    background-color: #f5f5f5;
}

.conversation-item.active {
    background-color: #e3f2fd;
    border-left-color: #1976d2;
}

.conversation-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
}

.conversation-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.conversation-info {
    flex: 1;
    min-width: 0;
}

.conversation-title {
    font-size: 14px;
    font-weight: 500;
    color: #333;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.conversation-preview {
    font-size: 12px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
}

.conversation-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
}

.conversation-time {
    font-size: 11px;
    color: #888;
}

.delete-btn {
    padding: 0;
    background: transparent;
    border: none;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background-color 0.2s;
}

.conversation-item:hover .delete-btn {
    opacity: 0.5;
}

.delete-btn:hover {
    opacity: 1 !important;
    background-color: #f0f0f0;
}

.delete-btn img {
    width: 16px;
    height: 16px;
}

.empty-list {
    padding: 20px;
    text-align: center;
    color: #888;
}
</style>