<script lang="ts">
	import JsonTreeNode from './JsonTreeNode.svelte';

	let { value } = $props<{ value: any }>();
</script>

<div class="w-full overflow-x-auto text-sm">
	<JsonTreeNode value={value} isLast={true} path={['$']} />
</div>
