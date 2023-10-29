default:
  just --list

# renders graphviz.dot file into ASCII graph
graph:
  graph-easy graphviz.dot # sudo apt install libgraph-easy-perl

build:
  cargo build

upgrade-bash-completion: build
  ./target/debug/dinasty generate-completion bash >/tmp/dinasty.bash
  sudo mv /tmp/dinasty.bash /usr/share/bash-completion/completions/

build-image:
  nix flake update github:bitcoin-dinasty/dinasty
  cd build-raspi4-image
  # export TMPDIR=/mnt/nvme/tmp
  nix build .#image.rpi4
  cd -