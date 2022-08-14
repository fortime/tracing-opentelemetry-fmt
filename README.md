# Introduction
Implement a `Layer` of `tracing-subscriber` which combines a `OpentelmetryLayer` and `FmtLayer`. This layer adds `trace.id` and `span.id` to `FmtLayer` so that we can log `trace.id` and `span.id` from opentelemetry.
