
use cortex_m::peripheral;

pub fn itm_init(peripherals: &mut peripheral::Peripherals) {
    unsafe {
        // Unlock ITM if needed
        peripherals.ITM.lar.write(0xC5ACCE55);
        
        // Enable DWT and ITM units, set up SWO trace
        peripherals.DCB.enable_trace(); // Enable tracing in the Debug Control Block
        
        // Configure TPIU for SWO output  
        peripherals.TPIU.sppr.write(2); // Set the Selected Pin Protocol Register to 2 (SWO NRZ)
        
        // Set prescaler to 0 for now (no prescaling)
        peripherals.TPIU.acpr.write(0);
        
        // Enable ITM with basic settings
        peripherals.ITM.tcr.write(0x0001000D); // Enable ITM with sync packet
        peripherals.ITM.ter[0].write(1); // Enable stimulus port 0
    }
}

pub fn itm_print(msg: &str) {
    let stim0 = unsafe { &mut (*peripheral::ITM::PTR).stim[0] };
    for byte in msg.bytes() {
        // Wait until the stimulus port is ready
        while stim0.is_fifo_ready() == false {}
        stim0.write_u8(byte);
    }
}
