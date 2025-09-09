import { useState, useEffect, useRef } from 'react';
import { initializeData } from './initialize-data';
import { Drawnix } from '@drawnix/drawnix';
import { PlaitElement, PlaitTheme, Viewport } from '@plait/core';
import type { BoardChangeData } from '@plait-board/react-board';
import { listen } from '@tauri-apps/api/event';

// 递归更新节点
function updateNode(nodes: PlaitElement[], node: PlaitElement): PlaitElement[] {
  return nodes.map((n) => {
    if (n.id === node.id) {
      return { ...n, ...node };
    }
    if (n.children) {
      return { ...n, children: updateNode(n.children, node) };
    }
    return n;
  });
}

// 递归删除节点
function removeNode(nodes: PlaitElement[], nodeId: string): PlaitElement[] {
  return nodes
    .filter((n) => n.id !== nodeId)
    .map((n) =>
      n.children ? { ...n, children: removeNode(n.children, nodeId) } : n
    );
}

export function App() {
  const [value, setValue] = useState<{
    children: PlaitElement[];
    viewport?: Viewport;
    theme?: PlaitTheme;
  }>({ children: structuredClone(initializeData) });

  const [logs, setLogs] = useState<string[]>([]);
  const logEndRef = useRef<HTMLDivElement>(null);
  const prevElementsRef = useRef<PlaitElement[]>([]);

  // 自动滚动日志到底部
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  // 本地操作处理
  // 本地操作处理
const handleBoardChange = (newValue: BoardChangeData) => {
  console.log("👉 收到本地 BoardChangeData:", newValue);

  const filteredOps =
    newValue.operations?.filter(
      (op: any) =>
        op.type === "insert_node" ||
        op.type === "remove_node" ||
        op.type === "set_node"
    ) || [];

  if (filteredOps.length > 0) {
    // ✅ 更新 React 状态
    setValue((prev) => {
      let updatedChildren = [...prev.children];

      filteredOps.forEach((op: any) => {
        if (op.type === "insert_node" && op.node) {
          if (!updatedChildren.find((n) => n.id === op.node.id)) {
            updatedChildren.push(op.node);
          }
        } else if (op.type === "remove_node" && op.node) {
          updatedChildren = removeNode(updatedChildren, op.node.id);
        } else if (op.type === "set_node" && op.node) {
          if (updatedChildren.find((n) => n.id === op.node.id)) {
            updatedChildren = updateNode(updatedChildren, op.node);
          }
        }
      });

      return { ...prev, children: updatedChildren };
    });

    // ✅ 写日志
    setLogs((prev) => [
      ...prev,
      ...filteredOps.map((op: any) => `本地操作: ${op.type}`),
    ]);
  }
};



  // 后端推送的 BoardChangeData 应用到白板（增量更新）
  const applyBoardChangeFromRust = (newValue: BoardChangeData) => {
  setValue((prev) => {
    let updatedChildren = [...prev.children];

    (newValue.operations || []).forEach((op: any) => {
      if (op.type === "insert_node" && op.node) {
        if (!updatedChildren.find((n) => n.id === op.node.id)) {
          updatedChildren.push(op.node);
        }
      } else if (op.type === "remove_node" && op.node) {
        updatedChildren = removeNode(updatedChildren, op.node.id);
      } else if (op.type === "set_node" && op.node) {
        if (updatedChildren.find((n) => n.id === op.node.id)) {
          updatedChildren = updateNode(updatedChildren, op.node);
        } else {
          console.warn("⚠️ set_node 收到未知 id, 忽略:", op.node.id);
        }
      }
    });

    return { ...prev, children: updatedChildren };
  });
};


  // 监听 Rust 发来的 board-change 事件
  useEffect(() => {
    const unlisten = listen<BoardChangeData>('board-change', (event) => {
      console.log('收到来自 Rust 的 BoardChangeData:', event.payload);
      applyBoardChangeFromRust(event.payload);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []); // 注意依赖数组为空，避免重复绑定

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex' }}>
      {/* 左侧：单个白板 */}
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

      {/* 右侧：操作日志 */}
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
