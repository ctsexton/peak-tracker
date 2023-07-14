#!/bin/bash

rm -rf ./target/release/reconstructor.lv2
cp -r src/lv2/ target/release/reconstructor.lv2
cp target/release/libreconstructor_lv2.so target/release/reconstructor.lv2/libreconstructor_lv2.so
rm -rf ~/.lv2/reconstructor.lv2
cp -r target/release/reconstructor.lv2 ~/.lv2/reconstructor.lv2
