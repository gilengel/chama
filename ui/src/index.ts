import "../app.scss";

const wasm = import("../pkg/index.js");

import { Editor } from "../pkg/index.js";

const editor = Editor.new("map_canvas");
//editor.switch_to_mode(1);

const data = { width: 800, height: 800, realPart: -0.8, imaginaryPart: 0.156 };

wasm.then((lib) => {});

const renderLoop = () => {
  //numStreets.textContent = mapEditor.streets_length();
  //numIntersections.textContent = mapEditor.intersections_length();
  editor.render();

  requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);

const canvas = document.getElementById("map_canvas");

function getMousePos(evt: MouseEvent) {
  var rect = canvas.getBoundingClientRect();
  return {
    x: evt.clientX - rect.left,
    y: evt.clientY - rect.top,
  };
}

canvas.addEventListener("contextmenu", (e) => {
  e.preventDefault();

  const pos = getMousePos(e);
  editor.mouse_up(pos.x, pos.y, e.button);
});

canvas.addEventListener("mousedown", (e) => {
  const pos = getMousePos(e);
  editor.mouse_down(pos.x, pos.y, e.button);
});

canvas.addEventListener("mousemove", (e) => {
  const pos = getMousePos(e);
  editor.mouse_move(pos.x, pos.y);
});

canvas.addEventListener("mouseup", (e) => {
  const pos = getMousePos(e);
  editor.mouse_up(pos.x, pos.y, e.button);
});

const actionButtons = document.getElementsByName("primaryAction");
for (var i = 0; i < actionButtons.length; i++) {
  actionButtons[i].addEventListener("change", function () {
    const button = this as HTMLInputElement;
    editor.switch_to_mode(parseInt(button.value));
  });
}
