import * as wasm from "orion";

let canvas = document.getElementById("canvas")



let game = wasm.Game.new()
console.log(`Created new game: ${game}`)

canvas.width = document.body.clientWidth;
canvas.height = document.body.clientHeight;

let prev_time = performance.now()
const renderLoop = (now) => {
  let dt = (now - prev_time) / 1000;
  prev_time = now;
  game.update(dt);

  // Clear
  const context = canvas.getContext('2d');
  context.clearRect(0, 0, canvas.width, canvas.height);

  game.render(canvas)

  requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);
