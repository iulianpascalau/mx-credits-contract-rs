#!/bin/bash

source ./config.cfg

mxpy contract call "${CONTRACT_ADDRESS}" --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=3000000 --function="changeNumCreditsPerEGLD" \
    --arguments "${NUM_CREDITS_PER_EGLD}" \
    --send --proxy="${PROXY}" --chain="${CHAIN_ID}"