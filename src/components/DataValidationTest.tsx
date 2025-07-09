import React, { useState, useEffect } from "react";
import { Button, Card, Table, Tag, Space, Alert, Spin, Collapse } from "antd";
import {
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ReloadOutlined,
  DatabaseOutlined,
  ApiOutlined,
} from "@ant-design/icons";
import { conversationApi, settingsApi, debugApi } from "../utils/api";
import { dataValidation } from "../utils/dataValidation";
import type { Conversation, Message, HealthStatus } from "../types";

const { Panel } = Collapse;

interface TestResult {
  name: string;
  status: "success" | "error" | "warning";
  message: string;
  details?: any;
  duration?: number;
}

interface DataStats {
  conversations: number;
  messages: number;
  consistencyIssues: string[];
  senderTypes: Record<string, number>;
  averageMessageLength: number;
  timeRange: {
    start: number;
    end: number;
    duration: number;
  };
}

export const DataValidationTest: React.FC = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [testResults, setTestResults] = useState<TestResult[]>([]);
  const [dataStats, setDataStats] = useState<DataStats | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [expandedPanels, setExpandedPanels] = useState<string[]>([]);

  // 运行所有测试
  const runAllTests = async () => {
    setIsRunning(true);
    setTestResults([]);
    setDataStats(null);
    setHealthStatus(null);

    const results: TestResult[] = [];

    try {
      // 测试1: 后端连接测试
      await runTest(
        "Backend Connection",
        async () => {
          const response = await settingsApi.ping();
          if (response.success && response.data === "pong") {
            return { success: true, message: "Backend is responsive" };
          }
          throw new Error("Invalid ping response");
        },
        results,
      );

      // 测试2: 获取健康状态
      await runTest(
        "Health Status",
        async () => {
          const response = await settingsApi.getHealthStatus();
          if (response.success && response.data) {
            setHealthStatus(response.data);
            return {
              success: true,
              message: "Health status retrieved successfully",
              data: response.data,
            };
          }
          throw new Error("Failed to get health status");
        },
        results,
      );

      // 测试3: 获取对话列表
      await runTest(
        "Load Conversations",
        async () => {
          const convs = await conversationApi.getAll();
          setConversations(convs);

          // 验证数据格式
          const validationResult = dataValidation.validateConversations(convs);
          if (!validationResult) {
            throw new Error("Conversation data validation failed");
          }

          return {
            success: true,
            message: `Loaded ${convs.length} conversations`,
            data: convs,
          };
        },
        results,
      );

      // 测试4: 获取消息数据
      await runTest(
        "Load Messages",
        async () => {
          const allMessages: Message[] = [];

          // 为每个对话加载消息
          for (const conv of conversations) {
            try {
              const msgs = await conversationApi.getMessages(conv.id);
              allMessages.push(...msgs);
            } catch (error) {
              console.warn(`Failed to load messages for conversation ${conv.id}:`, error);
            }
          }

          setMessages(allMessages);

          // 验证消息数据格式
          const validationResult = dataValidation.validateMessages(allMessages);
          if (!validationResult) {
            throw new Error("Message data validation failed");
          }

          return {
            success: true,
            message: `Loaded ${allMessages.length} messages from ${conversations.length} conversations`,
            data: allMessages,
          };
        },
        results,
      );

      // 测试5: 数据一致性检查
      await runTest(
        "Data Consistency",
        async () => {
          const consistencyCheck = dataValidation.checkDataConsistency(
            messages,
            conversations,
          );

          if (!consistencyCheck.isConsistent) {
            return {
              success: false,
              message: `Found ${consistencyCheck.issues.length} consistency issues`,
              data: consistencyCheck.issues,
            };
          }

          return {
            success: true,
            message: "All data consistency checks passed",
            data: consistencyCheck,
          };
        },
        results,
      );

      // 测试6: 数据类型验证
      await runTest(
        "Data Type Validation",
        async () => {
          const issues: string[] = [];

          // 检查消息sender字段值
          const senderTypes = new Set(messages.map(m => m.sender));
          const validSenders = ["user", "assistant", "system"];

          for (const sender of senderTypes) {
            if (!validSenders.includes(sender)) {
              issues.push(`Invalid sender type: ${sender}`);
            }
          }

          // 检查时间戳合理性
          const now = Date.now();
          const oneYearAgo = now - (365 * 24 * 60 * 60 * 1000);

          messages.forEach(msg => {
            if (msg.timestamp > now + 1000 || msg.timestamp < oneYearAgo) {
              issues.push(`Message ${msg.id} has unreasonable timestamp`);
            }
          });

          conversations.forEach(conv => {
            if (conv.timestamp > now + 1000 || conv.timestamp < oneYearAgo) {
              issues.push(`Conversation ${conv.id} has unreasonable timestamp`);
            }
          });

          if (issues.length > 0) {
            return {
              success: false,
              message: `Found ${issues.length} data type issues`,
              data: issues,
            };
          }

          return {
            success: true,
            message: "All data type validations passed",
            data: { senderTypes: Array.from(senderTypes) },
          };
        },
        results,
      );

      // 测试7: 数据库状态
      await runTest(
        "Database Status",
        async () => {
          const dbStatus = await debugApi.getDatabaseStatus();
          return {
            success: true,
            message: "Database status retrieved",
            data: dbStatus,
          };
        },
        results,
      );

      // 生成数据统计
      const stats = dataValidation.generateDataStats(messages, conversations);
      setDataStats({
        conversations: conversations.length,
        messages: messages.length,
        consistencyIssues: dataValidation.checkDataConsistency(messages, conversations).issues,
        senderTypes: stats.senderDistribution,
        averageMessageLength: stats.averageMessageLength,
        timeRange: stats.timeRange,
      });

    } catch (error) {
      console.error("Test suite failed:", error);
    } finally {
      setIsRunning(false);
      setTestResults(results);
    }
  };

  // 运行单个测试的辅助函数
  const runTest = async (
    name: string,
    testFn: () => Promise<{ success: boolean; message: string; data?: any }>,
    results: TestResult[],
  ) => {
    const startTime = Date.now();
    try {
      const result = await testFn();
      const duration = Date.now() - startTime;

      results.push({
        name,
        status: result.success ? "success" : "error",
        message: result.message,
        details: result.data,
        duration,
      });
    } catch (error) {
      const duration = Date.now() - startTime;
      results.push({
        name,
        status: "error",
        message: error instanceof Error ? error.message : String(error),
        duration,
      });
    }
  };

  // 表格列定义
  const testColumns = [
    {
      title: "Test Name",
      dataIndex: "name",
      key: "name",
      width: 200,
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      width: 100,
      render: (status: string) => {
        const config = {
          success: { color: "success", icon: <CheckCircleOutlined /> },
          error: { color: "error", icon: <ExclamationCircleOutlined /> },
          warning: { color: "warning", icon: <ExclamationCircleOutlined /> },
        };
        const { color, icon } = config[status as keyof typeof config];
        return <Tag color={color} icon={icon}>{status.toUpperCase()}</Tag>;
      },
    },
    {
      title: "Message",
      dataIndex: "message",
      key: "message",
    },
    {
      title: "Duration",
      dataIndex: "duration",
      key: "duration",
      width: 100,
      render: (duration: number) => `${duration}ms`,
    },
  ];

  const conversationColumns = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      width: 80,
    },
    {
      title: "Title",
      dataIndex: "title",
      key: "title",
    },
    {
      title: "Last Message",
      dataIndex: "last_message",
      key: "last_message",
      width: 200,
      render: (text: string) => text.length > 50 ? `${text.slice(0, 50)}...` : text,
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      width: 150,
      render: (timestamp: number) => new Date(timestamp).toLocaleString(),
    },
  ];

  const messageColumns = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      width: 80,
    },
    {
      title: "Sender",
      dataIndex: "sender",
      key: "sender",
      width: 100,
      render: (sender: string) => {
        const color = sender === "user" ? "blue" : sender === "assistant" ? "green" : "orange";
        return <Tag color={color}>{sender}</Tag>;
      },
    },
    {
      title: "Content",
      dataIndex: "content",
      key: "content",
      render: (text: string) => text.length > 100 ? `${text.slice(0, 100)}...` : text,
    },
    {
      title: "Conv ID",
      dataIndex: "conversation_id",
      key: "conversation_id",
      width: 80,
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      width: 150,
      render: (timestamp: number) => new Date(timestamp).toLocaleString(),
    },
  ];

  // 组件挂载时自动运行测试
  useEffect(() => {
    runAllTests();
  }, []);

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold mb-2">Data Validation Test Suite</h1>
        <p className="text-gray-600">
          This tool validates the data integrity and consistency between frontend and backend.
        </p>
      </div>

      <div className="mb-6">
        <Button
          type="primary"
          icon={<ReloadOutlined />}
          onClick={runAllTests}
          loading={isRunning}
          size="large"
        >
          Run All Tests
        </Button>
      </div>

      {/* Health Status Overview */}
      {healthStatus && (
        <Card title="System Health" className="mb-6" size="small">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className={`text-2xl ${healthStatus.config_loaded ? 'text-green-500' : 'text-red-500'}`}>
                {healthStatus.config_loaded ? '✓' : '✗'}
              </div>
              <div className="text-sm">Config Loaded</div>
            </div>
            <div className="text-center">
              <div className={`text-2xl ${healthStatus.database_connected ? 'text-green-500' : 'text-red-500'}`}>
                {healthStatus.database_connected ? '✓' : '✗'}
              </div>
              <div className="text-sm">Database</div>
            </div>
            <div className="text-center">
              <div className={`text-2xl ${healthStatus.llm_available ? 'text-green-500' : 'text-red-500'}`}>
                {healthStatus.llm_available ? '✓' : '✗'}
              </div>
              <div className="text-sm">LLM Available</div>
            </div>
            <div className="text-center">
              <div className={`text-2xl ${healthStatus.voice_recognition_available ? 'text-green-500' : 'text-red-500'}`}>
                {healthStatus.voice_recognition_available ? '✓' : '✗'}
              </div>
              <div className="text-sm">Voice Recognition</div>
            </div>
          </div>
        </Card>
      )}

      {/* Data Statistics */}
      {dataStats && (
        <Card title="Data Statistics" className="mb-6" size="small">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-500">{dataStats.conversations}</div>
              <div className="text-sm">Conversations</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-500">{dataStats.messages}</div>
              <div className="text-sm">Messages</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-orange-500">
                {Math.round(dataStats.averageMessageLength)}
              </div>
              <div className="text-sm">Avg Message Length</div>
            </div>
            <div className="text-center">
              <div className={`text-2xl font-bold ${dataStats.consistencyIssues.length === 0 ? 'text-green-500' : 'text-red-500'}`}>
                {dataStats.consistencyIssues.length}
              </div>
              <div className="text-sm">Consistency Issues</div>
            </div>
          </div>

          {/* Sender Distribution */}
          <div className="mt-4">
            <h4 className="font-semibold mb-2">Sender Distribution:</h4>
            <Space>
              {Object.entries(dataStats.senderTypes).map(([sender, count]) => (
                <Tag key={sender} color={sender === "user" ? "blue" : "green"}>
                  {sender}: {count}
                </Tag>
              ))}
            </Space>
          </div>

          {/* Consistency Issues */}
          {dataStats.consistencyIssues.length > 0 && (
            <div className="mt-4">
              <Alert
                type="warning"
                message="Data Consistency Issues"
                description={
                  <ul className="mt-2">
                    {dataStats.consistencyIssues.map((issue, index) => (
                      <li key={index}>• {issue}</li>
                    ))}
                  </ul>
                }
              />
            </div>
          )}
        </Card>
      )}

      {/* Test Results */}
      <Card title="Test Results" className="mb-6" size="small">
        {isRunning ? (
          <div className="text-center py-8">
            <Spin size="large" />
            <div className="mt-4 text-gray-600">Running tests...</div>
          </div>
        ) : (
          <Table
            dataSource={testResults}
            columns={testColumns}
            rowKey="name"
            size="small"
            pagination={false}
            expandable={{
              expandedRowRender: (record) => (
                <pre className="bg-gray-50 p-3 rounded text-xs overflow-auto max-h-40">
                  {JSON.stringify(record.details, null, 2)}
                </pre>
              ),
              rowExpandable: (record) => !!record.details,
            }}
          />
        )}
      </Card>

      {/* Data Tables */}
      <Collapse
        activeKey={expandedPanels}
        onChange={setExpandedPanels}
        className="mb-6"
      >
        <Panel
          header={
            <Space>
              <DatabaseOutlined />
              <span>Conversations Data ({conversations.length})</span>
            </Space>
          }
          key="conversations"
        >
          <Table
            dataSource={conversations}
            columns={conversationColumns}
            rowKey="id"
            size="small"
            pagination={{ pageSize: 10 }}
            scroll={{ x: 800 }}
          />
        </Panel>

        <Panel
          header={
            <Space>
              <ApiOutlined />
              <span>Messages Data ({messages.length})</span>
            </Space>
          }
          key="messages"
        >
          <Table
            dataSource={messages}
            columns={messageColumns}
            rowKey="id"
            size="small"
            pagination={{ pageSize: 10 }}
            scroll={{ x: 1000 }}
          />
        </Panel>
      </Collapse>

      {/* Debug Information */}
      <Card title="Debug Information" size="small">
        <div className="space-y-2 text-sm">
          <div><strong>User Agent:</strong> {navigator.userAgent}</div>
          <div><strong>Current Time:</strong> {new Date().toISOString()}</div>
          <div><strong>Timezone:</strong> {Intl.DateTimeFormat().resolvedOptions().timeZone}</div>
          <div><strong>Language:</strong> {navigator.language}</div>
        </div>
      </Card>
    </div>
  );
};

export default DataValidationTest;
