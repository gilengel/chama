<script>
  import wasm from "../../rust/Cargo.toml";

  let editor;
  async function loadEditor() {
    const { Editor } = await wasm();

    return Editor.new("map_canvas", canvas.width, canvas.height);
  }

  export let selectedAction = 0;
  $: if(editor) editor.switch_to_mode(selectedAction);

  export let showDebugInformation = false;
  $: if(editor) editor.set_enable_debug_information(showDebugInformation);

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

  function mouseMove(e) {
    if (!editor) return;

    const pos = getMousePos(e);
    editor.mouse_move(pos.x, pos.y);
  }

  function mouseUp(e) {
    if (!editor) return;

    const pos = getMousePos(e);
    editor.mouse_up(pos.x, pos.y, e.button);
  }
</script>

<canvas
  bind:this={canvas}
  on:mousemove={mouseMove}
  on:mousedown={mouseDown}
  on:mouseup={mouseUp}
  id="map_canvas"
  width="1920"
  height="1080"
/>

<style lang="scss">
  /* your styles go here */
</style>
