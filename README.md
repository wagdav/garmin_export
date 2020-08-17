# garmin_export
Export FIT files from connect.garmin.com

**Work in progress**

## Quickstart

To build:

```
nix develop --command cargo build
```

To run:

```
nix develop --command cargo run <USERNAME> <PASSWORD>
```

## Acknowledgement

I read the Python source code of Peter Gardfjäll's
[garminexport](https://github.com/petergardfjall/garminexport) project to
understand the idiosyncratic API of Garmin Connect.
