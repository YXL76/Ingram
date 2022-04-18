export {};

interface IKernel {
  RTC_CENTURY_REG: number;

  inb: (port: number) => number;
  outb: (port: number, value: number) => void;
  inw: (port: number) => number;
  outw: (port: number, value: number) => void;
  inl: (port: number) => number;
  outl: (port: number, value: number) => void;
}

declare global {
  const Kernel: IKernel;
}
