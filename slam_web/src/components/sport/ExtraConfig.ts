import { TEXTS } from '../../i18n';
import { SportType } from '../../services/sport';
import type { Swimming, Running, Track } from '../../services/sport';

export type FieldKind = 'select' | 'number' | 'text';
export type FieldOption = { value: string; label: string };
export type FieldConfig = {
  key: string;
  label: string;
  kind: FieldKind;
  options?: FieldOption[];
  parse?: (v: string) => any;
  default: string | number;
};

// 布局配置：定义每一行放多少个字段
export type LayoutConfig = {
  rowFieldCounts: number[];
};

// 根据布局配置将一维字段列表分组成行
export function groupByLayout(fields: FieldConfig[], layout: LayoutConfig): FieldConfig[][] {
  const rows: FieldConfig[][] = [];
  let idx = 0;
  for (const count of layout.rowFieldCounts) {
    if (count <= 0) continue;
    if (idx >= fields.length) break;
    rows.push(fields.slice(idx, idx + count));
    idx += count;
  }
  // 若还有剩余字段，按最后一行的数量继续切分；若布局为空则每行一个
  const last = layout.rowFieldCounts[layout.rowFieldCounts.length - 1] || 1;
  while (idx < fields.length) {
    rows.push(fields.slice(idx, idx + last));
    idx += last;
  }
  return rows;
}

// 生成均匀布局（每行固定数量）
export function makeUniformLayout(total: number, perRow: number): LayoutConfig {
  const counts: number[] = [];
  if (perRow <= 0) return { rowFieldCounts: [] };
  let remaining = total;
  while (remaining > 0) {
    counts.push(Math.min(perRow, remaining));
    remaining -= perRow;
  }
  return { rowFieldCounts: counts };
}

function buildSwimmingConfig(lang: 'zh' | 'en'): FieldConfig[] {
  return [
    {
      key: 'main_stroke',
      label: TEXTS[lang].addsports.submitStrokeLabel,
      kind: 'select',
      default: 'unknown',
      options: [
        { label: TEXTS[lang].addsports.strokeUnknown, value: 'unknown' },
        { label: TEXTS[lang].addsports.strokeFreestyle, value: 'freestyle' },
        { label: TEXTS[lang].addsports.strokeButterfly, value: 'butterfly' },
        { label: TEXTS[lang].addsports.strokeBreaststroke, value: 'breaststroke' },
        { label: TEXTS[lang].addsports.strokeBackstroke, value: 'backstroke' },
        { label: TEXTS[lang].addsports.strokeMedley, value: 'medley' },
      ],
    },
    {
      key: 'stroke_avg',
      label: TEXTS[lang].addsports.submitStrokeAvgLabel,
      kind: 'number',
      default: 0,
      parse: v => Number.parseInt(v || '0'),
    },
    {
      key: 'swolf_avg',
      label: TEXTS[lang].addsports.submitSwolfAvgLabel,
      kind: 'number',
      default: 0,
      parse: v => Number.parseInt(v || '0'),
    },
  ];
}

function buildRunningConfig(lang: 'zh' | 'en'): FieldConfig[] {
  return [
    {
      key: 'speed_avg',
      label: TEXTS[lang].addsports.runSpeedAvgLabel,
      kind: 'number',
      default: 0,
      parse: v => Number.parseFloat(v || '0'),
    },
    {
      key: 'cadence_avg',
      label: TEXTS[lang].addsports.runCadenceAvgLabel,
      kind: 'number',
      default: 0,
      parse: v => Number.parseInt(v || '0'),
    },
    {
      key: 'stride_length_avg',
      label: TEXTS[lang].addsports.runStrideLengthAvgLabel,
      kind: 'number',
      default: 0,
      parse: v => Number.parseInt(v || '0'),
    },
    {
      key: 'steps_total',
      label: TEXTS[lang].addsports.runStepsTotalLabel,
      kind: 'number',
      default: 0,
      parse: v => Number.parseInt(v || '0'),
    },
    {
      key: 'pace_min',
      label: TEXTS[lang].addsports.runPaceMinLabel,
      kind: 'text',
      default: '0\'00\'\'',
    },
    {
      key: 'pace_max',
      label: TEXTS[lang].addsports.runPaceMaxLabel,
      kind: 'text',
      default: '0\'00\'\'',
    },
  ];
}

export const EXTRA_CONFIG_BUILDERS: Record<SportType, (lang: 'zh' | 'en') => FieldConfig[]> = {
  [SportType.Swimming]: buildSwimmingConfig,
  [SportType.Running]: buildRunningConfig,
  [SportType.Cycling]: () => [],
  [SportType.Unknown]: () => [],
};

export function getExtraConfigByType(lang: 'zh' | 'en', sportType: SportType): FieldConfig[] {
  const builder = EXTRA_CONFIG_BUILDERS[sportType] || (() => []);
  return builder(lang);
}

// 默认专项数据（用于构建表单模型及重置）
export const DEFAULT_SWIMMING_EXTRA: Swimming = {
  main_stroke: 'unknown',
  stroke_avg: 0,
  swolf_avg: 0,
};

export const DEFAULT_RUNNING_EXTRA: Running = {
  speed_avg: 0,
  cadence_avg: 0,
  stride_length_avg: 0,
  steps_total: 0,
  pace_min: '0\'00\'\'',
  pace_max: '0\'00\'\'',
};

export function getDefaultExtraByType(sportType: SportType): Swimming | Running | null {
  switch (sportType) {
    case SportType.Swimming:
      return DEFAULT_SWIMMING_EXTRA;
    case SportType.Running:
      return DEFAULT_RUNNING_EXTRA;
    default:
      return null;
  }
}

// Track 默认值，结合专项默认 extra
export function getDefaultTrackByType(sportType: SportType): Track {
  const extra = getDefaultExtraByType(sportType);
  return {
    distance_meter: 0,
    duration_second: 0,
    pace_average: '0',
    extra: extra ?? undefined,
  } as Track;
}
