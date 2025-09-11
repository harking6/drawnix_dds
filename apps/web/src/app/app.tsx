import { useState, useEffect, useRef } from 'react';
import { initializeData } from './initialize-data';
import { Drawnix } from '@drawnix/drawnix';
import { PlaitElement, PlaitTheme, Viewport } from '@plait/core';
import type { BoardChangeData } from '@plait-board/react-board';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

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
  const sourceIdRef = useRef<string>(() => {
    // 简单生成一个实例ID（重启前一致即可）
    return (
      'src-' + Math.random().toString(36).slice(2) + Date.now().toString(36)
    );
  });
  // 初始化 sourceId（处理函数式初始值）
  if (typeof sourceIdRef.current === 'function') {
    // @ts-expect-error runtime init
    sourceIdRef.current = sourceIdRef.current();
  }

  // 笔划会话 & 缓冲
  const inStrokeRef = useRef<boolean>(false);
  const strokeOpsRef = useRef<any[]>([]);
  const latestChildrenRef = useRef<PlaitElement[]>(value.children);

  // 监听受控 children 变化，更新最新快照引用
  useEffect(() => {
    latestChildrenRef.current = value.children;
  }, [value.children]);

  // 自动滚动日志到底部
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

// 本地操作：收集为当前笔划的增量；若非笔划则立即发送
const handleBoardChange = (data: BoardChangeData) => {
  // 写日志
  const filteredOps =
    data.operations?.filter(
      (op: any) =>
        op.type === 'insert_node' ||
        op.type === 'remove_node' ||
        op.type === 'set_node'
    ) || [];
  try {
    const opTypes = (data.operations || []).map((o: any) => o.type);
    console.log('[DRAWNIX][onChange] 收到 BoardChangeData:', {
      inStroke: inStrokeRef.current,
      opsTotal: data.operations?.length || 0,
      filteredOps: filteredOps.length,
      opTypes,
      childrenCount: data.children?.length,
    });
  } catch (e) {
    console.warn('[DRAWNIX][onChange] 打印数据失败:', e);
  }
  if (filteredOps.length) {
    setLogs((prev) => [
      ...prev,
      ...filteredOps.map((op: any) => `本地操作: ${op.type}`),
    ]);
  }

  // 同步最新 children 快照，避免发送时“晚一拍”
  if (Array.isArray(data.children)) {
    latestChildrenRef.current = data.children as PlaitElement[];
  }

  // 笔划中：只缓冲
  if (inStrokeRef.current) {
    if (data.operations?.length) {
      strokeOpsRef.current.push(...data.operations);
      console.log('[DRAWNIX][onChange] 笔划缓冲追加 ops，当前缓冲长度:', strokeOpsRef.current.length);
    }
    return;
  }

  // 非笔划类操作：立即发送一次增量
  if (data.operations?.length) {
    const envelope = {
      kind: 'op',
      source_id: sourceIdRef.current,
      timestamp: new Date().toISOString(),
      // 携带最新 children 快照（使用当前 onChange 的数据，避免延迟）
      children: (data.children as PlaitElement[]) ?? latestChildrenRef.current,
      operations: data.operations,
    };
    console.log('[DRAWNIX][send-immediate] 非笔划操作，立即发送:', {
      ops: envelope.operations.length,
      children: envelope.children?.length,
      ts: envelope.timestamp,
      src: envelope.source_id,
    });
    invoke('relay_board_change', { data: JSON.stringify(envelope) }).catch((e) => {
      console.warn('relay_board_change 失败（非 Tauri 或未连接 DDS）:', e);
    });
  }
};



// 后端推送的 BoardChangeData 应用到白板
const applyBoardChangeFromRust = (payload: any) => {
  // 忽略自身消息（若有 source_id）
  if (payload?.source_id && payload.source_id === sourceIdRef.current) {
    console.log('[DRAWNIX][recv] 忽略自身消息 source_id =', payload.source_id);
    return;
  }

  // 优先使用 children 快照，避免回放误差
  if (Array.isArray(payload?.children)) {
    console.log('[DRAWNIX][recv] 应用远端快照 children，长度 =', payload.children.length);
    setValue((prev) => ({ ...prev, children: payload.children as PlaitElement[] }));
    setLogs((prev) => [...prev, `远端快照: ${payload?.operations?.length ?? 0} ops`]);
    return;
  }

  // 兼容仅有 operations 的旧格式
  const ops: any[] = payload?.operations || [];
  if (!ops.length) return;
  console.log('[DRAWNIX][recv] 应用远端增量 ops，数量 =', ops.length);
  setValue((prev) => {
    let updatedChildren = [...prev.children];
    ops.forEach((op) => {
      if (op.type === 'insert_node' && op.node) {
        if (!updatedChildren.find((n) => (n as any).id === op.node.id)) {
          updatedChildren.push(op.node);
        }
      } else if (op.type === 'remove_node' && op.node) {
        updatedChildren = removeNode(updatedChildren, op.node.id);
      } else if (op.type === 'set_node' && op.node) {
        if (updatedChildren.find((n) => (n as any).id === op.node.id)) {
          updatedChildren = updateNode(updatedChildren, op.node);
        }
      }
    });
    return { ...prev, children: updatedChildren };
  });
  setLogs((prev) => [...prev, `远端增量: ${ops.length} ops`]);
};


  // 监听 Rust 发来的 board-change 事件
  useEffect(() => {
    console.log('[DRAWNIX][setup] 绑定 board-change 事件监听');
    const unlisten = listen<BoardChangeData>('board-change', (event) => {
      try {
        console.log('[DRAWNIX][recv] 收到来自 Rust 的 board-change:', event.payload);
      } catch {}
      applyBoardChangeFromRust(event.payload as any);
    });

    return () => {
      console.log('[DRAWNIX][cleanup] 解除 board-change 事件监听');
      unlisten.then((f) => f());
    };
  }, []); // 注意依赖数组为空，避免重复绑定

  // 监听 pointer 事件来界定一笔
  const boardHostRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const host = boardHostRef.current ?? window;
    console.log('[DRAWNIX][setup] 安装 pointer 事件监听，host 是否存在:', !!boardHostRef.current);
    const onDown = () => {
      console.log('[DRAWNIX][stroke] pointerdown -> 开始新笔划');
      inStrokeRef.current = true;
      strokeOpsRef.current = [];
    };
    const flushStroke = () => {
      if (!inStrokeRef.current) return;
      inStrokeRef.current = false;
      const ops = strokeOpsRef.current;
      strokeOpsRef.current = [];
      if (!ops.length) return;
      const envelope = {
        kind: 'op',
        source_id: sourceIdRef.current,
        timestamp: new Date().toISOString(),
        children: latestChildrenRef.current,
        operations: ops,
      };
      console.log('[DRAWNIX][stroke-send] 笔划结束，发送增量:', {
        ops: envelope.operations.length,
        children: envelope.children?.length,
        ts: envelope.timestamp,
        src: envelope.source_id,
      });
      invoke('relay_board_change', { data: JSON.stringify(envelope) }).catch((e) => {
        console.warn('relay_board_change 失败（非 Tauri 或未连接 DDS）:', e);
      });
      setLogs((prev) => [...prev, `发送笔划增量: ${ops.length} ops`]);
    };

    // 在 window 上监听 up/cancel 更稳妥
    window.addEventListener('pointerup', flushStroke, true);
    window.addEventListener('pointercancel', flushStroke, true);
    // 在白板容器上监听 down
    if (boardHostRef.current) {
      boardHostRef.current.addEventListener('pointerdown', onDown, true);
    } else {
      window.addEventListener('pointerdown', onDown, true);
    }
    return () => {
      window.removeEventListener('pointerup', flushStroke, true);
      window.removeEventListener('pointercancel', flushStroke, true);
      if (boardHostRef.current) {
        boardHostRef.current.removeEventListener('pointerdown', onDown, true);
      } else {
        window.removeEventListener('pointerdown', onDown, true);
      }
      console.log('[DRAWNIX][cleanup] 卸载 pointer 事件监听');
    };
  }, []);

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex' }}>
      {/* 左侧：单个白板 */}
      <div style={{ flex: 3, display: 'flex', padding: '10px' }} ref={boardHostRef}>
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
            onValueChange={(children) => {
              console.log('[DRAWNIX][onValueChange] 更新受控 children，长度 =', children.length);
              setValue((prev) => ({ ...prev, children }));
            }}
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
