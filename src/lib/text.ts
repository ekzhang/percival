import AnsiUp from "ansi_up";
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkMath from "remark-math";
import remarkRehype from "remark-rehype";
import rehypeKatex from "rehype-katex";
import rehypeStringify from "rehype-stringify";

const pipeline = unified()
  .use(remarkParse)
  .use(remarkMath)
  .use(remarkRehype)
  .use(rehypeKatex)
  .use(rehypeStringify);

export function markdownToHtml(markdown: string): string {
  return pipeline.processSync(markdown) as any;
}

const ansi_up = new AnsiUp();

export function ansiToHtml(text: string): string {
  return ansi_up.ansi_to_html(text);
}
