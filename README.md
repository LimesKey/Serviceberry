# Serviceberry
The Serviceberry project provides fast, accurate location data to your system while enhancing the accuracy of public geolocation databases. It collects location information from your mobile device’s GNSS/GPS module and combines it with Wi-Fi and Bluetooth adapter data from a companion device. By default, Serviceberry contributes this data to BeaconDB, which anonymizes and obfuscates it to ensure privacy. Serviceberry operates as two interconnected components: the iOS app and the desktop app, which communicate via Wi-Fi or Bluetooth.

Currently, Serviceberry is only guaranteed to support Linux machines, iOS devices and contributing to [BeaconDB](https://beacondb.net/). See the todo below for current progress.

##  TODO
* [x] Add TLS encryption
* [x] Set up mDNS services
* [x] Support geo.provider browser requests
* [ ] Add Bluetooth connectivity
* [ ] Build iOS mobile app
* [ ] Build Tauri desktop app
* [ ] Ensure support for other geolocation databases

## System Requirements

### Linux Packages

Before running Serviceberry, install the required system dependencies:

**Debian/Ubuntu:**
```bash
sudo apt-get install pkg-config libdbus-1-dev bluez libbluetooth-dev wireless-tools iw wpasupplicant avahi-daemon openssl libssl-dev mkcert
```

**Fedora/RHEL:**
```bash
sudo dnf install pkgconf-pkg-config dbus-devel bluez bluez-libs-devel wireless-tools iw wpa_supplicant avahi avahi-tools openssl openssl-devel mkcert
```

**Arch Linux:**
```bash
sudo pacman -S pkgconf dbus bluez bluez-utils wireless_tools iw wpa_supplicant avahi openssl mkcert
```

### Enable Required Services

After installing packages, enable and start the necessary system services:

```bash
sudo systemctl enable --now bluetooth
sudo systemctl enable --now avahi-daemon
```

## Contributing

Come contribute now

### Quick Start

1. Ensure you have the latest stable version of [Rust](https://rust-lang.org/tools/install/) installed
2. Install all necessary system packages (see [System Requirements](#system-requirements))
3. Fork and clone the repository
4. Create a new branch for your changes
5. Make your changes following our coding standards
6. Run tests and linting: `cargo test && cargo fmt && cargo clippy`
7. Submit a pull request

For detailed contributing guidelines, please see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file. 
