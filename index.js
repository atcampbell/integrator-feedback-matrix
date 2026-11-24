import { createScope } from './js/scope.js';

import('./pkg').catch(console.error).then((rust_module) => {
    const SCOPE_ZOOM = 1;
    const AXIS_RANGE = {
        x: [-SCOPE_ZOOM, SCOPE_ZOOM],
        y: [-SCOPE_ZOOM, SCOPE_ZOOM],
        z: [-SCOPE_ZOOM, SCOPE_ZOOM],
    };
    // Which axes to plot horizontally and vertically: any of 'x', 'y', 'z'.
    const SCOPE_H = 'x';
    const SCOPE_V = 'y';

    const scope = createScope(document.getElementById('scope'), AXIS_RANGE[SCOPE_H], AXIS_RANGE[SCOPE_V]);

    // The synth is created on the first start and then kept: stop pauses the
    // audio rather than destroying it, so the matrix weights survive.
    let handle = null;
    let playing = false;

    const play_button = document.getElementById('play');
    play_button.addEventListener('click', () => {
        if (handle == null) {
            // Created inside a click because browsers only allow audio to
            // start from a user gesture.
            handle = rust_module.beep(
                param_inputs.map((input) => input.dataset.param),
                Float64Array.from(param_inputs, param_value),
            );
            playing = true;
        } else if (playing) {
            handle.pause();
            playing = false;
        } else {
            handle.play();
            playing = true;
        }
        play_button.value = playing ? 'stop' : 'start';
    });

    // Move every parameter slider to a random point in its range.
    const randomise_button = document.getElementById('randomise');
    randomise_button.addEventListener('click', () => {
        for (const input of param_inputs) {
            randomise_param(input);
        }
    });

    // New random matrix weights. Before the first start there's no synth yet;
    // it gets random weights when it's created.
    const randomise_matrix_button = document.getElementById('randomise-matrix');
    randomise_matrix_button.addEventListener('click', () => {
        if (handle != null) {
            handle.randomise_matrix();
        }
    });

    // Every element with a `data-param="name"` attribute in index.html is sent
    // to Rust under that name. An element with the id `<name>-value`, if there
    // is one, shows the current value.
    const param_inputs = [...document.querySelectorAll('[data-param]')];
    const param_value = (input) => parseFloat(input.value);

    // Move one slider to a random point in its range.
    const randomise_param = (input) => {
        const min = parseFloat(input.min);
        const max = parseFloat(input.max);
        input.value = min + Math.random() * (max - min);
        // Run the slider's own listener, which updates its display and sends
        // the new value to Rust.
        input.dispatchEvent(new Event('input'));
    };

    for (const input of param_inputs) {
        const name = input.dataset.param;
        const display = document.getElementById(`${name}-value`);
        input.addEventListener('input', () => {
            if (display != null) {
                display.textContent = input.value;
            }
            if (handle != null) {
                handle.set_param(name, param_value(input));
            }
        });

        // A randomise button for just this parameter, placed after its value
        // display (or after the slider if it has no display).
        const randomise = document.createElement('input');
        randomise.type = 'button';
        randomise.value = 'randomise';
        randomise.addEventListener('click', () => randomise_param(input));
        (display ?? input).after(randomise);
    }

    const animate = () => {
        // scope.fade();
        if (handle != null && playing) {
            const points = handle.take_points(); // has .x, .y, .z Float32Arrays
            scope.draw(points[SCOPE_H], points[SCOPE_V]);
            points.free(); // release the Rust-side memory
        }
        requestAnimationFrame(animate);
    };
    requestAnimationFrame(animate);
});
