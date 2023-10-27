#!/usr/bin/env bash

URL='https://crates.io/api/v1/keywords?sort=alpha&per_page=100'
PAGES='[1-275]'

curl -sS --compressed "${URL}&page=$PAGES" |
    jq -sr 'map(.keywords) | flatten | map(.keyword + "    count=" + (.crates_cnt | tostring))[]'
