format ELF64

section '.text' executable
public print
public printNum
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

; printNum function compiled on https://godbolt.org/ from the following C code:
;
; void print(const char *buf);
; 
; #define BUF_SIZE 30
; 
; void print_num(long x) {
;     char buf[BUF_SIZE];
;     if (x == 0) {
;         buf[0] = 1;
;         buf[1] = '0';
;         print(buf);
;     } else {
;         buf[0] = 0;
;         char *ptr = &buf[BUF_SIZE-1];
;         unsigned char len = 0;
;         while (x != 0) {
;             len++;
;             *ptr-- = '0' + (x%10);
;             x /= 10;
;         }
;         *ptr = len;
;         print(ptr);
;     }
; }
; 

printNum:
        sub     rsp, 40
        test    rdi, rdi
        je      .L8
        mov     BYTE [rsp], 0
        lea     r9, [rsp+29]
        mov     r8, 7378697629483820647
        mov     rcx, r9
.L4:
        mov     rax, rdi
        mov     rsi, rcx
        sub     rcx, 1
        imul    r8
        mov     rax, rdi
        sar     rax, 63
        sar     rdx, 2
        sub     rdx, rax
        lea     rax, [rdx+rdx*4]
        add     rax, rax
        sub     rdi, rax
        add     edi, 48
        mov     BYTE [rcx+1], dil
        mov     rdi, rdx
        test    rdx, rdx
        jne     .L4
        add     r9d, 1
        mov     rdi, rcx
        sub     r9d, esi
        mov     BYTE [rcx], r9b
        call    print
        add     rsp, 40
        ret
.L8:
        mov     eax, 12289
        mov     rdi, rsp
        mov     WORD [rsp], ax
        call    print
        add     rsp, 40
        ret