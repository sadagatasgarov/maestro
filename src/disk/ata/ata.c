#include <disk/ata/ata.h>
#include <memory/memory.h>

static cache_t *devices_cache;
ata_device_t *devices = NULL;

// TODO Optimize I/O

__attribute__((cold))
void ata_init(void)
{
	if(!(devices_cache = cache_create("ata_devices", sizeof(ata_device_t), 32,
		bzero, NULL)))
	{
		// TODO Error (failed to init driver)
		return;
	}
	devices = ata_init_device(ATA_PRIMARY_BUS, ATA_PRIMARY_CTRL);
	// TODO Check error
	// TODO Use PCI to get every devices
	// TODO Printfs
}

__attribute__((hot))
static inline int ata_has_err(ata_device_t *dev)
{
	return (inb(dev->bus + ATA_REG_STATUS) & ATA_STATUS_ERR);
}

__attribute__((hot))
void ata_irq(void)
{
	ata_device_t *dev;

	// TODO Check which device did the interrupt
	dev = devices;
	dev->wait_irq = 0;
}

__attribute__((hot))
void ata_err_check(void)
{
	ata_device_t *d;

	d = devices;
	while(d)
	{
		if(d->wait_irq && ata_has_err(d))
			d->wait_irq = 0;
		d = d->next;
	}
}

static inline void ata_wait(const uint16_t port)
{
	size_t i;

	for(i = 0; i < 4; ++i)
		inb(port);
}

static inline int ata_check_floating_bus(const uint16_t bus)
{
	return (inb(bus + ATA_REG_STATUS) == 0xff);
}

static inline int ata_is_ready(const uint16_t bus)
{
	return (inb(bus + ATA_REG_STATUS) & ATA_STATUS_RDY);
}

static inline int ata_is_busy(const uint16_t bus)
{
	return (inb(bus + ATA_REG_STATUS) & ATA_STATUS_BSY);
}

static inline void ata_wait_ready(ata_device_t *dev)
{
	dev->wait_irq = 1;
	while(dev->wait_irq && !ata_is_ready(dev->bus)) // TODO Is it even entering the loop?
		kernel_wait();
	dev->wait_irq = 0;
}

static inline void ata_command(const uint16_t bus, const uint8_t cmd)
{
	outb(bus + ATA_REG_COMMAND, cmd);
}

static inline void ata_select_drive(const uint16_t bus, const int slave)
{
	outb(bus + ATA_REG_DRIVE, slave ? 0xa0 : 0xb0);
}

static inline int ata_identify(const uint16_t bus, const int slave,
	uint16_t *init_data)
{
	uint8_t status;
	size_t i;

	ata_select_drive(bus, slave);
	outb(bus + ATA_REG_SECTOR_COUNT, 0x0);
	outb(bus + ATA_REG_SECTOR_NUMBER, 0x0);
	outb(bus + ATA_REG_CYLINDER_LOW, 0x0);
	outb(bus + ATA_REG_CYLINDER_HIGH, 0x0);
	ata_command(bus, ATA_CMD_IDENTIFY);
	if((status = inb(bus + ATA_REG_STATUS)) == 0)
		return 0;
	while(ata_is_busy(bus))
		;
	if(inb(bus + ATA_REG_CYLINDER_LOW) || inb(bus + ATA_REG_CYLINDER_HIGH))
		return 0;
	do
		status = inb(bus + ATA_REG_STATUS);
	while(!(status & ATA_STATUS_ERR) && !(status & ATA_STATUS_DRQ));
	// TODO Some ATAPI devices doesn't set ERR on abort
	if(status & ATA_STATUS_ERR)
		return 0;
	for(i = 0; i < 256; ++i)
		init_data[i] = inw(bus + ATA_REG_DATA);
	return 1;
}

static inline uint32_t ata_lba28_sectors(const uint16_t *data)
{
	return *(uint32_t *) (data + 60);
}

static inline int ata_supports_lba48(const uint16_t *data)
{
	return (data[83] & 0b10000000000);
}

// TODO Put printfs out of this functions
ata_device_t *ata_init_device(const uint16_t bus, const uint16_t ctrl)
{
	ata_device_t *dev;
	uint16_t init_data[256];
	uint32_t sectors = 0;

	if(!(dev = cache_alloc(devices_cache)))
		return NULL;
	dev->bus = bus;
	dev->ctrl = ctrl;
	if(ata_check_floating_bus(bus))
	{
		printf("ATA floating bus detected\n");
		cache_free(devices_cache, dev);
		return NULL;
	}
	if(!ata_identify(bus, 0, init_data))
	{
		printf("ATA identify failed\n");
		cache_free(devices_cache, dev);
		return NULL;
	}
	if((sectors = ata_lba28_sectors(init_data)) != 0)
		printf("ATA LBA28 sectors: %u\n", (unsigned) sectors);
	if(ata_supports_lba48(init_data))
	{
		printf("ATA LBA48 supported\n");
		// TODO Get sectors
	}
	printf("ATA disk size: %u bytes\n", (unsigned) sectors * ATA_SECTOR_SIZE);
	// TODO Set data in struct
	printf("ATA initialized!\n");
	return dev;
}

int ata_get_type(const ata_device_t *dev, const int slave)
{
	unsigned cl, ch;

	if(!dev)
		return ATA_TYPE_UNKNOWN;
	ata_reset(dev);
	ata_select_drive(dev->bus, slave);
	ata_wait(dev->ctrl);
	cl = inb(dev->bus + ATA_REG_CYLINDER_LOW);
	ch = inb(dev->bus + ATA_REG_CYLINDER_HIGH);
	if(cl == 0 && ch == 0)
		return ATA_TYPE_PATA;
	if(cl == 0x14 && ch == 0xeb)
		return ATA_TYPE_PATAPI;
	if(cl == 0x3c && ch == 0xc3)
		return ATA_TYPE_SATA;
	if(cl == 0x69 && ch == 0x96)
		return ATA_TYPE_SATAPI;
	return ATA_TYPE_UNKNOWN;
}

// TODO Set errnos?
// TODO sectors == 0
// TODO Doesn't work when lba > 0
// TODO Crashes with more than 1 sector
int ata_read(ata_device_t *dev, const int slave, const size_t lba,
	void *buff, const size_t sectors)
{
	size_t i, j;

	if(!dev || !buff || sectors > 0xff)
		return -1;
	spin_lock(&dev->spinlock);
	outb(dev->bus + ATA_REG_DRIVE, (slave ? 0xe0 : 0xf0)
		| ((lba >> 24) & 0xf));
	outb(dev->bus + ATA_REG_SECTOR_COUNT, (uint8_t) sectors);
	outb(dev->bus + ATA_REG_SECTOR_NUMBER, (uint8_t) lba);
	outb(dev->bus + ATA_REG_CYLINDER_LOW, (uint8_t) (lba >> 8));
	outb(dev->bus + ATA_REG_CYLINDER_HIGH, (uint8_t) (lba >> 16));
	ata_command(dev->bus, ATA_CMD_READ_SECTORS);
	for(i = 0; i < sectors; ++i)
	{
		ata_wait_ready(dev);
		if(ata_has_err(dev))
		{
			// TODO Clear err?
			spin_unlock(&dev->spinlock);
			return -1;
		}
		for(j = 0; j < 256; ++j)
		{
			*((uint16_t *) buff) = inw(dev->bus + ATA_REG_DATA);
			buff += sizeof(uint16_t);
		}
		if(i >= sectors)
			ata_wait(dev->ctrl);
	}
	spin_unlock(&dev->spinlock);
	return 0;
}

// TODO Set errnos?
// TODO sectors == 0
// TODO Doesn't work when lba > 0
// TODO Doesn't work with more than 1 sector
int ata_write(ata_device_t *dev, const int slave, const size_t lba,
	const void *buff, const size_t sectors)
{
	size_t i, j;

	if(!dev || !buff || sectors > 0xff)
		return -1;
	spin_lock(&dev->spinlock);
	outb(dev->bus + ATA_REG_DRIVE, (slave ? 0xe0 : 0xf0)
		| ((lba >> 24) & 0xf));
	outb(dev->bus + ATA_REG_SECTOR_COUNT, (uint8_t) sectors);
	outb(dev->bus + ATA_REG_SECTOR_NUMBER, (uint8_t) lba);
	outb(dev->bus + ATA_REG_CYLINDER_LOW, (uint8_t) (lba >> 8));
	outb(dev->bus + ATA_REG_CYLINDER_HIGH, (uint8_t) (lba >> 16));
	ata_command(dev->bus, ATA_CMD_WRITE_SECTORS);
	for(i = 0; i < sectors; ++i)
	{
		ata_wait_ready(dev);
		if(ata_has_err(dev))
		{
			// TODO Clear err?
			spin_unlock(&dev->spinlock);
			return -1;
		}
		for(j = 0; j < 256; ++j)
		{
			outw(dev->bus + ATA_REG_DATA, *((uint16_t *) buff));
			buff += sizeof(uint16_t);
		}
	}
	ata_command(dev->bus, ATA_CMD_CACHE_FLUSH);
	spin_unlock(&dev->spinlock);
	return 0;
}

void ata_reset(const ata_device_t *dev)
{
	uint8_t reg;

	if(!dev)
		return;
	reg = dev->ctrl + ATA_CTRL_DEVICE_CONTROL;
	outb(reg, inb(reg) | 0b100);
	outb(reg, inb(reg) & ~0b100);
}