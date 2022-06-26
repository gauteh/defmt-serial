#! /usr/bin/env bash

set -ep

$1 | defmt-print -e $1
