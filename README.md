# Serviceberry
The Serviceberry project aims to provide reliable, accurate location data to your system therefore improving the accuracy of public geolocation databases for everyone. It collects location data from your mobile phone's GNSS/GPS sensor, and combines it with Wi-Fi and Bluetooth adapter data from a companion computer. By default, Serviceberry contributes this data to BeaconDB, whom then anonymizes and obfuscates it to ensure privacy. Serviceberry operates as two interconnected components: the desktop program and the iOS app, which communicate to each other via Wi-Fi or Bluetooth.

Serviceberry responds to location requests at the hostname `https://serviceberry-<your-username>.local/request` for any browser implementing the [Google Maps Geolocation API](https://developers.google.com/maps/documentation/geolocation/requests-geolocation) request format. Before this can happen, Serviceberry registers two mDNS services `_serviceberry-https._tcp.local.` and `_serviceberry-http._tcp.local.`, at ports `8443` and `8080` respectively. In order to use HTTPS, you'll need to generate a self-signed certificate and private key pair, and import the certificate in your browser's Certificate Manager, authorizing the cert for use in identifying websites. Serviceberry is designed to submit to any database implementing the [MLS/Ichnaea API's Geosubmit V2 format](https://ichnaea.readthedocs.io/en/latest/api/geosubmit2.html), with [BeaconDB](https://beacondb.net/) being the default.

Currently, Serviceberry only offically supports Linux machines, iOS devices; and has only been confirmed to support [BeaconDB](https://beacondb.net/). See the todo for current progress.

  
## Prerequisites

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

#### Enable Required Services

After installing packages, enable and start the necessary system services:

```bash
sudo systemctl enable --now bluetooth
sudo systemctl enable --now avahi-daemon
```

## Step by Install Step Guide
1. Ensure you have installed the mobile IOS app with the correct permissions
2. Download the latest release
3. Install all the system packages
4. Set the `geo.provider.network.url` in `about:config` to `https://serviceberry-<your-username>.local/request`
5. Watch logs for sucessful location

## Contributing
For detailed contributing guidelines, please see [CONTRIBUTING.md](CONTRIBUTING.md).

### Checklist
1. Ensure you have the latest stable version of [Rust](https://rust-lang.org/tools/install/) installed
2. Install all necessary system packages (see [System Requirements](#system-requirements))
3. Fork and clone the repository
4. Create a new branch for your changes
5. Make your changes following our coding standards
6. Run tests and linting: `cargo test && cargo fmt && cargo clippy`
7. Submit a pull request
### Step by Step
#### 1. Install Rust for Linux, macOS or another Unix-like Os
- See [rust-lang.org](https://rust-lang.org/tools/install/) for additional help
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

##  TODO
* [x] Add TLS encryption
* [x] Set up mDNS services
* [x] Support geo.provider browser requests
* [ ] Verify support for Chromium-based browsers
* [ ] Add Bluetooth connectivity
* [ ] Build iOS mobile app
* [ ] Build Tauri desktop app
* [ ] Ensure support for other geolocation databases

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file. 
