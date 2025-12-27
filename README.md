# Serviceberry
Serviceberry started as a project to improve the accuracy & coverage of geolocation databases, it does this by submitting anonyomous sensor & location data to a configured geolocation service. Serviceberry is designed to run on your mobile IOS device, while running as a desktop app on a local computer or laptop, using either Bluetooth or Wifi connectivity. 

Currently, Serviceberry is only guaranteed to support [BeaconDB](https://beacondb.net/) and Linux machines. See the todo for the progress.

##  TODO
*   [x] Add TLS encryption
*   [ ] Add support for Bluetooth connectivity
*   [ ] Create IOS Mobile App
*   [ ] Build Tauri desktop app
*   [ ] Test support for other geolocation databases

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
