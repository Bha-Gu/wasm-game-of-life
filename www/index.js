import { Universe } from "wasm-game-of-life";

const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#000000";
const ALIVE_COLOR = "#FFFFFF";

// Construct the universe, and get its width and height.
Universe.new();
let width = Universe.width();
let height = Universe.height();

const CELL_SIZE = Math.floor(704 / (height + 1)) - 1; // px

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

// const ctx = canvas.getContext("2d");
const gl = canvas.getContext("webgl2");

const VERTEX_SRC = `#version 300 es
in vec2 a_position;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
}`;

const FRAGMENT_SRC = `#version 300 es
precision highp float;
precision highp usampler2D;

uniform usampler2D u_cells;
uniform vec2 u_dims;       // width, height in cells
uniform float u_cellSize;
uniform vec4 u_gridColor;
uniform vec4 u_aliveColor;
uniform vec4 u_deadColor;

out vec4 outColor;

void main() {
  vec2 fragCoord = gl_FragCoord.xy;
  float span = u_cellSize + 1.0;

  // grid lines (1px gaps between cells)
  if (mod(fragCoord.x, span) < 1.0 || mod(fragCoord.y, span) < 1.0) {
    outColor = u_gridColor;
    return;
  }

  float col = floor(fragCoord.x / span);
  // gl_FragCoord y=0 is the bottom; canvas row 0 is the top, so flip
  float row = u_dims.y - 1.0 - floor(fragCoord.y / span);

  vec2 uv = (vec2(col, row) + 0.5) / u_dims;
  uint alive = texture(u_cells, uv).r;
  outColor = alive > 0u ? u_aliveColor : u_deadColor;
}`;

function compileShader(gl, type, src) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, src);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error(gl.getShaderInfoLog(shader));
  }
  return shader;
}

const program = gl.createProgram();
gl.attachShader(program, compileShader(gl, gl.VERTEX_SHADER, VERTEX_SRC));
gl.attachShader(program, compileShader(gl, gl.FRAGMENT_SHADER, FRAGMENT_SRC));
gl.linkProgram(program);
gl.useProgram(program);

// Full-screen quad
const quad = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);
const posLoc = gl.getAttribLocation(program, "a_position");
gl.enableVertexAttribArray(posLoc);
gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

// Texture that holds one byte per cell (alive/dead)
const cellTexture = gl.createTexture();
gl.bindTexture(gl.TEXTURE_2D, cellTexture);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

const u = {
  dims: gl.getUniformLocation(program, "u_dims"),
  cellSize: gl.getUniformLocation(program, "u_cellSize"),
  gridColor: gl.getUniformLocation(program, "u_gridColor"),
  aliveColor: gl.getUniformLocation(program, "u_aliveColor"),
  deadColor: gl.getUniformLocation(program, "u_deadColor"),
  cells: gl.getUniformLocation(program, "u_cells"),
};

function hexToRGBA(hex) {
  const n = parseInt(hex.replace("#", ""), 16);
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255, 1];
}
const ticksRange = document.getElementById("ticks-range");
const ticksValue = document.getElementById("ticks-value");

let ticksPerFrame = Number(ticksRange.value);

ticksRange.addEventListener("input", (event) => {
  ticksPerFrame = Number(event.target.value);
  ticksValue.textContent = `${ticksPerFrame} tick${ticksPerFrame > 1 ? "s" : ""}/frame`;
});
let animationId = null;

let pausedStatus = true;

// const drawGrid = () => {
//   ctx.beginPath();
//   ctx.strokeStyle = GRID_COLOR;

//   // Vertical lines.
//   for (let i = 0; i <= width; i++) {
//     ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
//     ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
//   }

//   // Horizontal lines.
//   for (let j = 0; j <= height; j++) {
//     ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
//     ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
//   }

//   ctx.stroke();
// };
// Import the WebAssembly memory at the top of the file.
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg.wasm";

// ...

const getIndex = (row, column) => {
  return row * width + column;
};

const drawCells = () => {
  const cellsPtr = Universe.cells();
  const packed = new Uint8Array(memory.buffer, cellsPtr, (width * height) / 8);
  console.log(packed);
  // Unpack bit-packed cells into one byte per cell for the texture
  const unpacked = new Uint8Array(width * height);
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      unpacked[idx] = packed[idx >> 3] & (1 << (idx & 7)) ? 1 : 0;
    }
  }

  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, cellTexture);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.R8UI,
    width,
    height,
    0,
    gl.RED_INTEGER,
    gl.UNSIGNED_BYTE,
    unpacked,
  );

  gl.useProgram(program);
  gl.uniform2f(u.dims, width, height);
  gl.uniform1f(u.cellSize, CELL_SIZE);
  gl.uniform4fv(u.gridColor, hexToRGBA(GRID_COLOR));
  gl.uniform4fv(u.aliveColor, hexToRGBA(ALIVE_COLOR));
  gl.uniform4fv(u.deadColor, hexToRGBA(DEAD_COLOR));
  gl.uniform1i(u.cells, 0);

  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
};

const fps = new (class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = (1 / delta) * 1000;

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
})();

const renderLoop = () => {
  if (pausedStatus) {
  } else {
    // setTimeout(() => {
    fps.render();
    // debugger;
    for (let tick = 0; tick < ticksPerFrame; tick++) {
      Universe.tick();
    }
    // drawGrid();
    drawCells();
    animationId = requestAnimationFrame(renderLoop);
    // }, 1000 * ticksPerFrame);
  }
};

const playPauseButton = document.getElementById("play-pause");

const randomizeButton = document.getElementById("random");

const clearButton = document.getElementById("clear");

const play = () => {
  playPauseButton.textContent = "⏸ Pause";
  pausedStatus = false;
  animationId = requestAnimationFrame(renderLoop);
};

const pause = () => {
  playPauseButton.textContent = "▶ Play";
  pausedStatus = true;
  cancelAnimationFrame(animationId);
  animationId = null;
};

playPauseButton.addEventListener("click", (event) => {
  if (pausedStatus) {
    play();
  } else {
    pause();
  }
});

randomizeButton.addEventListener("click", (event) => {
  Universe.new();
  drawCells();
});

clearButton.addEventListener("click", (event) => {
  Universe.new_empty();
  drawCells();
});

// This used to be `requestAnimationFrame(renderLoop)`.
// play();

// drawGrid();
drawCells();
// requestAnimationFrame(renderLoop);
canvas.addEventListener("click", (event) => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  Universe.toggle_cell(row, col);

  // drawGrid();
  drawCells();
});
