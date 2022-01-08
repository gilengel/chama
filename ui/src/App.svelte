<script lang="ts">
  import { onMount } from "svelte";
  import MapEditor from "./MapEditor.svelte"

  // components
  import Toolbar from "./Toolbar.svelte";

  export let Editor;

  export let name: string;

  /*
  
  $: if(editor) {
    editor.switch_to_mode(selectedAction);
  }
  */
  let selectedAction = 1;
  const streetToolbarOptions = [
    {
      value: 1,
      icon: "add",
      tooltip: "Create Street",
    },
    {
      value: 2,
      icon: "delete",
      tooltip: "Delete Street",
    },
  ];

  const districtToolbarOptions = [
    {
      value: 3,
      icon: "add",
      tooltip: "Create Street2",
    },
    {
      value: 4,
      icon: "delete",
      tooltip: "Delete Street2",
    },    
  ]


</script>

<main>
  <MapEditor bind:selectedAction={selectedAction} />
  <div class="main-toolbar">
    <Toolbar options={streetToolbarOptions} bind:group={selectedAction} />
    <Toolbar options={districtToolbarOptions} bind:group={selectedAction} />
  </div>
</main>

<style lang="scss">
  .main-toolbar {
    position: absolute;
    left: 10px;
    top: 10px;
  }
</style>
