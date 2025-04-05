/* Memory layout for S32K148 microcontroller */
MEMORY
{
  /* Flash memory */
  FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K  /* Adjust size as needed, S32K148 has different flash sizes */
  
  /* RAM */
  RAM (rwx) : ORIGIN = 0x1FFF0000, LENGTH = 32K    /* Adjust RAM size and address according to datasheet */
  
  /* Additional RAM regions if needed */
  /* RAM2 (rwx) : ORIGIN = 0x20000000, LENGTH = 32K */
}

/* Define the boot loader reserved area - adjust as needed */
_bootloader_size = 32K;  /* Example size - adjust based on requirements */

/* This is where the application code should be placed */
_app_start = ORIGIN(FLASH) + _bootloader_size;

/* Export symbols */
PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));

/* Optional memory regions for bootloader specific needs */
/* SECTIONS
{
  .bootloader_data (NOLOAD) : ALIGN(4)
  {
    *(.bootloader_data)
    . = ALIGN(4);
  } > RAM
} */
