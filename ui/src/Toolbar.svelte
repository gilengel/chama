<script>
  export let group;
  export let options;

  const uniqueID = Math.floor(Math.random() * 100);
  const slugify = (str = "") =>
    str.toLowerCase().replace(/ /g, "-").replace(/\./g, "");
</script>

<ul class="toolbar" role="radiogroup">
  {#each options as { value, icon, tooltip }}
    <li>
      <input
        id={slugify(tooltip)}
        type="radio"
        bind:group={group}
        value={value}
      />
      <label for={slugify(tooltip)}>
        <span class="material-icons">{icon}</span>
      </label>
      <span class="tooltip">{tooltip}</span>
    </li>
  {/each}
</ul>

<style lang="scss">
  // app colors
  $primary: #1e88e5;
  $text: #ffffff;
  $background: #211f20;

  $border-radius: 4px;

  $padding: 4px;

  .toolbar:not(:last-child) {
    margin-bottom: $padding;
  }
  .toolbar {
    background: darken($background, 5);

    $icon-size: 24px;

    list-style: none;
    display: flex;
    flex-direction: column;

    width: $padding * 4 + $icon-size; //calc($padding * 4 + $icon-size);
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
