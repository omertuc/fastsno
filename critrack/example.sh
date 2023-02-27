#!/bin/bash

export RELEASE_IMAGE=registry.build05.ci.openshift.org/ci-ln-dxn2lj2/release:latest

./critrack/dump.sh | ./critrack/component.sh | ./critrack/pulls.sh
