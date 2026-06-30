Installation
============

Requirements
------------

Python 3.10 or higher. ``pip`` installs the NumPy and
`audio_samples <https://pypi.org/project/audio-samples/>`_ dependencies
automatically.

Install from PyPI
-----------------

.. code-block:: bash

   pip install audio-samples-qoe

This pulls a pre-built wheel, so no Rust toolchain is needed.

Build from Source
-----------------

Building from source requires a Rust toolchain (1.87 or newer) and
`maturin <https://maturin.rs>`_:

.. code-block:: bash

   git clone https://github.com/jmg049/audio_samples_qoe
   cd audio_samples_qoe
   maturin develop --release

``maturin develop`` builds the extension and installs it into the active
virtual environment. The project is a single Rust crate; the Python bindings
are compiled behind its ``python`` feature, which maturin enables
automatically.
