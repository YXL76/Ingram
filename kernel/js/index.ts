/// <reference path="./index.d.ts" />

import { getDate } from "./rtc.ts";
import { checkAllBuses } from "./pci.ts";

const date = getDate();

console.log(
  `${date.year}-${date.month}-${date.day} ${date.hour}:${date.minute}:${date.second}`,
);

checkAllBuses();
