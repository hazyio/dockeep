(function() {
    window.__dockeep_selection_0replace_key0 = null;

    const overlay = document.createElement('div');
    Object.assign(overlay.style, {
        position: 'fixed', inset: '0', zIndex: 2147483647, cursor: 'crosshair'
    });
    document.body.appendChild(overlay);

    let box, widthLabel, heightLabel;

    function cleanup() {
        if (box) { box.remove(); box = null; }
        if (widthLabel) { widthLabel.remove(); widthLabel = null; }
        if (heightLabel) { heightLabel.remove(); heightLabel = null; }
    }

    function reset() {
        cleanup();
        overlay.remove();
    }

    function initBox() {
        const w = Math.round(window.innerWidth  * 0.5);
        const h = Math.round(window.innerHeight * 0.5);
        const x = Math.round((window.innerWidth  - w) / 2);
        const y = Math.round((window.innerHeight - h) / 2);

        box = document.createElement('div');
        Object.assign(box.style, {
            position: 'fixed', border: '2px dashed red',
            background: 'rgba(255,0,0,0.1)', zIndex: 2147483647,
            overflow: 'visible', boxSizing: 'border-box',
            left: x + 'px', top: y + 'px',
            width: w + 'px', height: h + 'px'
        });
        document.body.appendChild(box);

        const labelStyle = {
            position: 'fixed', background: 'rgba(0,0,0,0.65)', color: '#fff',
            font: 'bold 11px monospace', padding: '1px 4px', borderRadius: '3px',
            pointerEvents: 'none', zIndex: 2147483647, whiteSpace: 'nowrap'
        };
        widthLabel = document.createElement('div');
        Object.assign(widthLabel.style, labelStyle);
        document.body.appendChild(widthLabel);

        heightLabel = document.createElement('div');
        Object.assign(heightLabel.style, labelStyle);
        document.body.appendChild(heightLabel);

        updateLabels();
        addBoxControls();

        overlay.style.pointerEvents = 'none';
        overlay.style.cursor = 'default';
    }

    document.addEventListener('keydown', e => {
        if (e.key === 'Escape') reset();
    });

    function updateLabels() {
        const x = parseFloat(box.style.left), y = parseFloat(box.style.top);
        const w = parseFloat(box.style.width),  h = parseFloat(box.style.height);
        widthLabel.textContent  = Math.round(w) + 'px';
        heightLabel.textContent = Math.round(h) + 'px';
        widthLabel.style.left  = (x + w / 2 - widthLabel.offsetWidth / 2) + 'px';
        widthLabel.style.top   = (y + h + 4) + 'px';
        heightLabel.style.left = (x + w + 4) + 'px';
        heightLabel.style.top  = (y + h / 2 - heightLabel.offsetHeight / 2) + 'px';
    }

    function addBoxControls() {
        // ── Buttons ──────────────────────────────────────────────────────────
        const btnWrap = document.createElement('div');
        Object.assign(btnWrap.style, {
            position: 'absolute', top: '50%', left: '50%',
            transform: 'translate(-50%, -50%)',
            display: 'flex', gap: '8px', zIndex: 10
        });
        box.appendChild(btnWrap);

        const btnStyle = {
            padding: '6px 16px', fontSize: '13px', fontWeight: 'bold',
            cursor: 'pointer', border: 'none', borderRadius: '6px',
            whiteSpace: 'nowrap', boxShadow: '0 2px 6px rgba(0,0,0,0.4)'
        };

        const btn = document.createElement('button');
        btn.textContent = 'Select';
        Object.assign(btn.style, { ...btnStyle, background: 'red', color: '#fff' });
        btnWrap.appendChild(btn);
        btn.addEventListener('click', () => {
            const rect = box.getBoundingClientRect();
            window.__dockeep_selection_0replace_key0 = {
                x: rect.left, y: rect.top,
                width: rect.width, height: rect.height
            };
            reset();
        });

        const cancelBtn = document.createElement('button');
        cancelBtn.textContent = 'Cancel';
        Object.assign(cancelBtn.style, { ...btnStyle, background: '#555', color: '#fff' });
        btnWrap.appendChild(cancelBtn);
        cancelBtn.addEventListener('click', reset);

        // ── Drag handle ──────────────────────────────────────────────────────
        const handle = document.createElement('div');
        Object.assign(handle.style, {
            position: 'absolute', top: '0', left: '0', right: '0',
            height: '100%', background: 'rgba(255,80,80,0.18)',
            cursor: 'move', userSelect: 'none', zIndex: 5
        });
        box.appendChild(handle);

        let dragStartX, dragStartY, boxStartLeft, boxStartTop;

        handle.addEventListener('pointerdown', e => {
            handle.setPointerCapture(e.pointerId);
            dragStartX = e.clientX; dragStartY = e.clientY;
            boxStartLeft = parseFloat(box.style.left);
            boxStartTop  = parseFloat(box.style.top);
            document.body.style.userSelect = 'none';
        });
        handle.addEventListener('pointermove', e => {
            if (!handle.hasPointerCapture(e.pointerId)) return;
            box.style.left = (boxStartLeft + e.clientX - dragStartX) + 'px';
            box.style.top  = (boxStartTop  + e.clientY - dragStartY) + 'px';
            updateLabels();
        });
        handle.addEventListener('pointerup', () => {
            document.body.style.userSelect = '';
        });

        // ── Resize handles ───────────────────────────────────────────────────
        // Each def describes which edges move and how to position the handle dot
        const resizeDefs = [
            { edges: ['n','w'], cursor: 'nw-resize', top: '-5px',    left: '-5px'              },
            { edges: ['n'],     cursor: 'n-resize',  top: '-5px',    left: 'calc(50% - 5px)'   },
            { edges: ['n','e'], cursor: 'ne-resize', top: '-5px',    right: '-5px'             },
            { edges: ['e'],     cursor: 'e-resize',  top: 'calc(50% - 5px)', right: '-5px'     },
            { edges: ['s','e'], cursor: 'se-resize', bottom: '-5px', right: '-5px'             },
            { edges: ['s'],     cursor: 's-resize',  bottom: '-5px', left: 'calc(50% - 5px)'  },
            { edges: ['s','w'], cursor: 'sw-resize', bottom: '-5px', left: '-5px'              },
            { edges: ['w'],     cursor: 'w-resize',  top: 'calc(50% - 5px)', left: '-5px'      },
        ];

        resizeDefs.forEach(def => {
            const rh = document.createElement('div');
            Object.assign(rh.style, {
                position: 'absolute', width: '10px', height: '10px',
                background: '#fff', border: '2px solid red', borderRadius: '2px',
                cursor: def.cursor, zIndex: 20, boxSizing: 'border-box'
            });
            if (def.top)    rh.style.top    = def.top;
            if (def.bottom) rh.style.bottom = def.bottom;
            if (def.left)   rh.style.left   = def.left;
            if (def.right)  rh.style.right  = def.right;
            box.appendChild(rh);

            let rsX0, rsY0, rsL, rsT, rsW, rsH;

            rh.addEventListener('pointerdown', e => {
                e.stopPropagation();
                rh.setPointerCapture(e.pointerId);
                rsX0 = e.clientX; rsY0 = e.clientY;
                rsL  = parseFloat(box.style.left);
                rsT  = parseFloat(box.style.top);
                rsW  = parseFloat(box.style.width);
                rsH  = parseFloat(box.style.height);
                document.body.style.userSelect = 'none';
            });

            rh.addEventListener('pointermove', e => {
                if (!rh.hasPointerCapture(e.pointerId)) return;
                const dx = e.clientX - rsX0, dy = e.clientY - rsY0;
                let l = rsL, t = rsT, w = rsW, h = rsH;

                if (def.edges.includes('e')) w = Math.max(30, rsW + dx);
                if (def.edges.includes('s')) h = Math.max(30, rsH + dy);
                if (def.edges.includes('w')) { w = Math.max(30, rsW - dx); l = rsL + rsW - w; }
                if (def.edges.includes('n')) { h = Math.max(30, rsH - dy); t = rsT + rsH - h; }

                box.style.left   = l + 'px';
                box.style.top    = t + 'px';
                box.style.width  = w + 'px';
                box.style.height = h + 'px';
                updateLabels();
            });

            rh.addEventListener('pointerup', () => {
                document.body.style.userSelect = '';
            });
        });
    }

    initBox();
})();
