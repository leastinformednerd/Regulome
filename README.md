# Regulome

For reasons of personal enjoyment this has been shelved and currently all my kernel dev is in: https://github.com/leastinformednerd/Kernel-Experiments

Shared repository for a pure rust x86\_64 kernel (Regulome) / its uefi bootloader (unnamed..., and at this point as I want to work on the kernel I will be using Limine and the bootloader is shelved)

## About Regulome
What I think makes Regulome interesting as a design is the (planned and not yet implemented as the project is very nascent and the kernel can't even be loaded right now) scheduler.

All processses in both kernel-space and user-space either 'promote' or 'inhibit' related processes. This works similarly to how biological cis- and trans- regulatory elements in cells modulate the expression of proteins, hence the name and the inspiration (I like the thought that in a sense the transcription related cell machinery is a form of scheduler).

## Work that is not my own
The work in this repository that is not my own (other than crates I am using through cargo) are:
   
   - The spleen font (located in ./spleen/)
   
   spleen- The kernel linker script (located at ./kern/linker.ld), which is taken from the limine-rust-template repository (I do not yet know how to write a good linker script)
