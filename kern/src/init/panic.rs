use core::panic::PanicInfo;

use crate::console::{kprintln};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("     (");
    kprintln!("    (      )     )");
    kprintln!("      )   (    (");
    kprintln!("     (          `");
    kprintln!(" .-\"\"^\"\"\"^\"\"\"\"\"^\"\"-.");
    kprintln!("(//|||/||//||//||//||//)");
    kprintln!("~|^^^^^^^^^^^^^^^^^^|~");
    kprintln!("  `================`");
    kprintln!("");
    kprintln!(" The pi is overdone.");
    kprintln!("");
    kprintln!("---------- PANIC ----------");
    kprintln!("");
    match info.location() {
        Some(location) => {
            kprintln!("panic occurred in file '{}' at line {}", location.file(),
                location.line());
        },
        None           => {
            kprintln!("panic occurred but can't get location information...");
        }
    }

    loop {}
}
