#!/bin/bash

find -iname '*.rs' -exec rustfmt {} \;
find -iname '*.rs.bk' -delete
