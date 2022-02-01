<script>
  import wasm from "../../rust/Cargo.toml";

  async function loadEditor() {
    const { Editor } = await wasm();

    //return Editor.new("map_canvas", canvas.width, canvas.height);
  }

  let main;
  $: {
    loadEditor().then(
      () => {
        //editor = e;
        //requestAnimationFrame(renderLoop);
      },
      (error) => {
        console.log(error);
      }
    );
  }

  let editor;
</script>

<header>
  <div id="num-clicks"></div>
  <button id="green-square">CLICK ME</button>
</header>

<main id="main" bind:this={main} />

<style lang="scss" global>
  // app colors
  $primary: #1e88e5;
  $text: #ffffff;
  $background: #211f20;
  $border-radius: 4px;

  $icon-size: 24px;
  $padding: 4px;
  // app colors
  $primary: #1e88e5;
  $text: #ffffff;
  $background: #211f20;

  $border-radius: 4px;

  $padding: 4px;

  main {
    #left_primary_toolbar {
      position: absolute;
      left: 0;
      top: 0;

      margin-top: $padding;
      margin-left: $padding;
    }

    #right_primary_toolbar {
      position: absolute;
      right: 0;
      top: 0;

      margin-top: $padding;
      margin-right: $padding;
    }
  }

  .toolbar:not(:last-child) {
    margin-bottom: $padding;
  }
  .toolbar {
    background: darken($background, 5);

    $icon-size: 24px;

    list-style: none;
    display: flex;
    flex-direction: column;

    width: $padding * 4 + $icon-size;
    padding: 0;
    margin: 0;

    border-radius: $border-radius;

    li {
      position: relative;
      text-align: center;
      display: flex;
      align-content: center;
      justify-content: center;

      padding: $padding;

      .tooltip {
        position: absolute;
        left: 0;
        white-space: nowrap;

        transform: translate(50px);
        visibility: hidden;

        background: darken($background, 8);
        color: $text;

        border-radius: $border-radius;

        padding: $padding * 2;

        font-family: "Heebo", sans-serif;
        font-size: 14px;
      }
    }
    li:hover {
      .tooltip {
        visibility: visible;
      }
    }
    
    input {
      position: absolute;
      visibility: collapse;
    }
    label {
      cursor: pointer;
      height: $icon-size;
      display: flex;
      align-items: center;
      padding: $padding;

      .material-icons {
        color: $text;
        font-size: $icon-size;
      }
    }
    input:checked + label {
      .material-icons {
        color: $primary;
      }
    }
  }
</style>