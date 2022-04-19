/// <reference path="./index.d.ts" />

// https://wiki.osdev.org/Pci

const enum PCI {
  CONFIG_ADDRESS = 0xCF8,
  CONFIG_DATA = 0xCFC,
}

const enum COMMOM_OFFSET {
  DEVICE_ID = 0x00,
  VENDOR_ID = 0x00,
  CLASS = 0x08,
  SUBCLASS = 0x08,
  PROG_IF = 0x08,
  HEADER_TYPE = 0x0C,
}

const enum HEADER_TYPE_00_OFFSET {
  SUBSYSTEM_ID = 0x2C,
}

const enum HEADER_TYPE_01_OFFSET {
  SECONDARY_BUS = 0x18,
}

function getData(bus: number, device: number, func: number, offset: number) {
  const addr = (bus << 16) | (device << 11) | (func << 8) | (offset & 0xFC) |
    0x80000000;
  Kernel.outl(PCI.CONFIG_ADDRESS, addr);
  return Kernel.inl(PCI.CONFIG_DATA);
}

function getDeviceID(bus: number, device: number, func: number) {
  return getData(bus, device, func, COMMOM_OFFSET.DEVICE_ID) >> 16;
}

function getVendorID(bus: number, device: number, func: number) {
  return getData(bus, device, func, COMMOM_OFFSET.VENDOR_ID) & 0xFFFF;
}

function getHeaderType(bus: number, device: number, func: number) {
  return (getData(bus, device, func, COMMOM_OFFSET.HEADER_TYPE) >> 16) & 0x7F;
}

function hasMultipleFunctions(bus: number, device: number, func: number) {
  return ((getData(bus, device, func, COMMOM_OFFSET.HEADER_TYPE) >> 16) &
    0x80) !== 0;
}

function getBaseClass(bus: number, device: number, func: number) {
  return getData(bus, device, func, COMMOM_OFFSET.CLASS) >> 24;
}

function getSubClass(bus: number, device: number, func: number) {
  return (getData(bus, device, func, COMMOM_OFFSET.SUBCLASS) >> 16) & 0xFF;
}

function getProgIF(bus: number, device: number, func: number) {
  return (getData(bus, device, func, COMMOM_OFFSET.PROG_IF) >> 8) & 0xFF;
}

function getSubsystemID(bus: number, device: number, func: number) {
  return getData(bus, device, func, HEADER_TYPE_00_OFFSET.SUBSYSTEM_ID) >> 16;
}

function getSecondaryBus(bus: number, device: number, func: number) {
  return (getData(bus, device, func, HEADER_TYPE_01_OFFSET.SECONDARY_BUS) >>
    8) & 0xFF;
}

function checkDevice(bus: number, device: number) {
  let func = 0;

  if (getVendorID(bus, device, func) === 0xFFFF) return; // Device doesn't exist
  checkFunction(bus, device, func);

  if (hasMultipleFunctions(bus, device, func)) {
    for (func = 1; func < 8; ++func) {
      if (getVendorID(bus, device, func) === 0xFFFF) continue;
      checkFunction(bus, device, func);
    }
  }
}

function checkFunction(bus: number, device: number, func: number) {
  const baseClass = getBaseClass(bus, device, func);
  const subClass = getSubClass(bus, device, func);
  console.log(baseClass, subClass, getProgIF(bus, device, func));

  if ((baseClass === 0x6) && (subClass === 0x4)) {
    const secondaryBus = getSecondaryBus(bus, device, func);
    checkBus(secondaryBus);
  }
}

function checkBus(bus: number) {
  for (let device = 0; device < 0x20; ++device) {
    checkDevice(bus, device);
  }
}

export function checkAllBuses() {
  if (hasMultipleFunctions(0, 0, 0)) {
    for (let func = 0; func < 8; ++func) {
      if (getVendorID(0, 0, func) !== 0xFFFF) break;
      checkBus(func);
    }
  } else {
    checkBus(0);
  }
}
