use coremotion_rs::{CMMotionManager, ICMAccelerometerData, ICMMotionManager, INSObject};

fn main() {
    let manager = CMMotionManager::alloc();
    unsafe {
        manager.init();
        let available = manager.isAccelerometerAvailable();
        println!("Accelerometer {available}");
        if available {
            manager.setAccelerometerUpdateInterval_(1.0/60.0); //60Hz
            manager.startAccelerometerUpdates();
            for i in 1..1000 {
                let data = manager.accelerometerData();
                let acceleration = data.acceleration();
                println!("Sample {i} - {acceleration:?}");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}