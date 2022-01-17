<script lang="ts">
  import wasm from "../../rust/Cargo.toml";

  import Dialog, { Title, Content, Actions } from "@smui/dialog";
  import Button, { Label } from "@smui/button";
  import Snackbar, { SnackbarComponentDev } from "@smui/snackbar";

  let width;
  let height;

  let editor;
  async function loadEditor() {
    const { Editor } = await wasm();

    return Editor.new("map_canvas", canvas.width, canvas.height);
  }

  export let selectedAction = 0;
  $: if (editor) editor.switch_to_mode(selectedAction);

  export let showDebugInformation = false;
  $: if (editor) editor.set_enable_debug_information(showDebugInformation);

  export let enableGrid = true;
  $: if (editor) editor.set_grid_enabled(enableGrid);

  export let gridOffset = 200;
  $: if (editor) editor.set_grid_offset(gridOffset);

  export let gridSubdivisions = 8;
  $: if (editor) editor.set_grid_subdivisions(gridSubdivisions);

  let canvas;
  $: {
    loadEditor().then(
      (e) => {
        editor = e;
        requestAnimationFrame(renderLoop);
      },
      (error) => {
        console.log(error);
      }
    );
  }

  function getMousePos(evt) {
    var rect = canvas.getBoundingClientRect();
    return {
      x: evt.clientX - rect.left,
      y: evt.clientY - rect.top,
    };
  }

  function renderLoop() {
    editor.render();

    requestAnimationFrame(renderLoop);
  }

  function mouseDown(e) {
    if (!editor) return;

    const pos = getMousePos(e);
    editor.mouse_down(pos.x, pos.y, e.button);
  }

  var timeout;
  function mouseMove(e: MouseEvent) {
    if (!editor) return;

    const pos = getMousePos(e);

    clearTimeout(timeout);
    timeout = setTimeout(function () {
      editor.mouse_move(pos.x, pos.y, 0, 0);
    }, 50);

    editor.mouse_move(pos.x, pos.y, e.movementX, e.movementY);
  }

  function mouseUp(e) {
    if (!editor) return;

    const pos = getMousePos(e);
    editor.mouse_up(pos.x, pos.y, e.button);
  }

  export function handleKeydown(event) {
    switch (event.keyCode) {
      case 112: // F1
        editor.save();
        showInfo("Map successfully saved");
        break;
      case 113: // F2
        if (!editor.is_empty()) {
          unsavedChanges = true;
        } else {
          editor.load();
          showInfo("Map successfully loaded");
        }

        break;
      case 114: // F3
        if (editor) editor.download();
    }
  }

  function dragOver(evt: DragEvent) {
    evt.preventDefault();
  }

  function drop(evt: DragEvent) {
    // Prevent default behavior (Prevent file from being opened)
    evt.preventDefault();

    const items = evt.dataTransfer.items;
    if (!items) {
      return;
    }

    if (items.length !== 1) {
      console.error(
        "You provided multiple files for import which is not supported. Please select a single map file instead."
      );
      return;
    }

    if (evt.dataTransfer.items[0].kind !== "file") {
      console.log("You didn't provide a valid file for import. Abort.");
      return;
    }

    const file = evt.dataTransfer.items[0].getAsFile();
    const type = file.type;

    if (type !== "application/json") {
      console.error(`The provided file has the invalid type "${type}" and was rejected.`);

      return;
    }

    
    file.text().then((text) => editor.import(text));
  }

  function showInfo(message) {
    infoMessage = message;

    info.open();
  }

  let infoMessage = "";

  let unsavedChanges = false;
  let confirmOverwrite = false;
  $: if (confirmOverwrite && editor) {
    editor.load();
    showInfo("Map successfully loaded");
  }

  let info: SnackbarComponentDev;
</script>

<Dialog
  bind:open={unsavedChanges}
  aria-labelledby="unsaved-changes-title"
  aria-describedby="unsaved-changes-content"
>
  <Title id="unsaved-changes-title">Unsaved Changes</Title>
  <Content id="unsaved-changes-content"
    >All made changes will be discarded which cannot be undone. Do you want to
    continue?</Content
  >
  <Actions>
    <Button on:click={() => (confirmOverwrite = false)}>
      <Label>Cancel</Label>
    </Button>
    <Button on:click={() => (confirmOverwrite = true)}>
      <Label>Load and discard changes</Label>
    </Button>
  </Actions>
</Dialog>

<Snackbar bind:this={info}>
  <Label>{infoMessage}</Label>
  <Actions />
</Snackbar>

<canvas
  bind:this={canvas}
  on:mousemove={mouseMove}
  on:mousedown={mouseDown}
  on:mouseup={mouseUp}
  on:drop={drop}
  on:dragover={dragOver}
  id="map_canvas"
  {width}
  {height}
/>

<svelte:window bind:innerWidth={width} bind:innerHeight={height} />

<style lang="scss">
  canvas {
    border-bottom: solid 1px rgb(40, 40, 40);
    width: 100%;
    height: 100%;
  }
</style>
