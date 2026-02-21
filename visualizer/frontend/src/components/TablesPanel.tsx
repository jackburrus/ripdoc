import type { TableData } from "../types";

interface Props {
  tables: TableData[];
}

export default function TablesPanel({ tables }: Props) {
  if (tables.length === 0) return null;

  return (
    <div className="tables-panel">
      <h3>Tables</h3>
      {tables.map((t, i) => (
        <div key={i} className="table-item">
          <div className="table-header">
            Table {i + 1} ({t.row_count} x {t.col_count})
          </div>
          <div
            className="table-html"
            dangerouslySetInnerHTML={{ __html: t.html }}
          />
        </div>
      ))}
    </div>
  );
}
