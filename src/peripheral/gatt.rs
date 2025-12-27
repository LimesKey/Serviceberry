use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex, mpsc};
use tracing::info;
use uuid::Uuid;

use ble_peripheral_rust::PeripheralImpl;
use ble_peripheral_rust::{
    Peripheral,
    gatt::{
        characteristic::Characteristic,
        descriptor::Descriptor,
        peripheral_event::{PeripheralEvent, RequestResponse, WriteRequestResponse},
        properties::{AttributePermission, CharacteristicProperty},
        service::Service,
    },
    uuid::ShortUuid,
};

use crate::server::handlers::PartialPayload;

pub async fn ble_peripheral(payload_tx: UnboundedSender<PartialPayload>) {
    let service_uuid =
        Uuid::parse_str("12345678-1234-5678-1234-56789abcdef0").expect("invalid service UUID");
    let char_uuid =
        Uuid::parse_str("abcdef01-1234-5678-1234-56789abcdef0").expect("invalid char UUID");

    let char_value = Arc::new(Mutex::new(b"Hello iOS".to_vec()));

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

    let (event_tx, mut event_rx) = mpsc::channel::<PeripheralEvent>(256);
    let mut peripheral = Peripheral::new(event_tx)
        .await
        .expect("failed to create peripheral");
    peripheral
        .add_service(&service)
        .await
        .expect("failed to add service");

    let mut write_buffer = Vec::new();
    let char_value_loop = Arc::clone(&char_value);

    info!("Advertising as Serviceberry...");
    let _ = peripheral
        .start_advertising("Serviceberry", &[service_uuid])
        .await;

    while let Some(event) = event_rx.recv().await {
        match event {
            PeripheralEvent::ReadRequest { responder, .. } => {
                let data = char_value_loop.lock().await;
                let _ = responder.send(
                    ble_peripheral_rust::gatt::peripheral_event::ReadRequestResponse {
                        value: data.clone(),
                        response: RequestResponse::Success,
                    },
                );
            }

            PeripheralEvent::WriteRequest {
                value, responder, ..
            } => {
                let _ = responder.send(WriteRequestResponse {
                    response: RequestResponse::Success,
                });

                {
                    let mut data = char_value_loop.lock().await;
                    *data = value.clone();
                }

                write_buffer.extend_from_slice(&value);

                if let Ok(text) = String::from_utf8(write_buffer.clone()) {
                    let mut success = false;

                    if text.contains('\n') {
                        for line in text.lines() {
                            if let Ok(payload) = serde_json::from_str::<PartialPayload>(line) {
                                let _ = payload_tx.send(payload);
                                success = true;
                            }
                        }
                    } else if let Ok(payload) = serde_json::from_str::<PartialPayload>(&text) {
                        let _ = payload_tx.send(payload);
                        success = true;
                    }

                    if success {
                        write_buffer.clear();
                    }
                }

                if write_buffer.len() > 2048 {
                    write_buffer.clear();
                }
            }

            _ => {
                let _ = peripheral
                    .start_advertising("Serviceberry", &[service_uuid])
                    .await;
            }
        }
    }
}
