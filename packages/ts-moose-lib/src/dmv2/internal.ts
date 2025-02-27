import process from "process";

const moose_internal = {
  tables: new Map(),
};

(globalThis as any).moose_internal = moose_internal;

export const getMooseInternal = (): typeof moose_internal =>
  (globalThis as any).moose_internal;

getMooseInternal()["tables"] = new Map();

export const loadIndex = async () => {
  await require(`${process.cwd()}/app/index.ts`);

  console.log(
    "___MOOSE_TABLES___",
    JSON.stringify(Object.fromEntries(getMooseInternal().tables)),
    "___MOOSE_TABLES___",
  );
};
