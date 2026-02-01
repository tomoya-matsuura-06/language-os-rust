#![no_std]
#![no_main]
#![feature(offset_of)]

// use core::panic::PanicInfo;
// use core::ptr::read_volatile;
// use core::ptr::write_volatile;
// use core::time::Duration;
// use wasabi::error;
// use wasabi::executor::sleep;
// use wasabi::executor::spawn_global;
// use wasabi::executor::start_global_executor;
// use wasabi::gui::set_global_vram;
// use wasabi::info;
// use wasabi::init::init_allocator;
// use wasabi::init::init_basic_runtime;
// use wasabi::init::init_display;
// use wasabi::init::init_hpet;
// use wasabi::init::init_paging;
// use wasabi::init::init_pci;
// use wasabi::input::input_task;
// use wasabi::print::hexdump_struct;
// use wasabi::println;
// use wasabi::qemu::exit_qemu;
// use wasabi::qemu::QemuExitCode;
// use wasabi::serial::SerialPort;
// use wasabi::uefi::init_vram;
// use wasabi::uefi::locate_loaded_image_protocol;
// use wasabi::uefi::EfiHandle;
// use wasabi::uefi::EfiSystemTable;
// use wasabi::warn;
// use wasabi::x86::init_exceptions;
use core::arch::asm;
use core::mem::offset_of;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use core::slice;

type EfiVoid = u8;
type EfiHandle = u64;
type Result<T> = core::result::Result<T, &'static str>;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EfiGuid {
    pub data0: u32,
    pub data1: u16,
    pub data2: u16,
    pub data3: [u8; 8],
}

const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data0: 0x9042a9de,
    data1: 0x23dc,
    data2: 0x4a38,
    data3: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[must_use]
#[repr(u64)]
enum EfiStatus {
    Success = 0,
}

#[repr(C)]
struct EfiBootServicesTable {
    _reserved0: [u64; 40],
    locate_protocol: extern "win64" fn(
        protocol: *const EfiGuid,
        registration: *const EfiVoid,
        interface: *mut *mut EfiVoid,
    ) -> EfiStatus,
}
const _: () = assert!(offset_of!(EfiBootServicesTable, locate_protocol) == 320);

#[repr(C)]
struct EfiSystemTable {
    _reserved0: [u64; 12],
    pub boot_services: &'static EfiBootServicesTable,
}
const _: () = assert!(offset_of!(EfiSystemTable, boot_services) ==96);

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolPixelInfo {
    version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    _padding0: [u32; 5],
    pub pixels_per_scan_line: u32,
}
const _: () = assert!(size_of::<EfiGraphicsOutputProtocolPixelInfo>() == 36);

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    // pub info: &'EfiGraphicsOutputProtocolPixelInfo,
    pub info: &'a EfiGraphicsOutputProtocolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize,
    pub frame_buffer_size: usize,
}

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocol<'a> {
    reserved: [u64; 3],
    pub mode: &'a EfiGraphicsOutputProtocolMode<'a>,
}
fn locate_graphic_protocol<'a>(
    efi_system_table: &EfiSystemTable,
) -> Result<&'a EfiGraphicsOutputProtocol<'a>> {
    let mut graphic_output_protocol = null_mut::<EfiGraphicsOutputProtocol>();
    let status = (efi_system_table.boot_services.locate_protocol)(
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        null_mut::<EfiVoid>(),
        &mut graphic_output_protocol as *mut *mut EfiGraphicsOutputProtocol
            as *mut *mut EfiVoid,
    );
    if status != EfiStatus::Success {
        return Err("Failed to locate graphics output protocol");
    }
    Ok(unsafe { &*graphic_output_protocol })
}

pub fn hlt() {
    unsafe { asm!("hlt") }
}

#[no_mangle]
fn efi_main(_image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let efi_graphics_output_protocol = locate_graphic_protocol(efi_system_table).unwrap();
    let vram_addr = efi_graphics_output_protocol.mode.frame_buffer_base;
    let vram_byte_size = efi_graphics_output_protocol.mode.frame_buffer_size;
    let vram = unsafe {
        slice::from_raw_parts_mut(
            vram_addr as *mut u32,
            vram_byte_size / size_of::<u32>(),
        )
    };
    for e in vram {
        *e = 0xffffff;
    }
    //println!("Hello, world!");
    //loop {}
    loop {
        hlt()
    }
}

//     println!("Booting WasabiOS...");
//     println!("image_handle: {:#018X}", image_handle);
//     println!("efi_system_table: {:#p}", efi_system_table);
//     let loaded_image_protocol =
//         locate_loaded_image_protocol(image_handle, efi_system_table)
//             .expect("Failed to get LoadedImageProtocol");
// 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 
//     println!("image_base: {:#018X}", loaded_image_protocol.image_base);
//     println!("image_size: {:#018X}", loaded_image_protocol.image_size);
//     info!("info");
//     warn!("warn");
//     error!("error");
//     hexdump_struct(efi_system_table);
//     let mut vram = init_vram(efi_system_table).expect("init_vram failed");
//     init_display(&mut vram);
//     set_global_vram(vram);
//     let acpi = efi_system_table.acpi_table().expect("ACPI table not found");

//     let memory_map = init_basic_runtime(image_handle, efi_system_table);
//     INFO!("hELlo, Non-UEFI world!");
//     init_allocator(&memory_map);
//     let (_gdt, _idt) = init_exceptions();
//     init_paging(&memory_map);
//     init_hpet(acpi);
//     init_pci(acpi);
//     let serial_task = 
//         let sp = SerialPort::default();
//         if let Err(e) = sp.loopback_test() {
//             error!("{e:?}");
//             return Err("serial: loopback test failed");
//         }
//         iasync 
//             nfo!("Started to monitor serial port");
//         loop {
//             if let Some(v) = sp.try_read() {
//                 let c = char::from_u32(v as u32);
//                 info!("serial input: {v:#04X} = {c:?}");
//             }
//             sleep(Duration::from_millis(20)).await;
//         }
//     spawn_global(serial_task);
//     let abp_uart_task = async {
//         // https://caro.su/msx/ocm_de1/16550.pdf
//         sleep(Duration::from_millis(1000)).await;
//         let base_addr = 0xfe032000_usize; // chromebook boten/bookem
//         let reg_rx_data = base_addr as *mut u8;
//         let reg_line_status = (base_addr + 0b101) as *mut u8;
//         unsafe {
//             write_volatile((base_addr + 1) as *mut u8, 0x00);
//             write_volatile((base_addr + 3) as *mut u8, 0x80);
//             write_volatile((base_addr) as *mut u8, 1);
//             write_volatile((base_addr + 1) as *mut u8, 0);
//             write_volatile((base_addr + 3) as *mut u8, 0x03);
//             write_volatile((base_addr + 2) as *mut u8, 0xC7);
//             write_volatile((base_addr + 4) as *mut u8, 0x0B);
//         }
//         loop {
//             sleep(Duration::from_millis(1000)).await;
//             info!("----");
//             let data = unsafe { read_volatile(reg_rx_data) };
//             info!("DATA:      {data:#010X}");
//             let status = unsafe { read_volatile(reg_line_status) };
//             info!("STATUS:    {status:#010b}");
//         }
//     };
//     spawn_global(abp_uart_task);
//     spawn_global(input_task());
//     start_global_executor()
// }


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
//     error!("PANIC: {info:?}");
//     exit_qemu(QemuExitCode::Fail);
// }
    //loop {}
    loop {
        hlt()
    }
}
