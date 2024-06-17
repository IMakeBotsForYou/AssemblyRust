number dd, 102
factors dd, 0
mov ECX, 0 ; counter
mov ESI, 1
loop:
  inc ESI
  cmp [number], 1
  je exit 

continue_dividing:

  mov EAX, DWORD PTR [number]
  mov EDX, 0
  div ESI
  cmp EDX, 0         ; No remainder ?
  je update_number  ; If no remainder, add number to factor list
  jne loop           ; If we have a remainder, go back to loop.

update_number:
  mov DWORD PTR [number], EAX
  mov DWORD PTR [factors+ECX*4], ESI
  inc ECX
  jmp continue_dividing
exit:
  print ECX, [factors]