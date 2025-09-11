import { invoke } from '@tauri-apps/api/core';
import { PlaitElement, PlaitOperation } from '@plait/core';

// 扩展 Window 接口以包含 Tauri 对象
declare global {
  interface Window {
    __TAURI__?: {
      core?: {
        invoke: typeof invoke;
      };
    };
  }
}

// 检查是否在 Tauri 环境中运行
export const isTauriEnvironment = (): boolean => {
  // 检查 Tauri 2.x 的全局对象
  if (typeof window !== 'undefined' && 
      window.__TAURI__ && 
      window.__TAURI__.core) {
    console.log('✅ [TAURI] 检测到 Tauri 2.x 环境 (全局对象)');
    return true;
  }
  
  // 检查是否有 Tauri API 包
  try {
    // 这个检查可能在某些情况下有效
    const isTauri = typeof window !== 'undefined' && 
                   'window' in window && 
                   '__TAURI__' in window;
    if (isTauri) {
      console.log('✅ [TAURI] 检测到 Tauri 环境 (window.__TAURI__)');
      return true;
    }
  } catch (error) {
    console.log('🔍 [TAURI] 检测过程中出现错误:', error);
  }
  
  console.log('🌐 [TAURI] 当前运行在 Web 环境中');
  return false;
};

// 板子变化数据结构
export interface BoardChangeData {
  children_count: number;
  operation_count: number;
  operations: PlaitOperation[];
  changes: ElementChange[];
  timestamp: string;
}

// 元素变化数据结构
export interface ElementChange {
  operation_type: string;
  element_id?: string;
  element_type?: string;
  element_data?: PlaitElement | Record<string, unknown>;
  timestamp: string;
}

// 测试 Tauri 连接
export const testTauriConnection = async (): Promise<boolean> => {
  if (!isTauriEnvironment()) {
    console.log('🚫 [TAURI] 不在 Tauri 环境中');
    return false;
  }

  try {
    const result = await invoke('test_connection');
    console.log('🧪 [TAURI] 连接测试结果:', result);
    return true;
  } catch (error) {
    console.error('❌ [TAURI] 连接测试失败:', error);
    return false;
  }
};


export const sendBoardChangeToRust = async (data: BoardChangeData): Promise<void> => {
  if (!isTauriEnvironment()) {
    console.log('🚫 [TAURI] 不在 Tauri 环境中，跳过 Rust 调用');
    return;
  }

  try {
    const result = await invoke('handle_board_change', { data });
    console.log('✅ [TAURI] Rust 处理结果:', result);
  } catch (error) {
    console.error('❌ [TAURI] 调用 Rust 失败:', error);
  }
};

// 发送元素变化详情到 Rust 后端
export const sendElementChangesToRust = async (
  added: PlaitElement[],
  removed: PlaitElement[],
  modified: PlaitElement[],
  timestamp: string
): Promise<void> => {
  if (!isTauriEnvironment()) {
    console.log('🚫 [TAURI] 不在 Tauri 环境中，跳过 Rust 调用');
    return;
  }

  try {
    const result = await invoke('handle_element_changes', {
      added,
      removed,
      modified,
      timestamp
    });
    console.log('✅ [TAURI] 元素变化处理结果:', result);
  } catch (error) {
    console.error('❌ [TAURI] 调用 Rust 失败:', error);
  }
};

// 将操作转换为元素变化
export const convertOperationsToChanges = (operations: PlaitOperation[]): ElementChange[] => {
  const changes: ElementChange[] = [];
  const timestamp = new Date().toISOString();

  operations.forEach(op => {
    const change: ElementChange = {
      operation_type: op.type,
      timestamp
    };

    switch (op.type) {
      case 'insert_node':
        if ('node' in op && op.node) {
          const node = op.node as PlaitElement;
          change.element_id = node.id;
          change.element_type = node.type;
          change.element_data = node;
        }
        break;
      
      case 'remove_node':
        if ('node' in op && op.node) {
          const node = op.node as PlaitElement;
          change.element_id = node.id;
          change.element_type = node.type;
          change.element_data = node;
        }
        break;
      
      case 'set_node':
        change.element_data = {
          path: 'path' in op ? op.path : undefined,
          properties: 'properties' in op ? op.properties : undefined,
          newProperties: 'newProperties' in op ? op.newProperties : undefined
        };
        break;
      
      default:
        change.element_data = op as Record<string, unknown>;
        break;
    }

    changes.push(change);
  });

  return changes;
};
