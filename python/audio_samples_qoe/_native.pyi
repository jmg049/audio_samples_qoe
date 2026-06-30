"""Type stubs for the private extension module ``audio_samples_qoe._native``.

The public API is ``audio_samples_qoe.visqol`` — see the package
``__init__``. This module only carries the low-level scorer it dispatches
to.
"""

from os import PathLike
from typing import Literal

import numpy as np
import numpy.typing as npt

__version__: str

def _read(path: str | PathLike[str]) -> tuple[npt.NDArray[np.float64], int]:
    """Decode an audio file (WAV/FLAC) with the native Rust reader. Private —
    used by ``audio_samples_qoe.visqol`` to handle path arguments.

    Returns ``(samples, sample_rate)`` where ``samples`` is a float64 array,
    1-D for mono or 2-D ``(channels, samples)``, normalised to [-1, 1].
    The GIL is released while the file is decoded.
    """

class VisqolError(Exception):
    """Raised when the ViSQOL computation itself fails.

    This covers errors inside the metric pipeline — e.g. signals that are
    too short to extract a single analysis patch, resampling failures, or
    alignment failures. Invalid arguments raise ``ValueError`` or
    ``TypeError`` instead, and unreadable files raise ``OSError``.
    """

def _score(
    reference: npt.NDArray[np.float64],
    reference_rate: int,
    degraded: npt.NDArray[np.float64],
    degraded_rate: int,
    mode: Literal["audio", "speech"],
) -> float:
    """Score two raw sample buffers. Private — use ``audio_samples_qoe.visqol``.

    ``reference`` and ``degraded`` are float64 arrays, 1-D mono or 2-D
    ``(channels, samples)``, with their sample rates in Hz. ``mode`` is
    ``"audio"`` or ``"speech"``. Returns the MOS-LQO score in [1.0, 5.0].
    The GIL is released while the score is computed.
    """
