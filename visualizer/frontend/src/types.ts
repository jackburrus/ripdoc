export interface UploadResult {
  page_count: number;
  metadata: Record<string, string>;
  filename: string;
}

export interface PageInfo {
  page_number: number;
  width: number;
  height: number;
  char_count: number;
  bbox: [number, number, number, number];
}

export interface Char {
  text: string;
  fontname: string;
  size: number;
  x0: number;
  x1: number;
  top: number;
  bottom: number;
  doctop: number;
  upright: boolean;
  adv: number;
}

export interface Word {
  text: string;
  x0: number;
  x1: number;
  top: number;
  bottom: number;
  doctop: number;
  upright: boolean;
}

export interface Line {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  top: number;
  bottom: number;
  width: number;
}

export interface Rect {
  x0: number;
  top: number;
  x1: number;
  bottom: number;
  width: number;
  height: number;
  linewidth: number;
}

export interface Edge {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  top: number;
  bottom: number;
  width: number;
}

export interface TableBBox {
  x0: number;
  top: number;
  x1: number;
  bottom: number;
}

export interface TableData {
  bbox: TableBBox;
  row_count: number;
  col_count: number;
  grid: (string | null)[][];
  html: string;
}

export interface SearchMatch {
  text: string;
  x0: number;
  top: number;
  x1: number;
  bottom: number;
  page_number: number;
}

export interface TimedResponse<T> {
  data: T;
  timing_ms: number;
}

export type BenchmarkResults = Record<string, Record<string, number>>;

export type LayerName =
  | "chars"
  | "words"
  | "lines"
  | "rects"
  | "edges"
  | "tables"
  | "search";

export interface LayerState {
  chars: boolean;
  words: boolean;
  lines: boolean;
  rects: boolean;
  edges: boolean;
  tables: boolean;
  search: boolean;
}
