/// <reference path="./index.d.ts" />
// https://wiki.osdev.org/CMOS

const enum CMOS {
  ADDRESS = 0x70,
  DATA = 0x71,
}

const enum REGISTER {
  SECOND = 0x00,
  MINUTE = 0x02,
  HOUR = 0x04,
  WEEKDAY = 0x06,
  DAY_OF_MONTH = 0x07,
  MONTH = 0x08,
  YEAR = 0x09,
  /** Update in progress */
  STATUS_A = 0x0A,
  /** Format of Bytes */
  STATUS_B = 0x0B,
}

function getRegister(reg: number) {
  Kernel.outb(CMOS.ADDRESS, reg);
  return Kernel.inb(CMOS.DATA);
}

export function getDate() {
  let second = getRegister(REGISTER.SECOND);
  let minute = getRegister(REGISTER.MINUTE);
  let hour = getRegister(REGISTER.HOUR);
  let day = getRegister(REGISTER.DAY_OF_MONTH);
  let month = getRegister(REGISTER.MONTH);
  let year = getRegister(REGISTER.YEAR);
  let century = getRegister(Kernel.RTC_CENTURY_REG);

  const format = getRegister(REGISTER.STATUS_B);

  // Convert BCD to binary values if necessary
  if ((format & 0x04) === 0) {
    second = (second & 0x0F) + (Math.floor(second / 16) * 10);
    minute = (minute & 0x0F) + (Math.floor(minute / 16) * 10);
    hour = ((hour & 0x0F) + (Math.floor((hour & 0x70) / 16) * 10)) |
      (hour & 0x80);
    day = (day & 0x0F) + (Math.floor(day / 16) * 10);
    month = (month & 0x0F) + (Math.floor(month / 16) * 10);
    year = (year & 0x0F) + (Math.floor(year / 16) * 10);
    century = (century & 0x0F) + (Math.floor(century / 16) * 10);
  }

  // Convert 12 hour clock to 24 hour clock if necessary
  if ((format & 0x02) === 0 && (hour & 0x80)) {
    hour = ((hour & 0x7F) + 12) % 24;
  }

  // Calculate the full (4-digit) year
  if (Kernel.RTC_CENTURY_REG !== 0) {
    year += century * 100;
  } else {
    year += (2022 / 100) * 100;
    if (year < 2022) year += 100;
  }

  return {
    second,
    minute,
    hour,
    day,
    month,
    year,
  };
}
