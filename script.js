// const MDNS_ENDPOINT = "https://serviceberry-limeskey.local:8443/submit";
// const MDNS_ENDPOINT = "http://serviceberry-limeskey.local:8080/submit";
const MDNS_ENDPOINT = "http://192.168.0.251:8080/submit";

let logBuffer = "";

function log(msg) {
  console.log(msg);
  logBuffer += msg + "\n";
}

async function getPosition() {
  try {
    const loc = await Location.current();
    log("[Location] Successfully retrieved GPS location.");
    return {
      latitude: loc.latitude ?? 0,
      longitude: loc.longitude ?? 0,
      accuracy: loc.horizontalAccuracy ?? 0,
      altitude: loc.altitude ?? 0,
      altitudeAccuracy: loc.verticalAccuracy ?? 0,
      heading: loc.course ?? 0,
      speed: loc.speed ?? 0,
      source: "gnss",
    };
  } catch (e) {
    log("[Location] Failed: " + e.toString());
    throw e;
  }
}

// Main
async function main() {
  try {
    // Only gather device GPS location and send a Position-only payload
    const position = await getPosition();

    const payload = {
      position: {
        latitude: position.latitude,
        longitude: position.longitude,
        accuracy: position.accuracy,
        altitude: position.altitude,
        altitudeAccuracy: position.altitudeAccuracy,
        heading: position.heading,
        speed: position.speed,
        source: position.source,
      },
    };

    log("[Payload] Position-only payload:\n" + JSON.stringify(payload, null, 2));

    async function sendPayload(url, body) {
      log(`[POST] Sending payload to: ${url}`);
      let req = new Request(url);
      req.method = "POST";
      req.headers = { "Content-Type": "application/json" };
      req.body = JSON.stringify(body);
      try {
        const res = await req.loadString();
        const status = req.response && req.response.statusCode ? req.response.statusCode : null;
        log(`[POST ${url}] Status: ${status}`);
        log(`[POST ${url}] Response length: ${res.length}`);
      } catch (e) {
        if (req && req.response && req.response.statusCode) {
          log(`[POST ${url}] HTTP Error Status: ${req.response.statusCode}`);
        } else {
          log(`[POST ${url}] Error: ${e.toString()}`);
        }
      }
    }

    // Send directly to IP
    await sendPayload(MDNS_ENDPOINT, payload);

    QuickLook.present(logBuffer);

  } catch (e) {
    QuickLook.present("[Error] " + e.toString() + "\n\nLogs:\n" + logBuffer);
  }
}

await main();
Script.complete();