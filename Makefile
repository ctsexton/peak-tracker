.PHONY: all lv2 clap_vst3 clean test mac_installer

all: lv2 clap_vst3

lv2:
	cargo +nightly build --release --manifest-path lv2/Cargo.toml

clap_vst3:
	cargo +nightly run --manifest-path nih/xtask/Cargo.toml --bin xtask bundle peak_tracker_nih --release

clean:
	cargo clean; \
	rm -rf ./PackageRoot ./PeakTracker.pkg ./PeakTrackerInstaller.pkg

test:
	cargo +nightly test --workspace

mac_installer: clap_vst3
	mkdir -p PackageRoot/Library/Audio/Plug-Ins/VST3/; \
	cp -R target/bundled/peak_tracker_nih.vst3 PackageRoot/Library/Audio/Plug-Ins/VST3/PeakTracker.vst3; \
	pkgbuild --root ./PackageRoot --identifier com.ctsexton.peak-tracker --version 0.1.0 --install-location / ./PeakTracker.pkg; \
	productbuild --package ./PeakTracker.pkg ./PeakTrackerInstaller.pkg

