# Regulome
Shared repository for a pure rust x86\_64 kernel (Regulome) / it's uefi bootloader (unnamed...)

## About Regulome
What I think makes Regulome interesting as a design is the (planned and not yet implemented as the project is very nascent and the kernel can't even be loaded right now) scheduler.

All processses in both kernel-space and user-space either 'promote' or 'inhibit' related processes. This works similarly to how biological cis- and trans- regulatory elements in cells modulate the expression of proteins, hence the name and the inspiration (I like the thought that in a sense the transcription related cell machinery is a form of scheduler).

## Build / Install
Right now I'm working on the boot loader. To build and install that:
    -   cd into the boot/ directory

    -   run cargo build --target=x86\_64-unknown-uefi
    
    -   (Seemingly cargo default build targets don't seem to like workspaces and it wasn't worth trying to fix)
    
    -   Grab a seperate installation device to write it onto (I've been using a usb drive)
    
    -   Make sure the drive is properly formatted to be used for UEFI (this can be found elsewhere)
    
    -   Then copy the bootloader (target/x86\_64-unknown-uefi/debug/boot.efi) to /EFI/BOOT/BOOTX64.EFI on the external drive
    
    -   There also needs to be a file named "font.bdf" in the root directory of the installation drive (the bootloader looks for certain hardcoded paths, currently on this is relavent, since we early exist before trying to load the kernel)

## Work that is not my own
The code presented in this repository is all my own (although it uses the hard work of others with crates from crates.io). The only thing in this repository that is not my own is the font I am using ([Spleen](https://github.com/fcambus/spleen)) which is stored alongside its associated license in the spleen/ directory
