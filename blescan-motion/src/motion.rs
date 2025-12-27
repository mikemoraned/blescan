use objc2_core_motion::CMMotionManager;

fn main() {
    let frequency = 1.0;
    let motion_manager = unsafe { CMMotionManager::new() };
    unsafe {
        motion_manager.setAccelerometerUpdateInterval(1.0 / frequency);
        motion_manager.startAccelerometerUpdates();
    }
    unsafe {
        loop {
            if let Some(accelerometer_data) = motion_manager.accelerometerData() {
                let acceleration = accelerometer_data.acceleration();
                println!("x: {}, y: {}, z: {}", acceleration.x, acceleration.y, acceleration.z);

            };
        }
    }
}