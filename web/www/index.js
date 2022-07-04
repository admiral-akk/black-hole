import * as wasm from "web";

var renderer = wasm.get_renderer();

function render() {
    renderer.render();
}

render();

document.getElementById('input').onchange = render;