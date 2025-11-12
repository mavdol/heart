import React, { useEffect, useRef } from "react";

interface AnimatedMeshGradientProps {
  colors?: string[];
  className?: string;
  speed?: number;
  darkenTop?: boolean;
  fullscreen?: boolean;
}

export default function AnimatedMeshGradient({
  colors = ["#00C6FF", "#0072FF", "#7F00FF", "#E100FF"],
  className = "",
  speed = 0.002,
  darkenTop = false,
  fullscreen = false,
}: AnimatedMeshGradientProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const gl = canvas.getContext("webgl", { antialias: true });

    if (!gl) {
      console.error("WebGL not supported");
      return;
    }

    // Get dimensions based on fullscreen prop
    const getDimensions = () => {
      if (fullscreen) {
        return {
          width: window.innerWidth,
          height: window.innerHeight,
        };
      } else {
        // Get parent container dimensions
        const parent = canvas.parentElement;
        const rect =
          parent?.getBoundingClientRect() || canvas.getBoundingClientRect();
        return {
          width: rect.width || 100,
          height: rect.height || 100,
        };
      }
    };

    let { width, height } = getDimensions();
    canvas.width = width;
    canvas.height = height;

    // --- SHADERS ---
    const vertexShaderSource = `
      attribute vec2 position;
      varying vec2 vUv;
      void main() {
        vUv = position * 0.5 + 0.5;
        gl_Position = vec4(position, 0.0, 1.0);
      }
    `;

    const fragmentShaderSource = `
      precision highp float;
      varying vec2 vUv;
      uniform float uTime;
      uniform vec3 uColors[4];
      uniform float uSpeed;
      uniform bool uDarkenTop;

      // Improved noise function for smoother gradients
      float noise(vec2 p) {
        return sin(p.x * 1.5) * sin(p.y * 1.5) * 0.5 + 0.5;
      }

      float smoothNoise(vec2 p) {
        vec2 i = floor(p);
        vec2 f = fract(p);
        f = f * f * (3.0 - 2.0 * f);

        float a = noise(i);
        float b = noise(i + vec2(1.0, 0.0));
        float c = noise(i + vec2(0.0, 1.0));
        float d = noise(i + vec2(1.0, 1.0));

        return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
      }

      void main() {
        vec2 uv = vUv;
        float t = uTime * uSpeed;

        // Multi-octave noise for more interesting patterns
        float n = smoothNoise(uv * 3.0 + t * 0.5) * 0.5 +
                  smoothNoise(uv * 6.0 - t * 0.3) * 0.3 +
                  smoothNoise(uv * 12.0 + t * 0.7) * 0.2;

        // Create more organic color mixing
        vec3 color = mix(uColors[0], uColors[1], smoothstep(0.0, 1.0, uv.x * 0.8 + n * 0.3));
        color = mix(color, uColors[2], smoothstep(0.0, 1.0, uv.y * 0.8 + n * 0.3));
        color = mix(color, uColors[3], smoothstep(0.0, 1.0, n));

        if (uDarkenTop) {
          color *= mix(0.7, 1.0, uv.y);
        }

        gl_FragColor = vec4(color, 1.0);
      }
    `;

    // --- BUILD PROGRAM ---
    const compileShader = (src: string, type: number) => {
      const shader = gl.createShader(type)!;
      gl.shaderSource(shader, src);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error(gl.getShaderInfoLog(shader));
      }
      return shader;
    };

    const vertexShader = compileShader(vertexShaderSource, gl.VERTEX_SHADER);
    const fragmentShader = compileShader(
      fragmentShaderSource,
      gl.FRAGMENT_SHADER
    );

    const program = gl.createProgram()!;
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    // Check if program linked successfully
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error("Program linking failed:", gl.getProgramInfoLog(program));
      return;
    }

    gl.useProgram(program);

    // --- GEOMETRY ---
    const position = gl.getAttribLocation(program, "position");
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]),
      gl.STATIC_DRAW
    );
    gl.enableVertexAttribArray(position);
    gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

    // --- UNIFORMS ---
    const uTime = gl.getUniformLocation(program, "uTime");
    const uColors = gl.getUniformLocation(program, "uColors");
    const uSpeed = gl.getUniformLocation(program, "uSpeed");
    const uDarkenTop = gl.getUniformLocation(program, "uDarkenTop");

    const parsedColors = colors.map((c) => {
      const bigint = parseInt(c.replace("#", ""), 16);
      return [
        ((bigint >> 16) & 255) / 255,
        ((bigint >> 8) & 255) / 255,
        (bigint & 255) / 255,
      ];
    });

    while (parsedColors.length < 4) parsedColors.push(parsedColors[0]);

    gl.uniform3fv(uColors, new Float32Array(parsedColors.flat()));
    gl.uniform1f(uSpeed, speed);
    gl.uniform1i(uDarkenTop, darkenTop ? 1 : 0);

    // --- RENDER LOOP ---
    let start = performance.now();
    let animationFrameId: number;
    const render = (now: number) => {
      const elapsed = (now - start) / 1000.0;
      gl.useProgram(program); // Ensure program is bound
      gl.viewport(0, 0, width, height);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.uniform1f(uTime, elapsed);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      animationFrameId = requestAnimationFrame(render);
    };
    animationFrameId = requestAnimationFrame(render);

    // --- RESIZE ---
    const updateDimensions = () => {
      const dims = getDimensions();
      width = dims.width;
      height = dims.height;
      canvas.width = width;
      canvas.height = height;
      gl.viewport(0, 0, width, height);
    };

    // Use ResizeObserver for container-relative sizing, window resize for fullscreen
    let resizeObserver: ResizeObserver | null = null;

    if (fullscreen) {
      const handleResize = () => {
        updateDimensions();
      };
      window.addEventListener("resize", handleResize);

      return () => {
        cancelAnimationFrame(animationFrameId);
        window.removeEventListener("resize", handleResize);
        gl.deleteProgram(program);
        gl.deleteShader(vertexShader);
        gl.deleteShader(fragmentShader);
        gl.deleteBuffer(buffer);
      };
    } else {
      // Use ResizeObserver to track container size changes
      resizeObserver = new ResizeObserver(() => {
        updateDimensions();
      });

      const parent = canvas.parentElement;
      if (parent) {
        resizeObserver.observe(parent);
      } else {
        resizeObserver.observe(canvas);
      }

      return () => {
        cancelAnimationFrame(animationFrameId);
        resizeObserver?.disconnect();
        gl.deleteProgram(program);
        gl.deleteShader(vertexShader);
        gl.deleteShader(fragmentShader);
        gl.deleteBuffer(buffer);
      };
    }
  }, [colors, speed, darkenTop, fullscreen]);

  const containerClasses = fullscreen
    ? "fixed inset-0"
    : "absolute inset-0 w-full h-full";

  return (
    <canvas
      ref={canvasRef}
      className={`${containerClasses} ${className}`}
      style={{ display: "block" }}
    />
  );
}
