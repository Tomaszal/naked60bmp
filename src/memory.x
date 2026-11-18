MEMORY
{
  /* Leave the MBR and Adafruit bootloader intact. RMK storage is at 0xA0000..0xBFFFF. */
  FLASH : ORIGIN = 0x00001000, LENGTH = 0x0009F000
  /* The bootloader uses the first eight RAM bytes for double-reset detection. */
  RAM : ORIGIN = 0x20000008, LENGTH = 0x0003FFF8
}
