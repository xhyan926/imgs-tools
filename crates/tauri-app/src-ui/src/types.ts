export type Provider = 'aliyun' | 'tencent' | 'huawei' | 'qiniu' | 'volcengine';

export type ConversionMode = 'strict' | 'lenient' | 'report';

export interface ConvertRequest {
  url: string;
  from: Provider;
  to: Provider;
  mode: ConversionMode;
}

export interface ConvertResponse {
  url: string;
  success: boolean;
  warnings: Warning[];
  dropped: Dropped[];
}

export interface Warning {
  operation: string;
  reason: string;
  suggestion: string | null;
}

export interface Dropped {
  name: string;
  original_value: string;
  reason: string;
}

export interface ValidateRequest {
  url: string;
  provider: Provider;
}

export interface ValidateResponse {
  valid: boolean;
  params: ValidatedParams | null;
  error: string | null;
}

export interface ValidatedParams {
  resize: ResizeParams | null;
  crop: CropParams | null;
  rotate: RotateParams | null;
  quality: QualityParams | null;
  format: FormatParams | null;
}

export interface ResizeParams {
  width: number | null;
  height: number | null;
  mode: string;
}

export interface CropParams {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RotateParams {
  angle: number;
}

export interface QualityParams {
  value: number | null;
  relative: number | null;
}

export interface FormatParams {
  format: string;
}

export interface FeaturesResponse {
  provider: string;
  operations: OperationInfo[];
}

export interface OperationInfo {
  name: string;
  supported: boolean;
}

export interface ModeInfo {
  name: string;
  display_name: string;
  description: string;
}
