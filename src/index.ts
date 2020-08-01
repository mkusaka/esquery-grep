import { parse } from "@typescript-eslint/parser";
import esquery from "esquery";

export const grep = (content: string, query: string) => {
  const ast = parse(content, {
    ecmaVersion: 2020,
    sourceType: "module",
    ecmaFeatures: {
      jsx: true
    }
  });

  // @ts-ignore
  return esquery(ast, query)
}
