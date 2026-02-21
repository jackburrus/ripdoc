import { useEffect, useRef } from "react";
import * as pdfjsLib from "pdfjs-dist";
import { getPdfFileUrl } from "../api";

// Use the worker from the pdfjs-dist package
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.mjs",
  import.meta.url
).toString();

interface Props {
  page: number;
  scale: number;
  onViewportReady?: (width: number, height: number) => void;
}

export default function PdfViewer({ page, scale, onViewportReady }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const renderTaskRef = useRef<any>(null);
  const pdfDocRef = useRef<pdfjsLib.PDFDocumentProxy | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function loadAndRender() {
      // Load the PDF document (cache it)
      if (!pdfDocRef.current) {
        const loadingTask = pdfjsLib.getDocument(getPdfFileUrl());
        pdfDocRef.current = await loadingTask.promise;
      }

      if (cancelled) return;

      const pdf = pdfDocRef.current;
      const pdfPage = await pdf.getPage(page);

      if (cancelled) return;

      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      // Cancel any in-progress render
      if (renderTaskRef.current) {
        renderTaskRef.current.cancel();
      }

      const viewport = pdfPage.getViewport({ scale });
      canvas.width = viewport.width;
      canvas.height = viewport.height;

      // Report actual viewport dimensions back to parent
      onViewportReady?.(viewport.width, viewport.height);

      const renderTask = pdfPage.render({
        canvasContext: ctx,
        viewport,
      });

      renderTaskRef.current = renderTask;

      try {
        await renderTask.promise;
      } catch (e: any) {
        if (e.name !== "RenderingCancelledException") {
          console.error("Render error:", e);
        }
      }
    }

    loadAndRender();

    return () => {
      cancelled = true;
      if (renderTaskRef.current) {
        renderTaskRef.current.cancel();
      }
    };
  }, [page, scale]);

  // Reset the cached PDF when a new file is uploaded
  useEffect(() => {
    return () => {
      pdfDocRef.current = null;
    };
  }, []);

  return <canvas ref={canvasRef} className="pdf-canvas" />;
}
