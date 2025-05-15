1. Set up `ufw`
    - Install it with your package manager
    - `ufw enable` as root
    - Start it as a service
    - `ufw deny in on tun0 proto ipv6` as root
2. Set up [yggdrasil](https://yggdrasil-network.github.io/documentation.html)
