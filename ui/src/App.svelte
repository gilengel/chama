<script>
  import TopAppBar, { Row, Section, Title } from "@smui/top-app-bar";
  import IconButton from "@smui/icon-button";
  import Checkbox from "@smui/checkbox";
  import FormField from "@smui/form-field";
  import MapEditor from "./MapEditor.svelte";
  import Textfield from "@smui/textfield";
  import HelperText from "@smui/textfield/helper-text";

  // components
  import Toolbar from "./Toolbar.svelte";
import ToggleToolbarButton from "./ToggleToolbarButton.svelte";

  let selectedAction = 2;
  const streetToolbarOptions = [
    {
      value: 1,
      icon: "add",
      tooltip: "Create Street",
    },
    {
      value: 2,
      icon: "brush",
      tooltip: "Freeform Street",
    },
    {
      value: 3,
      icon: "delete",
      tooltip: "Delete Street",
    },
  ];

  const districtToolbarOptions = [
    {
      value: 4,
      icon: "add",
      tooltip: "Create District",
    },
    {
      value: 5,
      icon: "brush",
      tooltip: "Freeform Street",
    },
    {
      value: 6,
      icon: "delete",
      tooltip: "Delete District",
    },
  ];

  const controlPointOptions = [
    {
      value: 7,
      icon: "control_camera",
      tooltip: "Move Control Point"
    }
  ]

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
      case 52: // 4
        selectedAction = 4;
      case 53: // 5
        selectedAction = 5;
        break;

      default:
        editor.handleKeydown(event);
    }
  }

  let enableGrid = false;
  let gridOffset = 200;
  let gridSubdivisions = 4;

  let editor;
</script>

<svelte:window on:keydown={handleKeydown} />

<header>
  <div class="toolbar">
    
      <span class="material-icons">undo</span>
      <span class="material-icons">redo</span>

      <ToggleToolbarButton bind:value={enableGrid}>
        <Textfield bind:value={gridOffset} label="Grid Size">
          <HelperText slot="helper"></HelperText>
        </Textfield>
        <Textfield bind:value={gridSubdivisions} label="Grid Subdivisions">
        </Textfield>
      </ToggleToolbarButton>
    
  </div>
</header>

<main>
  <MapEditor
    bind:this={editor}
    bind:selectedAction
    bind:showDebugInformation
    bind:enableGrid
    bind:gridOffset
    bind:gridSubdivisions
  />
  <div class="main-toolbar">
    <Toolbar options={streetToolbarOptions} bind:group={selectedAction} />
    <Toolbar options={districtToolbarOptions} bind:group={selectedAction} />
    <Toolbar options={controlPointOptions} bind:group={selectedAction} />
  </div>

  <div class="debug-panel">
    <FormField>
      <Checkbox bind:checked={showDebugInformation} />
      <span slot="label">Show Debug Information</span>
    </FormField>

  </div>
</main>

<style lang="scss">
  // app colors
  $primary: #1e88e5;
  $text: #ffffff;
  $background: #211f20;
  $border-radius: 4px;

  $icon-size: 24px;
  $padding: 4px;

  .toolbar {
    position: relative;
    z-index: 999;
    background: darken($background, 5);
    height: $padding * 4 + $icon-size;
    padding-left: $padding;
    padding-right: $padding;

    display: flex;
    align-items: center;

    border-radius: $border-radius;

      .material-icons {
      padding: $padding;

        color: $text;
        font-size: $icon-size;
      }    
  }
  main {
    position: relative;

    .main-toolbar {
      position: absolute;
      left: 10px;
      top: 10px;
    }

    .debug-panel {
      position: absolute;
      right: 20px;
      top: 10px;

      display: flex;
      flex-direction: column;
    }
  }
</style>
