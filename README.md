# TLSNotary proof visualizer

This is a Proof of Concept of a web app to visualized TLSNotary proofs.

This web app allows a user to upload TLSNotary proof (`proof.json`) and the app will check the validity and visualize the redacted parts.

You can use the deployed version via <https://tlsnotary.github.io/proof_viz/>

## Dependencies

This app is based on the [Yew](https://yew.rs/) [file upload example](https://github.com/yewstack/yew/tree/master/examples/file_upload). It is build with [Trunk](https://trunkrs.dev/)

This project requires a `clang` version newer than `16.0.0` to compile `ring` to `wasm`. If not, you will run into `warning: error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-unknown"'`

## Running

Run this application with the trunk development server:

```bash
trunk serve --open
```
