"use client";
import {
  createContext,
  Dispatch,
  PropsWithChildren,
  SetStateAction,
  useState,
} from "react";

export const VersionContext = createContext<{
  version: string;
  setVersion: Dispatch<SetStateAction<string>>;
}>({ version: "latest", setVersion: () => "" });

export const VersionProvider = ({
  version: initialVersion,
  children,
}: PropsWithChildren<{ version: string }>) => {
  const [version, setVersion] = useState(initialVersion);

  return (
    <VersionContext.Provider value={{ version, setVersion }}>
      {children}
    </VersionContext.Provider>
  );
};
