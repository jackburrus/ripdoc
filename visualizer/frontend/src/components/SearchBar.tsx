import { useState, useCallback, useRef } from "react";

interface Props {
  onSearch: (query: string) => void;
}

export default function SearchBar({ onSearch }: Props) {
  const [query, setQuery] = useState("");
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();

  const handleChange = useCallback(
    (value: string) => {
      setQuery(value);
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => {
        onSearch(value);
      }, 300);
    },
    [onSearch]
  );

  return (
    <div className="search-bar">
      <input
        type="text"
        placeholder="Search text..."
        value={query}
        onChange={(e) => handleChange(e.target.value)}
      />
    </div>
  );
}
