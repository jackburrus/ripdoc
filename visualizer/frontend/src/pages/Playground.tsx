import { useState, useCallback } from "react";
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
  LayerState,
  LayerName,
} from "../types";
import * as api from "../api";
import DropZone from "../components/DropZone";
import PageSelector from "../components/PageSelector";
import PdfViewer from "../components/PdfViewer";
import OverlayCanvas from "../components/OverlayCanvas";
import ControlPanel from "../components/ControlPanel";
import TextPanel from "../components/TextPanel";
import TablesPanel from "../components/TablesPanel";
import SearchBar from "../components/SearchBar";
import BenchmarkPanel from "../components/BenchmarkPanel";

const SCALE = 1.0;

export default function Playground() {
  const [upload, setUpload] = useState<UploadResult | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageInfo, setPageInfo] = useState<PageInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Actual canvas dimensions reported by pdf.js
  const [canvasSize, setCanvasSize] = useState<{ w: number; h: number }>({ w: 0, h: 0 });

  // Layer data (fetched lazily)
  const [chars, setChars] = useState<Char[]>([]);
  const [words, setWords] = useState<Word[]>([]);
  const [lines, setLines] = useState<Line[]>([]);
  const [rects, setRects] = useState<Rect[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [tables, setTables] = useState<TableData[]>([]);
  const [searchResults, setSearchResults] = useState<SearchMatch[]>([]);

  // Text extraction
  const [simpleText, setSimpleText] = useState("");
  const [layoutText, setLayoutText] = useState("");

  // Timing data per layer
  const [timings, setTimings] = useState<Record<string, number>>({});

  // Layer toggles
  const [layers, setLayers] = useState<LayerState>({
    chars: false,
    words: true,
    lines: false,
    rects: false,
    edges: false,
    tables: true,
    search: false,
  });

  // Track which layers have been fetched for current page
  const [fetched, setFetched] = useState<Set<string>>(new Set());

  const clearPageData = () => {
    setChars([]);
    setWords([]);
    setLines([]);
    setRects([]);
    setEdges([]);
    setTables([]);
    setSearchResults([]);
    setSimpleText("");
    setLayoutText("");
    setFetched(new Set());
    setTimings({});
  };

  const fetchLayerData = useCallback(
    async (layer: LayerName, page: number) => {
      const fetchers: Record<LayerName, () => Promise<void>> = {
        chars: async () => {
          const resp = await api.getChars(page);
          setChars(resp.data);
          setTimings((prev) => ({ ...prev, chars: resp.timing_ms }));
        },
        words: async () => {
          const resp = await api.getWords(page);
          setWords(resp.data);
          setTimings((prev) => ({ ...prev, words: resp.timing_ms }));
        },
        lines: async () => {
          const resp = await api.getLines(page);
          setLines(resp.data);
          setTimings((prev) => ({ ...prev, lines: resp.timing_ms }));
        },
        rects: async () => {
          const resp = await api.getRects(page);
          setRects(resp.data);
          setTimings((prev) => ({ ...prev, rects: resp.timing_ms }));
        },
        edges: async () => {
          const resp = await api.getEdges(page);
          setEdges(resp.data);
          setTimings((prev) => ({ ...prev, edges: resp.timing_ms }));
        },
        tables: async () => {
          const resp = await api.getTables(page);
          setTables(resp.data);
          setTimings((prev) => ({ ...prev, tables: resp.timing_ms }));
        },
        search: async () => {}, // handled by SearchBar
      };
      await fetchers[layer]();
    },
    []
  );

  const handleUpload = async (file: File) => {
    setLoading(true);
    setError(null);
    clearPageData();
    try {
      const result = await api.uploadPdf(file);
      setUpload(result);
      setCurrentPage(1);
      await loadPage(1);
    } catch (e: any) {
      setError(e.message || String(e));
    } finally {
      setLoading(false);
    }
  };

  const loadPage = async (page: number) => {
    setLoading(true);
    clearPageData();
    try {
      const info = await api.getPageInfo(page);
      setPageInfo(info);

      // Fetch text
      const [simple, layout] = await Promise.all([
        api.getPageText(page, false),
        api.getPageText(page, true),
      ]);
      setSimpleText(simple.text);
      setLayoutText(layout.text);

      // Fetch initially-enabled layers
      const newFetched = new Set<string>();
      const fetches: Promise<void>[] = [];
      for (const [layer, enabled] of Object.entries(layers)) {
        if (enabled && layer !== "search") {
          fetches.push(fetchLayerData(layer as LayerName, page));
          newFetched.add(layer);
        }
      }
      await Promise.all(fetches);
      setFetched(newFetched);
    } catch (e) {
      console.error("Failed to load page:", e);
    } finally {
      setLoading(false);
    }
  };

  const handlePageChange = async (page: number) => {
    setCurrentPage(page);
    await loadPage(page);
  };

  const handleToggleLayer = async (layer: LayerName) => {
    const newEnabled = !layers[layer];
    setLayers((prev) => ({ ...prev, [layer]: newEnabled }));

    if (newEnabled && !fetched.has(layer) && layer !== "search") {
      await fetchLayerData(layer, currentPage);
      setFetched((prev) => new Set(prev).add(layer));
    }
  };

  const handleSearch = async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);
      setLayers((prev) => ({ ...prev, search: false }));
      return;
    }
    const resp = await api.searchPage(currentPage, query);
    setSearchResults(resp.data);
    setTimings((prev) => ({ ...prev, search: resp.timing_ms }));
    setLayers((prev) => ({ ...prev, search: true }));
  };

  const handleViewportReady = useCallback((w: number, h: number) => {
    setCanvasSize({ w, h });
  }, []);

  // Compute actual scale: how many pixels per PDF point
  const effectiveScale =
    pageInfo && canvasSize.w > 0 ? canvasSize.w / pageInfo.width : SCALE;

  // Counts for control panel
  const counts = {
    chars: chars.length,
    words: words.length,
    lines: lines.length,
    rects: rects.length,
    edges: edges.length,
    tables: tables.length,
    search: searchResults.length,
  };

  if (!upload) {
    return (
      <div className="playground-container">
        <DropZone onUpload={handleUpload} loading={loading} error={error} />
      </div>
    );
  }

  return (
    <div className="playground-container">
      <header className="playground-toolbar">
        <span className="filename">{upload.filename}</span>
        <PageSelector
          currentPage={currentPage}
          pageCount={upload.page_count}
          onPageChange={handlePageChange}
        />
        <button className="new-file-btn" onClick={() => { setUpload(null); clearPageData(); setPageInfo(null); setCanvasSize({ w: 0, h: 0 }); }}>
          New file
        </button>
      </header>

      <div className="main-layout">
        <div className="sidebar-column">
          <div className="sidebar-section">
            <ControlPanel
              layers={layers}
              counts={counts}
              timings={timings}
              onToggle={handleToggleLayer}
            />
          </div>
          <div className="sidebar-section">
            <SearchBar onSearch={handleSearch} />
          </div>
          <div className="sidebar-section">
            <BenchmarkPanel currentPage={currentPage} autoRun />
          </div>
          <div className="sidebar-section sidebar-section--grow">
            <TextPanel simpleText={simpleText} layoutText={layoutText} />
          </div>
          <TablesPanel tables={tables} />
        </div>

        <div className="viewer-column">
          <div
            className="pdf-viewer-container"
            style={{
              width: canvasSize.w || undefined,
              height: canvasSize.h || undefined,
            }}
          >
            <PdfViewer
              page={currentPage}
              scale={SCALE}
              onViewportReady={handleViewportReady}
            />
            {pageInfo && canvasSize.w > 0 && (
              <OverlayCanvas
                width={canvasSize.w}
                height={canvasSize.h}
                scale={effectiveScale}
                layers={layers}
                chars={chars}
                words={words}
                lines={lines}
                rects={rects}
                edges={edges}
                tables={tables}
                searchResults={searchResults}
              />
            )}
            {loading && <div className="loading-overlay">Loading...</div>}
          </div>
        </div>
      </div>
    </div>
  );
}
