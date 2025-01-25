cargo build --profile wasm-release --target wasm32-unknown-unknown && \
wasm-bindgen --out-dir ./webapp/ --target web --no-typescript ./target/wasm32-unknown-unknown/wasm-release/rubiks_cube.wasm
cd webapp && \
wasm-opt -Oz -o rubiks_cube_bg.wasm rubiks_cube_bg.wasm
