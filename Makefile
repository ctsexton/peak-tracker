.PHONY: all lv2 clap_vst3 clean

all: lv2 clap_vst3

lv2:
	cargo +nightly build --release --manifest-path lv2/Cargo.toml

clap_vst3:
	cargo +nightly run --manifest-path nih/xtask/Cargo.toml --bin xtask bundle peak_tracker_nih --release

clean:
	cargo clean

test:
	cargo +nightly test --workspace
