#!/usr/bin/env bash

pwd

for seed in 17 42 4711
do
    for size in 5 10 15 20
    do
        for tiles in 5 10 15 20
        do
            ./target/release/minizinc-pentominoes-generator --seed $seed --size $size --tiles $tiles > ./data/size_${size}_tiles_${tiles}_seed_$seed.dzn
        done
    done
done
