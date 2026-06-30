SPHINXBUILD = uv run --group docs sphinx-build
SOURCEDIR   = .
BUILDDIR    = _build

.PHONY: help html clean livehtml

help:
	@$(SPHINXBUILD) -M help $(SOURCEDIR) $(BUILDDIR)

html:
	$(SPHINXBUILD) -b html $(SOURCEDIR) $(BUILDDIR)/html -W --keep-going
	@echo "Build finished. Open $(BUILDDIR)/html/index.html"

clean:
	rm -rf $(BUILDDIR)

# Live-reload server (requires sphinx-autobuild: uv add --group docs sphinx-autobuild)
livehtml:
	uv run --group docs sphinx-autobuild $(SOURCEDIR) $(BUILDDIR)/html
