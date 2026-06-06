// AudioWorkletProcessor: acumula los quantums de 128 muestras en lotes y los
// postea al hilo principal como Float32 (transferible, zero-copy).
const BATCH_FRAMES = 1600; // ~100 ms @ 16 kHz

class PcmCaptureProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this._buf = new Float32Array(BATCH_FRAMES);
    this._pos = 0;
  }
  process(inputs) {
    const ch = inputs[0] && inputs[0][0];
    if (!ch || ch.length === 0) return true;
    let i = 0;
    while (i < ch.length) {
      const room = BATCH_FRAMES - this._pos;
      const n = Math.min(room, ch.length - i);
      this._buf.set(ch.subarray(i, i + n), this._pos);
      this._pos += n;
      i += n;
      if (this._pos >= BATCH_FRAMES) {
        const b = this._buf.buffer;
        this.port.postMessage({ pcm: b }, [b]);
        this._buf = new Float32Array(BATCH_FRAMES);
        this._pos = 0;
      }
    }
    return true;
  }
}
registerProcessor('pcm-capture', PcmCaptureProcessor);
