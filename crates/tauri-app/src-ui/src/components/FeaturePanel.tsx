import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FeaturesResponse } from '../types';

interface FeaturePanelProps {
  providers: string[];
  providerNames: Record<string, string>;
}

export default function FeaturePanel({ providers, providerNames }: FeaturePanelProps) {
  const [allFeatures, setAllFeatures] = useState<Record<string, FeaturesResponse>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // 加载所有厂商的功能信息
    const loadAllFeatures = async () => {
      setLoading(true);
      const results: Record<string, FeaturesResponse> = {};

      for (const provider of providers) {
        try {
          const response = await invoke<FeaturesResponse>('get_features', {
            provider,
          });
          results[provider] = response;
        } catch (e) {
          console.error(`Failed to load features for ${provider}:`, e);
        }
      }

      setAllFeatures(results);
      setLoading(false);
    };

    loadAllFeatures();
  }, [providers]);

  const operationIcons: Record<string, string> = {
    '缩放': '🔍',
    '裁剪': '✂️',
    '旋转': '🔄',
    '质量': '⚙️',
    '格式转换': '📄',
    '渐进式加载': '📊',
  };

  return (
    <div className="p-8">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-800">功能对比</h2>
        <p className="text-sm text-gray-500 mt-1">各云厂商支持的操作对比</p>
      </div>

      {loading ? (
        <div className="text-center py-12">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          <p className="mt-4 text-gray-600">加载中...</p>
        </div>
      ) : (
        <div className="bg-white rounded-lg shadow-sm overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-48">
                  操作
                </th>
                {providers.map((provider) => (
                  <th
                    key={provider}
                    className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    {providerNames[provider] || provider}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {['缩放', '裁剪', '旋转', '质量', '格式转换', '渐进式加载'].map((operation) => (
                <tr key={operation} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <span className="mr-2">{operationIcons[operation]}</span>
                      <span className="text-sm font-medium text-gray-900">
                        {operation}
                      </span>
                    </div>
                  </td>
                  {providers.map((provider) => {
                    const providerFeatures = allFeatures[provider];
                    const op = providerFeatures?.operations.find((o) => o.name === operation);
                    const supported = op?.supported ?? false;

                    return (
                      <td key={provider} className="px-6 py-4 whitespace-nowrap text-center">
                        {supported ? (
                          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                            ✓ 支持
                          </span>
                        ) : (
                          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                            ✗ 不支持
                          </span>
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 说明 */}
      <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h4 className="text-sm font-medium text-blue-800 mb-2">💡 说明</h4>
        <ul className="text-xs text-blue-700 space-y-1">
          <li>• 不同云厂商的图片处理 API 格式各异，本工具提供统一的转换接口</li>
          <li>• 部分高级功能可能需要特定参数配置，转换时可能丢失或降级处理</li>
          <li>• 建议在实际使用前先验证转换后的 URL 是否符合预期效果</li>
        </ul>
      </div>
    </div>
  );
}
