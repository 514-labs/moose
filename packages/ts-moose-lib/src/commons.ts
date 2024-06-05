import fs from "node:fs";
import path from "node:path";

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

export const walkDir = (
  dir: string,
  fileExtension: string,
  fileList: string[],
) => {
  const files = fs.readdirSync(dir);

  files.forEach((file) => {
    if (fs.statSync(path.join(dir, file)).isDirectory()) {
      fileList = walkDir(path.join(dir, file), fileExtension, fileList);
    } else if (file.endsWith(fileExtension)) {
      fileList.push(path.join(dir, file));
    }
  });

  return fileList;
};

export const getFileName = (filePath: string) => {
  const regex = /\/([^\/]+)\.ts/;
  const matches = filePath.match(regex);
  if (matches && matches.length > 1) {
    return matches[1];
  }
  return "";
};
