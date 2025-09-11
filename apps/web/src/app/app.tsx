import { useState, useEffect, useRef } from 'react';
import { initializeData } from './initialize-data';
import { Drawnix } from '@drawnix/drawnix';
import { PlaitElement, PlaitTheme, Viewport } from '@plait/core';
import type { BoardChangeData } from '@plait-board/react-board';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core'; // 🔥 Updated import path

// 统一的元素状态管理
interface BoardState {
  elements: Map<string, PlaitElement>;
  viewport?: Viewport;
  theme?: PlaitTheme;
}

// 统一的更新操作类型
type UpdateSource = 'local' | 'remote';

export function App() {
  // 使用统一的状态管理
  const [boardState, setBoardState] = useState<BoardState>({
    elements: new Map(initializeData.map(el => [el.id, el]))
  });

  const [logs, setLogs] = useState<string[]>([]);
  const logEndRef = useRef<HTMLDivElement>(null);
  const [isConnected, setIsConnected] = useState(false);

  // 自动滚动日志到底部
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  // 统一的状态更新函数
  const applyBoardChanges = async (changeData: BoardChangeData, source: UpdateSource) => {
    console.log(`👉 收到${source === 'local' ? '本地' : '远程'}操作:`, changeData);

    const filteredOps = changeData.operations?.filter(
      (op: any) =>
        op.type === "insert_node" ||
        op.type === "remove_node" ||
        op.type === "set_node"
    ) || [];

    if (filteredOps.length === 0) return;

    // 如果是本地操作，广播到其他设备
    if (source === 'local') {
      try {
        await invoke('broadcast_board_change', {
          boardChange: {
            operations: filteredOps,
            timestamp: new Date().toISOString(),
            source_id: 'local-user' // 可以使用设备唯一标识
          }
        });
        console.log('✅ 本地操作已广播');
      } catch (error) {
        console.error('❌ 广播失败:', error);
      }
    }

    setBoardState(prevState => {
      const newElements = new Map(prevState.elements);
      let hasChanges = false;

      filteredOps.forEach((op: any) => {
        switch (op.type) {
          case "insert_node":
            if (op.node && !newElements.has(op.node.id)) {
              newElements.set(op.node.id, { ...op.node });
              console.log(`✅ ${source}插入节点: ${op.node.id}`);
              hasChanges = true;
            }
            break;

          case "remove_node":
            if (op.node && newElements.has(op.node.id)) {
              newElements.delete(op.node.id);
              console.log(`✅ ${source}删除节点: ${op.node.id}`);
              hasChanges = true;
            }
            break;

          case "set_node":
            if (op.node) {
              if (newElements.has(op.node.id)) {
                // 智能合并：保留现有属性，只更新传入的属性
                const existingNode = newElements.get(op.node.id)!;
                const updatedNode = { ...existingNode };
                
                // 只更新非空的属性
                Object.keys(op.node).forEach(key => {
                  if (op.node[key] !== null && op.node[key] !== undefined) {
                    updatedNode[key] = op.node[key];
                  }
                });
                
                newElements.set(op.node.id, updatedNode);
                console.log(`✅ ${source}更新节点: ${op.node.id}`);
                hasChanges = true;
              } else {
                // 如果节点不存在，直接添加
                newElements.set(op.node.id, { ...op.node });
                console.log(`✅ ${source}添加新节点: ${op.node.id}`);
                hasChanges = true;
              }
            }
            break;
        }
      });

      if (hasChanges) {
        return {
          ...prevState,
          elements: newElements
        };
      }
      
      return prevState;
    });

    // 记录日志
    setLogs(prev => [
      ...prev,
      ...filteredOps.map((op: any) => {
        const nodeInfo = op.node ? `${op.node.id}` : 'unknown';
        const sourceIcon = source === 'local' ? '📤' : '📥';
        return `${sourceIcon} ${source === 'local' ? '本地' : '远程'}操作: ${op.type} - ${nodeInfo}`;
      })
    ]);
  };

  // 本地操作处理
  const handleBoardChange = (newValue: BoardChangeData) => {
    applyBoardChanges(newValue, 'local');
  };

  // 远程操作处理
  const handleRemoteChange = (newValue: BoardChangeData) => {
    setIsConnected(true);
    applyBoardChanges(newValue, 'remote');
  };

  // 监听 Rust 发来的 board-change 事件
  useEffect(() => {
    const unlisten = listen<BoardChangeData>('board-change', (event) => {
      console.log('收到来自 Rust 的 BoardChangeData:', event.payload);
      handleRemoteChange(event.payload);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // 将Map转换为数组供Drawnix使用
  const elementsArray = Array.from(boardState.elements.values());

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex' }}>
      {/* 左侧：共享白板 */}
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
            background: isConnected ? '#e6f7ff' : '#f0f0f0', 
            padding: '4px 8px',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center'
          }}>
            <span>共享白板 (元素: {elementsArray.length})</span>
            <span style={{ 
              fontSize: '12px', 
              color: isConnected ? '#1890ff' : '#999'
            }}>
              {isConnected ? '🟢 已连接' : '🔴 离线'}
            </span>
          </div>
          <Drawnix
            value={elementsArray}
            viewport={boardState.viewport}
            theme={boardState.theme}
            onChange={handleBoardChange}
          />
        </div>
      </div>

      {/* 右侧：操作日志和状态信息 */}
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
        <h3 style={{ margin: '0 0 10px' }}>共享状态</h3>
        
        {/* 连接状态 */}
        <div style={{ marginBottom: '15px', fontSize: '12px' }}>
          <div style={{ fontWeight: 'bold', marginBottom: '5px' }}>连接状态:</div>
          <div style={{ 
            padding: '5px 8px', 
            background: isConnected ? '#e6f7ff' : '#fff2e8', 
            borderRadius: '4px',
            color: isConnected ? '#1890ff' : '#fa8c16'
          }}>
            {isConnected ? '✅ DDS连接正常' : '⚠️ 等待连接'}
          </div>
        </div>
        
        {/* 元素状态显示 */}
        <div style={{ marginBottom: '15px', fontSize: '12px' }}>
          <div style={{ fontWeight: 'bold', marginBottom: '5px' }}>当前元素:</div>
          <div style={{ maxHeight: '150px', overflowY: 'auto' }}>
            {elementsArray.map(el => {
              const positionInfo = el.points && el.points.length > 0 ? 
                ` [${el.points[0][0].toFixed(0)},${el.points[0][1].toFixed(0)}]` : '';
              return (
                <div key={el.id} style={{ 
                  padding: '2px 5px', 
                  background: '#e8f4fd', 
                  margin: '2px 0', 
                  borderRadius: '3px',
                  fontSize: '11px'
                }}>
                  {el.id} ({el.type}){positionInfo}
                </div>
              );
            })}
          </div>
        </div>

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
                color: log.includes('📥') ? '#1890ff' : '#52c41a'
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
