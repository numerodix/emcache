#!/bin/bash

find -iname '*.rs' -exec rustfmt {} \;
