declare module "shiki" {
  export interface Highlighter {
    codeToHtml(code: string, options: { lang: string }): Promise<string>;
  }
  export function getHighlighter(options: {
    theme: string;
  }): Promise<Highlighter>;
}
