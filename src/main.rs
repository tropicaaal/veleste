extern crate vex_sdk_jumptable;

use std::{cell::UnsafeCell, ffi::CStr, time::Duration};

use vex_sdk::{vexDisplayCopyRect, vexDisplayForegroundColor, vexDisplayRectFill};

mod input;

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_begin() {}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_end() {
    std::process::exit(0);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_millis() -> u32 {
    (unsafe {
        vex_sdk::vexSystemHighResTimeGet()
    } / 1000) as u32
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_sleep(millis: u32) {
    std::thread::sleep(Duration::from_millis(millis as _));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_push(
    x: core::ffi::c_int,
    y: core::ffi::c_int,
    mut w: core::ffi::c_int,
    mut h: core::ffi::c_int,
    px: *const u16,
) {
    if px.is_null() || w <= 0 || h <= 0 {
        return;
    }

    let len = (w as usize) * (h as usize);
    let src = unsafe { core::slice::from_raw_parts(px, len) };

    let mut buf: Vec<u32> = src
        .iter()
        .map(|&p| color_from_rgb565(p))
        .collect();

    unsafe {
        vexDisplayCopyRect(x + 80, y, x + w - 1 + 80, y + h - 1, buf.as_mut_ptr(), w);
    }
}

fn color_from_rgb565(color: u16) -> u32 {
    let r_5 = ((color >> 11) & 0x1F) as u8;
    let g_6 = ((color >> 5) & 0x3F) as u8;
    let b_5 = (color & 0x1F) as u8;

    let r = (r_5 << 3) | (r_5 >> 2);
    let g = (g_6 << 2) | (g_6 >> 4);
    let b = (b_5 << 3) | (b_5 >> 2);

    bytemuck::must_cast::<Color, u32>(Color { r, g, b, _reserved: 0 }).to_le()
}

#[repr(C, align(4))]
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    _reserved: u8,
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_fill(
    x: core::ffi::c_int,
    y: core::ffi::c_int,
    w: core::ffi::c_int,
    h: core::ffi::c_int,
    color: u16,
) {
    let color = color_from_rgb565(color);

    unsafe {
        vexDisplayForegroundColor(color);
        vexDisplayRectFill(x + 80, y, x + w + 80, y + h);
    }
}

struct SaveBuf(UnsafeCell<Vec<u8>>);
unsafe impl Sync for SaveBuf {}

static SAVE_BUF: SaveBuf = SaveBuf(UnsafeCell::new(Vec::new()));

unsafe fn path_from(name: *const core::ffi::c_char) -> Option<&'static str> {
    if name.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(name) }.to_str().ok()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_load(name: *const core::ffi::c_char, len: *mut u32) -> *const u8 {
    let Some(path) = (unsafe { path_from(name) }) else {
        return core::ptr::null();
    };

    let Ok(data) = std::fs::read(path) else {
        return core::ptr::null();
    };

    let buf = unsafe { &mut *SAVE_BUF.0.get() };
    *buf = data;

    if !len.is_null() {
        unsafe { *len = buf.len() as u32 };
    }
    buf.as_ptr()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_save(
    name: *const core::ffi::c_char,
    len: u32,
) -> core::ffi::c_int {
    let Some(path) = (unsafe { path_from(name) }) else {
        return 0;
    };

    let buf = unsafe { &*SAVE_BUF.0.get() };
    let len = len as usize;
    if len > buf.len() {
        return 0;
    }

    match std::fs::write(path, &buf[..len]) {
        Ok(()) => 1,
        Err(_) => {
            0
        },
    }
}

#[link(name = "celeste")]
unsafe extern "C" {
    static mut g_running: core::ffi::c_int;
    static mut cel_bin: *const u8;

    fn gfx_set_memory(strip: *mut u32, cmds: *mut u32);
    fn game_draw();
    fn game_frame();
    fn game_init();
    fn game_save();
}

#[repr(C, align(4))]
pub struct Align4<T: ?Sized>(pub T);

pub static CELESTE_DATA: Align4<[u8; 1488184]> = Align4(*include_bytes!("./data.bin"));

fn main() {
    let strip: &'static mut [u32] = Box::leak(vec![0u32; 9600 / 4].into_boxed_slice());
    let cmds: &'static mut [u32] = Box::leak(vec![0u32; 7200 / 4].into_boxed_slice());

    unsafe {
        gfx_set_memory(strip.as_mut_ptr(), cmds.as_mut_ptr());
        plat_begin();
        cel_bin = CELESTE_DATA.0.as_ptr() as _;
        game_init();

        // In thirds of a millisecond: 50 per update (60 updates a second)
        let mut next: u32 = plat_millis().wrapping_mul(3);

        while g_running != 0 {
            // Draw when caught up (at most 2 updates per frame)
            let mut n = 0;
            let now: u32 = plat_millis().wrapping_mul(3);

            while (now.wrapping_sub(next) as i32) >= 0 && n < 2 && g_running != 0 {
                game_frame();
                next = next.wrapping_add(50);
                n += 1;
            }

            if g_running == 0 {
                break;
            }

            // Far behind: do not rush
            if (now.wrapping_sub(next) as i32) > 300 {
                next = now;
            }

            if n > 0 {
                game_draw();
            } else {
                plat_sleep(1);
            }
        }

        game_save();
        plat_end();
    }
}
