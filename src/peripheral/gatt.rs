//! BLE GATT service setup and peripheral functionality
//!
//! Configures and runs a Bluetooth Low Energy peripheral with a GATT service
//! that allows read/write operations and notifications.

use std::io::{self, BufRead};
use tokio::{sync::mpsc, task};
use tracing::{error, info};
use uuid::Uuid;

use ble_peripheral_rust::PeripheralImpl;
use ble_peripheral_rust::{
    Peripheral,
    gatt::{
        characteristic::Characteristic,
        descriptor::Descriptor,
        peripheral_event::{
            PeripheralEvent, ReadRequestResponse, RequestResponse, WriteRequestResponse,
        },
        properties::{AttributePermission, CharacteristicProperty},
        service::Service,
    },
    uuid::ShortUuid,
};

/// Start the BLE peripheral and handle GATT operations
pub async fn ble_peripheral(line_rx: mpsc::UnboundedReceiver<String>) {
    // UUIDs for service and characteristic
    let service_uuid = Uuid::from_short(0x1234_u16);
    let char_uuid = Uuid::from_short(0x2A3D_u16);

    // Shared characteristic value
    let char_value = std::sync::Arc::new(tokio::sync::Mutex::new(b"Hello iOS".to_vec()));

    // GATT service definition
    let service = Service {
        uuid: service_uuid,
        primary: true,
        characteristics: vec![Characteristic {
            uuid: char_uuid,
            properties: vec![
                CharacteristicProperty::Read,
                CharacteristicProperty::Write,
                CharacteristicProperty::Notify,
            ],
            permissions: vec![
                AttributePermission::Readable,
                AttributePermission::Writeable,
            ],
            value: Some(char_value.lock().await.clone()),
            descriptors: vec![Descriptor {
                uuid: Uuid::from_short(0x2A13_u16),
                value: Some(vec![0, 1]),
                ..Default::default()
            }],
        }],
    };

    // Channels for BLE events
    let (event_tx, mut event_rx): (
        mpsc::Sender<PeripheralEvent>,
        mpsc::Receiver<PeripheralEvent>,
    ) = mpsc::channel(256);

    // Create peripheral
    let mut peripheral = Peripheral::new(event_tx).await.unwrap();

    // Add service
    peripheral.add_service(&service).await.unwrap();

    // Start advertising
    info!("Advertising as Serviceberry...");
    peripheral
        .start_advertising("Serviceberry", &[service.uuid])
        .await
        .unwrap();

    // Spawn task to handle BLE events
    let char_value_clone = char_value.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                PeripheralEvent::ReadRequest { responder, .. } => {
                    let value = char_value_clone.lock().await.clone();
                    responder
                        .send(ReadRequestResponse {
                            value,
                            response: RequestResponse::Success,
                        })
                        .unwrap();
                }
                PeripheralEvent::WriteRequest {
                    value, responder, ..
                } => {
                    {
                        let mut v = char_value_clone.lock().await;
                        *v = value.clone();
                    }
                    responder
                        .send(WriteRequestResponse {
                            response: RequestResponse::Success,
                        })
                        .unwrap();
                    info!("Characteristic updated via write: {:?}", value);
                }
                PeripheralEvent::CharacteristicSubscriptionUpdate {
                    request,
                    subscribed,
                } => {
                    info!("Subscription update: subscribed={subscribed:?}, request={request:?}");
                }
                _ => {}
            }
        }
    });

    // Optional: spawn task to read from stdin
    let (line_tx, mut line_rx_stdin): (
        mpsc::UnboundedSender<String>,
        mpsc::UnboundedReceiver<String>,
    ) = mpsc::unbounded_channel();
    let line_tx_clone = line_tx.clone();
    task::spawn_blocking(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(input) = line {
                if line_tx_clone.send(input).is_err() {
                    break;
                }
            }
        }
    });

    // Forward external line_rx into internal line_rx_stdin
    let mut line_rx = line_rx;
    tokio::spawn(async move {
        while let Some(line) = line_rx.recv().await {
            if line_tx.send(line).is_err() {
                error!("Failed to forward line to internal channel");
            }
        }
    });

    // Update characteristic with input from stdin or external channel
    while let Some(input) = line_rx_stdin.recv().await {
        info!("Writing '{input}' to characteristic {char_uuid}");
        {
            let mut v = char_value.lock().await;
            *v = input.clone().into_bytes();
        }
        if let Err(err) = peripheral
            .update_characteristic(char_uuid, input.into_bytes())
            .await
        {
            error!("Error updating characteristic: {}", err);
            break;
        }
    }
}
