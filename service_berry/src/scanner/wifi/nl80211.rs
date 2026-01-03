//! Low-level Wi-Fi scanning via nl80211 netlink interface.

use super::types::{AkmSuite, ChannelInfo, CipherSuite, PhyType, Security, WifiBssid, WpaVersion};
use btleplug::api::BDAddr as mac_address;
use futures::TryStreamExt;
use std::{collections::HashSet, error::Error, time::Duration};
use tracing::error;
use tracing::{debug, info};
use wl_nl80211::{
    Nl80211Attr, Nl80211BssInfo, Nl80211Element, Nl80211ExtFeature, Nl80211Scan, Nl80211ScanFlags,
    new_connection,
};

const SCAN_RETRY_DELAY: Duration = Duration::from_millis(300);
const SCAN_MAX_RETRIES: u32 = 2;
const SCAN_DURATION_MS: u64 = 5000; // Wait for scan to complete
const SCAN_PASSES: u8 = 1; // Multiple passes catch more networks randomly
const SCAN_DWELL_TU: u16 = 300; // Per-channel dwell time (TU = 1024µs), 100 - 300 is a good value

/// Scan for Wi-Fi networks (multiple passes, flushes cache for fresh results).
pub async fn scan(ifindex: u32) -> Result<Vec<WifiBssid>, Box<dyn Error + Send + Sync>> {
    let (conn, handle, _) = new_connection()?; // start nl80211 connection
    tokio::spawn(conn);

    // See if high_accuracy is supported
    let supports_high_accuracy = driver_feat(ifindex)
        .await?
        .iter()
        .any(|f| matches!(f, Nl80211ExtFeature::HighAccuracyScan));

    debug!(
        "[info] HIGH_ACCURACY scan: {}",
        if supports_high_accuracy {
            "supported"
        } else {
            "not supported"
        }
    );

    let mut all_networks = Vec::new();
    let mut seen: HashSet<([u8; 6], u32)> = HashSet::new();

    for pass in 1..=SCAN_PASSES {
        match trigger_scan(&handle, ifindex, supports_high_accuracy).await {
            Ok(()) => tokio::time::sleep(Duration::from_millis(SCAN_DURATION_MS)).await,
            Err(e) if e.to_string().contains("permission") => return Err(e),
            Err(e) => {
                // Log but continue - we can still dump cached results
                debug!(pass, error = %e, "scan trigger failed, using cached results");
                tokio::time::sleep(Duration::from_millis(SCAN_DURATION_MS / 2)).await;
            }
        }

        for net in dump_scan(&handle, ifindex).await? {
            if seen.insert((net.bssid.into_inner(), net.channel_info.frequency_mhz)) {
                all_networks.push(net);
            }
        }
        debug!(pass, count = all_networks.len(), "pass complete");
    }

    info!(ifindex, count = all_networks.len(), "scan complete");
    Ok(all_networks)
}

/// Print all driver extended features for diagnostics
pub async fn driver_feat(
    ifindex: u32,
) -> Result<Vec<Nl80211ExtFeature>, Box<dyn Error + Send + Sync>> {
    let (conn, handle, _) = new_connection()?;
    tokio::spawn(conn);

    println!(
        "Querying driver extended features for ifindex {}...\n",
        ifindex
    );

    let mut stream = handle.wireless_physic().get().execute().await;
    while let Ok(Some(msg)) = stream.try_next().await {
        for attribute in &msg.payload.attributes {
            // for all attributes
            if let Nl80211Attr::ExtFeatures(ext_features) = attribute {
                // find ExtFeatures attribute line
                debug!("\nAll Extended Features: {:?}", ext_features);
                return Ok(ext_features.clone());
            }
        }
    }

    error!("No extended features found (driver may not advertise them)");
    Err("no extended features found".into())
}

async fn trigger_scan(
    handle: &wl_nl80211::Nl80211Handle,
    ifindex: u32,
    use_high_accuracy: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut flags = Nl80211ScanFlags::Flush;
    if use_high_accuracy {
        flags |= Nl80211ScanFlags::HighAccuracy;
        info!("using HIGH_ACCURACY scan");
    }

    let attrs = Nl80211Scan::new(ifindex)
        .scan_flags(flags)
        .duration(SCAN_DWELL_TU)
        .passive(false)
        .build();

    for attempt in 1..=SCAN_MAX_RETRIES {
        let mut stream = handle.scan().trigger(attrs.clone()).execute().await;
        let mut ok = true;

        loop {
            match stream.try_next().await {
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(e) => {
                    let s = e.to_string();
                    if s.contains("EBUSY") || s.contains("os error 16") {
                        ok = false;
                        break;
                    }
                    if s.contains("EPERM") || s.contains("os error 1") {
                        return Err("permission denied (need CAP_NET_ADMIN)".into());
                    }
                    if s.contains("ENETDOWN") || s.contains("os error 100") {
                        return Err("interface is down".into());
                    }
                    return Err(s.into());
                }
            }
        }

        if ok {
            return Ok(());
        }
        tokio::time::sleep(SCAN_RETRY_DELAY).await;
    }

    Err("scan busy after max retries".into())
}

async fn dump_scan(
    handle: &wl_nl80211::Nl80211Handle,
    ifindex: u32,
) -> Result<Vec<WifiBssid>, Box<dyn Error + Send + Sync>> {
    let mut stream = handle.scan().dump(ifindex).execute().await;
    let mut networks = Vec::new();

    while let Some(msg) = stream.try_next().await.map_err(|e| e.to_string())? {
        for attr in &msg.payload.attributes {
            if let Nl80211Attr::Bss(bss) = attr {
                if let Some(net) = parse_bss(bss) {
                    networks.push(net);
                }
            }
        }
    }
    Ok(networks)
}

fn parse_bss(attrs: &[Nl80211BssInfo]) -> Option<WifiBssid> {
    let (
        mut bssid,
        mut frequency_mhz,
        mut signal_dbm,
        mut last_seen_ms,
        mut channel_width_mhz,
        mut capability,
    ) = (None, None, None, None, None, None);
    let (mut ies, mut beacon_ies): (Option<&Vec<Nl80211Element>>, _) = (None, None);
    let (mut ssid, mut ht, mut vht, mut he, mut eht, mut has_rsn, mut has_wpa) =
        (None, false, false, false, false, false, false);
    let (mut group_cipher, mut pairwise_ciphers, mut akm_suites) = (None, Vec::new(), Vec::new());

    for attr in attrs {
        match attr {
            Nl80211BssInfo::Bssid(b) => bssid = Some(*b),
            Nl80211BssInfo::Frequency(f) => frequency_mhz = Some(*f),
            Nl80211BssInfo::SignalMbm(s) => signal_dbm = Some(*s / 100),
            Nl80211BssInfo::SeenMsAgo(s) => last_seen_ms = Some(*s),
            Nl80211BssInfo::Capability(c) => capability = Some(c.bits()),
            Nl80211BssInfo::ChanWidth(w) => {
                channel_width_mhz = Some([20, 10, 5].get(*w as usize).copied().unwrap_or(20))
            }
            Nl80211BssInfo::InformationElements(d) => ies = Some(d),
            Nl80211BssInfo::BeaconInformationElements(d) => beacon_ies = Some(d),
            _ => {}
        }
    }

    let (bssid, frequency_mhz) = (bssid?, frequency_mhz?);
    let elems = match (ies, beacon_ies) {
        (Some(r), Some(b)) if b.len() > r.len() => b,
        (Some(r), _) => r,
        (None, Some(b)) => b,
        _ => return None,
    };

    for e in elems {
        match e {
            Nl80211Element::Ssid(s) if !s.is_empty() => ssid = Some(s.clone()),
            Nl80211Element::HtCapability(_) => ht = true,
            Nl80211Element::Rsn(r) => {
                has_rsn = true;
                group_cipher = r
                    .group_cipher
                    .as_ref()
                    .map(|gc| rsn_cipher(&format!("{gc:?}")));
                pairwise_ciphers.extend(
                    r.pairwise_ciphers
                        .iter()
                        .map(|c| rsn_cipher(&format!("{c:?}"))),
                );
                akm_suites.extend(r.akm_suits.iter().map(|a| akm(&format!("{a:?}"))));
            }
            Nl80211Element::Vendor(d) if d.get(..4) == Some(&[0x00, 0x50, 0xF2, 0x01]) => {
                has_wpa = true
            }
            Nl80211Element::Other(id, d) => match *id {
                45 => ht = true,
                61 if d.len() >= 2 => {
                    ht = true;
                    channel_width_mhz.get_or_insert(if d[1] & 0x03 != 0 { 40 } else { 20 });
                }
                191 => vht = true,
                192 if !d.is_empty() => {
                    vht = true;
                    channel_width_mhz =
                        Some([40, 80, 160, 160].get(d[0] as usize).copied().unwrap_or(40));
                }
                255 if !d.is_empty() => match d[0] {
                    35 | 36 | 106 => he = true,
                    108 => eht = true,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    // PHY types (newest first)
    let mut phy: Vec<_> = [
        (eht, PhyType::Eht),
        (he, PhyType::He),
        (vht, PhyType::Vht),
        (ht, PhyType::Ht),
    ]
    .into_iter()
    .filter_map(|(b, p)| b.then_some(p))
    .collect();
    if phy.is_empty() {
        phy.push(PhyType::Legacy);
    }

    let version = if akm_suites
        .iter()
        .any(|a| matches!(a, AkmSuite::Sae | AkmSuite::FtSae))
    {
        Some(WpaVersion::Wpa3)
    } else if has_rsn {
        Some(WpaVersion::Wpa2)
    } else if has_wpa {
        Some(WpaVersion::Wpa)
    } else {
        None
    };

    let caps: Vec<_> = capability
        .map(|c| {
            [
                (0x0001, "ESS"),
                (0x0002, "IBSS"),
                (0x0010, "Privacy"),
                (0x0020, "ShortPreamble"),
                (0x0400, "ShortSlotTime"),
            ]
            .into_iter()
            .filter_map(|(m, n)| (c & m != 0).then(|| n.into()))
            .collect()
        })
        .unwrap_or_default();

    Some(WifiBssid {
        ssid,
        bssid: mac_address::from(bssid),
        age: last_seen_ms.map(u64::from),
        channel_info: ChannelInfo {
            frequency_mhz,
            channel: freq_to_channel(frequency_mhz),
            bandwidth_mhz: channel_width_mhz,
        },
        phy,
        rssi: signal_dbm.unwrap_or(-100),
        wifi_security: Security {
            version,
            group_cipher,
            pairwise_ciphers,
            auth_suites: akm_suites,
            mfp: None,
        },
        capabilities: caps,
    })
}

fn akm(s: &str) -> AkmSuite {
    match s {
        "Psk" => AkmSuite::Psk,
        "Sae" => AkmSuite::Sae,
        "Ieee8021x" => AkmSuite::Ieee8021x,
        "PskSha256" => AkmSuite::PskSha256,
        "Ieee8021xSha256" => AkmSuite::Ieee8021xSha256,
        "FtPsk" => AkmSuite::FtPsk,
        "FtSae" => AkmSuite::FtSae,
        "FtIeee8021x" => AkmSuite::FtIeee8021x,
        "SuiteB" => AkmSuite::SuiteB,
        "SuiteB192" => AkmSuite::SuiteB192,
        _ => AkmSuite::Other,
    }
}

fn rsn_cipher(s: &str) -> CipherSuite {
    match s {
        "UseGroup" => CipherSuite::UseGroup,
        "Wep40" => CipherSuite::Wep40,
        "Tkip" => CipherSuite::Tkip,
        "Ccmp128" => CipherSuite::Ccmp,
        "Wep104" => CipherSuite::Wep104,
        "BipCmac128" => CipherSuite::AesCmac,
        "Gcmp128" => CipherSuite::Gcmp,
        "Gcmp256" => CipherSuite::Gcmp256,
        "Ccmp256" => CipherSuite::Ccmp256,
        "BipGmac128" => CipherSuite::BipGmac128,
        "BipGmac256" => CipherSuite::BipGmac256,
        "BipCmac256" => CipherSuite::BipCmac256,
        _ => CipherSuite::Other(0),
    }
}

fn freq_to_channel(f: u32) -> Option<u8> {
    match f {
        2412..=2472 => Some(((f - 2407) / 5) as u8),
        2484 => Some(14),
        5170..=5330 | 5490..=5895 => Some(((f - 5000) / 5) as u8),
        5935 => Some(2),
        5955..=7115 => Some(((f - 5950) / 5) as u8),
        _ => None,
    }
}
