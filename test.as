1
1
section .data
HEAP:   times 1024 dq 0
section .text
        global start_here
        extern snake_error
        extern print_snake_val
start_here:
        mov r15, HEAP
        call main
        ret
main:
;;; Let
;;; Array
        mov rax, 0x0000000000000000
        mov QWORD [r15 + 0], rax
        mov rax, 0x0000000000000004
        mov QWORD [r15 + 8], rax
        mov rax, 0
        mov QWORD [r15 + 16], rax
        mov rax, 0
        mov QWORD [r15 + 24], rax
        mov rax, r15
        add rax, 0x00000001
        add r15, 0x00000020
        mov QWORD [rsp + -8], rax
;;; Let
;;; MakeClosure
        mov rax, 0x0000000000000002
        mov QWORD [r15 + 0], rax
        mov rax, Dummy_14_SetDummy_2
        mov QWORD [r15 + 8], rax
        mov rax, QWORD [rsp + -8]
        mov QWORD [r15 + 16], rax
        mov rax, r15
        add rax, 0x00000003
        add r15, 0x00000018
        mov QWORD [rsp + -16], rax
;;; Let
;;; MakeClosure
        mov rax, 0x0000000000000001
        mov QWORD [r15 + 0], rax
        mov rax, Dummy_14_GetDummy_4
        mov QWORD [r15 + 8], rax
        mov rax, QWORD [rsp + -8]
        mov QWORD [r15 + 16], rax
        mov rax, r15
        add rax, 0x00000003
        add r15, 0x00000018
        mov QWORD [rsp + -24], rax
;;; Let
;;; ArraySet
        mov rax, QWORD [rsp + -8]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
        mov r10, 0
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rbx, QWORD [rsp + -16]
        mov QWORD [rax + r10 * 4 + 16], rbx
        add rax, 0x00000001
        mov QWORD [rsp + -32], rax
;;; Let
;;; ArraySet
        mov rax, QWORD [rsp + -8]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
        mov r10, 2
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rbx, QWORD [rsp + -24]
        mov QWORD [rax + r10 * 4 + 16], rbx
        add rax, 0x00000001
        mov QWORD [rsp + -40], rax
;;; Let
;;; Object
        mov rax, 0x0000000000000001
        mov QWORD [r15 + 0], rax
        mov rax, 0x0000000000000002
        mov QWORD [r15 + 8], rax
        mov rax, 0
        mov QWORD [r15 + 16], rax
        mov rax, r15
        add rax, 0x00000001
        add r15, 0x00000018
        mov QWORD [rsp + -48], rax
;;; Let
;;; CallMethod
        mov rax, QWORD [rsp + -48]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
;;; Check object and method type
        mov rdi, 0x000000000000000b
        mov r11, 0x0000000000000001
        cmp r11, QWORD [rax + 0]
        je Found_18
        jmp snake_err
Found_18:
        mov r10, QWORD [rsp + -16]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000009
        mov rbx, 0x0000000000000007
        and rbx, r10
        cmp rbx, 0x00000003
        jne snake_err
        sub r10, 0x00000003
;;; Check Arity Number
        mov rdi, 0x000000000000000a
        mov rbx, 0x0000000000000001
        cmp rbx, QWORD [r10 + 0]
        jne snake_err
        mov rax, QWORD [r10 + 16]
        mov QWORD [rsp + -72], rax
        mov rax, QWORD [rsp + -48]
        mov QWORD [rsp + -80], rax
        mov rax, 966
        mov QWORD [rsp + -88], rax
;;; CallClosure-Non Tail Recursion
        sub rsp, 56
        mov rax, QWORD [r10 + 8]
        call rax
        add rsp, 56
        mov QWORD [rsp + -56], rax
;;; CallMethod
        mov rax, QWORD [rsp + -48]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
;;; Check object and method type
        mov rdi, 0x000000000000000b
        mov r11, 0x0000000000000001
        cmp r11, QWORD [rax + 0]
        je Found_19
        jmp snake_err
Found_19:
        mov r10, QWORD [rsp + -24]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000009
        mov rbx, 0x0000000000000007
        and rbx, r10
        cmp rbx, 0x00000003
        jne snake_err
        sub r10, 0x00000003
;;; Check Arity Number
        mov rdi, 0x000000000000000a
        mov rbx, 0x0000000000000000
        cmp rbx, QWORD [r10 + 0]
        jne snake_err
        mov rax, QWORD [r10 + 16]
        mov QWORD [rsp + -72], rax
        mov rax, QWORD [rsp + -48]
        mov QWORD [rsp + -80], rax
;;; CallClosure-Tail Recursion
        mov rax, QWORD [rsp + -72]
        mov QWORD [rsp + -8], rax
        mov rax, QWORD [rsp + -80]
        mov QWORD [rsp + -16], rax
        mov rax, QWORD [r10 + 8]
        jmp rax
        ret
Dummy_14_SetDummy_2:
;;; Let
;;; Prim2
        mov rax, QWORD [rsp + -8]
        mov r10, 0
;;; ArrayGet
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rax, QWORD [rax + r10 * 4 + 16]
        mov QWORD [rsp + -32], rax
;;; Let
;;; Prim2
        mov rax, QWORD [rsp + -8]
        mov r10, 2
;;; ArrayGet
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rax, QWORD [rax + r10 * 4 + 16]
        mov QWORD [rsp + -40], rax
;;; ArraySet
        mov rax, QWORD [rsp + -16]
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
        mov r10, 0
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rbx, QWORD [rsp + -24]
        mov QWORD [rax + r10 * 4 + 16], rbx
        add rax, 0x00000001
        ret
Dummy_14_GetDummy_4:
;;; Let
;;; Prim2
        mov rax, QWORD [rsp + -8]
        mov r10, 0
;;; ArrayGet
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rax, QWORD [rax + r10 * 4 + 16]
        mov QWORD [rsp + -24], rax
;;; Let
;;; Prim2
        mov rax, QWORD [rsp + -8]
        mov r10, 2
;;; ArrayGet
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rax, QWORD [rax + r10 * 4 + 16]
        mov QWORD [rsp + -32], rax
;;; Prim2
        mov rax, QWORD [rsp + -16]
        mov r10, 0
;;; ArrayGet
;;; Check Boolean/Array/Closure type
        mov rdi, 0x0000000000000005
        mov rbx, 0x0000000000000007
        and rbx, rax
        cmp rbx, 0x00000001
        jne snake_err
        sub rax, 0x00000001
;;; Check Number type
        mov rdi, 0x0000000000000006
        mov rbx, 0x0000000000000001
        test rbx, r10
        jnz snake_err
;;; Check Array Index Bounding
        mov rdi, 0x0000000000000007
        cmp r10, QWORD [rax + 8]
        jge snake_err
        cmp r10, 0
        jl snake_err
        mov rax, QWORD [rax + r10 * 4 + 16]
        ret
snake_err:
        call snake_error


