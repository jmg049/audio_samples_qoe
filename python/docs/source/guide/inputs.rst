Input Types
===========

:func:`audio_samples_qoe.visqol` accepts two kinds of input for each signal
argument: a file path or an in-memory :class:`audio_samples.AudioSamples`. Any
combination across the two arguments is valid.

File Paths
----------

Pass a ``str`` or any ``os.PathLike`` (including ``pathlib.Path``). The file is
decoded by the native Rust reader, so no Python audio library is needed.

.. code-block:: python

   from pathlib import Path
   from audio_samples_qoe import visqol

   score = visqol("reference.wav", Path("degraded.flac"))

**Supported formats**: WAV (PCM 16/24/32-bit, float) and FLAC.

AudioSamples
------------

In-memory signals are passed as :class:`audio_samples.AudioSamples`. A signal
produced by an ``audio_samples`` generator or loader is already an
``AudioSamples`` and can be passed straight in:

.. code-block:: python

   from audio_samples import sine_wave
   from audio_samples_qoe import visqol

   ref = sine_wave(440, 5.0, 48_000)
   score = visqol(ref, "degraded.wav")

To use a raw NumPy array you already hold, attach a sample rate with
``new_mono`` (1-D) or ``new_multi`` (2-D, shape ``(channels, samples)``):

.. code-block:: python

   from audio_samples import AudioSamples

   signal = AudioSamples.new_mono(samples, 48_000)
   signal = AudioSamples.new_multi(channels, 48_000)

Any NumPy integer or float dtype is accepted. Integer formats (``int16``,
``int32``, ...) are normalised to ``[-1, 1]`` internally before scoring.

Sample Rates
------------

Signals at any sample rate are accepted. In **audio mode** both signals are
resampled to 48 kHz internally. In **speech mode** the reference runs at its
native rate and the degraded signal is resampled to match. Providing 16 kHz
speech input directly is recommended.

.. code-block:: python

   ref = AudioSamples.new_mono(ref_arr, 48_000)
   deg = AudioSamples.new_mono(deg_arr, 16_000)   # resampled to 48 kHz in audio mode
   score = visqol(ref, deg)

Channel Count
-------------

Any channel count is accepted. All signals are mixed down to mono by averaging
the channels before scoring.

What Is Not Accepted
--------------------

Raw NumPy arrays without a sample rate are rejected with a ``TypeError`` that
explains how to fix it:

.. code-block:: python

   import numpy as np
   arr = np.zeros(48_000 * 5)
   visqol(arr, arr)
   # TypeError: reference must be an audio_samples.AudioSamples, str, or
   # os.PathLike, got ndarray; wrap raw sample arrays with
   # audio_samples.AudioSamples.new_mono(arr, sample_rate) or new_multi

The sample rate is meaningful to the metric, so it must be attached
explicitly rather than guessed.
