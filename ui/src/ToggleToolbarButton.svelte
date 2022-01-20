<script lang="ts">
  let expanded = false;

  export let value;

  let timeout;
  function mouseOver(e: FocusEvent) {
    timeout = setTimeout(function () {
      expanded = true;
    }, 800);
  }

  function mouseOut(e: MouseEvent) {
    setTimeout(function () {
      expanded = !expanded;
      clearTimeout(timeout);
    }, 500);
  }
</script>

<div on:mouseenter={mouseOver}>
  <span
    class="material-icons"
    class:selected={value}
    on:click={() => (value = !value)}>grid_on</span
  >
  <div class="drop-down" on:click={() => (expanded = !expanded)}>
    <span class="material-icons">arrow_drop_down</span>
  </div>

  <div class="drop-content" class:visible={expanded} on:mouseleave={mouseOut}>
    <slot>No Content was provided</slot>
  </div>
</div>

<style lang="scss">
  $primary: #1e88e5;
  $background: #211f20;
  $padding: 4px;

  $border-radius: 4px;

  div:hover {
    background: lighten($background, 5);
  }
  div {
    position: relative;
    display: flex;
    align-items: center;
    height: 100%;
    z-index: 999;
    padding-left: $padding;

    .material-icons {
      z-index: 999;
    }
    .material-icons:hover {
      cursor: pointer;
    }

    .selected {
      color: $primary;
    }

    .drop-down {
      position: relative;
      width: 10px;
      padding-left: 4px;
      padding-right: 4px;
      margin: 0;

      .material-icons {
        transform: translate(-7px, 0);
      }
    }

    .drop-down:hover {
      color: $primary;
      cursor: pointer;
    }

    .drop-content {
      position: absolute;
      top: 100%;
      width: 200px;
      min-height: 150px;
      display: flex;
      flex-direction: column;

      padding: $padding * 4;

      background: darken($background, 5);

      border-bottom-left-radius: $border-radius;
      border-bottom-right-radius: $border-radius;

      //z-index: 999;
      transition: all 0.2s ease-out;
      transform: translate(-4px, -130%);

      z-index: 0;
    }

    .visible {
      transform: translate(-4px, 0%);
      background: lighten($background, 5);
    }
  }
</style>
