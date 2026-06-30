import sys
from pathlib import Path

# Make the installed package importable (works after `maturin develop`).
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

project = "audio-samples-qoe"
author = "Jack Geraghty"
copyright = f"2025, {author}"
release = "0.1.0"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "sphinx.ext.intersphinx",
    "sphinx_autodoc_typehints",
    "sphinx_copybutton",
    "myst_parser",
]

# -- autodoc ------------------------------------------------------------------
autodoc_typehints = "description"
autodoc_typehints_format = "short"
autodoc_member_order = "bysource"
autodoc_default_options = {
    "members": True,
    "show-inheritance": True,
    "undoc-members": False,
}

# -- napoleon (Google-style docstrings) ---------------------------------------
napoleon_google_docstring = True
napoleon_numpy_docstring = False
napoleon_use_param = True
napoleon_use_rtype = False

# -- intersphinx --------------------------------------------------------------
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "numpy": ("https://numpy.org/doc/stable", None),
}

# -- MyST ---------------------------------------------------------------------
myst_enable_extensions = ["colon_fence"]

# -- HTML output --------------------------------------------------------------
html_theme = "furo"
html_title = "audio-samples-qoe"
html_static_path = ["_static"]
html_theme_options = {
    "sidebar_hide_name": False,
    "navigation_with_keys": True,
}

# -- copy-button --------------------------------------------------------------
copybutton_prompt_text = r">>> |\.\.\. |\$ "
copybutton_prompt_is_regexp = True
