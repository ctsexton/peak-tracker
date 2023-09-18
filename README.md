![linux-headless](https://github.com/ctsexton/peak-tracker/actions/workflows/linux-headless.yaml/badge.svg)
![mac-headless](https://github.com/ctsexton/peak-tracker/actions/workflows/mac-headless.yaml/badge.svg)
# Peak Tracker

Attempts to identify strongest frequency partials from an incoming audio stream and resynthesize them with oscillators. Inspired by Miller Puckette's peak tracking mode in the Pd/Max ~sigmund object.

### Demo Video
[![Peak Tracker Audio Plugin demo](http://img.youtube.com/vi/JVvRggFd4_g/0.jpg)](http://www.youtube.com/watch?v=JVvRggFd4_g)

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

To build Mac VST3 installer package:
```
make mac_installer
```
