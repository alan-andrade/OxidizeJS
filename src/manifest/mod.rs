// Manifest
//
// This library will look for "manifesto.json" and dig into the
// "manifest" key. The value should be an array of files.
//
// This is an example of how the manifest.json should look like.
//
// "manifest": [
//      "js/foo.js",
//      "js/bar.js",
//      "js/*.js"
// ]
pub use manifest::manifest::with_options;

pub mod paths;
pub mod manifest;
