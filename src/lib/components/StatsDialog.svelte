<script lang="ts">
	import { BarChart, X, Check, ArrowRight } from 'lucide-svelte';
	import { createEventDispatcher } from 'svelte';
	import { fileStore } from '$lib/stores/fileStore';
	import { searchStore } from '$lib/stores/searchStore';
	import Button from '$lib/components/ui/button.svelte';

	export let show = false;

	const dispatch = createEventDispatcher();

	interface ColumnStats {
		name: string;
		type: string;
		count: number;
		unique: number;
		min?: number;
		max?: number;
		sum?: number;
		avg?: number;
	}

	let stats: ColumnStats[] = [];
	let isComputing = false;

	$: if (show) {
		computeStats();
	}

	function getValue(obj: any, path: string): any {
		if (!obj) return undefined;
		const parts = path.split('_'); // Using our flattened key logic
		let current = obj;
		// This is tricky because our store has flattened column names but data is nested.
		// fileStore.columns are like "user_id", "user_name".
		// But data is { user: { id: 1, name: "foo" } }
		// We actually need to reconstruct or simply use the same flattening logic,
		// OR just inspect the `fileStore.columns` derived from flattening.

		// Let's assume we want to match the columns displayed in the FileViewer
		// But accessing nested data via "user_name" string isn't direct.
		// For simplicity/robustness, let's re-flatten the object temporarily or use a helper.
		// Actually, `fileStore` logic was: key = "user_name" -> obj["user"]["name"]

		for (const part of parts) {
			if (current === undefined || current === null) return undefined;
			current = current[part];
		}
		return current;
	}

	async function computeStats() {
		if (!$fileStore.lines.length) return;

		isComputing = true;

		// Run in timeout to unblock UI render
		await new Promise(r => setTimeout(r, 50));

		const lines = $searchStore.results.length > 0
			? $searchStore.results.map(r => { try { return JSON.parse(r.context) } catch (e) { return {} } })
			: $fileStore.lines.map(l => l.parsed);

		const columnInfo = $fileStore.columnInfo;
		const columns = columnInfo.map(c => c.path);
		const tempStats: Record<string, ColumnStats> = {};

		// Initialize stats for known columns
		columns.forEach(col => {
			tempStats[col] = {
				name: col,
				type: 'string', // Default
				count: 0,
				unique: 0,
				sum: 0
			};
		});

		const uniqueSets: Record<string, Set<any>> = {};
		columns.forEach(c => uniqueSets[c] = new Set());

		// Iterate ALL lines (might be slow for >100k, but let's try)
		for (const row of lines) {
			for (const col of columns) {
				const val = getValue(row, col);

				if (val !== undefined && val !== null && val !== '') {
					tempStats[col].count++;
					uniqueSets[col].add(val);

					const type = typeof val;
					if (tempStats[col].type === 'string' && type === 'number') {
						tempStats[col].type = 'number';
					}

					if (type === 'number') {
						const num = val as number;
						if (tempStats[col].min === undefined || num < tempStats[col].min) tempStats[col].min = num;
						if (tempStats[col].max === undefined || num > tempStats[col].max) tempStats[col].max = num;
						tempStats[col].sum = (tempStats[col].sum || 0) + num;
					}
				}
			}
		}

		// Finalize
		stats = Object.values(tempStats).map(s => {
			s.unique = uniqueSets[s.name].size;
			if (s.type === 'number' && s.count > 0) {
				s.avg = (s.sum || 0) / s.count;
			}
			return s;
		}).sort((a, b) => b.count - a.count); // Most populated first

		isComputing = false;
	}

	function close() {
		dispatch('close');
	}
</script>

{#if show}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-200">
		<div class="bg-background rounded-lg shadow-xl w-full max-w-4xl max-h-[85vh] flex flex-col overflow-hidden border border-border animate-in zoom-in-95 duration-200">
			<!-- Header -->
			<div class="flex items-center justify-between p-4 border-b border-border bg-muted/20">
				<div class="flex items-center gap-3">
					<div class="p-2 bg-primary/10 rounded-full text-primary">
						<BarChart class="w-5 h-5" />
					</div>
					<div>
						<h2 class="text-lg font-semibold">Dataset Statistics</h2>
						<p class="text-xs text-muted-foreground">
							Analyzing {$searchStore.results.length > 0 ? $searchStore.results.length : $fileStore.lines.length} rows
							({$fileStore.columnInfo.length} columns)
						</p>
					</div>
				</div>
				<button onclick={close} class="p-2 hover:bg-muted rounded-full transition-colors">
					<X class="w-5 h-5 text-muted-foreground" />
				</button>
			</div>

			<!-- Body -->
			<div class="flex-1 overflow-auto p-4">
				{#if isComputing}
					<div class="flex flex-col items-center justify-center h-40">
						<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mb-4"></div>
						<p class="text-muted-foreground">Calculating statistics...</p>
					</div>
				{:else}
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{#each stats as col}
							<div class="p-4 rounded-lg border border-border bg-card hover:border-primary/50 transition-colors">
								<div class="flex justify-between items-start mb-2">
									<h3 class="font-medium truncate mr-2" title={col.name}>{col.name}</h3>
									<span class="text-xs px-2 py-0.5 rounded-full bg-muted font-mono uppercase">
										{col.type}
									</span>
								</div>

								<div class="space-y-1 text-sm">
									<div class="flex justify-between">
										<span class="text-muted-foreground">Filled</span>
										<span>{col.count}</span>
									</div>
									<div class="flex justify-between">
										<span class="text-muted-foreground">Unique</span>
										<span>{col.unique}</span>
									</div>

									{#if col.type === 'number'}
										<div class="my-2 border-t border-border/50 pt-2 space-y-1">
											<div class="flex justify-between">
												<span class="text-muted-foreground">Min</span>
												<span class="font-mono">{col.min}</span>
											</div>
											<div class="flex justify-between">
												<span class="text-muted-foreground">Max</span>
												<span class="font-mono">{col.max}</span>
											</div>
											<div class="flex justify-between">
												<span class="text-muted-foreground">Avg</span>
												<span class="font-mono">{col.avg?.toFixed(2)}</span>
											</div>
										</div>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="p-4 border-t border-border bg-muted/20 flex justify-end">
				<Button onclick={close}>Close</Button>
			</div>
		</div>
	</div>
{/if}
