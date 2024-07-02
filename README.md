# Regulome
Shared repository for a pure rust x86\_64 kernel (Regulome) / its uefi bootloader (unnamed..., and at this point as I want to work on the kernel I will be using Limine and the bootloader is shelved)

## About Regulome
What I think makes Regulome interesting as a design is the (planned and not yet implemented as the project is very nascent and the kernel can't even be loaded right now) scheduler.

All processses in both kernel-space and user-space either 'promote' or 'inhibit' related processes. This works similarly to how biological cis- and trans- regulatory elements in cells modulate the expression of proteins, hence the name and the inspiration (I like the thought that in a sense the transcription related cell machinery is a form of scheduler).

## Work that is not my own
The code presented in this repository is all my own (although it uses the hard work of others with crates from crates.io). The only thing in this repository that is not my own is the font I am using ([Spleen](https://github.com/fcambus/spleen)) which is stored alongside its associated license in the spleen/ directory
