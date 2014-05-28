# Oxidize JS

A Rust library that compress Javascripts in parallel.

## How it works

1. Load Javascript files from `manifest.json`.
2. Split evenly the work among your cpus.
3. Output the result into one file.

## Inspiration

I want to speedup the JS asset compression process. This will help
people to deploy faster.

I started this project to teach myself some Rust and I hope it can
evolve into something really useful.

## Technical details

Not much really. I just fire up an `uglifyjs` process with different
chunk of files. The first pass, I don't mangle nor compress.

I uglify it once more after all children processes finish.

## Roadmap

I've heard that playing around with the Js AST might be faster. I'm
interested in that option but I still don't have the technical
expertise.
