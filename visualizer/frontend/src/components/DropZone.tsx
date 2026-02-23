import { useState, useCallback, DragEvent } from "react";

interface Props {
  onUpload: (file: File) => void;
  loading: boolean;
  error?: string | null;
}

export default function DropZone({ onUpload, loading, error }: Props) {
  const [dragging, setDragging] = useState(false);

  const handleDrag = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragIn = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragging(true);
  }, []);

  const handleDragOut = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragging(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragging(false);
      const files = e.dataTransfer.files;
      if (files.length > 0 && files[0].type === "application/pdf") {
        onUpload(files[0]);
      }
    },
    [onUpload]
  );

  const handleClick = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".pdf";
    input.onchange = () => {
      if (input.files && input.files[0]) {
        onUpload(input.files[0]);
      }
    };
    input.click();
  };

  return (
    <div
      className={`dropzone ${dragging ? "dropzone-active" : ""}`}
      onDragOver={handleDrag}
      onDragEnter={handleDragIn}
      onDragLeave={handleDragOut}
      onDrop={handleDrop}
      onClick={loading ? undefined : handleClick}
    >
      {loading ? (
        <p>Processing PDF...</p>
      ) : (
        <>
          <p className="dropzone-icon">PDF</p>
          <p>Drop a PDF here or click to browse</p>
          {error && (
            <div className="dropzone-error">
              <pre>{error}</pre>
            </div>
          )}
        </>
      )}
    </div>
  );
}
