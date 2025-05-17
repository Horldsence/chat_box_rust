<template>
  <div class="database-config-panel">
    <h2>数据库配置</h2>
    
    <el-form label-position="top">
      <el-form-item label="启用数据库">
        <el-switch
          v-model="localConfig.database.enabled"
          active-text="启用"
          inactive-text="禁用"
        />
      </el-form-item>
      
      <template v-if="localConfig.database.enabled">
        <el-form-item label="数据库路径">
          <el-input v-model="localConfig.database.path" placeholder="请输入数据库路径">
            <template #append>
              <el-button @click="selectDatabasePath">选择路径</el-button>
            </template>
          </el-input>
        </el-form-item>
      </template>
    </el-form>
    
    <div class="action-buttons">
      <el-button type="primary" @click="handleSaveConfig">保存配置</el-button>
      <el-button @click="resetConfig">重置</el-button>
    </div>

    <!-- 数据库内容管理 -->
    <div v-if="localConfig.database.enabled" class="database-management">
      <h3>数据库内容管理</h3>
      
      <el-table 
        v-loading="loading"
        :data="conversations" 
        style="width: 100%"
        border
      >
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="title" label="对话标题" />
        <el-table-column prop="last_message" label="最后消息" />
        <el-table-column label="时间">
          <template #default="scope">
            {{ formatDate(scope.row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="scope">
            <el-button 
              type="danger" 
              size="small" 
              @click="handleDelete(scope.row)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      
      <div class="pagination-container">
        <el-pagination
          background
          layout="prev, pager, next"
          :total="totalConversations"
          :page-size="pageSize"
          @current-change="handleCurrentChange"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, reactive, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { ElMessageBox, ElMessage } from 'element-plus';

interface Conversation {
  id: number;
  title: string;
  last_message?: string;
  timestamp: number;
}

interface DatabaseConfig {
  enabled: boolean;
  path: string;
}

interface AppConfig {
  database: DatabaseConfig;
  // 其他配置...
  [key: string]: any;
}

// 定义组件接收的props
const props = defineProps<{
  config: AppConfig;
}>();

// 定义组件触发的事件
const emit = defineEmits<{
  (e: 'save', config: AppConfig): void;
}>();

// 创建本地配置副本
const localConfig = reactive<AppConfig>({
  ...props.config
});

// 监听外部配置变化
watch(() => props.config, (newConfig) => {
  Object.assign(localConfig, JSON.parse(JSON.stringify(newConfig)));
}, { deep: true });

const loading = ref(false);
const conversations = ref<Conversation[]>([]);
const totalConversations = ref(0);
const currentPage = ref(1);
const pageSize = ref(10);

// 加载配置
onMounted(async () => {
  if (localConfig.database.enabled) {
    await loadConversations();
  }
});
// 选择数据库路径
async function selectDatabasePath() {
  try {
    const selected = await open({
      filters: [{
        name: 'Database',
        extensions: ['db', 'sqlite', 'sqlite3']
      }],
      multiple: false,
      title: '选择数据库文件'
    });
    if (selected && typeof selected === 'string') {
      localConfig.database.path = selected;
    }
  } catch (error) {
    ElMessage.error('选择文件失败: ' + error);
  }
}

// 保存配置
function handleSaveConfig() {
  // 创建一个新对象发送给父组件，避免引用传递
  const configToSave = JSON.parse(JSON.stringify(localConfig));
  emit('save', configToSave);
}

// 重置配置
function resetConfig() {
  // 从props中重新复制配置
  Object.assign(localConfig, JSON.parse(JSON.stringify(props.config)));
}

// 加载对话列表
async function loadConversations() {
  loading.value = true;
  try {
    const allConversations = await invoke('get_database_conversations') as Conversation[];
    totalConversations.value = allConversations.length;
    
    // 简单的分页实现
    const start = (currentPage.value - 1) * pageSize.value;
    const end = start + pageSize.value;
    conversations.value = allConversations.slice(start, end);
  } catch (error) {
    ElMessage.error('加载对话失败: ' + error);
  } finally {
    loading.value = false;
  }
}

// 处理分页变化
function handleCurrentChange(page: number) {
  currentPage.value = page;
  loadConversations();
}

// 删除对话
async function handleDelete(row: Conversation) {
  try {
    await ElMessageBox.confirm(
      `确定要删除对话 "${row.title}" 吗？这将同时删除所有相关消息。`,
      '警告',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    
    await invoke('delete_database_conversation', { conversationId: row.id });
    ElMessage.success('删除成功');
    loadConversations();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败: ' + error);
    }
  }
}

// 格式化日期
function formatDate(timestamp: number): string {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  return date.toLocaleString();
}
</script>

<style scoped>
.database-config-panel {
  padding: 20px;
}

.action-buttons {
  margin-top: 20px;
  margin-bottom: 30px;
  display: flex;
  gap: 10px;
}

.database-management {
  margin-top: 30px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: center;
}
</style>