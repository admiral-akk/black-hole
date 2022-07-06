import * as wasm from "web";

var renderer = wasm.get_renderer();

function render() {
    renderer.render();
    requestAnimationFrame(render);
}

render();
