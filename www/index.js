import {Universe} from "wasm-game-of-life";

const universe = Universe.new("canvas", 64, 64);
const CELL_SIZE = 15;

// Dom
const canvas = document.querySelector("#canvas");
const canvasHeightInput = document.querySelector("#canvas-height");
const canvasWidthInput = document.querySelector("#canvas-width");
const createNewCanvasButton = document.querySelector("#create-new-canvas");
const playPauseButton = document.querySelector("#play-pause");
const randomMutateButton = document.querySelector("#random-mutate");

let animationId = null;
let isDrawing = false;
let isPause = false;

const fps = new class {
  constructor() {
    this.fps = document.querySelector("#fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = 1 / delta * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    // Render the statistics.
    this.fps.textContent = `
Frames per Second:
         latest = ${Math.round(fps)}
avg of last 100 = ${Math.round(mean)}
min of last 100 = ${Math.round(min)}
max of last 100 = ${Math.round(max)}
`.trim();
  }
};


// Game loop
const renderLoop = () => {
  fps.render();
  universe.draw_grid();
  universe.draw_cells();
  universe.tick();
  animationId = requestAnimationFrame(renderLoop);
}

// Actions
const play = () => {
  isPause = false;
  playPauseButton.textContent = "⏸";
  renderLoop();
}

const pause = () => {
  isPause = true;
  playPauseButton.textContent = ">";
  cancelAnimationFrame(animationId);
  animationId = null;
}


createNewCanvasButton.addEventListener("click", () => {
  universe.set_width(canvasWidthInput.value);
  universe.set_height(canvasHeightInput.value);
})

// NOTE: auto play when clicked randomMutateButton.
randomMutateButton.addEventListener("click", () => {
  playPauseButton.textContent = "⏸";
  universe.random_mutate();
})

playPauseButton.addEventListener("click", () => {
  isPause = !isPause;
  if (animationId == null) {
    play();
  } else {
    pause();
  }
})

canvas.addEventListener("mousedown", () => {
  pause();
  isDrawing = true;
})

canvas.addEventListener("mouseup", () => {
  isDrawing = false;
})

canvas.addEventListener("mousemove", (event) => {
  if (!isDrawing || !isPause) return;
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), universe.height() - 1);
  const column = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), universe.width() - 1);

  universe.set_alive_cell(row, column);
  universe.draw_cells();
})

play();

