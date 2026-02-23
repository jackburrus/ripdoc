import type {
  UploadResult,
  PageInfo,
  Char,
  Word,
  Line,
  Rect,
  Edge,
  TableData,
  SearchMatch,
  TimedResponse,
  BenchmarkResults,
} from "./types";

async function fetchJSON<T>(url: string, method: "GET" | "POST" = "GET"): Promise<T> {
  const res = await fetch(url, { method });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status}: ${text}`);
  }
  return res.json();
}

export async function uploadPdf(file: File): Promise<UploadResult> {
  const form = new FormData();
  form.append("file", file);
  let res: Response;
  try {
    res = await fetch("/api/upload", { method: "POST", body: form });
  } catch {
    throw new Error(
      "Cannot reach the backend server.\n\n" +
      "Start it with:\n" +
      "  cd visualizer/backend && uvicorn app:app --port 8000"
    );
  }
  if (!res.ok) {
    let detail = `${res.status}`;
    try {
      const json = await res.json();
      if (json.detail) detail = json.detail;
    } catch {
      const text = await res.text();
      if (text) detail = text;
    }
    throw new Error(detail);
  }
  return res.json();
}

export const getPdfFileUrl = () => "/api/pdf-file";

export const getPageInfo = (n: number) =>
  fetchJSON<PageInfo>(`/api/pages/${n}`);

export const getPageText = (n: number, layout: boolean) =>
  fetchJSON<{ text: string; timing_ms: number }>(`/api/pages/${n}/text?layout=${layout}`);

export const getChars = (n: number) =>
  fetchJSON<TimedResponse<Char[]>>(`/api/pages/${n}/chars`);

export const getWords = (n: number) =>
  fetchJSON<TimedResponse<Word[]>>(`/api/pages/${n}/words`);

export const getLines = (n: number) =>
  fetchJSON<TimedResponse<Line[]>>(`/api/pages/${n}/lines`);

export const getRects = (n: number) =>
  fetchJSON<TimedResponse<Rect[]>>(`/api/pages/${n}/rects`);

export const getEdges = (n: number) =>
  fetchJSON<TimedResponse<Edge[]>>(`/api/pages/${n}/edges`);

export const getTables = (n: number) =>
  fetchJSON<TimedResponse<TableData[]>>(`/api/pages/${n}/tables`);

export const searchPage = (n: number, q: string) =>
  fetchJSON<TimedResponse<SearchMatch[]>>(`/api/pages/${n}/search?q=${encodeURIComponent(q)}`);

export const getBenchmarkLibraries = () =>
  fetchJSON<{ available: string[] }>("/api/benchmark/libraries");

export const runBenchmark = (n: number, iterations = 3) =>
  fetchJSON<BenchmarkResults>(`/api/benchmark/${n}?iterations=${iterations}`, "POST");
