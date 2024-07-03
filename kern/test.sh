sudo -u leastinformednerd cargo build --target=os-dev-target.json
echo "Compiled"
losetup /dev/loop0 disk.img
echo "Loopback created"
mkdir ./tmp_mnt
echo "Made mountpoint"
mount /dev/loop0 ./tmp_mnt
echo "Mounted"
cp ../target/os-dev-target/debug/kern ./tmp_mnt/kernel.elf
echo "Copied over"
umount ./tmp_mnt
echo "Unmounted"
rmdir ./tmp_mnt
echo "Deleted mountpoint"
losetup -d /dev/loop0
echo "Deleted loopback"
qemu-system-x86_64 -display gtk,zoom-to-fit=on --bios /usr/share/OVMF/x64/OVMF.fd -drive file=disk.img,format=raw,index=0,media=disk
