interface Props {
  currentPage: number;
  pageCount: number;
  onPageChange: (page: number) => void;
}

export default function PageSelector({
  currentPage,
  pageCount,
  onPageChange,
}: Props) {
  return (
    <div className="page-selector">
      <button
        disabled={currentPage <= 1}
        onClick={() => onPageChange(currentPage - 1)}
      >
        Prev
      </button>
      <select
        value={currentPage}
        onChange={(e) => onPageChange(Number(e.target.value))}
      >
        {Array.from({ length: pageCount }, (_, i) => i + 1).map((p) => (
          <option key={p} value={p}>
            Page {p}
          </option>
        ))}
      </select>
      <span className="page-count">of {pageCount}</span>
      <button
        disabled={currentPage >= pageCount}
        onClick={() => onPageChange(currentPage + 1)}
      >
        Next
      </button>
    </div>
  );
}
