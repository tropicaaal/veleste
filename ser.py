import pygame, serial

KEY_LEFT = 'a'
KEY_RIGHT = 'd'
KEY_UP = 'w'
KEY_DOWN = 's'
KEY_OK = 'o'
KEY_EXE = 'e'
KEY_BACK = 'b'
KEY_TOOLBOX = 't'
KEY_BACKSPACE = 'k'
KEY_VAR = 'v'
KEY_HOME = 'h'
KEY_ON_OFF = 'q'

PORT = "/dev/ttyACM12"
MAP = {
    pygame.K_LEFT:   KEY_LEFT,
    pygame.K_RIGHT:  KEY_RIGHT,
    pygame.K_UP:     KEY_UP,
    pygame.K_DOWN:   KEY_DOWN,
    pygame.K_c:      KEY_OK,
    pygame.K_x:      KEY_BACK,
    pygame.K_z:      KEY_TOOLBOX,
    pygame.K_ESCAPE: KEY_BACKSPACE,
    pygame.K_RETURN: KEY_EXE,
    pygame.K_v:      KEY_VAR,
}

ser = serial.Serial(PORT, 115200)
pygame.init()
pygame.display.set_mode((200, 100))
try:
    x = 0
    while True:
        for e in pygame.event.get():
            if e.type == pygame.QUIT:
                raise SystemExit
            if e.type in (pygame.KEYDOWN, pygame.KEYUP) and e.key in MAP:
                c = MAP[e.key]
                print(f"meow {x}")
                x += 1
                ser.write((c if e.type == pygame.KEYDOWN else c.upper()).encode())
finally:
    ser.write(b"0")