![linux-headless](https://github.com/ctsexton/peak-tracker/actions/workflows/linux-headless.yaml/badge.svg)
# Peak Tracker

Attempts to identify strongest frequency partials from an incoming audio stream and resynthesize them with oscillators. Inspired by Miller Puckette's peak tracking mode in the Pd/Max ~sigmund object.

To build LV2:
```
make lv2
```

To build CLAP/VST3:
```
make clap_vst3
```

Build all:
```
make
```

Run tests:
```
make test
```

CLAP and VST3 plugins will be output to `target/bundled/peak_tracker_nih.<clap/vst3>`.
