# Multi-reactor drifting

This project provides an integration between [tokio](https://tokio.rs/) and [rocket](https://rocket.rs/).

It should not be necessary in the long term since rocket [plans to have an async story](https://github.com/SergioBenitez/Rocket/blob/v0.3.6/README.md#future-improvements) itself in the future.

In the meanwhile, this crate provides the easiest way I know of to work with
tokio and/or futures as part of a rocket request.

See the [examples](examples) folder for what usage of this library looks like.

