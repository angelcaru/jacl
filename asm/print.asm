format ELF64

section '.text' executable
public print
print:
    mov rax, 1

    xor rdx, rdx
    mov dl, [rdi]

    lea rsi, [rdi+1]
    mov rdi, 1
    syscall

    push 10
    mov rax, 1
    mov rdi, 1
    mov rsi, rsp
    mov rdx, 1
    syscall
    pop rax
    ret