<template>
  <div class="database-config-panel">
    <h2>数据库配置</h2>
    
    <el-form label-position="top">
      <el-form-item label="启用数据库">
        <el-switch
          v-model="config.database.enabled"
          active-text="启用"
          inactive-text="禁用"
        />
      </el-form-item>
      
      <template v-if="config.database.enabled">
        <el-form-item label="数据库路径">
          <el-input v-model="config.database.path" placeholder="请输入数据库路径">
            <template #append>
              <el-button @click="selectDatabasePath">选择路径</el-button>
            </template>
          </el-input>
        </el-form-item>
      </template>
    </el-form>
    
    <div class="action-buttons">
      <el-button type="primary" @click="saveConfig">保存配置</el-button>
      <el-button @click="resetConfig">重置</el-button>
    </div>

    <!-- 数据库内容管理 -->
    <div v-if="config.database.enabled" class="database-management">
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

<script setup>
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/api/event';
import { ElMessageBox } from 'element-plus';
import { ElMessage } from 'element-plus';

const config = reactive({
  database: {
    enabled: true,
    path: ''
  }
});

const loading = ref(false);
const conversations = ref([]);
const totalConversations = ref(0);
const currentPage = ref(1);
const pageSize = ref(10);

// 加载配置
onMounted(async () => {
  try {
    const appConfig = await invoke('get_app_config');
    config.database = appConfig.database;
    
    if (config.database.enabled) {
      loadConversations();
    }
  } catch (error) {
    ElMessage({
      message:'加载配置失败: ' + error,
      type: 'error'
    });
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
    if (selected) {
      config.database.path = selected;
    }
  } catch (error) {
    ElMessage({
      message:'选择文件失败: ' + error,
      type: 'error'
    });
  }
}

// 保存配置
async function saveConfig() {
  try {
    await invoke('save_app_config', { config });
    ElMessage({
      message:'保存配置成功',
      type: 'success'
    });
  } catch (error) {
    ElMessage({
      message:'保存配置失败: ' + error,
      type: 'error'
    });
  }
}

// 重置配置
async function resetConfig() {
  try {
    const appConfig = await invoke('get_app_config');
    config.database = appConfig.database;
  } catch (error) {
    ElMessage({
      message:'重置配置失败: ' + error,
      type: 'error'
    });
  }
}

// 加载对话列表
async function loadConversations() {
  loading.value = true;
  try {
    const allConversations = await invoke('get_database_conversations');
    totalConversations.value = allConversations.length;
    
    // 简单的分页实现
    const start = (currentPage.value - 1) * pageSize.value;
    const end = start + pageSize.value;
    conversations.value = allConversations.slice(start, end);
  } catch (error) {
    ElMessage({
      message:'加载对话失败: ' + error,
      type: 'error'
    });
  } finally {
    loading.value = false;
  }
}

// 处理分页变化
function handleCurrentChange(page) {
  currentPage.value = page;
  loadConversations();
}

// 删除对话
async function handleDelete(row) {
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
    ElMessage({
      message:'删除成功',
      type: 'success'
    });
    loadConversations();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage({
        message:'删除失败: ' + error,
        type: 'error'
      });
    }
  }
}

// 格式化日期
function formatDate(timestamp) {
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