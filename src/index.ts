import { parse } from "acorn";
import esquery from "esquery";

export const grep = (content: string, query: string) => {
  const ast = parse(content, {
    ecmaVersion: 2020,
  });

  // @ts-ignore
  return esquery(ast, query)
}
