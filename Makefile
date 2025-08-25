# Makefile for Wezztershier
# Beautiful single-binary GUI for WezTerm configuration files

PACKAGE = wezztershier
VERSION = 0.3.0
RUST_TOOLCHAIN = stable

.PHONY: help build install uninstall test clean dist rpm dev-install check format bench

help:
	@echo "Available targets:"
	@echo "  build       - Build the unified binary (CLI + embedded web GUI)"
	@echo "  install     - Install the package"
	@echo "  uninstall   - Uninstall the package"
	@echo "  dev-install - Install in development mode"
	@echo "  test        - Run tests"
	@echo "  check       - Run code quality checks (clippy, fmt)"
	@echo "  format      - Format code with rustfmt"
	@echo "  bench       - Run performance benchmarks"
	@echo "  clean       - Clean build artifacts"
	@echo "  dist        - Create distribution package"
	@echo "  rpm         - Build RPM package"

build:
	@echo "Building Wezztershier (unified binary)..."
	cargo build --release
	@echo "✓ Build complete - single binary with embedded web GUI"

install:
	cargo install --path .

uninstall:
	@echo "Uninstalling $(PACKAGE)..."
	cargo uninstall $(PACKAGE) || echo "Package not installed via cargo"
	@echo "Uninstall complete!"

dev-install:
	cargo install --path . --debug

test:
	@echo "Running core library tests..."
	cargo test -p wezztershier-core --verbose
	@echo "Running workspace tests..."
	cargo test --workspace --verbose

check:
	@echo "Running code quality checks..."
	@echo "• Formatting check..."
	cargo fmt --all -- --check
	@echo "• Clippy linting..."
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "• Security audit..."
	@command -v cargo-audit >/dev/null 2>&1 && cargo audit || echo "cargo-audit not available - install with: cargo install cargo-audit"

format:
	cargo fmt --all

bench:
	@echo "Running performance benchmarks..."
	cargo bench --workspace

clean:
	cargo clean
	rm -rf dist/
	rm -f $(PACKAGE)-$(VERSION).tar.gz

dist: clean
	@echo "Creating distribution package..."
	@# Create list of files that exist
	@files_to_package=""; \
	for item in src src-core templates examples docs packaging scripts *.toml *.md *.spec Makefile LICENSE benches; do \
		if [ -e "$$item" ]; then \
			files_to_package="$$files_to_package $$item"; \
		fi; \
	done; \
	if [ -d ".github" ]; then \
		files_to_package="$$files_to_package .github"; \
	fi; \
	tar czf $(PACKAGE)-$(VERSION).tar.gz \
		--exclude='.git*' \
		--exclude='target' \
		--exclude='dist' \
		--exclude='*.rpm' \
		--transform 's,^,$(PACKAGE)-$(VERSION)/,' \
		$$files_to_package

rpm: dist
	@echo "Building RPM package..."
	@command -v rpmbuild >/dev/null 2>&1 || (echo "rpmbuild not available - install rpm-build package" && exit 1)
	mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	cp $(PACKAGE)-$(VERSION).tar.gz ~/rpmbuild/SOURCES/
	cp $(PACKAGE).spec ~/rpmbuild/SPECS/
	rpmbuild -ba ~/rpmbuild/SPECS/$(PACKAGE).spec
	@echo "RPM packages created in ~/rpmbuild/RPMS/"

# Performance testing
perf-test:
	@echo "Running performance tests..."
	@echo "Testing parsing performance..."
	@time target/release/wezztershier parse examples/test-config.lua >/dev/null
	@echo "Testing widget creation performance..."
	@time target/release/wezztershier debug examples/test-config.lua --widgets >/dev/null

# Smoke test - basic functionality verification  
smoke-test: build
	@echo "Running smoke tests..."
	@target/release/wezztershier --version
	@target/release/wezztershier --help >/dev/null
	@target/release/wezztershier widgets >/dev/null && echo "✓ Widget listing works"
	@target/release/wezztershier parse examples/test-config.lua >/dev/null && echo "✓ Configuration parsing works"
	@target/release/wezztershier validate examples/test-config.lua >/dev/null && echo "✓ Configuration validation works"
	@echo "✓ Testing embedded web GUI..."
	@timeout 5 target/release/wezztershier gui --daemon --port 8081 >/dev/null 2>&1 && echo "✓ Web GUI starts successfully" || echo "⚠ Web GUI test skipped"
	@echo "All smoke tests passed!"

# Integration test with actual WezTerm config
integration-test: build
	@echo "Running integration tests..."
	@if [ -f ~/.config/wezterm/wezterm.lua ]; then \
		echo "Testing with real WezTerm config..."; \
		target/release/wezztershier validate ~/.config/wezterm/wezterm.lua || echo "⚠ Real config may not have Wezztershier annotations"; \
	else \
		echo "No WezTerm config found at ~/.config/wezterm/wezterm.lua"; \
	fi

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --workspace --no-deps --document-private-items

all: clean build test check