import { useState, useEffect, useRef } from 'react';
import { initializeData } from './initialize-data';
import { Drawnix } from '@drawnix/drawnix';
import { PlaitElement, PlaitTheme, Viewport } from '@plait/core';
import type { BoardChangeData } from '@plait-board/react-board';
import { listen } from '@tauri-apps/api/event';

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

  // 自动滚动日志到底部
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

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
  // 前端操作：只改结构 + 写日志
  const handleBoardChange = (newValue: BoardChangeData) => {
    console.log('👉 收到本地 BoardChangeData:', newValue);
    const filteredOps =
      newValue.operations?.filter((op: any) =>
        ['insert_node', 'remove_node', 'set_node'].includes(op.type)
      ) || [];
    if (filteredOps.length > 0) {
      applyOperationsToBoardState(filteredOps);
      setLogs((prev) => [...prev, ...filteredOps.map((op) => `本地操作: ${op.type}`)]);
      // ⚠️ 注意：这里不调用 setValue，不触发渲染
    }
  };

  // 后端推送：改结构 + 渲染 + 写日志
  const applyBoardChangeFromRust = (newValue: BoardChangeData) => {
    console.log('收到来自 Rust 的 BoardChangeData:', newValue);
    if (newValue.operations?.length) {
      applyOperationsToBoardState(newValue.operations);

      // ⚡ 渲染（后端为准）
      setValue((prev) => ({
        ...prev,
        children: boardStateRef.current,
      }));

      setLogs((prev) => [
        ...prev,
        ...newValue.operations.map((op: any) => `后端操作: ${op.type}`),
      ]);
    }
  };

  // ======================================================== //
  // 监听 Rust 事件
  useEffect(() => {
    const unlisten = listen<BoardChangeData>('board-change', (event) => {
      applyBoardChangeFromRust(event.payload);
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
          <div style={{ background: '#f0f0f0', padding: '4px 8px' }}>白板</div>
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
