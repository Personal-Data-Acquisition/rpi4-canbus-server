#!/bin/sh
# Author: Jake G
# Date: 2024
# Filename: test_cam.sh

#Descripion: 
#show how can works on the pi by running a test of can1 --> can0


SPEED=1000000
QUEUE_LEN=65536

setup_can () {
    sudo ip link set can0 up type can bitrate ${SPEED}
    sudo ip link set can1 up type can bitrate ${SPEED}
    sudo ifconfig can0 txqueuelen ${QUEUE_LEN}
    sudo ifconfig can1 txqueuelen ${QUEUE_LEN}

    # Sometimes required
    sudo ip link set can0 up
    sudo ip link set can1 up
}

tear_down_can () {
    sudo ip link set can0 down
    sudo ip link set can1 down
}

print_info () {
    echo "##########################"
    echo "CAN TEST v0.0.0"
    echo "##########################"
    echo "For: SBC 2ch CAN"
    echo "SPEED: ${SPEED}"
    echo "QUEUE LENGTH: ${QUEUE_LEN}"
}

log_can () {
    candump -adex can0
}

send_can='
    i=1
    while [ "$i" -ne 26 ]
    do
        cansend can1 002#R0
        i=$((i + 1))
    done
'

spawn_tmux_session () {
    tmux new-session -d -s cantest
    tmux split-window -h
    tmux send-keys -t cantest:0.0 'candump -adex can0' C-m
    tmux send-keys -t cantest:0.1 "${send_can}" C-m
    tmux attach-session -t cantest 
}

main () {
    print_info
    sleep 2
    sleep 4 && tmux kill-session -t cantest&
    spawn_tmux_session
}

main
