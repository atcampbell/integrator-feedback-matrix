// Canvas plot of two of the oscillator's axes, with fading trails.
// Knows nothing about wasm: it just draws matching arrays of horizontal and
// vertical values. The caller picks which axes those are.

// Opacity of the black layer painted each frame; lower = longer trails.
const FADE = 0.05;

// h_range / v_range: [min, max] of the values drawn on each axis.
export function createScope(canvas, h_range, v_range) {
    const ctx = canvas.getContext('2d');
    let last_point = null;

    const scale = (value, [min, max]) => (value - min) / (max - min);
    const toPx = (h, v) => [
        scale(h, h_range) * canvas.width,
        // Canvas y increases downwards, so flip it.
        (1 - scale(v, v_range)) * canvas.height,
    ];

    return {
        // Fade out what's already drawn. Call once per animation frame.
        fade() {
            ctx.fillStyle = `rgba(0, 0, 0, ${FADE})`;
            ctx.fillRect(0, 0, canvas.width, canvas.height);
        },

        // Draw a line through the points (hs[i], vs[i]), continuing from the
        // last call.
        draw(hs, vs) {
            ctx.fillRect(0, 0, canvas.width, canvas.height);

            const n = Math.min(hs.length, vs.length);
            if (n === 0) {
                return;
            }

            ctx.strokeStyle = '#0f0';
            ctx.lineWidth = 1;
            ctx.beginPath();
            const start = last_point ?? toPx(hs[0], vs[0]);
            ctx.moveTo(start[0], start[1]);
            for (let i = 0; i < n; i++) {
                const [px, py] = toPx(hs[i], vs[i]);
                ctx.lineTo(px, py);
            }
            ctx.stroke();
            last_point = toPx(hs[n - 1], vs[n - 1]);
        },

        // Forget the last point so the next line doesn't join onto it.
        reset() {
            last_point = null;
        },
    };
}
