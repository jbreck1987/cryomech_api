fn main() {
    println!("Hello, world!");

    // Desired API
    /*
    // From new raw (all constructor fields required)
    let smdp_api = CryomechApiSmdp::new(com_port: "/dev/ttyUSB0", baud: 115200, read_timeout_ms: 50, dev_addr: 16)?;
    let modbus_api = CryomechApiModbusTCP::new(com_port: "/dev/ttyUSB0", baud: 115200, read_timeout_ms: 50, dev_addr: 42)?;

    // Using Default
    let smdp_api = CryomechApiSmdp {com_port: "/dev/ttyUSB0", dev_addr: 16, ..Default::default()?}
    ..

    // Using Builder
    let smdp_api = CryomechApiSmdpBuilder::new(com_port: "/dev/ttyUSB0", dev_addr: 16).read_timeout_ms(50).build()?;
    ..

    // Read-only requests
    let cpu_temp = smdp_api.cpu_temp()?;
    let mem_loss = smdp_api.mem_loss()?;
    .. Same for modbus API

     */
}
