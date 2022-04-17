export {};

interface IKernel {
  RTC_CENTURY_REG: number;

  inb: (port: number) => number;
  outb: (port: number, value: number) => void;
}

declare global {
  const Kernel: IKernel;
}
