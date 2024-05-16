# Regulome
Shared repository for a pure rust kernel (Regulome) / it's uefi bootloader (unnamed)

## About Regulome
What I think makes Regulome interesting as a design is the (planned and not yet implemented as the project is very nascent and the kernel can't even be loaded right now) scheduler.

All processses in both kernel-space and user-space either 'promote' or 'inhibit' related processes. This works similarly to how biological cis- and trans- regulatory elements in cells modulate the expression of proteins, hence the name and the inspiration (in a sense the cell machinery is a form of scheduler).
