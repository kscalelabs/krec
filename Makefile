# Makefile

# ------------------------ #
#       Static Checks      #
# ------------------------ #

format:
	@black krec tests
	@ruff format krec tests
	@cargo fmt
.PHONY: format

static-checks:
	@black --diff --check krec tests
	@ruff check krec tests
	@mypy krec tests
.PHONY: lint

# ------------------------ #
#        Unit tests        #
# ------------------------ #

test:
	python -m pytest
.PHONY: test
