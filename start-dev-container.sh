#!/bin/bash

devcontainer up --mount "type=bind,source=$HOME/.config/nvim,target=/home/vscode/.config/nvim" --workspace-folder .
devcontainer exec nvim
