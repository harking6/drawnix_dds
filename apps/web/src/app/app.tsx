import { useState, useEffect, useRef } from 'react';
import { initializeData } from './initialize-data';
import { Drawnix } from '@drawnix/drawnix';
import { PlaitElement, PlaitTheme, Viewport } from '@plait/core';
import type { BoardChangeData } from '@plait-board/react-board';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// ===================== 全局维护的结构 ===================== //
// 用 useRef 存储所有节点和位置，不随渲染丢失
const boardStateRef: { current: PlaitElement[] } = { current: structuredClone(initializeData) };

export function App() {
  // React 状态仅用于驱动渲染
  const [value, setValue] = useState<{
    children: PlaitElement[];
    viewport?: Viewport;
    theme?: PlaitTheme;
  }>({ children: boardStateRef.current });

  // 日志列表
  const [logs, setLogs] = useState<string[]>([]);
  const logEndRef = useRef<HTMLDivElement>(null);
  
  // DDS连接状态
  const [ddsConnected, setDdsConnected] = useState<boolean>(false);
  
  // 防止回环发布的标志
  const isApplyingRemoteChange = useRef<boolean>(false);

  // 自动滚动日志到底部
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  // 检查DDS连接状态
  useEffect(() => {
    const checkDdsStatus = async () => {
      try {
        const status = await invoke<boolean>('get_dds_status');
        setDdsConnected(status);
        setLogs(prev => [...prev, status ? '✅ DDS连接已建立' : '⚠️ DDS未连接，仅本地模式']);
      } catch (error) {
        console.error('检查DDS状态失败:', error);
        setDdsConnected(false);
      }
    };
    
    checkDdsStatus();
  }, []);

  // ======================================================== //
  // 工具函数：递归删除节点
  const removeNodeAtPath = (
    nodes: PlaitElement[],
    path: number[]
  ): [PlaitElement[], PlaitElement | null] => {
    const updated = [...nodes];
    const [head, ...rest] = path;
    if (rest.length === 0) {
      const removed = updated.splice(head, 1)[0];
      return [updated, removed || null];
    } else if (updated[head]?.children) {
      const [newChildren, removed] = removeNodeAtPath(updated[head].children!, rest);
      updated[head] = { ...updated[head], children: newChildren };
      return [updated, removed];
    }
    return [updated, null];
  };

  // 工具函数：递归插入节点
  const insertNodeAtPath = (
    nodes: PlaitElement[],
    path: number[],
    node: PlaitElement
  ): PlaitElement[] => {
    const updated = [...nodes];
    const [head, ...rest] = path;
    if (rest.length === 0) {
      updated.splice(head, 0, node);
    } else if (updated[head]) {
      updated[head] = {
        ...updated[head],
        children: insertNodeAtPath(updated[head].children || [], rest, node),
      };
    }
    return updated;
  };

  // 递归更新节点的部分属性（合并更新）
  const updateNodePropertiesAtPath = (
    nodes: PlaitElement[],
    path: number[],
    properties: Partial<PlaitElement>
  ): PlaitElement[] => {
    const updated = [...nodes];
    const [head, ...rest] = path;
    if (rest.length === 0) {
      if (updated[head]) {
        updated[head] = { ...updated[head], ...properties };
      }
    } else if (updated[head]?.children) {
      updated[head] = {
        ...updated[head],
        children: updateNodePropertiesAtPath(updated[head].children!, rest, properties),
      };
    }
    return updated;
  };

  // 工具函数：递归更新节点（这里直接替换）
  const updateNodeAtPath = (
    nodes: PlaitElement[],
    path: number[],
    newNode: PlaitElement
  ): PlaitElement[] => {
    const updated = [...nodes];
    const [head, ...rest] = path;
    if (rest.length === 0) {
      if (updated[head]) {
        updated[head] = newNode; // ⚡ 直接替换为新的节点
      }
    } else if (updated[head]?.children) {
      updated[head] = {
        ...updated[head],
        children: updateNodeAtPath(updated[head].children!, rest, newNode),
      };
    }
    return updated;
  };

  // ======================================================== //
  // 统一操作处理：只改 boardStateRef，不立即触发渲染
  const applyOperationsToBoardState = (ops: any[]) => {
    let updatedChildren = [...boardStateRef.current];

    ops.forEach((op) => {
      switch (op.type) {
        case 'insert_node':
          if (op.node && op.path) {
            updatedChildren = insertNodeAtPath(updatedChildren, op.path, op.node);
          }
          break;

        case 'remove_node':
          if (op.path) {
            const [newChildren] = removeNodeAtPath(updatedChildren, op.path);
            updatedChildren = newChildren;
          }
          break;

        case 'set_node':
          if (op.path) {
            if (op.node) {
              // 情况1：全量 node（后端规约）
              updatedChildren = updateNodeAtPath(updatedChildren, op.path, op.node);
            } else if (op.properties) {
              // 情况2：增量 properties（前端拖动）
              updatedChildren = updateNodePropertiesAtPath(updatedChildren, op.path, op.properties);
            }
          }
          break;

        default:
          console.warn(`未知操作类型: ${op.type}`, op);
      }
    });

    boardStateRef.current = updatedChildren;
  };

  // ======================================================== //
  // 发布白板变化到DDS
  const publishToDDS = async (boardChangeData: BoardChangeData) => {
    if (!ddsConnected) {
      return; // DDS未连接，不发布
    }
    
    try {
      await invoke('publish_board_change', { boardData: boardChangeData });
      setLogs(prev => [...prev, `📤 已发布到DDS: ${boardChangeData.operations?.length || 0} 个操作`]);
    } catch (error) {
      console.error('DDS发布失败:', error);
      setLogs(prev => [...prev, `❌ DDS发布失败: ${error}`]);
    }
  };

  // 前端操作：改结构 + 发布到DDS + 写日志
  const handleBoardChange = async (newValue: BoardChangeData) => {
    // 如果正在应用远程变化，不处理本地变化
    if (isApplyingRemoteChange.current) {
      return;
    }
    
    console.log('👉 收到本地 BoardChangeData:', newValue);
    const filteredOps =
      newValue.operations?.filter((op: any) =>
        ['insert_node', 'remove_node', 'set_node'].includes(op.type)
      ) || [];
      
    if (filteredOps.length > 0) {
      // 应用到本地状态
      applyOperationsToBoardState(filteredOps);
      
      // 发布到DDS
      const boardChangeForDDS = {
        ...newValue,
        operations: filteredOps
      };
      await publishToDDS(boardChangeForDDS);
      
      // 记录日志
      setLogs((prev) => [...prev, ...filteredOps.map((op) => `🔧 本地操作: ${op.type}`)]);
      
      // ⚠️ 注意：这里不调用 setValue，不触发渲染
    }
  };

  // 处理来自远程的白板变化
  const applyRemoteBoardChange = (remoteData: BoardChangeData) => {
    console.log('📨 收到来自远程的 BoardChangeData:', remoteData);
    
    if (remoteData.operations?.length) {
      // 设置标志，防止回环
      isApplyingRemoteChange.current = true;
      
      // 应用远程操作到本地状态
      applyOperationsToBoardState(remoteData.operations);

      // ⚡ 触发渲染（远程变化需要立即显示）
      setValue((prev) => ({
        ...prev,
        children: boardStateRef.current,
      }));

      // 记录日志
      setLogs((prev) => [
        ...prev,
        ...remoteData.operations.map((op: any) => `📡 远程操作: ${op.type}`),
      ]);
      
      // 重置标志
      setTimeout(() => {
        isApplyingRemoteChange.current = false;
      }, 100);
    }
  };

  // ======================================================== //
  // 监听来自Tauri后端的远程白板变化
  useEffect(() => {
    const unlisten = listen<BoardChangeData>('remote-board-change', (event) => {
      applyRemoteBoardChange(event.payload);
    });
    
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // ======================================================== //
  return (
    <div style={{ width: '100%', height: '100%', display: 'flex' }}>
      {/* 左侧：白板 */}
      <div style={{ flex: 3, display: 'flex', padding: '10px' }}>
        <div
          style={{
            flex: 1,
            border: '1px solid #ddd',
            borderRadius: '8px',
            overflow: 'hidden',
          }}
        >
          <div style={{ 
            background: ddsConnected ? '#e6f7ff' : '#fff2e8', 
            padding: '4px 8px',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center'
          }}>
            <span>白板</span>
            <span style={{ 
              fontSize: '12px', 
              color: ddsConnected ? '#52c41a' : '#fa8c16'
            }}>
              {ddsConnected ? '🟢 DDS已连接' : '🟡 仅本地模式'}
            </span>
          </div>
          <Drawnix
            value={value.children}
            viewport={value.viewport}
            theme={value.theme}
            onChange={handleBoardChange}
          />
        </div>
      </div>

      {/* 右侧：日志 */}
      <div
        style={{
          flex: 1,
          borderLeft: '2px solid #eee',
          background: '#fafafa',
          padding: '10px',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <h3 style={{ margin: '0 0 10px' }}>操作日志</h3>
        <div style={{ flex: 1, overflowY: 'auto' }}>
          {logs.map((log, i) => (
            <div
              key={i}
              style={{
                padding: '6px 10px',
                marginBottom: '6px',
                borderRadius: '6px',
                background: i === logs.length - 1 ? '#e6f7ff' : '#fff',
                border: '1px solid #ddd',
                fontSize: '14px',
              }}
            >
              {log}
            </div>
          ))}
          <div ref={logEndRef}></div>
        </div>
      </div>
    </div>
  );
}

export default App;
