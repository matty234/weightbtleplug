use btleplug::api::CentralEvent;
use btleplug::api::bleuuid::uuid_from_u32;
use btleplug::api::{
    bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::time;
use futures::stream::StreamExt;
use btleplug::api::bleuuid::BleUuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let manager = Manager::new().await.unwrap();


    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");



    let mut events = central.events().await?;


    let filter = ScanFilter { services: vec![ uuid_from_u32(0x0000ffc0) ] };
    central.start_scan(filter).await?;


    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                println!("DeviceDiscovered: {:?}", id);
                let p = central.peripheral(&id).await?;
                p.connect().await?;

        



            }
            CentralEvent::DeviceConnected(id) => {
                println!("DeviceConnected: {:?}", id);
                let local_peripheral = central.peripheral(&id).await?;           

                let services = local_peripheral.discover_services().await?;
                println!("Services: {:?}", services);


                let fff1Id = uuid_from_u32(0x0000fff1);
                let fff2Id = uuid_from_u32(0x0000fff2);

                
                let characteristics = local_peripheral.characteristics();
                
                let mut fff1 = characteristics.iter().find(|c| c.uuid == fff1Id).unwrap();
                let mut fff2 = characteristics.iter().find(|c| c.uuid == fff2Id).unwrap();

                println!("fff1Id: {:?}", fff1.uuid);
                println!("fff2Id: {:?}", fff2.uuid);



                
                local_peripheral.write(&mut fff2, &[0x01, 0x00], WriteType::WithoutResponse).await?;


                // Value: 1000 00C5 1303 0100 0085 9301 B918 0000 0037 38

                local_peripheral.write(&mut fff2, &[0x10, 0x00, 0x00, 0xC5, 0x13, 0x03, 0x01, 0x00, 0x00, 0x85, 0x93, 0x03, 0xB9, 0x18, 0x00, 0x00, 0x00, 0x37, 0x38], WriteType::WithoutResponse).await?;

                //	Value: 1000 00C5 0B03 0285 9300 28

                local_peripheral.write(&mut fff2, &[0x10, 0x00, 0x00, 0xC5, 0x0B, 0x03, 0x02, 0x85, 0x93, 0x00, 0x28], WriteType::WithoutResponse).await.expect("Unable to write to characteristic");



                local_peripheral.subscribe(&fff1).await?;

                let mut events = local_peripheral.notifications().await?;

                while let Some(event) = events.next().await {
                
                    
                    let weight = u16::from_be_bytes(event.value[7..9].try_into().unwrap());
                    
                    println!("Notification: {:02X?}", event.value.to_vec());

                    match event.value[4] {
                        0x0E => println!("Weight: {}kg" , weight as f32 / 100.0),
                        0x1E => {
                            let weight = u16::from_be_bytes(event.value[10..12].try_into().unwrap());
                            let fat = u16::from_be_bytes(event.value[12..14].try_into().unwrap());
                            let water = u16::from_be_bytes(event.value[16..18].try_into().unwrap());
                            let muscle = u16::from_be_bytes(event.value[18..20].try_into().unwrap());
                            
                            println!("Weight: {}kg" , weight as f32 / 100.0);
                            println!("Fat: {}%" , fat as f32 / 10.0);
                            println!("Water: {}%" , water as f32 / 10.0);
                            println!("Muscle: {}%" , muscle as f32 / 10.0);
                            local_peripheral.unsubscribe(&fff1).await?;
                            local_peripheral.disconnect().await?;
                        },
                        _ => println!("Unknown message"),
                    }
                }
            }
            CentralEvent::DeviceDisconnected(id) => {
                println!("DeviceDisconnected: {:?}", id);
                let filter = ScanFilter { services: vec![ uuid_from_u32(0x0000ffc0) ] };
                central.peripheral(&id).await?.disconnect().await?;
                central.start_scan(filter).await?;
            }
      
            _ => {}
        }

    }
    Ok(())
}
