<script>
  import Checkbox from "@smui/checkbox";
  import FormField from "@smui/form-field";
  import MapEditor from "./MapEditor.svelte";

  // components
  import Toolbar from "./Toolbar.svelte";

  export let Editor;

  export let name;

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
      tooltip: "Create District",
    },
    {
      value: 4,
      icon: "delete",
      tooltip: "Delete District",
    },
  ];

  let showDebugInformation = false;

  // for handling shortcuts
  function handleKeydown(event) {
    switch (event.keyCode) {
      case 49: // 1
        selectedAction = 1;
        break;
      case 50: // 2
        selectedAction = 2;
        break;
      case 51: // 3
        selectedAction = 3;
        break;
      case 52:
        selectedAction = 4;
        break;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main>
  <MapEditor bind:selectedAction bind:showDebugInformation />
  <div class="main-toolbar">
    <Toolbar options={streetToolbarOptions} bind:group={selectedAction} />
    <Toolbar options={districtToolbarOptions} bind:group={selectedAction} />
  </div>

  <div class="debug-panel">
    <FormField>
      <Checkbox bind:checked={showDebugInformation} />
      <span slot="label">Show Debug Information</span>
    </FormField>
  </div>
</main>

<style lang="scss">
  .main-toolbar {
    position: absolute;
    left: 10px;
    top: 10px;
  }

  .debug-panel {
    position: absolute;
    right: 20px;
    top: 10px;
  }
</style>
