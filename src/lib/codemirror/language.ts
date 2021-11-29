import { parser } from "./percival.grammar";
import {
  foldNodeProp,
  foldInside,
  indentNodeProp,
  LanguageSupport,
  LRLanguage,
} from "@codemirror/language";
import { styleTags, tags as t } from "@codemirror/highlight";

let parserWithMetadata = parser.configure({
  props: [
    styleTags({
      Identifier: t.local(t.variableName),
      TableName: t.definition(t.variableName),
      PropName: t.definition(t.propertyName),
      String: t.string,
      Number: t.number,
      Boolean: t.bool,
      Expr: [t.regexp, t.emphasis],
      LineComment: t.lineComment,
      BlockComment: t.blockComment,
      ImportKeyword: t.keyword,
      FromKeyword: t.keyword,
      Goal: t.string,
      "( )": t.paren,
      ":-": t.punctuation,
      ".": t.punctuation,
      ":": t.punctuation,
      ",": t.punctuation,
    }),
    indentNodeProp.add({
      Rule: (context) => context.column(context.node.from) + context.unit,
    }),
    foldNodeProp.add({
      Rule: foldInside,
    }),
  ],
});

export const percivalLanguage = LRLanguage.define({
  parser: parserWithMetadata,
  languageData: {
    commentTokens: {
      line: "//",
      block: { open: "/*", close: "*/" },
    },
  },
});

/** CodeMirror extension for Percival language support. */
export function percival() {
  return new LanguageSupport(percivalLanguage, []);
}
