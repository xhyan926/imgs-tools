import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Provider, ConversionMode, ConvertResponse, ModeInfo } from '../types';

interface ConvertPanelProps {
  providers: string[];
  providerNames: Record<string, string>;
  modes: ModeInfo[];
}

export default function ConvertPanel({ providers, providerNames, modes }: ConvertPanelProps) {
  const [url, setUrl] = useState('');
  const [from, setFrom] = useState<Provider>('aliyun');
  const [to, setTo] = useState<Provider>('tencent');
  const [mode, setMode] = useState<ConversionMode>('lenient');
  const [result, setResult] = useState<ConvertResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConvert = async () => {
    if (!url.trim()) {
      setError('请输入 URL');
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const response = await invoke<ConvertResponse>('convert_url', {
        request: {
          url,
          from,
          to,
          mode,
        },
      });
      setResult(response);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleSwap = () => {
    setFrom(to);
    setTo(from);
  };

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="bg-white rounded-lg shadow-sm p-6 mb-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-6">参数转换</h2>

        {/* 输入区域 */}
        <div className="space-y-4">
          {/* URL 输入 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              源 URL
            </label>
            <textarea
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://example.com/image.jpg?x-oss-process=image/resize,w_100,h_200"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              rows={3}
            />
          </div>

          {/* 厂商选择 */}
          <div className="grid grid-cols-3 gap-4 items-center">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                源厂商
              </label>
              <select
                value={from}
                onChange={(e) => setFrom(e.target.value as Provider)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              >
                {providers.map((p) => (
                  <option key={p} value={p}>
                    {providerNames[p] || p}
                  </option>
                ))}
              </select>
            </div>

            <div className="flex justify-center">
              <button
                onClick={handleSwap}
                className="mt-6 p-2 text-gray-400 hover:text-gray-600 transition-colors"
                title="交换厂商"
              >
                <svg
                  className="w-6 h-6"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"
                  />
                </svg>
              </button>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                目标厂商
              </label>
              <select
                value={to}
                onChange={(e) => setTo(e.target.value as Provider)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              >
                {providers.map((p) => (
                  <option key={p} value={p}>
                    {providerNames[p] || p}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* 转换模式 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              转换模式
            </label>
            <div className="grid grid-cols-3 gap-3">
              {modes.map((m) => (
                <button
                  key={m.name}
                  onClick={() => setMode(m.name as ConversionMode)}
                  className={`px-4 py-2 rounded-lg text-sm transition-colors ${
                    mode === m.name
                      ? 'bg-primary-600 text-white'
                      : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  }`}
                  title={m.description}
                >
                  {m.display_name}
                </button>
              ))}
            </div>
          </div>

          {/* 转换按钮 */}
          <div className="flex gap-3">
            <button
              onClick={handleConvert}
              disabled={loading}
              className="flex-1 bg-primary-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-primary-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
            >
              {loading ? '转换中...' : '开始转换'}
            </button>
            <button
              onClick={() => {
                setUrl('');
                setResult(null);
                setError(null);
              }}
              className="px-6 py-3 border border-gray-300 rounded-lg font-medium text-gray-700 hover:bg-gray-50 transition-colors"
            >
              清空
            </button>
          </div>
        </div>

        {/* 错误信息 */}
        {error && (
          <div className="mt-4 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-sm text-red-600">{error}</p>
          </div>
        )}
      </div>

      {/* 转换结果 */}
      {result && (
        <div className="bg-white rounded-lg shadow-sm p-6">
          <h3 className="text-lg font-bold text-gray-800 mb-4">转换结果</h3>

          {/* 状态 */}
          <div className="mb-4">
            <span
              className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                result.success
                  ? 'bg-green-100 text-green-700'
                  : 'bg-yellow-100 text-yellow-700'
              }`}
            >
              {result.success ? '✓ 转换成功' : '⚠ 部分成功'}
            </span>
          </div>

          {/* 结果 URL */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              转换后的 URL
            </label>
            <div className="relative">
              <input
                type="text"
                value={result.url}
                readOnly
                className="w-full px-4 py-2 bg-gray-50 border border-gray-300 rounded-lg text-sm"
              />
              <button
                onClick={() => navigator.clipboard.writeText(result.url)}
                className="absolute right-2 top-1/2 -translate-y-1/2 px-3 py-1 bg-white border border-gray-300 rounded text-sm hover:bg-gray-50"
              >
                复制
              </button>
            </div>
          </div>

          {/* 警告信息 */}
          {result.warnings.length > 0 && (
            <div className="mb-4">
              <h4 className="text-sm font-medium text-gray-700 mb-2">警告</h4>
              <div className="space-y-2">
                {result.warnings.map((warning, idx) => (
                  <div
                    key={idx}
                    className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg"
                  >
                    <p className="text-sm text-yellow-800">
                      <span className="font-medium">{warning.operation}:</span>{' '}
                      {warning.reason}
                    </p>
                    {warning.suggestion && (
                      <p className="text-xs text-yellow-700 mt-1">
                        建议: {warning.suggestion}
                      </p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 忽略的操作 */}
          {result.dropped.length > 0 && (
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-2">忽略的操作</h4>
              <div className="space-y-2">
                {result.dropped.map((dropped, idx) => (
                  <div
                    key={idx}
                    className="p-3 bg-gray-50 border border-gray-200 rounded-lg"
                  >
                    <p className="text-sm text-gray-800">
                      <span className="font-medium">{dropped.name}:</span>{' '}
                      {dropped.reason}
                    </p>
                    <p className="text-xs text-gray-600 mt-1">
                      原始值: {dropped.original_value}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
