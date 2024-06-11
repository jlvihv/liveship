<script lang="ts">
	import Sidebar from '$lib/components/sidebar.svelte';
	import NewRecord from '$lib/components/new-record.svelte';
	import RecordList from '$lib/components/record-list.svelte';
	import RecordPlan from '$lib/components/record-plan.svelte';
	import Config from '$lib/components/config.svelte';

	let currentPath = $state('new');
</script>

<div class="flex">
	<Sidebar bind:currentPath />

	<div class="w-full">
		{#if currentPath === 'new'}
			<NewRecord />
		{:else if currentPath === 'list'}
			<RecordList />
		{:else if currentPath === 'plan'}
			<RecordPlan />
		{:else if currentPath === 'config'}
			<Config />
		{/if}
	</div>
</div>
