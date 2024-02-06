#!/usr/bin/env bash

echo "generate asset.mol"
moleculec --language rust --schema-file ./schema/asset.mol | rustfmt > ./src/asset.rs

echo "generate charactors.mol"
moleculec --language rust --schema-file ./schema/charactors.mol | rustfmt > ./src/charactors.rs

echo "generate game.mol"
moleculec --language rust --schema-file ./schema/game.mol | rustfmt > ./src/game.rs

echo "generate resources.mol"
moleculec --language rust --schema-file ./schema/resources.mol | rustfmt > ./src/resources.rs

echo "generate scene.mol"
moleculec --language rust --schema-file ./schema/scene.mol | rustfmt > ./src/scene.rs

echo "generate types.mol"
moleculec --language rust --schema-file ./schema/types.mol | rustfmt > ./src/types.rs
