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

<header />

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
  $error: #d64933;
  $text: #ffffff;
  $background: #211f20;

  $border-radius: 4px;

  $padding: 4px;





  /// MATERIAL COMPONENTS
  $snackbar-height: 48px;
  $snackbar-padding: 16px;
  .md-snackbar {
    position: absolute;
    background-color: #d64933;
    display: block;

    width: 344px;
    line-height: $snackbar-height;
    height: $snackbar-height;
    padding-left: $snackbar-padding;
    padding-right: $snackbar-padding;
    border-radius: 4px;
    bottom: $snackbar-padding * 1.5;
  }

  .left {
    left: $snackbar-padding * 1.5;
  }

  .center {
    left: 50%;
    transform: translateX(-50%);
  }

  .right {
    right: $snackbar-padding * 1.5;
  }

  /// MATERIAL COMPONENTS







  body {
    font-family: "Heebo", sans-serif;
  }

  main {
    .dialog {
      position: absolute;
      //left: 50%;
      right: 0;
      top: 0%;
      height: 100%;
      box-shadow: 0 10px 20px rgba(0,0,0,0.19), 0 6px 6px rgba(0,0,0,0.23);

      //transform: translate(-50%, -50%);

      min-width: 500px;
      max-width: 800px;
      border-radius: $border-radius;

      background-color: lighten($background, 5%);
      padding: $padding * 6;

      display: flex;
      flex-direction: column;

      button.close {
        position: absolute;
        top: 0;
        right: 12px;
        width: 28px;
        height: 28px;
        border: none;
        display: flex;
        justify-content: center;
        align-items: center;
        border: solid lighten($background, 20%) 1px;
        border-top: none;
        border-bottom-left-radius: $border-radius;
        border-bottom-right-radius: $border-radius;
        background-color: transparent;
        transition: all 0.05s ease-out;
        
        span {
          color: lighten($background, 20%);
        }
      }

      button.close:hover {
        background-color: $error;
        cursor: pointer;

        span {
          color: white;
        }
      }

      div:first-child {
        h2 {
          margin-top: 0;
        }
      }

      div {
        h2 {
          margin-top: 1em;
        }

        .textbox {
          height: 4em;
          display: flex;
          flex-direction: column;

          .info {
            color: $error;
            font-size: 0.8em;
            text-align: right;
            padding: 0;
            padding-top: 0.1em;
          }

          input[type="number"],
          input[type="text"] {
            -moz-appearance: textfield;

            background-color: $background;
            border: none;
            border-radius: calc($border-radius / 2);
            color: $text;
            padding-left: $padding;

            font-size: 1em;
            height: 2em;
          }

          input[type="number"]:focus,
          input[type="text"]:focus {
            outline: none;
          }

          input[type="number"]:focus:not(.error),
          input[type="text"]:focus:not(.error) {
            outline: none;
            border-bottom: solid $primary 2px !important;
          }

          .error {
            border: none !important;
            border-bottom: solid $error 2px !important;
          }

          input[type="number"]::-webkit-inner-spin-button,
          input[type="number"]::-webkit-outer-spin-button {
            -webkit-appearance: none;
            margin: 0;
          }
        }
        div {
          position: relative;
          padding-top: $padding;
          padding-bottom: $padding;
          display: flex;
          justify-content: space-between;

          line-height: em;

          label {
            padding-right: 1em;
            font-size: 1em;
          }
        }
      }
    }

    canvas {
      position: absolute;
      z-index: 0;
    }

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

    #top_primary_toolbar {
      position: absolute;
      left: 0;
      top: 0;

      margin-top: $padding;
      margin-left: $padding;

      display: flex;

      .toolbar {
        
        flex-direction: row;
        width: auto;

        height: $padding * 4 + $icon-size;

        li {
          .tooltip {
            bottom: calc(-100% - $padding);
            left: 0;
            transform: translate(-50%);
          }
        }
      }

      .toolbar:not(:last-child) {
        margin-right: $padding;
      }
    }
  }

  .toolbar:not(:last-child) {
    margin-bottom: $padding;
  }
  .toolbar {
    position: relative;
    z-index: 999;
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

    button {
      color: white;
      background-color: transparent;
      border: none;

      cursor: pointer;
      display: flex;
      align-items: center;
      padding: $padding;
    }

    button.selected {
      color: $primary;
    }

    button:hover {
      color: $primary;
    }
    button:disabled {
      color: #424242;
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
