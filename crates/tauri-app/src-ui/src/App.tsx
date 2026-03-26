import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ConvertPanel from './components/ConvertPanel';
import FeaturePanel from './components/FeaturePanel';
import type { ModeInfo } from './types';

function App() {
  const [providers, setProviders] = useState<string[]>([]);
  const [modes, setModes] = useState<ModeInfo[]>([]);
  const [activeTab, setActiveTab] = useState<'convert' | 'features'>('convert');

  useEffect(() => {
    // 加载厂商列表
    invoke<string[]>('get_providers').then(setProviders).catch(console.error);

    // 加载转换模式
    invoke<ModeInfo[]>('get_conversion_modes').then(setModes).catch(console.error);
  }, []);

  const providerNames: Record<string, string> = {
    'aliyun': '阿里云 OSS',
    'tencent': '腾讯云 CI',
    'huawei': '华为云 OBS',
    'qiniu': '七牛云',
    'volcengine': '火山引擎',
  };

  return (
    <div className="h-screen w-screen bg-gray-50 flex">
      {/* 侧边栏 */}
      <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-6 border-b border-gray-200">
          <h1 className="text-xl font-bold text-gray-800">云厂商图片转换器</h1>
          <p className="text-sm text-gray-500 mt-1">图片处理参数转换工具</p>
        </div>

        <nav className="flex-1 p-4">
          <button
            onClick={() => setActiveTab('convert')}
            className={`w-full text-left px-4 py-2 rounded-lg mb-2 transition-colors ${
              activeTab === 'convert'
                ? 'bg-primary-100 text-primary-700 font-medium'
                : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            🔄 参数转换
          </button>
          <button
            onClick={() => setActiveTab('features')}
            className={`w-full text-left px-4 py-2 rounded-lg transition-colors ${
              activeTab === 'features'
                ? 'bg-primary-100 text-primary-700 font-medium'
                : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            📋 功能对比
          </button>
        </nav>

        <div className="p-4 border-t border-gray-200">
          <div className="text-xs text-gray-400">
            版本 0.1.0
          </div>
        </div>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 overflow-auto">
        {activeTab === 'convert' ? (
          <ConvertPanel
            providers={providers}
            providerNames={providerNames}
            modes={modes}
          />
        ) : (
          <FeaturePanel
            providers={providers}
            providerNames={providerNames}
          />
        )}
      </div>
    </div>
  );
}

export default App;
