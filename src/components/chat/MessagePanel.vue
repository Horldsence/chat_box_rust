<template>
    <div class="message-panel">
        <!-- 头部 -->
        <div class="message-panel-header">
            <div class="conversation-info">
                <div class="avatar">
                    <img :src="defaultAvatar" alt="avatar" />
                </div>
                <div class="info">
                    <h2 class="title">{{ conversation?.title || '请选择对话' }}</h2>
                    <div class="subtitle">
                        {{ isLoading ? '正在输入...' : lastActivityTime }}
                    </div>
                </div>
            </div>

            <div class="actions">
                <button class="action-btn" title="复制对话">
                    <img :src="copyIcon" alt="Copy" />
                </button>
                <button class="action-btn" title="导出对话">
                    <img :src="exportIcon" alt="Export" />
                </button>
                <button class="action-btn" title="更多选项">
                    <img :src="moreIcon" alt="More" />
                </button>
            </div>
        </div>

        <!-- 消息区域 -->
        <div class="message-list" ref="messageListRef">
            <div v-if="!messages.length" class="empty-state">
                <img :src="chatIcon" alt="Start chatting" class="empty-icon" />
                <p>开始新的对话吧！</p>
            </div>

            <template v-else>
                <div v-for="message in messages" :key="message.id" class="message-container" :class="message.sender">
                    <div class="message-avatar" v-if="message.sender === 'bot'">
                        <img :src="defaultAvatar" alt="Bot" />
                    </div>

                    <div class="message-bubble" :class="`${message.sender}-bubble`">
                        <message-item :message="message" />

                        <div class="message-actions">
                            <button class="message-action" @click="copyMessage(message.content)">
                                <img :src="copySmallIcon" alt="Copy" />
                            </button>
                        </div>
                    </div>

                    <div class="message-avatar user-avatar" v-if="message.sender === 'user'">
                        <img :src="userAvatar" alt="User" />
                    </div>
                </div>

                <div class="typing-indicator" v-if="isLoading">
                    <div class="message-avatar">
                        <img :src="defaultAvatar" alt="Bot" />
                    </div>
                    <div class="typing-bubble">
                        <span></span>
                        <span></span>
                        <span></span>
                    </div>
                </div>
            </template>
        </div>

        <!-- 输入区域 -->
        <div class="message-input">
            <div class="input-container">
                <textarea ref="inputRef" v-model="inputMessage" @keydown.enter.prevent="onSendMessage"
                    placeholder="输入消息..." :disabled="isLoading || !conversation || isVoiceRecording"
                    rows="1"></textarea>

                <div class="input-actions">
                    <button class="input-action" title="语音输入" @click="startVoiceInput"
                        :class="{ 'recording': isVoiceRecording }" :disabled="isLoading || !conversation">
                        <img :src="isVoiceRecording ? micActiveIcon : micIcon" alt="Voice" />
                    </button>
                    <button class="input-action" title="添加表情">
                        <img :src="emojiIcon" alt="Emoji" />
                    </button>
                    <button class="input-action" title="上传文件">
                        <img :src="attachmentIcon" alt="Attach" />
                    </button>
                    <button class="send-button" :class="{ disabled: !canSend }" @click="onSendMessage">
                        <img :src="sendIcon" alt="Send" />
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import MessageItem from './MessageItem.vue';
import type { Message, Conversation } from '../../types';

// 图标导入
import defaultAvatar from '../../assets/account-circle.svg';
import userAvatar from '../../assets/account.svg';
import sendIcon from '../../assets/send.svg';
import copyIcon from '../../assets/clipboard-outline.svg';
import copySmallIcon from '../../assets/clipboard-outline.svg';
import exportIcon from '../../assets/download.svg';
import emojiIcon from '../../assets/emoticon.svg';
import attachmentIcon from '../../assets/image.svg';
import moreIcon from '../../assets/more.svg';
import chatIcon from '../../assets/chat.svg'; // 需添加此图标
import micIcon from '../../assets/microphone.svg'; // 添加麦克风图标
import micActiveIcon from '../../assets/microphone-active.svg'; // 添加激活状态的麦克风图标

const realTimeText = ref('');
const isProcessing = ref(false);

// 监听实时识别事件
onMounted(async () => {
    await listen('voice_partial', (event) => {
        const text = event.payload as string;
        realTimeText.value = text;
        isProcessing.value = true;

        // 自动滚动到底部
        nextTick(() => {
            const container = messageListRef.value;
            if (container && autoScrollEnabled) {
                container.scrollTop = container.scrollHeight;
            }
        });
    });
});

const props = defineProps<{
    messages: Message[];
    conversation: Conversation | null;
    isLoading: boolean;
}>();

const emit = defineEmits<{
    sendMessage: [content: string];
}>();

// 状态
const inputMessage = ref('');
const messageListRef = ref<HTMLDivElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);
const isVoiceRecording = ref(false);
let autoScrollEnabled = true;

// 格式化时间
const formatTime = (timestamp: number): string => {
    return new Date(timestamp).toLocaleString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit'
    });
};

// 最后活动时间
const lastActivityTime = computed(() => {
    if (!props.conversation) return '';

    if (props.messages.length > 0) {
        const lastMessage = props.messages[props.messages.length - 1];
        return formatTime(lastMessage.timestamp);
    }

    return '在线';
});

// 是否可以发送消息
const canSend = computed(() => {
    return !!props.conversation &&
        inputMessage.value.trim().length > 0 &&
        !props.isLoading &&
        !isVoiceRecording.value;
});

// 发送消息
const onSendMessage = () => {
    if (!canSend.value) return;

    emit('sendMessage', inputMessage.value);
    inputMessage.value = '';
    autoScrollEnabled = true;

    // 自动聚焦输入框
    nextTick(() => {
        inputRef.value?.focus();
    });
};

// 复制消息
const copyMessage = (content: string) => {
    navigator.clipboard.writeText(content)
        .then(() => {
            // 可显示复制成功提示
        })
        .catch(err => {
            console.error('复制失败:', err);
        });
};

// 修改后的语音输入方法
const startVoiceInput = async () => {
    try {
        realTimeText.value = ''; // 重置实时文本
        isProcessing.value = true;

        const result = await invoke<string>('voice_input', {
            conversationId: props.conversation?.id
        });

        // 最终结果处理
        if (result) {
            inputMessage.value = result;
            realTimeText.value = ''; // 清空实时显示
        }
    } finally {
        isProcessing.value = false;
    }
};

// 自动滚动到底部
const scrollToBottom = async () => {
    if (!autoScrollEnabled || !messageListRef.value) return;

    await nextTick();
    const container = messageListRef.value;
    container.scrollTop = container.scrollHeight;
};

// 处理滚动事件
const handleScroll = () => {
    if (!messageListRef.value) return;

    const { scrollTop, scrollHeight, clientHeight } = messageListRef.value;
    autoScrollEnabled = scrollHeight - scrollTop - clientHeight < 100;
};

// 监听消息变化，自动滚动
watch(() => props.messages.length, scrollToBottom);
watch(() => props.isLoading, (newVal) => {
    if (newVal === false) {
        // 当加载完成时滚动到底部
        scrollToBottom();
    }
});

// 监听conversation变化，自动滚动
watch(() => props.conversation?.id, () => {
    autoScrollEnabled = true;
    nextTick(scrollToBottom);
});

// 设置滚动监听
onMounted(async () => {
    if (messageListRef.value) {
        messageListRef.value.addEventListener('scroll', handleScroll);
        scrollToBottom();
    }

    // 聚焦输入框
    inputRef.value?.focus();

    // 监听语音状态事件
    await listen('voice_status', (event) => {
        const status = event.payload as string;
        if (status === 'recording') {
            isVoiceRecording.value = true;
        } else if (status === 'completed') {
            isVoiceRecording.value = false;
        }
    });
});
</script>

<style scoped>
.message-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #f8f9fa;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
}

.message-panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #eaeaea;
    background-color: #ffffff;
}

.conversation-info {
    display: flex;
    align-items: center;
    gap: 12px;
}

.avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    overflow: hidden;
}

.avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.info {
    display: flex;
    flex-direction: column;
}

.title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: #333;
}

.subtitle {
    font-size: 12px;
    color: #999;
}

.actions {
    display: flex;
    gap: 8px;
}

.action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    border: none;
    background: transparent;
    cursor: pointer;
    transition: background-color 0.2s;
}

.action-btn:hover {
    background-color: #f0f2f5;
}

.action-btn img {
    width: 20px;
    height: 20px;
}

.message-list {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    opacity: 0.7;
}

.empty-icon {
    width: 64px;
    height: 64px;
    margin-bottom: 16px;
}

.message-container {
    display: flex;
    gap: 8px;
    max-width: 85%;
}

.message-container.user {
    align-self: flex-end;
    flex-direction: row-reverse;
}

.message-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
}

.message-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.message-bubble {
    padding: 12px 16px;
    border-radius: 18px;
    background-color: #ffffff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    position: relative;
}

.bot-bubble {
    border-top-left-radius: 4px;
}

.user-bubble {
    background: linear-gradient(135deg, #1976d2, #1565c0);
    color: white;
    border-top-right-radius: 4px;
}

.message-actions {
    position: absolute;
    top: 8px;
    right: 8px;
    opacity: 0;
    transition: opacity 0.2s;
}

.message-bubble:hover .message-actions {
    opacity: 1;
}

.message-action {
    background: transparent;
    border: none;
    padding: 4px;
    cursor: pointer;
    opacity: 0.7;
}

.message-action:hover {
    opacity: 1;
}

.message-action img {
    width: 16px;
    height: 16px;
}

.typing-indicator {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    max-width: 85%;
}

.typing-bubble {
    padding: 12px 16px;
    border-radius: 18px;
    background-color: #f1f3f4;
    border-top-left-radius: 4px;
    display: flex;
    align-items: center;
    gap: 4px;
}

.typing-bubble span {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: #aaa;
    animation: typing-animation 1.4s infinite ease-in-out both;
}

.typing-bubble span:nth-child(1) {
    animation-delay: 0s;
}

.typing-bubble span:nth-child(2) {
    animation-delay: 0.2s;
}

.typing-bubble span:nth-child(3) {
    animation-delay: 0.4s;
}

@keyframes typing-animation {
    0% {
        transform: scale(0.6);
    }

    40% {
        transform: scale(1);
    }

    80%,
    100% {
        transform: scale(0.6);
    }
}

.message-input {
    padding: 12px 20px;
    background-color: #ffffff;
    border-top: 1px solid #eaeaea;
}

.input-container {
    display: flex;
    align-items: flex-end;
    gap: 12px;
    background-color: #f0f2f5;
    border-radius: 24px;
    padding: 8px 16px;
}

textarea {
    flex: 1;
    border: none;
    background: transparent;
    resize: none;
    outline: none;
    font-family: inherit;
    font-size: 14px;
    padding: 8px 0;
    max-height: 120px;
}

.input-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.input-action {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    width: 36px;
    height: 36px;
    cursor: pointer;
    border-radius: 50%;
}

.input-action:hover {
    background-color: #e6e6e6;
}

.input-action img {
    width: 20px;
    height: 20px;
}

.input-action.recording {
    background-color: rgba(255, 0, 0, 0.1);
    animation: pulse 1.5s infinite;
}

@keyframes pulse {
    0% {
        box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.4);
    }

    70% {
        box-shadow: 0 0 0 10px rgba(255, 0, 0, 0);
    }

    100% {
        box-shadow: 0 0 0 0 rgba(255, 0, 0, 0);
    }
}

.send-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: none;
    background-color: #1976d2;
    cursor: pointer;
}

.send-button.disabled {
    background-color: #ccc;
    cursor: not-allowed;
}

.send-button img {
    width: 18px;
    height: 18px;
    filter: brightness(0) invert(1);
}

.realtime-result {
    position: relative;
    min-height: 40px;
    margin: 8px 0;
    opacity: 0;
    transition: opacity 0.3s ease;

    &.visible {
        opacity: 1;
    }
}

.partial-text {
    background: rgba(255, 255, 255, 0.9);
    border-radius: 12px;
    padding: 12px 16px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    font-size: 0.95em;
    color: #333;
    display: flex;
    align-items: center;
    backdrop-filter: blur(4px);
}

.waveform {
    width: 40px;
    height: 20px;
    margin-left: 12px;
    background:
        repeating-linear-gradient(90deg,
            #4a90e2 0px,
            #4a90e2 3px,
            transparent 3px,
            transparent 6px);
    animation: wave 1s infinite linear;
}

@keyframes wave {
    from {
        background-position: 0 0;
    }

    to {
        background-position: 40px 0;
    }
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.5s;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>