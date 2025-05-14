<template>
  <div class="conversation-list-container">
    <!-- 头部 -->
    <div class="header-wrapper">
      <el-row :gutter="20" align="middle" justify="space-between">
        <el-col :span="12">
          <h2>聊天</h2>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-button type="primary" size="small" @click="showCreateDialog" :icon="Plus">
            新对话
          </el-button>
        </el-col>
      </el-row>
    </div>

    <!-- 搜索框 -->
    <div class="search-wrapper">
      <el-input
        v-model="searchQuery"
        placeholder="搜索对话..."
        clearable
        :prefix-icon="Search"
      />
    </div>

    <!-- 对话列表 -->
    <el-scrollbar class="scrollbar-wrapper">
      <el-empty v-if="filteredConversations.length === 0" description="没有找到对话" />
      
      <el-menu
        v-else
        :default-active="currentConversationId ? currentConversationId.toString() : ''"
      >
        <el-menu-item
          v-for="conversation in filteredConversations"
          :key="conversation.id"
          :index="conversation.id.toString()"
          @click="selectConversation(conversation.id)"
        >
          <div class="conversation-item">
            <el-avatar :size="40" :icon="UserFilled" class="conversation-avatar" />
            
            <div class="conversation-content">
              <div class="conversation-title">{{ conversation.title }}</div>
              <div class="conversation-preview">{{ conversation.lastMessage }}</div>
            </div>
            
            <div class="conversation-meta">
              <span class="conversation-time">{{ formatTime(conversation.timestamp) }}</span>
              <el-button
                class="delete-button"
                type="danger"
                :icon="Delete"
                circle
                size="small"
                plain
                @click.stop="confirmDelete(conversation)"
              />
            </div>
          </div>
        </el-menu-item>
      </el-menu>
    </el-scrollbar>

    <!-- 创建对话对话框 -->
    <el-dialog v-model="createDialogVisible" title="创建新对话" width="300px" :append-to-body="true">
      <el-input v-model="newConversationTitle" placeholder="对话标题" autofocus />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="createDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="createConversation">创建</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 删除确认 -->
    <el-dialog v-model="deleteDialogVisible" title="删除对话" width="300px" :append-to-body="true">
      <p>确定要删除"{{ conversationToDelete?.title }}"吗？此操作不可撤销。</p>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="deleteDialogVisible = false">取消</el-button>
          <el-button type="danger" @click="deleteConversation">删除</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Conversation } from '../../types';
import { Search, Plus, Delete, UserFilled } from '@element-plus/icons-vue';

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
      conv.lastMessage?.toLowerCase().includes(query)
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
.conversation-list-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header-wrapper {
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.search-wrapper {
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.scrollbar-wrapper {
  flex: 1;
  overflow: hidden;
}

.conversation-item {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.conversation-content {
  flex: 1;
  overflow: hidden;
}

.conversation-title {
  font-size: var(--el-font-size-base);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.conversation-preview {
  font-size: var(--el-font-size-extra-small);
  color: var(--el-text-color-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.conversation-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.conversation-time {
  font-size: var(--el-font-size-extra-small);
  color: var(--el-text-color-secondary);
}

.delete-button {
  padding: 2px;
  font-size: 12px;
  visibility: hidden;
}

.el-menu-item:hover .delete-button {
  visibility: visible;
}

/* 覆盖一些Element Plus默认样式 */
.el-menu-item {
  height: auto;
  line-height: normal;
  padding: 10px 16px;
}

.el-menu {
  border-right: none;
}

/* 处理选中项样式 */
.el-menu-item.is-active {
  background-color: var(--el-menu-hover-bg-color);
}
</style>