# Serviceberry
The Serviceberry project aims to improve the accuracy & coverage of geolocation databases. It does this by submitting anonymized sensor & location data to a configured geolocation service. Serviceberry is designed to run in two interconnected parts: the Serviceberry IOS app and the Serviceberry desktop app. These two applications will communicate with each other via Wi-Fi or Bluetooth.

Currently, Serviceberry is only guaranteed to support [BeaconDB](https://beacondb.net/) and Linux machines. See the todo below for current progress.

##  TODO
*   [x] Add TLS encryption
*   [ ] Add support for Bluetooth connectivity
*   [ ] Create the IOS Mobile App
*   [ ] Build Tauri desktop app
*   [ ] Add support for other geolocation databases

## System Requirements

### Linux Packages

Before running Serviceberry, install the required system dependencies:

**Debian/Ubuntu:**
```bash
sudo apt-get install pkg-config libdbus-1-dev bluez libbluetooth-dev wireless-tools iw wpasupplicant avahi-daemon openssl libssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install pkgconf-pkg-config dbus-devel bluez bluez-libs-devel wireless-tools iw wpa_supplicant avahi avahi-tools openssl openssl-devel
```

**Arch Linux:**
```bash
sudo pacman -S pkgconf dbus bluez bluez-utils wireless_tools iw wpa_supplicant avahi openssl
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
