import "./after_frame.s" AfterFrame &[$1000] {
  player: [!PLAYER],
  player_x: [!PLAYER_X],
  player_y: [!PLAYER_Y],
  gravity: [!GRAVITY],
  jump_force: [!JUMP_FORCE],
}

const PLAYER = $2000
const PLAYER_X = $2001
const PLAYER_Y = $2002
const PLAYER_FLAGS = $2003
const MOVE_SPEED = $2
const GRAVITY = $5
const JUMP_FORCE = $5

const INPUT_ADDR = $677C
const INTERRUPT_ADDR = $676C

start:
setup_sprites:
  mov8 &[!PLAYER], $04
  mov8 &[!PLAYER_X], $10
  mov8 &[!PLAYER_Y], $10

setup_interrupts:
  mov &[!INTERRUPT_ADDR + $2], $3280

check_inputs:
  mov8 r8, &[!INPUT_ADDR]

check_left_press:
  mov acc, r8
  and acc, $80
  rsh acc, $7
  jne &[!check_down_press], $1
  mov8 r7, &[!PLAYER_X]
  sub r7, !MOVE_SPEED
  mov8 &[!PLAYER_X], r7
  call &[!look_left]

check_down_press:
  mov acc, r8
  and acc, $40
  rsh acc, $6
  jne &[!check_up_press], $1
  mov8 r7, &[!PLAYER_Y]
  add r7, !MOVE_SPEED
  mov8 &[!PLAYER_Y], r7

check_up_press:
  mov acc, r8
  and acc, $20
  rsh acc, $5
  jne &[!check_right_press], $1
  mov8 r7, &[!PLAYER_Y]
  sub r7, !MOVE_SPEED
  mov8 &[!PLAYER_Y], r7

check_right_press:
  mov acc, r8
  and acc, $10
  rsh acc, $4
  jne &[!clear_input], $1
  mov8 r7, &[!PLAYER_X]
  add r7, !MOVE_SPEED
  mov8 &[!PLAYER_X], r7
  call &[!look_right]

clear_input:
  mov8 &[!INPUT_ADDR], $0

game_loop:
  jmp &[!check_inputs]
  jmp &[!game_loop]
  hlt

; clear the first bit of the sprite flags (MIRROR_X)
look_right:
  mov8 r7, &[!PLAYER_FLAGS]
  and r7, $FE
  mov &[!PLAYER_FLAGS], r7
  ret

; set the first bit of the sprite flags (MIRROR_X)
look_left:
  mov8 r7, &[!PLAYER_FLAGS]
  or r7, $1
  mov &[!PLAYER_FLAGS], r7
  ret
