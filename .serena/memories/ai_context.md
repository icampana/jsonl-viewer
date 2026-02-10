# AI CONTEXT & ARCHITECTURAL MAP

> Last Updated: 2025-02-10
> Project: JSONL Viewer v0.4.0

## 1. Tech Stack & Versions

### Core Framework
- **Frontend**: Svelte 5.45.6 (using Svelte 5 features: `$derived`, `$state`, `$props`, snippets)
- **Desktop Framework**: Tauri v2.0.0 (Rust backend + Web frontend)
- **Build Tool**: Vite 7.2.6
- **Language**: TypeScript 5.9.3 (strict mode enabled)

### UI & Styling
- **CSS Framework**: TailwindCSS 3.4.0
- **UI Components**: bits-ui 0.21.0 (shadcn/ui-like patterns)
- **Icons**: lucide-svelte 0.460.0
- **Utilities**: clsx, tailwind-merge (via `cn()` helper)

### Backend (Rust)
- **Runtime**: Tokio 1.0 (async I/O)
- **Serialization**: serde 1.0, serde_json 1.0
- **CSV Export**: csv 1.3
- **Excel Export**: rust_xlsxwriter 0.63.0, calamine 0.24
- **Search**: regex 1.10, jsonpath-rust 0.5
- **Network**: reqwest 0.11 (with stream support)
- **Utilities**: uuid 1.0

### Development
- **Node Version**: v18+
- **Rust Version**: v1.70+
- **Package Manager**: npm (or ppn)
- **Tauri CLI**: v2.9.6

## 2. High-Level Architecture

**Pattern**: Hybrid Desktop Application (Tauri v2)

### Architecture Overview
```
┌─────────────────────────────────────────────────────────────┐
│                     Desktop Window                        │
│  ┌──────────────────────────────────────────────────────┐ │
│  │           SvelteKit Frontend (Static Build)          │ │
│  │  ┌─────────────────────────────────────────────────┐ │ │
│  │  │  Components (VirtualList, FileViewer, etc.)   │ │ │
│  │  └─────────────────────────────────────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────┐ │ │
│  │  │  Svelte Stores (State Management)             │ │ │
│  │  │  - fileStore: File data, metadata             │ │ │
│  │  │  - searchStore: Search query, results         │ │ │
│  │  │  - themeStore: Theme (dark/light)             │ │ │
│  │  │  - toastStore: Notifications                  │ │ │
│  │  └─────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────┘ │
│                      │ Tauri IPC                          │
│                      │ Channels (Streaming)               │
│  ┌──────────────────────────────────────────────────────┐ │
│  │              Rust Backend (Tauri)                   │ │
│  │  ┌─────────────────────────────────────────────────┐ │ │
│  │  │  Commands                                      │ │ │
│  │  │  - parse_file_streaming: Streaming parser       │ │ │
│  │  │  - search_in_file: Text/JSONPath search        │ │ │
│  │  │  - export_to_csv/excel: Export functionality   │ │ │
│  │  │  - download_url_to_temp: URL download           │ │ │
│  │  └─────────────────────────────────────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────┐ │ │
│  │  │  File System & Dialog Plugins                   │ │ │
│  │  └─────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Principles
1. **Streaming First**: Large files are parsed and sent in chunks (2000 lines per chunk) to avoid blocking UI
2. **Virtual Scrolling**: Custom VirtualList component renders only visible items (supports millions of rows)
3. **Event-Driven**: Tauri events handle menu actions and drag-drop
4. **State-Driven UI**: Svelte stores drive all UI updates using reactive patterns
5. **Column Auto-Detection**: First 50 lines are sampled to detect and prioritize columns

## 3. Critical Data Flows

### 3.1 File Loading Flow
```
User Action (Cmd+O, Drag-Drop)
  ↓
Tauri Dialog (plugin-dialog)
  ↓
invoke("parse_file_streaming", {path, channel})
  ↓
Rust: file_parser.rs::parse_file_streaming()
  - Reads file with Tokio AsyncBufRead
  - Detects format (JsonL vs JsonArray) by first line
  - Parses JSON line-by-line with serde_json
  - Sends chunks via Channel (CHUNK_SIZE: 2000)
  ↓
Frontend: Channel.onmessage
  - Buffers chunks (MAX_BUFFER_SIZE: 5000, FLUSH_INTERVAL: 100ms)
  - fileStore.addLines(buffer)
  - fileStore.setMetadata(metadata)
  ↓
FileViewer.svelte
  - Detects columns from first 50 lines
  - Prioritizes: id, timestamp, level, message, etc.
  - VirtualList renders visible items
```

### 3.2 Search Flow
```
User enters search query (text or JSONPath)
  ↓
searchStore.setQuery({...})
  ↓
invoke("search_in_file", {path, query, format, channel})
  ↓
Rust: search.rs::search_in_file()
  - Supports: plain text, regex, JSONPath
  - Can combine: JSONPath results + text filter
  - Sends chunks via Channel (CHUNK_SIZE: 100)
  ↓
Frontend: Channel.onmessage
  - searchStore.addResults(chunk)
  - searchStore.setStats(stats)
  ↓
FileViewer switches to search results mode
  - VirtualList renders filtered items
```

### 3.3 Export Flow
```
User clicks Export (Cmd+E)
  ↓
Tauri Save Dialog (choose CSV or Excel)
  ↓
invoke("export_to_csv" or "export_to_excel")
  ↓
Rust: export.rs::export_to_csv/excel()
  - Scans first 1000 lines for header detection
  - Collects all unique keys (flatten nested structures)
  - Writes CSV/Excel with proper formatting
  - Excel: Creates grouped headers (2-row header)
  ↓
showSuccess toast with export stats
```

### 3.4 URL Download Flow
```
User enters URL (Cmd+Shift+O)
  ↓
invoke("download_url_to_temp", {url})
  ↓
Rust: network.rs::download_url_to_temp()
  - Validates URL scheme (http/https only)
  - Downloads via reqwest
  - Saves to temp file with UUID name
  ↓
loadFile(tempPath)
  → File Loading Flow
```

## 4. Key Directory Map

### Frontend Structure
```
src/
├── routes/
│   ├── +page.svelte          # Main application page (File open, search, layout)
│   ├── +layout.svelte        # Theme provider, favicon
│   └── +layout.ts           # Route configuration
├── lib/
│   ├── components/
│   │   ├── ui/              # Basic UI components (button, spinner)
│   │   ├── json-tree/       # JSON tree viewer (JsonTree, JsonTreeNode)
│   │   ├── FileViewer.svelte # Main data grid with virtual scrolling
│   │   ├── VirtualList.svelte # Custom virtual scrolling component
│   │   ├── SearchBar.svelte # Search interface (text + JSONPath)
│   │   ├── DetailPanel.svelte # Resizable right sidebar for line details
│   │   ├── StatsDialog.svelte # Dataset statistics dialog
│   │   ├── Header.svelte    # App header with actions
│   │   ├── StatusBar.svelte # Status bar with file info
│   │   ├── UrlDialog.svelte # URL download dialog
│   │   ├── Toast.svelte     # Toast notifications
│   │   └── ExportButton.svelte # Export button with format selection
│   ├── stores/
│   │   ├── fileStore.ts     # File data, metadata, selected line (134 lines)
│   │   ├── searchStore.ts   # Search query, results, stats (58 lines)
│   │   ├── themeStore.ts    # Theme state (15 lines)
│   │   └── toastStore.ts    # Toast notifications (50 lines)
│   ├── types.ts             # Shared TypeScript interfaces (43 lines)
│   └── utils.ts             # cn() helper for class merging (6 lines)
├── app.css                  # Global styles (Tailwind CSS)
├── app.html                 # HTML template
└── app.d.ts                # Global type declarations
```

### Backend Structure (Rust)
```
src-tauri/src/
├── main.rs                 # Entry point, menu setup, Tauri config (109 lines)
├── lib.rs                  # Library entry point
└── commands/
    ├── mod.rs              # Command module declarations
    ├── file_parser.rs      # Streaming file parser (213 lines)
    ├── search.rs           # Search implementation (241 lines)
    ├── export.rs           # CSV/Excel export (318 lines)
    └── network.rs         # URL download (42 lines)
```

### Configuration Files
```
├── package.json            # npm scripts, dependencies
├── tsconfig.json           # TypeScript config (strict mode)
├── svelte.config.js        # SvelteKit config (static adapter)
├── tailwind.config.js      # Tailwind CSS config (custom colors)
├── vite.config.ts          # Vite config (devtools plugin)
├── Cargo.toml             # Rust dependencies
└── tauri.conf.json        # Tauri app config (windows, plugins)
```

## 5. Developer Guide / Conventions

### 5.1 Coding Patterns

#### Svelte 5 Features
- **State**: Use `$state()` for component-local reactive state
- **Props**: Use `$props<{...}>()` for component props
- **Derived**: Use `$derived(...)` for computed values
- **Effects**: Use `$effect(() => {...})` for side effects
- **Snippets**: Use `{#snippet children(item)}` for render props
- **Store Access**: Use `$store` to subscribe and read store values

#### File Naming
- Components: PascalCase (e.g., `FileViewer.svelte`)
- Utilities: camelCase (e.g., `cn()`)
- Stores: camelCase with "Store" suffix (e.g., `fileStore`)
- Types: PascalCase interfaces (e.g., `JsonLine`)

#### TypeScript Conventions
- Strict mode enabled (`"strict": true`)
- Use interfaces for object shapes
- Use types for unions/primitives
- Explicit return types on public functions
- Type assertions sparingly, prefer type guards

#### CSS Conventions
- Utility-first: Use Tailwind classes
- For dynamic values: Use `style="..."` with template literals
- Custom styles: Minimal, only in `<style>` blocks when necessary
- Dark mode: Class-based (`.dark` on html element)

### 5.2 Testing
**Current State**: No test framework configured
- No `*.test.*` or `*.spec.*` files found
- No test scripts in package.json
- **Recommendation**: Add Vitest for unit testing, Playwright for E2E

### 5.3 Build & Development Commands

```bash
# Development (both frontend + backend)
npm run tauri:dev

# Frontend only (Vite dev server)
npm run dev

# Type checking
npm run check

# Watch type checking
npm run check:watch

# Production build (creates desktop app)
npm run tauri:build

# Frontend build only
npm run build

# Preview frontend build
npm run preview
```

### 5.4 Code Quality Commands
```bash
# Type checking (no lint command configured)
npm run check

# Rust checks (manual)
cd src-tauri
cargo check
cargo clippy
cargo fmt
```

### 5.5 Linting/Formatting
**TypeScript/Svelte**: No ESLint/Prettier configured
- Uses `svelte-check` for type checking only
- Manual formatting expected

**Rust**: Standard Rust tooling
- `cargo fmt` for formatting
- `cargo clippy` for linting

### 5.6 State Management Patterns

#### Creating a Store
```typescript
// Follow the pattern in existing stores:
function createXxxStore() {
  const { subscribe, set, update } = writable<XxxState>({ ... });
  return {
    subscribe,
    // Action methods
    setXxx: (value) => update(state => ({ ...state, xxx: value })),
    reset: () => set({ ...initialState })
  };
}
export const xxxStore = createXxxStore();
```

#### Tauri Commands
```rust
// All commands are async and use streaming via Channel
#[tauri::command]
pub async fn xxx_command(
  param: Type,
  channel: Channel<Vec<ChunkType>>,
) -> Result<ReturnType, String> {
  // Do work
  channel.send(chunk)?;
  Ok(result)
}
```

### 5.7 Component Architecture

#### Component Props Pattern (Svelte 5)
```typescript
let {
  prop1 = defaultValue,
  prop2 = $bindable(bindableValue),
  children,
} = $props<{ ... }>();
```

#### Virtual List Pattern
For large datasets, always use `VirtualList.svelte`:
- Set `itemHeight` (constant row height)
- Set `overscan` (10 is default)
- Pass children as snippet

#### Error Handling
- Frontend: Try/catch + `showError()` toast
- Backend: `Result<T, String>` return type
- Errors displayed in UI via `fileStore.error` or `searchStore.error`

## 6. Known Technical Debt / Watchlist

### 6.1 Large Files (Potential God Objects)

**FileViewer.svelte** (263 lines)
- **Risk**: Complex logic mixed with rendering
- **Watchlist**: Value extraction, header grouping, search result mapping
- **Impact**: High - core data display component
- **Recommendation**: Extract `getValue()`, `smartFormat()`, header grouping to utils

**StatsDialog.svelte** (202 lines)
- **Risk**: Statistics computation logic mixed with UI
- **Watchlist**: `computeStats()` function, `getValue()` helper
- **Impact**: Medium - statistics feature
- **Recommendation**: Extract stats computation to separate utility

**export.rs** (318 lines)
- **Risk**: CSV and Excel logic in one file
- **Watchlist**: Header collection, flat value extraction, Excel formatting
- **Impact**: Medium - export functionality
- **Recommendation**: Split into `csv_export.rs` and `excel_export.rs`

**search.rs** (241 lines)
- **Risk**: Complex search logic with multiple paths
- **Watchlist**: `check_match` closure, `evaluate_jsonpath`, format detection
- **Impact**: High - core search functionality
- **Recommendation**: Extract text search and JSONPath to separate modules

### 6.2 Magic Numbers / Hardcoded Values

**Rust Commands**
- `CHUNK_SIZE: 2000` (file_parser.rs) - Line chunk size
- `CHUNK_SIZE: 100` (search.rs, export.rs) - Search/export chunk size
- `1000` (export.rs) - Header sample size
- `100` (VirtualList.svelte) - Default overscan

**Frontend Stores**
- `50` (fileStore.ts) - Column detection sample size
- `MAX_BUFFER_SIZE: 5000` (+page.svelte) - Channel buffer max
- `FLUSH_INTERVAL: 100` (+page.svelte) - Buffer flush interval (ms)
- `100` (fileStore.ts) - Column cap
- `2` (fileStore.ts) - Flattening depth limit

**Components**
- `itemHeight: 36` (FileViewer.svelte) - Row height
- `MIN_WIDTH: 300, MAX_WIDTH: 1200` (DetailPanel.svelte) - Sidebar width constraints

**Recommendation**: Extract to constants module
```rust
// src-tauri/src/constants.rs
pub const CHUNK_SIZE_PARSE: usize = 2000;
pub const CHUNK_SIZE_SEARCH: usize = 100;
pub const HEADER_SAMPLE_SIZE: usize = 1000;
```

### 6.3 Code Duplication

**Flat Value Extraction**
- Duplicate logic in:
  - `src-tauri/src/commands/export.rs`: `get_flat_value()`, `collect_headers()`
  - `src/lib/components/FileViewer.svelte`: `getValue()`, `smartFormat()`
  - `src/lib/components/StatsDialog.svelte`: `getValue()`
- **Recommendation**: Create shared utilities in `src/lib/utils/json.ts`

**Header Grouping**
- Similar logic in:
  - `src-tauri/src/commands/export.rs`: Excel header grouping
  - `src/lib/components/FileViewer.svelte`: UI header grouping
- **Recommendation**: Share grouping algorithm via Tauri command or shared types

### 6.4 Missing Tests

**Critical Paths Without Tests**:
1. File parser streaming logic (file_parser.rs)
2. Search algorithm (search.rs)
3. Export formatting (export.rs)
4. Virtual list rendering (VirtualList.svelte)
5. Column detection logic (fileStore.ts)

**Recommendation**: Set up testing framework before adding features

### 6.5 Performance Considerations

**Potential Issues**:
1. **Stats computation** - Synchronous computation over all lines (StatsDialog:81-103)
   - Blocks UI thread for large datasets
   - Consider: Web Worker or Tauri command

2. **Column detection** - Samples only 50 lines
   - May miss columns in large, sparse files
   - Consider: Adaptive sampling or full scan for small files

3. **Value formatting** - `smartFormat()` in FileViewer
   - Called for every visible cell on every render
   - Consider: Memoization or caching

### 6.6 Error Handling Gaps

**Areas with limited error handling**:
1. File parser - Continues parsing after single line error
2. Search - Partial failure handling unclear
3. Export - No progress indication for large files

### 6.7 Internationalization (i18n)
**Current Status**: No i18n framework
- All strings are hardcoded in English
- Consider: Adding svelte-i18n if multi-language support needed

### 6.8 Security Considerations

**Known Safe Practices**:
- URL scheme validation (http/https only)
- Tauri file system plugin configured
- CSP disabled (acceptable for desktop app)

**Watchlist**:
- JSONPath injection vulnerabilities (search.rs)
- Very large JSON parsing (DoS potential)
- Temp file cleanup (network.rs)

## 7. Quick Reference

### Common Imports
```typescript
// Svelte stores
import { writable, derived } from 'svelte/store';

// Tauri
import { invoke, Channel } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

// Local imports
import { fileStore } from "$lib/stores/fileStore";
import { searchStore } from "$lib/stores/searchStore";
import { showSuccess, showError } from "$lib/stores/toastStore";
```

### Common Patterns
```typescript
// Derived values
let derivedValue = $derived(compute($state1, $state2));

// Conditional rendering
{#if condition}
  <Component />
{/if}

// Lists with keys
{#each items as item (item.id)}
  <Item {item} />
{/each}

// Bindings
<input bind:value={state} />
<input bind:checked={booleanState} />

// Event handlers
<button onclick={handleClick}>Click</button>
<input oninput={handleInput} />

// Classes with cn()
<div class={cn("base-class", condition && "conditional-class")}>
```

### Tauri Command Invocation
```typescript
// Simple command
const result = await invoke("command_name", { param1: value1 });

// Streaming command
const channel = new Channel<ChunkType>();
channel.onmessage = (chunk) => { /* handle chunk */ };
const result = await invoke("streaming_command", { channel });
```

---

**End of AI Context Document**
