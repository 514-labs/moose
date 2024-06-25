import { PageViewEvent, PageViewProcessed } from "../datamodels/models";

export default function run(source: PageViewEvent): PageViewProcessed {
  return source;
}
