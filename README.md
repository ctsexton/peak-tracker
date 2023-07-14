# Peak Tracker

Attempts to identify strongest frequency partials from an incoming audio stream and resynthesize them with oscillators. Inspired by Miller Puckette's peak tracking mode in the Pd/Max ~sigmund object.

To build LV2:
```
cd lv2
cargo +nightly build --release 
```

To build CLAP/VST3:
```
cd nih
cargo +nightly xtask bundle peak_tracker_nih --release
```
