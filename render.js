const IlliniBlue = new Float32Array([0.075, 0.16, 0.292, 1])

const { mat4, vec3 } = glMatrix;

const PLANE_SIZE = 500;

/** @type {WebGL2RenderingContext} */
var gl;

/** @type {WebGLProgram} */
var program

var geom;

const m = mat4.create();
const v = mat4.create();
const p = mat4.create();
const up = vec3.fromValues(0, 0, 1);
const center = vec3.fromValues(0, 0, 0);
const NEAR_PLANE = 50;
const FAR_PLANE = 2 * PLANE_SIZE;
const CAMERA_DIST = PLANE_SIZE;
const CAMERA_HEIGHT = PLANE_SIZE * 0.8;

/**
 * Draw one frame
 */
function draw(t) {
  gl.clearColor(...IlliniBlue) // f(...[1,2,3]) means f(1,2,3)
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT)

  if (!geom) return;

  gl.useProgram(program);
  gl.bindVertexArray(geom.vao);

  const angle = Math.PI / 2 * (1 + Math.sin(t / 1000));
  const eye = vec3.fromValues(
    CAMERA_DIST * Math.cos(angle),
    CAMERA_DIST * Math.sin(angle),
    CAMERA_HEIGHT,
  );

  gl.uniform3fv(gl.getUniformLocation(program, 'eye'), eye);


  gl.uniformMatrix4fv(gl.getUniformLocation(program, 'm'), false, m);

  mat4.lookAt(v, eye, center, up);
  gl.uniformMatrix4fv(gl.getUniformLocation(program, 'v'), false, v);

  gl.uniformMatrix4fv(gl.getUniformLocation(program, 'p'), false, p);

  gl.drawElements(geom.mode, geom.count, geom.type, 0);
}

/**
 * Resizes the canvas to completely fill the screen
 */
function fillScreen(t) {
  let canvas = document.querySelector('canvas')
  document.body.style.margin = '0'
  canvas.style.width = '100%'
  canvas.style.height = '100%'
  canvas.width = canvas.clientWidth
  canvas.height = canvas.clientHeight
  canvas.style.width = ''
  canvas.style.height = ''
  // to do: update aspect ratio of projection matrix here
  mat4.perspective(p, Math.PI / 2, canvas.width / canvas.height, NEAR_PLANE, FAR_PLANE);

  if (gl) {
    gl.viewport(0, 0, canvas.width, canvas.height)
    draw(t)
  }

  requestAnimationFrame(fillScreen);
}

/**
 * Compile, link, other option-independent setup
 */
async function setup(event) {
  gl = document.querySelector('canvas').getContext('webgl2',
    // optional configuration object: see https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/getContext
    { antialias: false, depth: true, preserveDrawingBuffer: true }
  )

  gl.enable(gl.DEPTH_TEST);

  program = await compileAndLinkGLSL(gl, 'terrain_vertex.glsl', 'terrain_fragment.glsl');
  geom = await setupGeomery(gl, program, generateTerrain(100, 100));

  requestAnimationFrame(fillScreen);
}


window.addEventListener('load', setup)
window.addEventListener('resize', fillScreen)

function computeNormals(positions, triangles) {
  const normals = positions.map(() => vec3.create());
  const e1 = vec3.create();
  const e2 = vec3.create();
  const faceNormal = vec3.create();

  for (const indices of triangles) {
    const vs = indices.map(i => vec3.fromValues(...positions[i]));
    vec3.sub(e1, vs[1], vs[0]);
    vec3.sub(e2, vs[2], vs[0]);
    vec3.cross(faceNormal, e1, e2);
    for (const i of indices) {
      const vertexNormal = normals[i];
      vec3.add(vertexNormal, vertexNormal, faceNormal);
    }
  }

  for (const normal of normals) {
    vec3.normalize(normal, normal);
  }

  return normals;
}

function generateTerrain(resolution, slices) {
  const R = PLANE_SIZE / 10;
  const step = PLANE_SIZE / resolution;
  let delta = PLANE_SIZE / 10;

  const ind = (i, j) => i * resolution + j;

  const positions = [];
  for (let i = 0; i < resolution; ++i) {
    for (let j = 0; j < resolution; ++j) {
      positions.push(vec3.fromValues(
        step * i - PLANE_SIZE / 2,
        step * j - PLANE_SIZE / 2,
        0,
      ))
    }
  }

  for (let i = 0; i < slices; ++i, delta *= 0.99) {
    const p = vec3.fromValues(
      Math.random() * resolution * step - step * resolution / 2,
      Math.random() * resolution * step - step * resolution / 2,
      0
    );
    const theta = Math.random() * Math.PI * 2;
    const normal = vec3.fromValues(Math.cos(theta), Math.sin(theta), 0);
    const dir = vec3.create();
    for (const v of positions) {
      vec3.sub(dir, v, p);
      const r = Math.sqrt(vec3.dot(normal, dir));
      const g = r < R ? (1 - (r / R) ** 2) ** 2 : 0;
      if (r > 0) {
        v[2] += delta * g;
      } else {
        v[2] -= delta * g;
      }
    }
  }

  let zMax = -Infinity;
  let zMin = Infinity;
  for (const v of positions) {
    zMax = Math.max(zMax, v[2]);
    zMin = Math.min(zMin, v[2]);
  }
  for (const v of positions) {
    v[2] = (v[2] - zMin) / (zMax - zMin) * (PLANE_SIZE / 2);
  }

  const triangles = [];
  for (let i = 0; i < resolution - 1; ++i) {
    for (let j = 0; j < resolution - 1; ++j) {
      triangles.push([ind(i, j), ind(i + 1, j), ind(i, j + 1)]);
      triangles.push([ind(i, j + 1), ind(i + 1, j), ind(i + 1, j + 1)]);
    }
  }

  spheroidalWeathering(positions, resolution);

  const normals = computeNormals(positions, triangles);

  // TODO
  const colors = true 
    ? computeColors(positions)
    : positions.map(() => vec3.fromValues(0.5, 0.5, 0.5));

  return {
    triangles,
    attributes: {
      position: positions.map(v => [...v]),
      normal: normals.map(v => [...v]),
      color: colors.map(v => [...v])
    },
  }
}

function computeColors(positions) {
  const c1 = vec3.fromValues(0, 0, 1);
  const c2 = vec3.fromValues(0, 0.8, 0);
  const c3 = vec3.fromValues(1, 0, 0);

  const h1 = 0;
  const h2 = PLANE_SIZE / 4;
  const h3 = PLANE_SIZE / 2;

  const tmp1 = vec3.create();
  const tmp2 = vec3.create();

  return positions.map(v => {
    return v[2] > h3
      ? c3
      : v[2] > h2
        ? vec3.add(
          vec3.create(),
          vec3.scale(tmp1, c2, (h3 - v[2]) / (h3 - h2)),
          vec3.scale(tmp2, c3, (v[2] - h2) / (h3 - h2)))
        : v[2] > h1
          ? vec3.add(
            vec3.create(),
            vec3.scale(tmp1, c1, (h2 - v[2]) / (h2 - h1)),
            vec3.scale(tmp2, c2, (v[2] - h1) / (h2 - h1)))
          : c1;
  });
}

function spheroidalWeathering(positions, resolution, n = 5) {
  const ind = (i, j) => i * resolution + j;
  const tmp = vec3.create();
  while (n--) {
    const average = positions.map(() => vec3.create());
    for (let i = 0; i < resolution; ++i) {
      for (let j = 0; j < resolution; ++j) {
        const a = average[ind(i, j)];
        vec3.add(a, a, vec3.scale(tmp, positions[ind(Math.max(0, i - 1), j)], 0.25));
        vec3.add(a, a, vec3.scale(tmp, positions[ind(Math.min(resolution - 1, i + 1), j)], 0.25));
        vec3.add(a, a, vec3.scale(tmp, positions[ind(i, Math.max(0, j - 1))], 0.25));
        vec3.add(a, a, vec3.scale(tmp, positions[ind(i, Math.min(resolution - 1, j + 1))], 0.25));
      }
    };
    for (let i = 0; i < resolution; ++i) {
      for (let j = 0; j < resolution; ++j) {
        const p = positions[ind(i, j)];
        const a = average[ind(i, j)];
        vec3.add(p, vec3.scale(p, p, 0.5), vec3.scale(a, a, 0.5));
      }
    }
  }
}