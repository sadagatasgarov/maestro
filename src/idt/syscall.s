.text

.global syscall

syscall:
	cli
	push %ebp
	mov %esp, %ebp

	push %edi
	push %esi
	push %edx
	push %ecx
	push %ebx
	push %eax

	mov %ebp, %eax
	add $8, %eax
	push %eax

	push 4(%ebp)

	mov %ebp, %eax
	add $20, %eax
	push %eax

	mov %ebp, %eax
	sub $4, %eax
	push (%eax)

	mov $GDT_KERNEL_DATA_OFFSET, %ax
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs

	push %esp
	call syscall_handler
	add $44, %esp

	xor %ebx, %ebx
	mov $GDT_USER_DATA_OFFSET, %bx
	or $3, %bx
	mov %bx, %ds
	mov %bx, %es
	mov %bx, %fs
	mov %bx, %gs

	mov %ebp, %esp
	pop %ebp
	sti
	iret
