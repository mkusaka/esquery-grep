import fg from "fast-glob"

import { grep } from "./";
import * as fs from "fs";

const readAndParse = (path: string) => {
  const content = fs.readFileSync(path, "utf-8")
  grep(content, process.argv[3])
}

export const run = async () => {
  const entries = await fg(process.argv[2], { dot: true })
  await Promise.all(entries.map((path) => readAndParse(path)))
};
