# Payments Engine

This is a small toy project for processing payment transactions from a CSV file.

The project showcases the Rust programming language and best practices related to code quality and testing.

## Design

This diagram shows the high level components of the application. Obviously for this exercise we don't have a truly distributed system but the concepts are meant to show the start of a scalable design.

For the purposes of this toy application, the CLI and local filesystem will be the only implemented path through the system.

![Payments Engine](design.png)

* `CLI` is the main entry point of the application for the purposes of this toy project
* `Other Clients` is meant to show that other hypothetical clients can consume the ingestion api with different URI schemes 
* `Ingestion Service` exposes a URI based interface for submitting a payments CSV for processing
* Files can be read from either local filesystem or other storage mechanisms
* `Payments Queue` is a thread-safe mechanism to queue payments that are ready for processing
* `Payment Processor`'s are a pool of workers available to process payments and output results
