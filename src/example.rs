pub const TYPESCRIPT_SOURCE_CODE: &str = r#"
export interface NoteEvent {
    noteNumber: number; // MIDI note (0-127)
    startTime: number;  // Seconds
    duration: number;   // Seconds
    velocity: number;   // 0.0 to 1.0
}

export interface SynthPatch {
    cutoffFreq: number;
    resonance: number;
    drive: number;
    attack: number;
    decay: number;
    sustain: number;
    release: number;
}

// 1. Core DSP Math Utilities
function midiToFreq(note: number): number {
    return 440 * Math.pow(2, (note - 69) / 12);
}

const generateOscillatorSample = (
    freq: number,
    time: number,
    type: "sine" | "saw" | "square"
): number => {
    const phase = 2 * Math.PI * freq * time;
    switch (type) {
        case "sine": return Math.sin(phase);
        case "saw": return 2 * (phase / (2 * Math.PI) - Math.floor(0.5 + phase / (2 * Math.PI)));
        case "square": return Math.sin(phase) >= 0 ? 1 : -1;
    }
};

const calculateAdsrGain = (
    time: number,
    duration: number,
    a: number,
    d: number,
    s: number,
    r: number
): number => {
    if (time < a) return time / a;
    if (time < a + d) return 1.0 - (1.0 - s) * ((time - a) / d);
    if (time < duration) return s;
    if (time < duration + r) return s * (1.0 - (time - duration) / r);
    return 0.0;
};

// 2. Non-Linear Wave Shaping & Filtering
function applySoftClipping(sample: number, drive: number): number {
    const driven = sample * (1 + drive);
    return Math.tanh(driven); // Hyperbolic tangent saturator
}

const applyLowPassFilter = (
    buffer: Float32Array,
    cutoff: number,
    sampleRate: number
): Float32Array => {
    const rc = 1.0 / (2 * Math.PI * cutoff);
    const dt = 1.0 / sampleRate;
    const alpha = dt / (rc + dt);

    const output = new Float32Array(buffer.length);
    let previousSample = 0;

    for (let i = 0; i < buffer.length; i++) {
        output[i] = previousSample + alpha * (buffer[i] - previousSample);
        previousSample = output[i];
    }
    return output;
};

// 3. Voice Synthesis Engine
export function synthesizeVoice(
    note: NoteEvent,
    patch: SynthPatch,
    sampleRate: number = 44100
): Float32Array {
    const freq = midiToFreq(note.noteNumber);
    const totalSamples = Math.ceil((note.duration + patch.release) * sampleRate);
    const buffer = new Float32Array(totalSamples);

    for (let i = 0; i < totalSamples; i++) {
        const time = i / sampleRate;

        // Multi-oscillator mix (Detuned Saw + Sine sub)
        const osc1 = generateOscillatorSample(freq, time, "saw");
        const osc2 = generateOscillatorSample(freq * 1.005, time, "saw"); // Detuned
        const sub = generateOscillatorSample(freq / 2, time, "sine") * 0.5; // Sub-octave

        const rawMix = (osc1 + osc2 + sub) / 2.5;
        const envelope = calculateAdsrGain(
            time,
            note.duration,
            patch.attack,
            patch.decay,
            patch.sustain,
            patch.release
        );

        buffer[i] = rawMix * envelope * note.velocity;
    }

    return buffer;
}

// 4. Effects Processor & Master Mix
function processMasterEffects(
    rawBuffer: Float32Array,
    patch: SynthPatch,
    sampleRate: number
): Float32Array {
    const filtered = applyLowPassFilter(rawBuffer, patch.cutoffFreq, sampleRate);

    // Apply saturation per sample
    for (let i = 0; i < filtered.length; i++) {
        filtered[i] = applySoftClipping(filtered[i], patch.drive);
    }
    return filtered;
}

export function renderSequence(
    notes: NoteEvent[],
    patch: SynthPatch,
    sampleRate: number = 44100
): Float32Array {
    const maxEndTime = Math.max(...notes.map(n => n.startTime + n.duration + patch.release));
    const masterBuffer = new Float32Array(Math.ceil(maxEndTime * sampleRate));

    for (const note of notes) {
        const voiceBuffer = synthesizeVoice(note, patch, sampleRate);
        const startSample = Math.floor(note.startTime * sampleRate);

        // Mix voice into master timeline
        for (let i = 0; i < voiceBuffer.length; i++) {
            if (startSample + i < masterBuffer.length) {
                masterBuffer[startSample + i] += voiceBuffer[i];
            }
        }
    }

    return processMasterEffects(masterBuffer, patch, sampleRate);
}
"#;
