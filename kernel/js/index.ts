/// <reference path="./index.d.ts" />

import { getDate } from "./rtc.ts";
import { checkAllBuses } from "./pci.ts";

const date = getDate();

console.log(
  `${date.year}-${date.month}-${date.day} ${date.hour}:${date.minute}:${date.second}`,
);

checkAllBuses();

declare const USER_CODES: Array<number[]>;
const procs = USER_CODES.map((i) => Kernel.spawn(Uint8Array.from(i).buffer));

let idx = 0;
const schedule = () => {
  if (!procs[idx].steps()) {
    procs.splice(idx, 1);
    if (procs.length === 0) return;
    if (idx === procs.length) idx = 0;
  }
  if (Kernel.shouldSchedule()) {
    idx = (idx + 1) % procs.length;
  }
  queueMicrotask(schedule);
};
queueMicrotask(schedule);
