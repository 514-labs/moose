"use client"; // Error components must be Client Components

import { ReactNode, useEffect } from "react";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}): ReactNode {
  useEffect(() => {
    // Log the error to an error reporting service
    console.error(error);
  }, [error]);

  return (
    <div>
      <h2>Table Not Found</h2>
      <a href="/"> go back</a>
    </div>
  );
}
