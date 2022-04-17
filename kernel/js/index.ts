/// <reference path="./index.d.ts" />

import { getDate } from "./rtc.ts";

const date = getDate();

console.log(
  `${date.year}-${date.month}-${date.day} ${date.hour}:${date.minute}:${date.second}`,
);
