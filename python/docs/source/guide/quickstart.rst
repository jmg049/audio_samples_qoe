Quickstart
==========

Scoring Two Files
-----------------

The simplest usage: pass two file paths and read the score back.

.. code-block:: python

   from audio_samples_qoe import visqol

   score = visqol("reference.wav", "degraded.wav")
   print(f"MOS-LQO: {score:.3f}")   # e.g. 3.842

Files are decoded with the native Rust reader, so no Python audio library is
needed. WAV and FLAC are supported. Any sample rate and channel count is
accepted; the pipeline mixes down to mono and resamples internally.

Scoring In-Memory Signals
-------------------------

Build signals with the :mod:`audio_samples` generators. They return
:class:`audio_samples.AudioSamples` already carrying a sample rate, so you can
pass them straight in:

.. code-block:: python

   from audio_samples import sine_wave, white_noise
   from audio_samples_qoe import visqol

   RATE = 48_000

   ref = sine_wave(440, 5.0, RATE)
   deg = ref + white_noise(5.0, RATE, amplitude=0.02, seed=0)

   score = visqol(ref, deg)

Mixing Paths and In-Memory Signals
----------------------------------

The two arguments are independent, and any combination works:

.. code-block:: python

   from pathlib import Path

   score = visqol(Path("reference.wav"), deg)       # path + AudioSamples
   score = visqol(ref, "degraded.wav")              # AudioSamples + path
   score = visqol("reference.wav", "degraded.wav")  # both paths
   score = visqol(ref, deg)                         # both in-memory

Selecting a Mode
----------------

``audio`` mode (the default) is for full-bandwidth content; ``speech`` mode is
for narrowband and wideband voice:

.. code-block:: python

   score = visqol(ref, deg, mode="speech")

See :doc:`modes` for the differences between them.

Parallel Scoring
----------------

The GIL is released while ViSQOL computes, so many pairs can be scored
concurrently with a thread pool:

.. code-block:: python

   from concurrent.futures import ThreadPoolExecutor
   from audio_samples_qoe import visqol

   pairs = [("ref1.wav", "deg1.wav"), ("ref2.wav", "deg2.wav")]

   with ThreadPoolExecutor() as pool:
       scores = list(pool.map(lambda p: visqol(*p), pairs))
