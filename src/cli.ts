import meow from "meow";
import glob from "glob"

import { grep } from "./";
import * as fs from "fs";

const cli = meow(
  `
    Usage
      $ esquery-grep [glob pattern] -q [query]
 
    Options
      -q query

    Examples
      $ esquery-grep "src/**/*.js" -q "VariableDeclaration"
`,
  {
    flags: {
      q: {
        type: "string"
      }
    },
    autoHelp: true,
    autoVersion: true
  }
);

export const run = async () => {
  glob(cli.input[0], async (err, matches) => {
    console.log("here")
    if (err) {
      throw Error("file not matches");
    }
    const readAndParse = async (path: string) => {
      const content = fs.readFileSync(path, "utf-8")
      console.log(content)
      grep(content, cli.flags.q)
    }
    await Promise.all(matches.map((match) => readAndParse(match)))
  })
};
