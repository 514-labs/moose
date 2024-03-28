"use client";
import { getCliData } from "app/db";
import {
  CLI_DATA_ID,
  CliData,
  DataModel,
  VersionKey,
  defaultCliData,
} from "app/types";
import { getModelsByVersion } from "lib/utils";
import {
  createContext,
  Dispatch,
  PropsWithChildren,
  SetStateAction,
  useEffect,
  useMemo,
  useState,
} from "react";

export const VersionContext = createContext<{
  version: VersionKey;
  setVersion: Dispatch<SetStateAction<string>>;
  models: DataModel[];
  cliData: CliData;
}>({
  version: "latest",
  setVersion: () => "",
  models: [],
  cliData: defaultCliData[CLI_DATA_ID],
});

export const VersionProvider = ({
  version: initialVersion,
  children,
}: PropsWithChildren<{ version: string }>) => {
  const [version, setVersion] = useState(initialVersion);
  const [data, setData] = useState<CliData>(defaultCliData[CLI_DATA_ID]); // Define the type of 'data' to include 'models'

  useEffect(() => {
    getCliData().then((cliData) => {
      if (cliData) {
        setData(cliData);
      } else {
        throw new Error("Data is undefined");
      }
    });
  }, []);

  const models = useMemo(
    () => getModelsByVersion(data, version),
    [version, data]
  );

  return (
    <VersionContext.Provider
      value={{ version, setVersion, models, cliData: data }}
    >
      {children}
    </VersionContext.Provider>
  );
};
