# Implementation Plan: Column Sorting (Backend)

## Goal
Add column sorting functionality for single-value columns with backend-based sorting for large file support. Sort operates on current data (file lines OR search results) as separate operations.

---

## Architecture Overview

### Design Decisions
1. **Sort is separate from search**: Sorting is a standalone operation on retrieved data
2. **Always backend sorting**: No client-side fallback, always use Rust backend for sorting
3. **Stream results**: Use Tauri Channel to stream sorted results in chunks
4. **Auto-detect value types**: Numeric and date detection for proper ordering

### Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Scenario 1: Not Searching (Sort All File Lines)           │
├─────────────────────────────────────────────────────────────┤
│ User clicks column header                                   │
│   ↓                                                        │
│ Frontend: invoke('sort_file_lines', path, col, dir)        │
│   ↓                                                        │
│ Backend: Read file → Extract values → Sort → Stream         │
│   ↓                                                        │
│ Frontend: Receive chunks → fileStore.replaceLines()         │
│   ↓                                                        │
│ FileViewer displays sorted data                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Scenario 2: Searching (Sort Search Results)                │
├─────────────────────────────────────────────────────────────┤
│ User has active search ($searchStore.results)               │
│ User clicks column header                                   │
│   ↓                                                        │
│ Frontend: invoke('sort_search_results', results, col, dir) │
│   ↓                                                        │
│ Backend: Extract values → Sort → Stream                     │
│   ↓                                                        │
│ Frontend: Receive chunks → searchStore.replaceResults()     │
│   ↓                                                        │
│ FileViewer displays sorted search results                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Type Definitions

### Files: `src/lib/types.ts`

Add new interfaces:

```typescript
export interface SortState {
  column: string | null;  // Column path (e.g., "user_id")
  direction: 'asc' | 'desc';
}

export interface ColumnInfo {
  path: string;
  isSortable: boolean;
  displayName: string;
}

// Update SearchQuery to NOT include sort (they're separate operations)
// No changes needed
```

---

## Phase 2: Backend - Sort File Lines Command

### File: `src-tauri/src/commands/sort.rs` (NEW)

Create new sorting module:

```rust
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SortColumn {
  pub column: String,      // e.g., "user_id"
  pub direction: String,   // "asc" or "desc"
}

#[tauri::command]
pub async fn sort_file_lines(
  path: String,
  sort_column: SortColumn,
  file_format: FileFormat,
  channel: Channel<Vec<JsonLine>>,
) -> Result<usize, String> {
  // 1. Read all lines from file (or streaming approach)
  // 2. Extract values for sort_column from each line
  // 3. Sort lines based on extracted values (numeric/date detection)
  // 4. Stream sorted chunks (2000 lines per chunk)
  // 5. Return total lines sorted
}
```

**Key Implementation Details:**
- Value extraction: Navigate nested objects using underscore-separated paths
- Value type detection:
  - Try to parse as `f64` for numeric sorting
  - Try ISO 8601 date parsing for dates
  - Fall back to string comparison
- Null/empty handling: Place at the end of sorted results
- Stable sort: Maintain original order for equal values

### File: `src-tauri/src/commands/sort.rs` - Helper Functions

```rust
// Extract value from nested JSON by path
fn get_nested_value(json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
  let parts: Vec<&str> = path.split('_').collect();
  let mut current = json;
  for part in parts {
    match current {
      serde_json::Value::Object(obj) => {
        current = obj.get(part)?;
      }
      _ => return None,
    }
  }
  Some(current.clone())
}

// Convert value to sortable representation
enum SortValue {
  Null,
  Number(f64),
  Date(i64),  // Unix timestamp
  String(String),
}

fn to_sort_value(val: &serde_json::Value) -> SortValue {
  // Implementation details...
}
```

---

## Phase 3: Backend - Sort Search Results Command

### File: `src-tauri/src/commands/sort.rs` (continue)

Add command for sorting search results:

```rust
#[tauri::command]
pub async fn sort_search_results(
  results: Vec<SearchResult>,
  sort_column: SortColumn,
  channel: Channel<Vec<SearchResult>>,
) -> Result<usize, String> {
  // 1. Parse context JSON from each SearchResult
  // 2. Extract values for sort_column
  // 3. Sort based on extracted values
  // 4. Stream sorted chunks (100 results per chunk)
  // 5. Return total sorted
}
```

---

## Phase 4: Register Sort Commands

### File: `src-tauri/src/main.rs`

```rust
mod commands;

use commands::{file_parser, search, export, network, sort};  // Add sort

// In invoke_handler:
.invoke_handler(tauri::generate_handler![
  file_parser::parse_file_streaming,
  search::search_in_file,
  export::export_to_csv,
  export::export_to_excel,
  network::download_url_to_temp,
  sort::sort_file_lines,         // NEW
  sort::sort_search_results,      // NEW
])
```

---

## Phase 5: Frontend - Sort Store

### File: `src/lib/stores/sortStore.ts` (NEW)

```typescript
import { writable, derived } from 'svelte/store';
import type { SortState, SortColumn } from '$lib/types';

interface SortStore {
  state: SortState;
  isSorting: boolean;
  error: string | null;
}

function createSortStore() {
  const { subscribe, set, update } = writable<SortStore>({
    state: { column: null, direction: 'asc' },
    isSorting: false,
    error: null,
  });

  return {
    subscribe,
    setColumn: (column: string | null, direction: 'asc' | 'desc') =>
      update(state => ({
        ...state,
        state: { column, direction }
      })),
    setSorting: (sorting: boolean) =>
      update(state => ({ ...state, isSorting: sorting })),
    setError: (error: string | null) =>
      update(state => ({ ...state, error })),
    toggleColumn: (column: string) =>
      update(state => {
        const currentCol = state.state.column;
        const currentDir = state.state.direction;
        
        if (currentCol === column) {
          // Toggle direction
          return {
            ...state,
            state: { column, direction: currentDir === 'asc' ? 'desc' : 'asc' }
          };
        } else {
          // New column, default asc
          return {
            ...state,
            state: { column, direction: 'asc' }
          };
        }
      }),
    reset: () => set({
      state: { column: null, direction: 'asc' },
      isSorting: false,
      error: null,
    }),
  };
}

export const sortStore = createSortStore();
```

---

## Phase 6: Frontend - Detect Sortable Columns

### File: `src/lib/stores/fileStore.ts`

Modify to detect which columns contain only single (non-complex) values:

```typescript
// Add to FileState:
interface FileState {
  // ... existing fields
  columnInfo: ColumnInfo[];  // NEW: Replace `columns` with ColumnInfo[]
}

// Modify addLines to detect sortable columns:
if (columns.length === 0 && newLines.length > 0) {
  const sampleSize = Math.min(newLines.length, 50);
  const keyCounts = new Map<string, number>();
  const columnComplexity = new Map<string, boolean>();  // NEW

  // ... existing collectKeys logic ...

  // Check if column values are all simple (not complex)
  for (const col of Array.from(keyCounts.keys())) {
    let isSortable = true;
    
    for (let i = 0; i < sampleSize; i++) {
      const { isComplex } = smartFormat(newLines[i].parsed, col);
      if (isComplex) {
        isSortable = false;
        break;
      }
    }
    
    columnComplexity.set(col, isSortable);
  }

  // Sort and build ColumnInfo[]
  columns = Array.from(keyCounts.keys()).sort(/* existing sort logic */)
    .slice(0, 100)
    .map(path => ({
      path,
      isSortable: columnComplexity.get(path) ?? false,
      displayName: path.split('_').slice(1).join('_') || path,
    }));
}
```

**Helper Function Needed:**
Move `smartFormat` logic from FileViewer.svelte to a shared utility function to use in fileStore.

---

## Phase 7: Frontend - Update FileViewer

### File: `src/lib/components/FileViewer.svelte`

#### 7.1 Import sortStore

```typescript
import { sortStore } from "$lib/stores/sortStore";
```

#### 7.2 Add sort command handlers

```typescript
import { invoke } from '@tauri-apps/api/core';

// Sort file lines (when not searching)
async function sortFileLines(column: string, direction: 'asc' | 'desc') {
  sortStore.setSorting(true);
  sortStore.setColumn(column, direction);

  try {
    const channel = new Channel();
    channel.onmessage = (event) => {
      const chunk = event.payload as JsonLine[];
      // Replace existing lines with sorted chunk
      fileStore.replaceLines(chunk, channel.isComplete);
    };

    await invoke('sort_file_lines', {
      path: $fileStore.filePath,
      sortColumn: { column, direction },
      fileFormat: $fileStore.format,
      channel
    });
  } catch (error) {
    sortStore.setError(String(error));
  } finally {
    sortStore.setSorting(false);
  }
}

// Sort search results (when searching)
async function sortSearchResults(column: string, direction: 'asc' | 'desc') {
  sortStore.setSorting(true);
  sortStore.setColumn(column, direction);

  try {
    const channel = new Channel();
    channel.onmessage = (event) => {
      const chunk = event.payload as SearchResult[];
      searchStore.replaceResults(chunk, channel.isComplete);
    };

    await invoke('sort_search_results', {
      results: $searchStore.results,
      sortColumn: { column, direction },
      channel
    });
  } catch (error) {
    sortStore.setError(String(error));
  } finally {
    sortStore.setSorting(false);
  }
}
```

#### 7.3 Update header row with click handlers

Modify the bottom row (sub-keys) to add click handlers and sort indicators:

```svelte
<!-- Bottom Row (Sub-keys) -->
<div class="grid border-t border-border/30 bg-background/50" style="grid-template-columns: {gridCols}; padding-right: 8px; min-width: {typeof totalMinWidth === 'number' ? totalMinWidth + 'px' : totalMinWidth};">
  <div class="bg-muted/10 border-r border-border/30"></div>

  {#each $fileStore.columnInfo as col}
    <div
      class="p-1 px-2 text-muted-foreground text-xs truncate border-r border-border/30
        {col.isSortable ? 'cursor-pointer hover:bg-muted/50' : 'opacity-50'}
        {$sortStore.state.column === col.path ? 'font-bold text-foreground' : ''}"
      title={col.path}
      onmousedown={() => col.isSortable && handleHeaderClick(col.path)}
      onkeydown={(e) => col.isSortable && e.key === 'Enter' && handleHeaderClick(col.path)}
      role="columnheader"
      tabindex={col.isSortable ? 0 : -1}
      aria-sort={$sortStore.state.column === col.path 
        ? $sortStore.state.direction === 'asc' ? 'ascending' : 'descending'
        : 'none'}
    >
      <div class="flex items-center gap-1">
        <span>{col.displayName}</span>
        {#if $sortStore.state.column === col.path}
          {#if $sortStore.state.direction === 'asc'}
            <ArrowUp class="w-3 h-3" />
          {:else}
            <ArrowDown class="w-3 h-3" />
          {/if}
        {/if}
      </div>
    </div>
  {/each}
</div>
```

#### 7.4 Add click handler

```typescript
import { ArrowUp, ArrowDown } from "lucide-svelte";

function handleHeaderClick(colPath: string) {
  const currentCol = $sortStore.state.column;
  const currentDir = $sortStore.state.direction;

  // Toggle or set new column
  let newDirection: 'asc' | 'desc';
  if (currentCol === colPath) {
    newDirection = currentDir === 'asc' ? 'desc' : 'asc';
  } else {
    newDirection = 'asc';
  }

  // Execute sort based on current context
  if (isSearching) {
    sortSearchResults(colPath, newDirection);
  } else {
    sortFileLines(colPath, newDirection);
  }
}
```

---

## Phase 8: Store Updates

### File: `src/lib/stores/fileStore.ts`

Add method to replace lines with sorted version:

```typescript
replaceLines: (newLines: JsonLine[], isComplete: boolean = false) =>
  update(state => ({
    ...state,
    lines: isComplete ? newLines : [...state.lines, ...newLines],
  })),
```

### File: `src/lib/stores/searchStore.ts`

Add method to replace results with sorted version:

```typescript
replaceResults: (newResults: SearchResult[], isComplete: boolean = false) =>
  update(state => ({
    ...state,
    results: isComplete ? newResults : [...state.results, ...newResults],
  })),
```

---

## Phase 9: Utility Function Extraction

### File: `src/lib/utils/valueFormat.ts` (NEW)

Extract `smartFormat` from FileViewer to make it reusable:

```typescript
export function smartFormat(value: any): { text: string; isComplex: boolean } {
  // Implementation from FileViewer.svelte
}

export function getValue(parsed: any, colPath: string): { text: string; isComplex: boolean } {
  // Implementation from FileViewer.svelte
}
```

### File: `src/lib/components/FileViewer.svelte`

Update to use imported utils:

```typescript
import { getValue } from "$lib/utils/valueFormat";
// smartFormat can remain local if only used for display
```

---

## Phase 10: Reset Sorting on File Load

### File: `src/lib/stores/fileStore.ts`

When loading a new file, reset sorting state:

```typescript
setFilePath: (path: string | null) =>
  update(state => {
    // When path changes (new file), reset sort
    if (path !== state.filePath) {
      sortStore.reset();
    }
    return { ...state, filePath: path };
  }),
```

---

## Phase 11: Loading States

### File: `src/lib/components/FileViewer.svelte`

Show sorting indicator in UI:

```svelte
{#if $sortStore.isSorting}
  <div class="flex items-center justify-center h-full">
    <Spinner size="lg" text="Sorting..." />
  </div>
{/if}
```

---

## Implementation Order

1. ✅ **Phase 1**: Type definitions
2. ✅ **Phase 2**: Backend sort module (file lines)
3. ✅ **Phase 3**: Backend sort command (search results)
4. ✅ **Phase 4**: Register commands in main.rs
5. ✅ **Phase 5**: Create sortStore
6. ✅ **Phase 9**: Extract smartFormat to utils (needed for Phase 6)
7. ✅ **Phase 6**: Update fileStore to detect sortable columns
8. ✅ **Phase 7**: Update FileViewer with sort UI
9. ✅ **Phase 8**: Add store methods for replacing data
10. ✅ **Phase 10**: Reset sort on file load
11. ✅ **Phase 11**: Add loading states

---

## Files to Create

| File                               | Purpose                         |
| ---------------------------------- | ------------------------------- |
| `src-tauri/src/commands/sort.rs`  | Backend sorting logic           |
| `src/lib/stores/sortStore.ts`      | Frontend sort state management  |
| `src/lib/utils/valueFormat.ts`     | Shared value formatting utils    |

## Files to Modify

| File                                 | Changes                                   |
| ------------------------------------ | ----------------------------------------- |
| `src/lib/types.ts`                     | Add SortState, ColumnInfo interfaces       |
| `src-tauri/src/main.rs`               | Register sort commands                     |
| `src/lib/stores/fileStore.ts`          | Detect sortable columns, add replaceLines   |
| `src/lib/stores/searchStore.ts`        | Add replaceResults method                  |
| `src/lib/components/FileViewer.svelte` | Add sort UI, handlers, loading state      |

---

## Edge Cases & Considerations

### 1. Sort + Search Interaction
- **Requirement**: Sort is separate from search
- **Implementation**: Search returns unsorted, then sort applies to results
- **UX Flow**: Search → Sort results → Clear search → File shows sorted lines

### 2. Large Files
- **Streaming**: Both sort commands stream results (2000 lines for file, 100 for search)
- **Memory**: Backend loads all data to sort, then streams sorted results
- **Optimization**: Consider chunked sorting for very large files (>1M lines) if needed

### 3. Value Detection
- **Numbers**: Parse as f64 for proper numeric sorting
- **Dates**: Detect ISO 8601 format (`YYYY-MM-DD`, `YYYY-MM-DDTHH:mm:ss`)
- **Strings**: Case-insensitive comparison
- **Null/Empty**: Place at end of sorted results

### 4. Column Complexity
- **Detection**: Check if `smartFormat().isComplex` is false for ALL samples
- **Max depth**: Sample first 50 lines per current column detection
- **Dynamic columns**: If new columns appear after first 50 lines, they won't be sortable

### 5. Performance
- **Chunk sizes**: 2000 lines for file sort, 100 for search results
- **Blocking**: Sorting blocks Rust thread until complete (acceptable for this use case)
- **Frontend**: Channel streaming prevents UI freeze

### 6. Error Handling
- **Invalid column**: Return error from backend
- **Missing values**: Handle gracefully (treat as null/empty)
- **Parse errors**: Log and skip invalid lines

---

## Testing Checklist

- [ ] Sort file lines ascending by text column
- [ ] Sort file lines descending by text column  
- [ ] Sort file lines by numeric column (numbers sort numerically)
- [ ] Sort file lines by date column (dates sort chronologically)
- [ ] Sort search results
- [ ] Toggle sort direction on same column click
- [ ] Click different column changes sort column
- [ ] Non-sortable columns (complex values) show no cursor
- [ ] Sort indicators (↑/↓) display correctly
- [ ] Loading spinner shows during sort
- [ ] Clear search shows sorted file lines
- [ ] Load new file resets sort state
- [ ] Null/empty values appear at end
- [ ] Large file (>10k lines) sorts and streams correctly
