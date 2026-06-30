audio_samples_qoe Documentation
===============================

**audio-samples-qoe** computes **ViSQOL** (Virtual Speech Quality Objective
Listener), a full-reference perceptual audio quality metric, from Python. The
scoring core is pure Rust.

Given a clean **reference** signal and a **degraded** signal (codec artefacts,
bandwidth limiting, added noise), it returns a **MOS-LQO** (mean opinion score,
listening quality objective) in the range **1.0 to 5.0**. Higher is better.

The implementation is conformance-tested against Google's C++ reference in
both audio and speech mode. The GIL is released during computation, so scoring
parallelises across Python threads.

Features
--------

- **Two modes**: full-band ``audio`` (32 gammatone bands, SVR mapping) and
  ``speech`` (21 bands ≤ 8 kHz, voice-activity-gated, exponential NSIM fit).
- **Flexible inputs**: an :class:`audio_samples.AudioSamples` object or a path
  to a WAV/FLAC file, in any combination. File decoding is handled by the
  native Rust reader, so no Python audio library is required.
- **Any rate, any channel count**: signals are resampled and mixed to mono
  internally to match the reference pipeline.
- **Typed**: ships full type stubs and a ``py.typed`` marker.

Quick Example
-------------

.. code-block:: python

   from audio_samples_qoe import visqol

   # Score two files. No Python audio I/O library needed.
   score = visqol("reference.wav", "degraded.wav")
   print(f"MOS-LQO: {score:.3f}")

.. code-block:: python

   from audio_samples import sine_wave, white_noise
   from audio_samples_qoe import visqol

   ref = sine_wave(440, 5.0, 48_000)
   deg = ref + white_noise(5.0, 48_000, amplitude=0.02, seed=0)

   score = visqol(ref, deg)            # audio mode (default)
   score = visqol(ref, deg, mode="speech")

Installation
------------

.. code-block:: bash

   pip install audio-samples-qoe

Contents
--------

.. toctree::
   :maxdepth: 2
   :caption: User Guide

   guide/installation
   guide/quickstart
   guide/inputs
   guide/modes
   guide/errors

.. toctree::
   :maxdepth: 2
   :caption: API Reference

   api/index

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
