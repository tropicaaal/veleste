use std::sync::atomic::{AtomicU64, Ordering};

const KEY_LEFT: u8 = 0;
const KEY_UP: u8 = 1;
const KEY_DOWN: u8 = 2;
const KEY_RIGHT: u8 = 3;
const KEY_OK: u8 = 4;
const KEY_BACK: u8 = 5;
const KEY_HOME: u8 = 6;
const KEY_ON_OFF: u8 = 7;
const KEY_VAR: u8 = 15;
const KEY_TOOLBOX: u8 = 16;
const KEY_BACKSPACE: u8 = 17;
const KEY_EXE: u8 = 52;

// action bits
const K_LEFT: u32 = 1 << 0;
const K_RIGHT: u32 = 1 << 1;
const K_UP: u32 = 1 << 2;
const K_DOWN: u32 = 1 << 3;
const K_JUMP: u32 = 1 << 4;
const K_DASH: u32 = 1 << 5;
const K_GRAB: u32 = 1 << 6;
const K_TALK: u32 = 1 << 7;
const K_PAUSE: u32 = 1 << 8;
const K_HOME: u32 = 1 << 9;
const K_OK: u32 = 1 << 10;
const K_BACK: u32 = 1 << 11;
const K_JOURNAL: u32 = 1 << 12;
const K_ANY: u32 = 1 << 15;

#[inline]
const fn key(k: u8) -> u64 {
    1u64 << k
}

fn key_from_byte(b: u8) -> Option<(u8, bool)> {
    let down = b.is_ascii_lowercase();
    let id = match b.to_ascii_lowercase() {
        b'a' => KEY_LEFT,
        b'd' => KEY_RIGHT,
        b'w' => KEY_UP,
        b's' => KEY_DOWN,
        b'o' => KEY_OK,
        b'e' => KEY_EXE,
        b'b' => KEY_BACK,
        b't' => KEY_TOOLBOX,
        b'k' => KEY_BACKSPACE,
        b'v' => KEY_VAR,
        b'h' => KEY_HOME,
        b'q' => KEY_ON_OFF,
        _ => return None,
    };
    Some((id, down))
}

static SCAN: AtomicU64 = AtomicU64::new(0);
static SCAN_PREV: AtomicU64 = AtomicU64::new(0);
static HELD: AtomicU64 = AtomicU64::new(0);

fn poll_stdin() {
    let mut held = HELD.load(Ordering::Relaxed);
    for _ in 0..256 {
        let c = unsafe { vex_sdk::vexSerialReadChar(1) };
        if c < 0 {
            break;
        }
        let b = c as u8;
        if b == b'0' {
            held = 0;
        } else if let Some((id, down)) = key_from_byte(b) {
            if down {
                held |= key(id);
            } else {
                held &= !key(id);
            }
        }
    }
    HELD.store(held, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub static mut g_bind: [u8; 4] = [KEY_OK, KEY_BACK, KEY_TOOLBOX, KEY_BACK];

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_default_binds() {
    unsafe { g_bind = [KEY_OK, KEY_BACK, KEY_TOOLBOX, KEY_BACK] };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_key_pressed() -> core::ffi::c_int {
    let scan = SCAN.load(Ordering::Relaxed);
    let prev = SCAN_PREV.load(Ordering::Relaxed);
    let d = scan & !prev;
    if d != 0 { d.trailing_zeros() as core::ffi::c_int } else { -1 }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn plat_keys() -> u32 {
    poll_stdin();

    let k = HELD.load(Ordering::Relaxed);
    SCAN_PREV.store(SCAN.load(Ordering::Relaxed), Ordering::Relaxed);
    SCAN.store(k, Ordering::Relaxed);

    let bind = unsafe { g_bind };
    let mut r = 0u32;
    if k & key(KEY_LEFT) != 0 { r |= K_LEFT; }
    if k & key(KEY_RIGHT) != 0 { r |= K_RIGHT; }
    if k & key(KEY_UP) != 0 { r |= K_UP; }
    if k & key(KEY_DOWN) != 0 { r |= K_DOWN; }
    if k & key(bind[0]) != 0 { r |= K_JUMP; }
    if k & key(bind[1]) != 0 { r |= K_DASH; }
    if k & key(bind[2]) != 0 { r |= K_GRAB; }
    if k & key(bind[3]) != 0 { r |= K_TALK; }
    if k & (key(KEY_OK) | key(KEY_EXE)) != 0 { r |= K_OK; }
    if k & key(KEY_BACK) != 0 { r |= K_BACK; }
    if k & key(KEY_BACKSPACE) != 0 { r |= K_PAUSE; }
    if k & key(KEY_VAR) != 0 { r |= K_JOURNAL; }
    if k & (key(KEY_HOME) | key(KEY_ON_OFF)) != 0 { r |= K_HOME; }
    if k != 0 { r |= K_ANY; }
    r
}