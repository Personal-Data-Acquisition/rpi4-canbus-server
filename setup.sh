#!/bin/sh

BCM_VER='1.75'
# Define the lines to be added
lines_to_add="dtparam=spi=on
dtoverlay=mcp2515-can1,oscillator=16000000,interrupt=25
dtoverlay=mcp2515-can0,oscillator=16000000,interrupt=23
dtoverlay=spi-bcm2835-overlay"

# Check if the lines are already present
if grep -Fxq "$lines_to_add" /boot/config.txt; then
  echo "Lines are already present in /boot/config.txt. Bringing up canbus."
  dmesg | grep spi
  sudo ip link set can0 up type can bitrate 1000000
  sudo ip link set can1 up type can bitrate 1000000
  sudo ifconfig can0 txqueuelen 65536
  sudo ifconfig can1 txqueuelen 65536
  ifconfig | grep can
  exit 0
fi

cd

# Append the lines to /boot/config.txt
echo -e "$lines_to_add" | sudo tee -a /boot/config.txt
echo "Lines added to /boot/config.txt successfully."

#https://www.waveshare.com/wiki/2-CH_CAN_HAT
wget http://www.airspayce.com/mikem/bcm2835/bcm2835-${BCM_VER}.tar.gz
tar zxvf bcm2835-${BCM_VER}.tar.gz 
cd bcm2835-${BCM_VER}/
sudo ./configure
sudo make
sudo make check
sudo make install


sudo apt-get install can-utils

# Allows on pi sessions and terminal multiplexing(useful for can testing)
sudo apt install tmux

echo "Install complete, rebooting to enable spi"
sleep 5
sudo reboot
