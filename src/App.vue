<template>
    <el-container class="app-container">
        <el-aside width="280px" class="sidebar">
            <ConversationList :conversations="conversations"
                :current-conversation-id="currentConversation ? currentConversation.id : null"
                @select="selectConversation" @create="createNewConversation" @delete="deleteConversation" />
        </el-aside>
        <el-main class="main-content">
            <MessagePanel :messages="currentMessages" :conversation="currentConversation" :is-loading="isLoading"
                @send-message="sendMessage" />
        </el-main>
    </el-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import ConversationList from './components/chat/ConversationList.vue';
import MessagePanel from './components/chat/MessagePanel.vue';
import type { Message, Conversation, MessageChunk } from './types';

// 状态
const conversations = ref<Conversation[]>([]);
const currentConversation = ref<Conversation | null>(null);
const allMessages = ref<Message[]>([]);
const isLoading = ref(false);

// 当前对话的消息
const currentMessages = computed(() => {
    if (!currentConversation.value) return [];
    return allMessages.value
        .filter(m => m.conversation_id === currentConversation.value?.id)
        .sort((a, b) => a.timestamp - b.timestamp);
});

// 初始化
onMounted(async () => {
    await Promise.all([
        loadConversations(),
        setupMessageListeners()
    ]);
});

// 监听后端消息更新
const setupMessageListeners = async () => {
    await listen<MessageChunk>('message_chunk', (event) => {
        const { conversation_id, content, is_complete } = event.payload;

        // 如果不是结束信号并且有内容，将内容追加到当前消息
        if (!is_complete && content) {
            // 查找当前消息集中最后一条机器人消息
            const messages = allMessages.value.filter(
                m => m.conversation_id === conversation_id
            );
            const lastBotMessage = messages
                .filter(m => m.sender === 'bot')
                .sort((a, b) => b.timestamp - a.timestamp)[0];

            if (lastBotMessage) {
                // 直接修改消息内容，Vue 会检测到变化
                lastBotMessage.content += content;
            }
        }

        // 更新对话最后消息时间
        if (is_complete) {
            updateConversationTimestamp(conversation_id);
        }

        // 如果是当前正在查看的对话，自动滚动到底部
        if (currentConversation.value?.id === conversation_id) {
            // 这里可以通过事件通知MessagePanel组件滚动
        }
    });
};

// 加载对话列表
const loadConversations = async () => {
    try {
        const convs = await invoke<Conversation[]>('get_conversations');
        conversations.value = convs;

        // 如果有对话但没有选中的对话，自动选择第一个
        if (convs.length > 0 && !currentConversation.value) {
            await selectConversation(convs[0].id);
        }
    } catch (error) {
        console.error('加载对话失败:', error);
    }
};

// 选择对话
const selectConversation = async (id: number) => {
    try {
        const conv = conversations.value.find(c => c.id === id);
        if (!conv) return;

        currentConversation.value = conv;
        await loadConversationMessages(id);
    } catch (error) {
        console.error('切换对话失败:', error);
    }
};

// 加载特定对话的所有消息
const loadConversationMessages = async (conversationId: number) => {
    try {
        const messages = await invoke<Message[]>('get_conversation_messages', {
            conversationId
        });

        // 更新消息缓存
        // 删除当前对话的旧消息
        allMessages.value = allMessages.value.filter(
            m => m.conversation_id !== conversationId
        );
        // 添加新获取的消息
        allMessages.value.push(...messages);
    } catch (error) {
        console.error('加载对话消息失败:', error);
    }
};

// 发送消息
const sendMessage = async (content: string) => {
    if (!currentConversation.value || !content.trim()) return;

    const conversationId = currentConversation.value.id;
    isLoading.value = true;

    try {
        // 1. 创建用户消息并添加到UI
        const userMsg = await invoke<Message>('send_user_message', {
            content,
            conversationId
        });

        // 2. 立即将新消息添加到UI，不等待后端响应
        allMessages.value.push(userMsg);

        // 3. 启动AI响应生成
        await invoke('generate_ai_response', {
            userMessageContent: content,
            conversationId
        });

        // 4. 更新对话列表（更新时间戳和最新消息）
        await loadConversations();
    } catch (error) {
        console.error('发送消息失败:', error);
    } finally {
        isLoading.value = false;
    }
};

// 更新对话时间戳
const updateConversationTimestamp = (conversationId: number) => {
    const conv = conversations.value.find(c => c.id === conversationId);
    if (conv) {
        conv.timestamp = Date.now();
        // 重新排序对话列表，最新的排在前面
        conversations.value.sort((a, b) => b.timestamp - a.timestamp);
    }
};

// 创建新对话
const createNewConversation = async (title: string) => {
    try {
        const newConv = await invoke<Conversation>('create_conversation', { title });
        conversations.value.unshift(newConv); // 添加到列表顶部
        await selectConversation(newConv.id); // 自动选中新对话
    } catch (error) {
        console.error('创建对话失败:', error);
    }
};

// 删除对话
const deleteConversation = async (id: number) => {
    try {
        await invoke('delete_conversation', { conversationId: id });

        // 从列表中移除
        conversations.value = conversations.value.filter(c => c.id !== id);

        // 如果删除的是当前对话，自动选择另一个
        if (currentConversation.value?.id === id) {
            currentConversation.value = conversations.value.length > 0 ?
                conversations.value[0] : null;

            // 如果有新选中的对话，加载其消息
            if (currentConversation.value) {
                await loadConversationMessages(currentConversation.value.id);
            }
        }
    } catch (error) {
        console.error('删除对话失败:', error);
    }
};
</script>

<style>
html,
body {
    margin: 0;
    padding: 0;
    height: 100vh;
    overflow: hidden;
}

#app {
    height: 100vh;
    display: flex;
    flex-direction: column;
}

.app-container {
    height: 100vh;
    display: flex;
    overflow: hidden;
}

.sidebar {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

.main-content {
    height: 100%;
    overflow: hidden;
    padding: 0;
    /* 移除内边距 */
}

/* 为ConversationList和MessagePanel设置最小高度 */
.el-aside,
.el-main {
    min-height: 400px;
}
</style>